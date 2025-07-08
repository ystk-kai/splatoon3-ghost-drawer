use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde_json::json;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info};

/// Stream logs to WebSocket connection
pub async fn stream_logs(mut socket: WebSocket) {
    info!("Starting log streaming");

    // Send connection established message
    let connect_msg = create_log_message(
        "INFO",
        "Log streaming connected",
        "log_streamer",
    );
    
    if socket.send(Message::Text(connect_msg.into())).await.is_err() {
        return;
    }

    // Create interval for periodic log updates
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                // Send periodic status update
                let status_msg = create_log_message(
                    "DEBUG",
                    "Log stream heartbeat",
                    "log_streamer",
                );
                
                if socket.send(Message::Text(status_msg.into())).await.is_err() {
                    break;
                }
            }
            
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        debug!("Received client message: {}", text);
                        
                        // Parse client command if any
                        if text.trim() == "ping" {
                            let pong_msg = create_log_message(
                                "DEBUG",
                                "Pong",
                                "log_streamer",
                            );
                            
                            if socket.send(Message::Text(pong_msg.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("Client closed connection");
                        break;
                    }
                    _ => break,
                }
            }
        }
    }

    info!("Log streaming ended");
}

/// Create a properly formatted log message
fn create_log_message(level: &str, message: &str, target: &str) -> String {
    json!({
        "timestamp": Utc::now().to_rfc3339(),
        "level": level,
        "message": message,
        "target": target,
        "fields": {},
        "span": null
    })
    .to_string()
}

