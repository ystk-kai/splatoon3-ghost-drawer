use super::{
    ArtworkState, create_artwork, delete_artwork, get_artwork, get_hardware_status,
    get_system_info, list_artworks, paint_artwork, upload_artwork, websocket_handler,
};
use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, services::ServeDir};
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
        // Serve static files from web directory as fallback
        .fallback_service(ServeDir::new("web").append_index_html_on_directories(true));

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
