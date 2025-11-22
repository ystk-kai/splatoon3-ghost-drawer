use axum::extract::ws::{Message, WebSocket};
use chrono::Utc;
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{info, warn};

// Global log channel
lazy_static::lazy_static! {
    pub static ref LOG_CHANNEL: broadcast::Sender<String> = {
        let (tx, _) = broadcast::channel(100);
        tx
    };
    
    pub static ref PROGRESS_CHANNEL: broadcast::Sender<String> = {
        let (tx, _) = broadcast::channel(100);
        tx
    };
}

/// Custom tracing subscriber layer to capture logs
pub struct LogCaptureLayer;

impl<S> tracing_subscriber::Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let mut visitor = JsonVisitor::new();
        event.record(&mut visitor);

        let level = event.metadata().level().as_str();
        let target = event.metadata().target();
        
        // Format message
        let log_entry = json!({
            "type": "log",
            "timestamp": Utc::now().to_rfc3339(),
            "level": level,
            "message": visitor.message,
            "target": target,
        })
        .to_string();

        // Send to channel (ignore errors if no receivers)
        let _ = LOG_CHANNEL.send(log_entry);
    }
}

struct JsonVisitor {
    message: String,
}

impl JsonVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
        }
    }
}

impl tracing::field::Visit for JsonVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        }
    }
    
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        }
    }
}

/// Stream logs to WebSocket connection
pub async fn stream_logs(mut socket: WebSocket) {
    info!("Starting log streaming");

    // Subscribe to channels
    let mut log_rx = LOG_CHANNEL.subscribe();
    let mut progress_rx = PROGRESS_CHANNEL.subscribe();

    // Send connection established message
    let connect_msg = json!({
        "type": "log",
        "timestamp": Utc::now().to_rfc3339(),
        "level": "INFO",
        "message": "Log streaming connected",
        "target": "log_streamer"
    }).to_string();

    if socket.send(Message::Text(connect_msg.into())).await.is_err() {
        return;
    }

    loop {
        tokio::select! {
            // Receive log from channel
            result = log_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(count)) => {
                        warn!("Log stream lagged by {} messages", count);
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            
            // Receive progress from channel
            result = progress_rx.recv() => {
                match result {
                    Ok(msg) => {
                        if socket.send(Message::Text(msg.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Skip lagged progress messages
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Handle client messages (keepalive/close)
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        info!("Client closed connection");
                        break;
                    }
                    Some(Err(_)) | None => {
                        break;
                    }
                    _ => {} // Ignore other messages
                }
            }
        }
    }

    info!("Log streaming ended");
}
