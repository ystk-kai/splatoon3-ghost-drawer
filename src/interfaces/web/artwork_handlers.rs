use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

// Import domain entities
use super::error_response::ErrorResponse;
use crate::domain::artwork::entities::{Artwork, ArtworkMetadata, Canvas, Dot};
use crate::domain::shared::value_objects::{Color, Coordinates};

#[derive(Debug, Clone)]
pub struct ArtworkState {
    pub artworks: Arc<RwLock<HashMap<String, Artwork>>>,
}

impl ArtworkState {
    pub fn new() -> Self {
        Self {
            artworks: Arc::new(RwLock::new(HashMap::new())),
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
    pub speed: Option<f32>,
    pub preview: Option<bool>,
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

/// Paint an artwork
pub async fn paint_artwork(
    State(state): State<Arc<ArtworkState>>,
    Path(id): Path<String>,
    Json(request): Json<PaintRequest>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let artworks = state.artworks.read().await;

    match artworks.get(&id) {
        Some(artwork) => {
            let speed = request.speed.unwrap_or(2.0);
            let preview = request.preview.unwrap_or(false);

            info!(
                "Starting painting for artwork {} (speed: {}, preview: {})",
                id, speed, preview
            );

            // TODO: Implement actual painting logic
            let estimated_time = artwork.estimated_painting_time(speed as f64);

            Ok(Json(ApiResponse {
                success: true,
                message: format!("Painting started (estimated time: {estimated_time} seconds)"),
            }))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
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
