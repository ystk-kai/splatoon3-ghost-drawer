//! Web インターフェース
//!
//! HTTPベースのWeb APIとWebSocketエンドポイントを提供します。
//! アートワーク管理、システム情報、ハードウェア状態の監視、
//! リアルタイムログストリーミングなどの機能を含みます。

mod artwork_handlers;
mod embedded_assets;
mod error_response;
mod handlers;
mod log_streamer;
mod models;

pub mod server;

// 内部使用のため、必要な型のみを再エクスポート
pub(crate) use artwork_handlers::{
    ArtworkState, create_artwork, delete_artwork, get_artwork, list_artworks, paint_artwork,
    upload_artwork,
};
pub(crate) use handlers::{get_hardware_status, get_system_info, websocket_handler};
