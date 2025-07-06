//! # Splatoon3 Ghost Drawer
//!
//! Raspberry Pi 4 を使用して Nintendo Switch の Pro Controller をシミュレートし、
//! Splatoon3 の指定された画像のイラスト投稿を自動化するシステム
//!
//! このクレートは Domain-Driven Design (DDD) 原則に基づいて設計されており、
//! 以下の層に分かれています：
//!
//! - **Domain Layer**: ビジネスロジックとドメインモデル
//! - **Application Layer**: ユースケースとアプリケーションサービス
//! - **Infrastructure Layer**: 外部システムとの統合
//! - **Interface Layer**: ユーザーインターフェース

// Rust 2024 Edition 準拠の構造
pub mod domain;
pub mod debug;
pub mod application;
pub mod infrastructure;
pub mod interfaces;

// 公開API
pub use domain::*;

// エラー型の定義
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// アプリケーション全体の設定
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: String,
    pub debug: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            environment: "development".to_string(),
            debug: true,
        }
    }
} 