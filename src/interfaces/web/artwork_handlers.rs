use axum::{
    Json,
    extract::{Multipart, Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

// Import domain entities
use super::error_response::ErrorResponse;
use super::dto::{StrategyStats, StrategyComparisonResponse};
use crate::domain::artwork::entities::{Artwork, ArtworkMetadata, Canvas, Dot};
use crate::domain::shared::value_objects::{Color, Coordinates};
use crate::domain::painting::{ArtworkToCommandConverter, DrawingCanvasConfig, DrawingStrategy};

use crate::domain::controller::{Button, ControllerAction, ControllerCommand, ControllerEmulator, DPad, StickPosition};
use crate::domain::hardware::errors::HardwareError;

/// ボタンを1回タップする共通処理（デフォルト: 押下300ms、離す200ms、待機400ms）
fn tap_button(controller: &Arc<dyn ControllerEmulator>, button: Button, name: &str) -> Result<(), HardwareError> {
    tap_button_with_duration(controller, button, name, 300, 200, 400)
}

/// ボタンを1回タップする共通処理（時間指定版）
fn tap_button_with_duration(
    controller: &Arc<dyn ControllerEmulator>,
    button: Button,
    name: &str,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u64,
) -> Result<(), HardwareError> {
    let tap_cmd = ControllerCommand::new(name)
        .add_action(ControllerAction::press_button(button, press_ms))
        .add_action(ControllerAction::release_button(button, release_ms));
    controller.execute_command(&tap_cmd)?;
    if wait_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(wait_ms));
    }
    Ok(())
}

/// 十字キーを1回タップする共通処理（デフォルト: 押下100ms、離す50ms、待機50ms）
#[allow(dead_code)]
fn tap_dpad(controller: &Arc<dyn ControllerEmulator>, dpad: DPad, name: &str) -> Result<(), HardwareError> {
    tap_dpad_with_duration(controller, dpad, name, 100, 50, 50)
}

/// 十字キーを1回タップする共通処理（時間指定版）
fn tap_dpad_with_duration(
    controller: &Arc<dyn ControllerEmulator>,
    dpad: DPad,
    name: &str,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u64,
) -> Result<(), HardwareError> {
    let tap_cmd = ControllerCommand::new(name)
        .add_action(ControllerAction::set_dpad(dpad, press_ms))
        .add_action(ControllerAction::set_dpad(DPad::NEUTRAL, release_ms));
    controller.execute_command(&tap_cmd)?;
    if wait_ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(wait_ms));
    }
    Ok(())
}

#[derive(Clone)]
pub struct PaintingControl {
    pub stop_signal: Arc<AtomicBool>,
    pub pause_signal: Arc<AtomicBool>,
}

impl PaintingControl {
    pub fn new() -> Self {
        Self {
            stop_signal: Arc::new(AtomicBool::new(false)),
            pause_signal: Arc::new(AtomicBool::new(false)),
        }
    }
}

#[derive(Clone)]
pub struct ArtworkState {
    pub artworks: Arc<RwLock<HashMap<String, Artwork>>>,
    pub controller: Arc<dyn ControllerEmulator>,
    pub active_painting: Arc<RwLock<Option<PaintingControl>>>,
}

impl ArtworkState {
    pub fn new(controller: Arc<dyn ControllerEmulator>) -> Self {
        Self {
            artworks: Arc::new(RwLock::new(HashMap::new())),
            controller,
            active_painting: Arc::new(RwLock::new(None)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtworkSummary {
    pub id: String,
    pub name: String,
    pub format: String,
    pub canvas_size: String,
    pub total_dots: usize,
    pub drawable_dots: usize,
    pub completion_ratio: f32,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateArtworkRequest {
    pub name: String,
    pub width: u16,
    pub height: u16,
    pub dots: Vec<DotData>,
}

#[derive(Debug, Deserialize)]
pub struct DotData {
    pub x: u16,
    pub y: u16,
    pub color: String,
}

#[derive(Debug, Serialize)]
pub struct ArtworkResponse {
    pub id: String,
    pub message: String,
    pub artwork: Option<ArtworkSummary>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PaintRequest {
    pub press_ms: Option<u32>,
    pub release_ms: Option<u32>,
    pub wait_ms: Option<u32>,
    pub preview: Option<bool>,
    pub strategy: Option<DrawingStrategy>,
}

#[derive(Debug, Deserialize)]
pub struct GetPathRequest {
    pub strategy: Option<DrawingStrategy>,
}

#[derive(Debug, Serialize)]
pub struct PathResponse {
    pub path: Vec<Coordinates>,
    pub estimated_time_sec: f64,
}

/// List all artworks
pub async fn list_artworks(State(state): State<Arc<ArtworkState>>) -> Json<Vec<ArtworkSummary>> {
    let artworks = state.artworks.read().await;
    let summaries: Vec<ArtworkSummary> = artworks
        .values()
        .map(|artwork| ArtworkSummary {
            id: artwork.id.as_str().to_string(),
            name: artwork.metadata.name.clone(),
            format: artwork.original_format.clone(),
            canvas_size: format!("{}x{}", artwork.canvas.width, artwork.canvas.height),
            total_dots: artwork.total_dots(),
            drawable_dots: artwork.drawable_dots(),
            completion_ratio: artwork.completion_ratio() as f32,
            created_at: artwork.created_at.epoch_millis as i64,
            updated_at: artwork.updated_at.epoch_millis as i64,
        })
        .collect();

    Json(summaries)
}

/// Create a new artwork
pub async fn create_artwork(
    State(state): State<Arc<ArtworkState>>,
    request: Result<Json<CreateArtworkRequest>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<ArtworkResponse>, impl IntoResponse> {
    // Handle JSON parsing errors
    let Json(request) = match request {
        Ok(json) => json,
        Err(e) => {
            warn!("JSON parsing error: {:?}", e);
            return Err(ErrorResponse::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Invalid JSON: {e}"),
            ));
        }
    };

    info!("Creating artwork: {}", request.name);
    info!("Dimensions: {}x{}", request.width, request.height);
    info!("Number of dots: {}", request.dots.len());

    // Validate dimensions
    if request.width == 0 || request.height == 0 {
        warn!("Invalid dimensions: {}x{}", request.width, request.height);
        return Err(ErrorResponse::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Width and height must be greater than 0",
        ));
    }

    if request.width > 1000 || request.height > 1000 {
        warn!("Dimensions too large: {}x{}", request.width, request.height);
        return Err(ErrorResponse::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Width and height must not exceed 1000 pixels",
        ));
    }

    // Validate dots
    if request.dots.is_empty() {
        warn!("No dots provided");
        return Err(ErrorResponse::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "At least one dot is required",
        ));
    }

    // Create canvas from dots
    let mut canvas = Canvas::new(request.width, request.height);

    // Add dots to canvas
    for (index, dot_data) in request.dots.iter().enumerate() {
        // Validate dot coordinates
        if dot_data.x >= request.width || dot_data.y >= request.height {
            warn!(
                "Dot {} has invalid coordinates: ({}, {})",
                index, dot_data.x, dot_data.y
            );
            return Err(ErrorResponse::new(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Dot at index {index} has coordinates outside canvas bounds"),
            ));
        }

        let color = parse_color(&dot_data.color).unwrap_or(Color::new(0, 0, 0, 255));
        let coordinates = Coordinates::new(dot_data.x, dot_data.y);
        let dot = Dot::new(color, 255);
        if let Err(e) = canvas.set_dot(coordinates, dot) {
            warn!(
                "Failed to set dot at ({}, {}): {:?}",
                dot_data.x, dot_data.y, e
            );
        }
    }

    // Create metadata
    let metadata =
        ArtworkMetadata::new(request.name.clone()).with_description("Created via API".to_string());

    // Create artwork
    let artwork = Artwork::new(metadata, "api".to_string(), canvas);
    let artwork_id = artwork.id.as_str().to_string();

    // Store artwork
    state
        .artworks
        .write()
        .await
        .insert(artwork_id.clone(), artwork);

    info!("Artwork created with ID: {}", artwork_id);

    Ok(Json(ArtworkResponse {
        id: artwork_id,
        message: format!("Artwork '{}' created successfully", request.name),
        artwork: None,
    }))
}

/// Get a specific artwork
pub async fn get_artwork(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
) -> Result<Json<ArtworkSummary>, StatusCode> {
    let artworks = state.artworks.read().await;

    match artworks.get(&id) {
        Some(artwork) => Ok(Json(ArtworkSummary {
            id: artwork.id.as_str().to_string(),
            name: artwork.metadata.name.clone(),
            format: artwork.original_format.clone(),
            canvas_size: format!("{}x{}", artwork.canvas.width, artwork.canvas.height),
            total_dots: artwork.total_dots(),
            drawable_dots: artwork.drawable_dots(),
            completion_ratio: artwork.completion_ratio() as f32,
            created_at: artwork.created_at.epoch_millis as i64,
            updated_at: artwork.updated_at.epoch_millis as i64,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete an artwork
pub async fn delete_artwork(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let mut artworks = state.artworks.write().await;

    match artworks.remove(&id) {
        Some(_) => {
            info!("Artwork {} deleted", id);
            Ok(Json(ApiResponse {
                success: true,
                message: "Artwork deleted successfully".to_string(),
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get drawing path for an artwork
pub async fn get_artwork_path(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
    Query(params): Query<GetPathRequest>,
) -> Result<Json<PathResponse>, StatusCode> {
    let artworks = state.artworks.read().await;

    match artworks.get(&id) {
        Some(artwork) => {
            let strategy = params.strategy.unwrap_or(DrawingStrategy::GreedyTwoOpt);
            let config = DrawingCanvasConfig::default();
            let converter = ArtworkToCommandConverter::new(config, strategy);
            let drawing_path = converter.create_drawing_path(&artwork.canvas);
            
            Ok(Json(PathResponse {
                path: drawing_path.coordinates,
                estimated_time_sec: drawing_path.estimated_time_ms as f64 / 1000.0,
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get stats for all drawing strategies
pub async fn get_artwork_strategies(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
) -> Result<Json<StrategyComparisonResponse>, StatusCode> {
    let artworks = state.artworks.read().await;

    match artworks.get(&id) {
        Some(artwork) => {
            let strategies = vec![
                DrawingStrategy::GreedyTwoOpt,
                DrawingStrategy::NearestNeighbor,
                DrawingStrategy::ZigZag,
                DrawingStrategy::RasterScan,
            ];

            let mut stats_list = Vec::new();

            for strategy in strategies {
                let config = DrawingCanvasConfig::default();
                let converter = ArtworkToCommandConverter::new(config, strategy);
                let drawing_path = converter.create_drawing_path(&artwork.canvas);

                // Calculate operations
                let mut dpad_operations = 0;
                let mut a_button_presses = 0;
                
                // Initial position (0,0)
                let mut current_x = 0;
                let mut current_y = 0;

                for coord in &drawing_path.coordinates {
                    // Move operations
                    let dx = (coord.x as i32 - current_x as i32).abs();
                    let dy = (coord.y as i32 - current_y as i32).abs();
                    dpad_operations += (dx + dy) as usize;

                    // Paint operation
                    a_button_presses += 1;

                    current_x = coord.x;
                    current_y = coord.y;
                }

                stats_list.push(StrategyStats {
                    strategy,
                    dpad_operations,
                    a_button_presses,
                    estimated_time_seconds: drawing_path.estimated_time_ms as f64 / 1000.0,
                });
            }

            Ok(Json(StrategyComparisonResponse {
                strategies: stats_list,
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Stop current painting
pub async fn stop_painting(
    State(state): State<Arc<ArtworkState>>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let active_painting = state.active_painting.read().await;

    if let Some(control) = active_painting.as_ref() {
        control.stop_signal.store(true, Ordering::SeqCst);
        info!("Stop signal sent to active painting");
        Ok(Json(ApiResponse {
            success: true,
            message: "Painting stopped".to_string(),
        }))
    } else {
        Ok(Json(ApiResponse {
            success: false,
            message: "No active painting found".to_string(),
        }))
    }
}

/// Pause/Resume current painting
pub async fn pause_painting(
    State(state): State<Arc<ArtworkState>>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let active_painting = state.active_painting.read().await;
    
    if let Some(control) = active_painting.as_ref() {
        let current = control.pause_signal.load(Ordering::SeqCst);
        control.pause_signal.store(!current, Ordering::SeqCst);
        
        let status = if !current { "paused" } else { "resumed" };
        info!("Painting {}", status);
        
        Ok(Json(ApiResponse {
            success: true,
            message: format!("Painting {}", status),
        }))
    } else {
        Ok(Json(ApiResponse {
            success: false,
            message: "No active painting found".to_string(),
        }))
    }
}

/// Paint an artwork
pub async fn paint_artwork(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
    Json(request): Json<PaintRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let artworks = state.artworks.read().await;

    match artworks.get(&id) {
        Some(artwork) => {
            let press_ms = request.press_ms.unwrap_or(100);
            let release_ms = request.release_ms.unwrap_or(60);
            let wait_ms = request.wait_ms.unwrap_or(40);
            let preview = request.preview.unwrap_or(false);
            let strategy = request.strategy.unwrap_or(DrawingStrategy::GreedyTwoOpt);

            info!(
                "Starting painting for artwork {} (timing: {}+{}+{}ms/px, preview: {}, strategy: {:?})",
                id, press_ms, release_ms, wait_ms, preview, strategy
            );

            let artwork_clone = artwork.clone();
            let controller = state.controller.clone();

            // Setup control signals
            let control = PaintingControl::new();
            let stop_signal = control.stop_signal.clone();
            let pause_signal = control.pause_signal.clone();

            // Store active painting control
            {
                let mut active = state.active_painting.write().await;
                *active = Some(control);
            }

            let active_painting_store = state.active_painting.clone();

            // Spawn painting task
            tokio::spawn(async move {
                // Run blocking controller operations in a blocking thread
                let result = tokio::task::spawn_blocking(move || {
                    perform_painting(controller, artwork_clone, press_ms, release_ms, wait_ms as u64, strategy, stop_signal, pause_signal)
                }).await;

                // Clear active painting when done
                {
                    let mut active = active_painting_store.write().await;
                    *active = None;
                }

                match result {
                    Ok(Ok(_)) => info!("Painting completed successfully"),
                    Ok(Err(e)) => error!("Painting failed with hardware error: {}", e),
                    Err(e) => error!("Painting task panicked or was cancelled: {}", e),
                }
            });

            let total_ms_per_px = press_ms + release_ms + wait_ms;
            let estimated_time = (artwork.drawable_dots() as f64 * total_ms_per_px as f64) / 1000.0;

            Ok(Json(ApiResponse {
                success: true,
                message: format!("Painting started (estimated time: {:.1} seconds)", estimated_time),
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

fn perform_painting(
    controller: Arc<dyn ControllerEmulator>,
    artwork: Artwork,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u64,
    strategy: DrawingStrategy,
    stop_signal: Arc<AtomicBool>,
    pause_signal: Arc<AtomicBool>,
) -> Result<(), HardwareError> {
    info!("Initializing painting sequence...");

    // Check stop signal
    if stop_signal.load(Ordering::SeqCst) {
        // 停止時も必ずNEUTRAL状態にリセット
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        return Ok(());
    }

    // 1. Initialization Sequence
    // Press L multiple times to ensure pen size is set to small
    // Pen size cycles: small → medium → large → small
    // Press 5 times to guarantee we cycle through all sizes and land on small
    // (Even if some presses are missed, we should still reach small)
    info!("Setting pen size to small (pressing L button 5 times)...");
    for i in 1..=5 {
        info!("Pressing L button ({}/5)...", i);
        tap_button(&controller, Button::L, &format!("L Tap {}", i))?;
        // Wait between presses to ensure each is recognized
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    // Wait for pen menu to fully close
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Check stop signal
    if stop_signal.load(Ordering::SeqCst) {
        // 停止時も必ずNEUTRAL状態にリセット
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
        std::thread::sleep(std::time::Duration::from_millis(200));
        return Ok(());
    }

    // Move to Top-Left using left stick for fast movement
    // Switch-Fightstick uses ~250 frames (~4 seconds) of left stick at minimum position
    // StickPosition: x=0 is LEFT, y=0 is UP, so (0,0) moves to top-left
    info!("Moving to home position (Top-Left) using left stick...");

    // Move to top-left corner using left stick (5 seconds to ensure we hit the edge)
    let move_home_cmd = ControllerCommand::new("Move Home Left Stick")
        .add_action(ControllerAction::move_left_stick(StickPosition::new(0, 0), 5000))
        .add_action(ControllerAction::move_left_stick(StickPosition::CENTER, 100));
    controller.execute_command(&move_home_cmd)?;

    info!("Home position reached (0, 0)");

    // Wait before starting dot painting
    std::thread::sleep(std::time::Duration::from_millis(500));

    let total_dots = artwork.drawable_dots();
    info!("Starting dot painting... Total dots: {}", total_dots);

    // Generate drawing path using the selected strategy
    info!("Generating drawing path using strategy: {:?}", strategy);
    let config = DrawingCanvasConfig {
        cursor_speed_ms: 100, // These values are used for estimation, not actual drawing
        dot_draw_delay_ms: 100,
        ..Default::default()
    };
    let converter = ArtworkToCommandConverter::new(config, strategy);
    let drawing_path = converter.create_drawing_path(&artwork.canvas);
    let dots_to_paint = drawing_path.coordinates;

    info!("Path generated with {} dots", dots_to_paint.len());

    let mut current_x = 0;
    let mut current_y = 0;

    // カウンタを初期化
    let mut dpad_operations = 0u32;
    let mut a_button_presses = 0u32;

    // Adjust timing based on speed
    // タイミング値は引数として渡されています
    // - press_ms: 方向キーを保持する時間
    // - release_ms: ニュートラル状態を保持する時間
    // - wait_ms: 入力間の追加待機時間
    // Total time per pixel = press_ms + release_ms + wait_ms

    info!("Using timing: press={}ms, release={}ms, wait={}ms", press_ms, release_ms, wait_ms);

    for (i, coords) in dots_to_paint.into_iter().enumerate() {
        // Check stop signal
        if stop_signal.load(Ordering::SeqCst) {
            info!("Painting stopped by user");
            // 停止時も必ずNEUTRAL状態にリセット
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            return Ok(());
        }

        // Check pause signal
        while pause_signal.load(Ordering::SeqCst) {
            if stop_signal.load(Ordering::SeqCst) {
                info!("Painting stopped by user while paused");
                // 停止時も必ずNEUTRAL状態にリセット
                tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
                std::thread::sleep(std::time::Duration::from_millis(200));
                return Ok(());
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }

        let target_x = coords.x;
        let target_y = coords.y;

        // Calculate movement
        let dx = target_x as i32 - current_x as i32;
        let dy = target_y as i32 - current_y as i32;

        // Move X first
        if dx > 0 {
            for _ in 0..dx {
                if stop_signal.load(Ordering::SeqCst) { return Ok(()); } // Check stop signal during movement
                tap_dpad_with_duration(&controller, DPad::RIGHT, "Move Right", press_ms, release_ms, wait_ms)?;
                dpad_operations += 1;
                current_x += 1;
            }
        } else if dx < 0 {
            for _ in 0..dx.abs() {
                if stop_signal.load(Ordering::SeqCst) { return Ok(()); } // Check stop signal during movement
                tap_dpad_with_duration(&controller, DPad::LEFT, "Move Left", press_ms, release_ms, wait_ms)?;
                dpad_operations += 1;
                current_x -= 1;
            }
        }

        // Move Y
        if dy > 0 {
            for _ in 0..dy {
                if stop_signal.load(Ordering::SeqCst) { return Ok(()); } // Check stop signal during movement
                tap_dpad_with_duration(&controller, DPad::DOWN, "Move Down", press_ms, release_ms, wait_ms)?;
                dpad_operations += 1;
                current_y += 1;
            }
        } else if dy < 0 {
            for _ in 0..dy.abs() {
                if stop_signal.load(Ordering::SeqCst) { return Ok(()); } // Check stop signal during movement
                tap_dpad_with_duration(&controller, DPad::UP, "Move Up", press_ms, release_ms, wait_ms)?;
                dpad_operations += 1;
                current_y -= 1;
            }
        }

        // Send cursor move update (only once per dot to avoid flooding)
        use crate::interfaces::web::log_streamer::PROGRESS_CHANNEL;
        let move_msg = serde_json::json!({
            "type": "progress",
            "current": i + 1,
            "total": total_dots,
            "x": current_x,
            "y": current_y,
            "dpad_operations": dpad_operations,
            "a_button_presses": a_button_presses,
            "is_paint": false
        }).to_string();
        let _ = PROGRESS_CHANNEL.send(move_msg);

        // D-pad状態を完全にクリア（描画前）
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad Before Paint", 10, 10, 0)?;

        // Paint Dot (Press A)
        tap_button_with_duration(&controller, Button::A, "Paint Dot", press_ms, release_ms, wait_ms)?;
        a_button_presses += 1;

        // Send paint progress update
        let progress_msg = serde_json::json!({
            "type": "progress",
            "current": i + 1,
            "total": total_dots,
            "x": current_x,
            "y": current_y,
            "dpad_operations": dpad_operations,
            "a_button_presses": a_button_presses,
            "is_paint": true
        }).to_string();
        let _ = PROGRESS_CHANNEL.send(progress_msg);

        // Log progress every 100 dots
        if i % 100 == 0 {
            info!("Painted {}/{} dots", i, total_dots);
        }
    }

    info!("Painting completed!");
    Ok(())
}

/// 速度キャリブレーションテスト
/// 指定された速度パラメータで横20ドットを5行描画
/// ドットが乱れたらその速度はSwitchの限界を超えている
pub fn perform_speed_calibration(
    controller: Arc<dyn ControllerEmulator>,
    stop_signal: Arc<AtomicBool>,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u32,
    skip_initialization: bool,
) -> Result<(), HardwareError> {
    let total_ms = press_ms + release_ms + wait_ms;
    info!("Starting speed calibration test ({}ms/pixel: press={}ms, release={}ms, wait={}ms, skip_init={})...",
          total_ms, press_ms, release_ms, wait_ms, skip_initialization);

    // Initialize controller
    controller.initialize()?;

    if !skip_initialization {
        // ペンサイズを小に設定（5回押下）
        info!("Setting pen size to small...");
        for i in 1..=5 {
            if stop_signal.load(Ordering::SeqCst) { return Ok(()); }
            tap_button(&controller, Button::L, &format!("L Tap {}", i))?;
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
        std::thread::sleep(std::time::Duration::from_millis(500));

        // まず左上に移動（左スティック使用）
        info!("Moving to top-left corner...");
        let move_home_cmd = ControllerCommand::new("Move Home")
            .add_action(ControllerAction::move_left_stick(StickPosition::new(0, 0), 5000))
            .add_action(ControllerAction::move_left_stick(StickPosition::CENTER, 100));
        controller.execute_command(&move_home_cmd)?;
        std::thread::sleep(std::time::Duration::from_millis(500));

        // キャンバス中央付近に移動（D-padで確実に移動）
        // Switchキャンバス: 320x180ピクセル
        // テストパターン: 5行×20ドット
        // 中央に配置するため、左上から (150, 85) の位置に移動
        info!("Moving to center position for calibration test...");
        let center_x = 150;
        let center_y = 85;

        // 右に150ピクセル移動（速めのパラメータで高速化）
        for _ in 0..center_x {
            if stop_signal.load(Ordering::SeqCst) { return Ok(()); }
            tap_dpad_with_duration(&controller, DPad::RIGHT, "Move Right", 30, 15, 5)?;
        }

        // 下に85ピクセル移動
        for _ in 0..center_y {
            if stop_signal.load(Ordering::SeqCst) { return Ok(()); }
            tap_dpad_with_duration(&controller, DPad::DOWN, "Move Down", 30, 15, 5)?;
        }

        info!("Calibration test position reached: ({}, {})", center_x, center_y);
        std::thread::sleep(std::time::Duration::from_millis(500));
    } else {
        info!("Skipping initialization (pen size, home position, center position)");
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    // 初期化完了後、確実にNEUTRAL状態にリセット
    tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Reset after initialization", 50, 50, 0)?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    // 5行のテスト（各行異なるパターン、ビーストロフェドン方式）
    // 行1: 1px描画+1px空白 (●_●_●_●_...) 左→右
    // 行2: 2px描画+2px空白 (●●__●●__...) 右→左
    // 行3: 3px描画+3px空白 (●●●___●●●___...) 左→右
    // 行4: 4px描画+4px空白 (●●●●____...) 右→左
    // 行5: 5px描画+5px空白 (●●●●●_____...) 左→右
    let rows = 5;
    let total_width = 20; // 各行の幅（ピクセル数）

    for row_idx in 0..rows {
        if stop_signal.load(Ordering::SeqCst) {
            info!("Calibration stopped by user");
            // 停止時も必ずNEUTRAL状態にリセット
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            return Ok(());
        }

        let pattern_size = row_idx + 1; // 1px, 2px, 3px, 4px, 5px

        // ビーストロフェドン方式: 偶数行は左→右、奇数行は右→左
        let is_left_to_right = row_idx % 2 == 0;
        let direction = if is_left_to_right { DPad::RIGHT } else { DPad::LEFT };
        let direction_name = if is_left_to_right { "LEFT→RIGHT" } else { "RIGHT←LEFT" };

        info!("Testing row {}/{} ({}px draw + {}px gap pattern, {})...",
              row_idx + 1, rows, pattern_size, pattern_size, direction_name);

        let mut dots_drawn = 0;
        let mut position = 0;

        // パターンを繰り返し描画
        while position < total_width {
            if stop_signal.load(Ordering::SeqCst) {
                // 停止時も必ずNEUTRAL状態にリセット
                tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
                std::thread::sleep(std::time::Duration::from_millis(200));
                return Ok(());
            }

            // N個のドットを描画
            for _ in 0..pattern_size {
                if position >= total_width {
                    break;
                }

                // D-pad状態を完全にクリア（描画前）
                tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad Before Paint", 10, 10, 0)?;

                // ドットを打つ
                tap_button_with_duration(&controller, Button::A, "Paint Dot", press_ms, release_ms, wait_ms as u64)?;
                dots_drawn += 1;
                position += 1;

                // D-pad状態を完全にクリア（移動前）
                tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad Before Move", 10, 10, 0)?;

                // 描画方向に移動（行末でない限り）
                if position < total_width {
                    tap_dpad_with_duration(&controller, direction, "Move", press_ms, release_ms, wait_ms as u64)?;
                }
            }

            // N個分空白（移動のみ）
            for _ in 0..pattern_size {
                if position >= total_width {
                    break;
                }

                position += 1;

                // D-pad状態をクリア
                tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", 10, 10, 0)?;

                // 描画方向に移動（行末でない限り）
                if position < total_width {
                    tap_dpad_with_duration(&controller, direction, "Move", press_ms, release_ms, wait_ms as u64)?;
                }
            }
        }

        info!("Row {} complete: {} dots drawn in {}px draw/{}px gap pattern ({})",
              row_idx + 1, dots_drawn, pattern_size, pattern_size, direction_name);

        // 次の行に移動（ビーストロフェドン方式: 下に2ピクセル移動するだけ、左端には戻らない）
        if row_idx < rows - 1 {
            // D-pad状態をクリア（NEUTRAL状態を送信）
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", press_ms, release_ms, wait_ms as u64)?;
            std::thread::sleep(std::time::Duration::from_millis(100));

            // 下に2ピクセル移動（行間を空ける）
            // ユーザー指定のパラメータを使用
            info!("Moving down 2 pixels for next row (boustrophedon pattern)");
            for _ in 0..2 {
                tap_dpad_with_duration(&controller, DPad::DOWN, "Move Down", press_ms, release_ms, wait_ms as u64)?;
            }

            // D-pad状態をクリア（次の行の開始前）
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", press_ms, release_ms, wait_ms as u64)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    // テスト完了後、確実にNEUTRAL状態にリセット
    tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset", 100, 100, 0)?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    info!("Speed calibration test completed!");
    info!("Check the screen: If dots are aligned correctly, this speed is safe.");
    Ok(())
}

/// 描画移動テスト（Aボタン押しながら右移動）
fn test_paint_move(
    controller: Arc<dyn ControllerEmulator>,
    stop_signal: Arc<AtomicBool>,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u32,
) -> Result<(), HardwareError> {
    info!("Starting paint move test (A button + RIGHT)");

    // 10回描画移動
    for i in 0..10 {
        if stop_signal.load(Ordering::SeqCst) {
            // 停止時も必ずNEUTRAL状態にリセット
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            return Ok(());
        }

        info!("Paint move {}/10", i + 1);

        // D-pad状態をクリア
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", 10, 10, 0)?;

        // ドットを打つ
        tap_button_with_duration(&controller, Button::A, "Paint Dot", press_ms, release_ms, wait_ms as u64)?;

        // D-pad状態をクリア
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", 10, 10, 0)?;

        // 右に移動
        tap_dpad_with_duration(&controller, DPad::RIGHT, "Move Right", press_ms, release_ms, wait_ms as u64)?;
    }

    // テスト完了後、確実にNEUTRAL状態にリセット
    tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset", 100, 100, 0)?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    info!("Paint move test completed");
    Ok(())
}

/// 空白移動テスト（Aボタンなしで右移動）
fn test_gap_move(
    controller: Arc<dyn ControllerEmulator>,
    stop_signal: Arc<AtomicBool>,
    press_ms: u32,
    release_ms: u32,
    wait_ms: u32,
) -> Result<(), HardwareError> {
    info!("Starting gap move test (RIGHT only, no A button)");

    // 10回空白移動
    for i in 0..10 {
        if stop_signal.load(Ordering::SeqCst) {
            // 停止時も必ずNEUTRAL状態にリセット
            tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset on Stop", 100, 100, 0)?;
            std::thread::sleep(std::time::Duration::from_millis(200));
            return Ok(());
        }

        info!("Gap move {}/10", i + 1);

        // D-pad状態をクリア
        tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Clear DPad", 10, 10, 0)?;

        // 右に移動（Aボタンなし）
        tap_dpad_with_duration(&controller, DPad::RIGHT, "Move Right", press_ms, release_ms, wait_ms as u64)?;
    }

    // テスト完了後、確実にNEUTRAL状態にリセット
    tap_dpad_with_duration(&controller, DPad::NEUTRAL, "Final Reset", 100, 100, 0)?;
    std::thread::sleep(std::time::Duration::from_millis(200));

    info!("Gap move test completed");
    Ok(())
}

/// 速度キャリブレーションテストを開始するAPIハンドラー
pub async fn start_calibration(
    State(state): State<Arc<ArtworkState>>,
    Json(request): Json<super::models::CalibrationRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Starting speed calibration test with params: press={}ms, release={}ms, wait={}ms, skip_init={}",
          request.press_ms, request.release_ms, request.wait_ms, request.skip_initialization);

    let controller = state.controller.clone();
    let press_ms = request.press_ms;
    let release_ms = request.release_ms;
    let wait_ms = request.wait_ms;
    let skip_initialization = request.skip_initialization;

    // Setup control signals
    let control = PaintingControl::new();
    let stop_signal = control.stop_signal.clone();

    // Store active painting control
    {
        let mut active = state.active_painting.write().await;
        *active = Some(control);
    }

    let active_painting_store = state.active_painting.clone();

    // Spawn calibration task
    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            perform_speed_calibration(controller, stop_signal, press_ms, release_ms, wait_ms, skip_initialization)
        }).await;

        // Clear active painting when done
        {
            let mut active = active_painting_store.write().await;
            *active = None;
        }

        // Send completion status through PROGRESS_CHANNEL for frontend notification
        use crate::interfaces::web::log_streamer::PROGRESS_CHANNEL;
        use serde_json::json;
        use chrono::Utc;

        match result {
            Ok(Ok(_)) => {
                info!("Calibration completed successfully");
                // Send calibration completion event
                let completion_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "success",
                    "message": "キャリブレーションテストが完了しました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(completion_msg);
            },
            Ok(Err(e)) => {
                error!("Calibration failed with hardware error: {}", e);
                // Send calibration failure event
                let failure_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "error",
                    "message": format!("キャリブレーションテストが失敗しました: {}", e)
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(failure_msg);
            },
            Err(e) => {
                error!("Calibration task panicked or was cancelled: {}", e);
                // Send calibration cancellation event
                let cancel_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "cancelled",
                    "message": "キャリブレーションテストが中断されました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(cancel_msg);
            }
        }
    });

    Ok(Json(ApiResponse {
        success: true,
        message: "Speed calibration test started".to_string(),
    }))
}

/// 描画移動テストを開始するAPIハンドラー
pub async fn start_paint_move_test(
    State(state): State<Arc<ArtworkState>>,
    Json(request): Json<super::models::CalibrationRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Starting paint move test");

    let controller = state.controller.clone();
    let press_ms = request.press_ms;
    let release_ms = request.release_ms;
    let wait_ms = request.wait_ms;

    let control = PaintingControl::new();
    let stop_signal = control.stop_signal.clone();

    {
        let mut active = state.active_painting.write().await;
        *active = Some(control);
    }

    let active_painting_store = state.active_painting.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            test_paint_move(controller, stop_signal, press_ms, release_ms, wait_ms)
        }).await;

        {
            let mut active = active_painting_store.write().await;
            *active = None;
        }

        use crate::interfaces::web::log_streamer::PROGRESS_CHANNEL;
        use serde_json::json;
        use chrono::Utc;

        match result {
            Ok(Ok(_)) => {
                let completion_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "success",
                    "message": "描画移動テストが完了しました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(completion_msg);
            },
            _ => {
                let error_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "error",
                    "message": "描画移動テストが失敗しました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(error_msg);
            }
        }
    });

    Ok(Json(ApiResponse {
        success: true,
        message: "Paint move test started".to_string(),
    }))
}

/// 空白移動テストを開始するAPIハンドラー
pub async fn start_gap_move_test(
    State(state): State<Arc<ArtworkState>>,
    Json(request): Json<super::models::CalibrationRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    info!("Starting gap move test");

    let controller = state.controller.clone();
    let press_ms = request.press_ms;
    let release_ms = request.release_ms;
    let wait_ms = request.wait_ms;

    let control = PaintingControl::new();
    let stop_signal = control.stop_signal.clone();

    {
        let mut active = state.active_painting.write().await;
        *active = Some(control);
    }

    let active_painting_store = state.active_painting.clone();

    tokio::spawn(async move {
        let result = tokio::task::spawn_blocking(move || {
            test_gap_move(controller, stop_signal, press_ms, release_ms, wait_ms)
        }).await;

        {
            let mut active = active_painting_store.write().await;
            *active = None;
        }

        use crate::interfaces::web::log_streamer::PROGRESS_CHANNEL;
        use serde_json::json;
        use chrono::Utc;

        match result {
            Ok(Ok(_)) => {
                let completion_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "success",
                    "message": "空白移動テストが完了しました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(completion_msg);
            },
            _ => {
                let error_msg = json!({
                    "type": "calibration_complete",
                    "timestamp": Utc::now().to_rfc3339(),
                    "status": "error",
                    "message": "空白移動テストが失敗しました"
                }).to_string();
                let _ = PROGRESS_CHANNEL.send(error_msg);
            }
        }
    });

    Ok(Json(ApiResponse {
        success: true,
        message: "Gap move test started".to_string(),
    }))
}

/// Upload artwork image
pub async fn upload_artwork(
    State(state): State<Arc<ArtworkState>>,
    mut multipart: Multipart,
) -> Result<Json<ArtworkResponse>, StatusCode> {
    let mut name = String::new();
    let mut image_data = Vec::new();

    // Process multipart form
    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "name" => {
                name = field.text().await.unwrap_or_default();
            }
            "file" => {
                image_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    if name.is_empty() || image_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    info!("Uploading artwork: {} ({} bytes)", name, image_data.len());

    // Create simple canvas (TODO: implement actual image processing)
    let canvas = Canvas::new(320, 180);

    // Create metadata
    let metadata =
        ArtworkMetadata::new(name.clone()).with_description("Uploaded image".to_string());

    // Create artwork
    let artwork = Artwork::new(metadata, "png".to_string(), canvas);
    let artwork_id = artwork.id.as_str().to_string();

    // Store artwork
    state
        .artworks
        .write()
        .await
        .insert(artwork_id.clone(), artwork);

    Ok(Json(ArtworkResponse {
        id: artwork_id,
        message: format!("Image '{name}' uploaded successfully"),
        artwork: None,
    }))
}

// Helper function to parse color from string
fn parse_color(color_str: &str) -> Option<Color> {
    if color_str.starts_with('#') && color_str.len() == 7 {
        let r = u8::from_str_radix(&color_str[1..3], 16).ok()?;
        let g = u8::from_str_radix(&color_str[3..5], 16).ok()?;
        let b = u8::from_str_radix(&color_str[5..7], 16).ok()?;
        Some(Color::new(r, g, b, 255))
    } else {
        None
    }
}
