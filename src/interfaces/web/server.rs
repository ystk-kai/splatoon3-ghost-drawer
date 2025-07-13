use super::{
    ArtworkState, create_artwork, delete_artwork, embedded_assets::WebAssets, get_artwork,
    get_hardware_status, get_system_info, list_artworks, paint_artwork, upload_artwork,
    websocket_handler,
};
use axum::{
    Router,
    body::Body,
    extract::DefaultBodyLimit,
    http::{StatusCode, Uri, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tracing::info;

pub async fn create_server(host: String, port: u16) -> anyhow::Result<()> {
    info!("Starting Splatoon3 Ghost Drawer web server...");

    // Parse socket address
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    // Create shared application state
    let app_state = Arc::new(ArtworkState::new());

    // Create the application router with all endpoints
    let app = Router::new()
        // API endpoints
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/system/info", get(get_system_info))
        .route("/api/hardware/status", get(get_hardware_status))
        // Artwork endpoints
        .route("/api/artworks", get(list_artworks).post(create_artwork))
        .route("/api/artworks/upload", post(upload_artwork))
        .route(
            "/api/artworks/{id}",
            get(get_artwork).delete(delete_artwork),
        )
        .route("/api/artworks/{id}/paint", post(paint_artwork))
        // WebSocket endpoint
        .route("/ws/logs", get(websocket_handler))
        // Add state
        .with_state(app_state)
        // Add CORS support and body size limit
        .layer(
            ServiceBuilder::new()
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
                .layer(CorsLayer::permissive()),
        )
        // Serve embedded static files as fallback
        .fallback(static_handler);

    // Create TCP listener
    let listener = TcpListener::bind(&addr).await?;

    println!("🌐 Web server started successfully!");
    println!("   URL: http://{}", addr);
    println!("   Press Ctrl+C to stop");

    // Run the server
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

/// 埋め込まれた静的ファイルを提供するハンドラ
async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    // ルートパスの場合はindex.htmlを提供
    let path = if path.is_empty() || path == "/" {
        "index.html"
    } else {
        path
    };

    // ファイルを取得
    match WebAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();

            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.to_vec()))
                .unwrap()
        }
        None => {
            // ファイルが見つからない場合はindex.htmlを返す（SPAのため）
            if let Some(content) = WebAssets::get("index.html") {
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/html")
                    .body(Body::from(content.data.to_vec()))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap()
            }
        }
    }
}
