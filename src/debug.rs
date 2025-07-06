//! デバッグとログ機能
//! 
//! プロジェクト全体のデバッグとログ機能を提供

use std::fs;
use tracing::{info, debug, Level};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;

/// デバッグ設定
#[derive(Debug, Clone)]
pub struct DebugConfig {
    /// ログレベル
    pub log_level: Level,
    /// ファイルログを有効にするか
    pub enable_file_logging: bool,
    /// ログファイルのディレクトリ
    pub log_directory: String,
    /// コンソールログを有効にするか
    pub enable_console_logging: bool,
    /// JSONフォーマットを使用するか
    pub use_json_format: bool,
    /// パフォーマンス測定を有効にするか
    pub enable_performance_tracking: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            log_level: Level::INFO,
            enable_file_logging: true,
            log_directory: "logs".to_string(),
            enable_console_logging: true,
            use_json_format: false,
            enable_performance_tracking: true,
        }
    }
}

impl DebugConfig {
    /// 開発環境用の設定
    pub fn development() -> Self {
        Self {
            log_level: Level::DEBUG,
            enable_file_logging: true,
            log_directory: "logs".to_string(),
            enable_console_logging: true,
            use_json_format: false,
            enable_performance_tracking: true,
        }
    }

    /// 本番環境用の設定
    pub fn production() -> Self {
        Self {
            log_level: Level::INFO,
            enable_file_logging: true,
            log_directory: "/var/log/splatoon3-ghost-drawer".to_string(),
            enable_console_logging: false,
            use_json_format: true,
            enable_performance_tracking: false,
        }
    }

    /// テスト環境用の設定
    pub fn test() -> Self {
        Self {
            log_level: Level::WARN,
            enable_file_logging: false,
            log_directory: "test_logs".to_string(),
            enable_console_logging: true,
            use_json_format: false,
            enable_performance_tracking: false,
        }
    }
}

/// ログシステムを初期化
pub fn init_logging(config: &DebugConfig) -> Result<(), Box<dyn std::error::Error>> {
    // ログディレクトリを作成
    if config.enable_file_logging {
        fs::create_dir_all(&config.log_directory)?;
    }

    // 環境変数からのフィルター設定
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&format!("splatoon3_ghost_drawer={}", config.log_level)))
        .unwrap();

    // シンプルな設定でサブスクライバーを初期化
    if config.enable_file_logging {
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            &config.log_directory,
            "splatoon3-ghost-drawer.log",
        );
        
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(file_appender)
            .json()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .pretty()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .init();
    }

    info!("ログシステムが初期化されました");
    debug!("デバッグ設定: {:?}", config);

    Ok(())
}

/// パフォーマンス測定用のマクロ
#[macro_export]
macro_rules! measure_time {
    ($name:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        tracing::info!(
            operation = $name,
            duration_ms = duration.as_millis(),
            "操作完了"
        );
        result
    }};
}

    /// デバッグ用のヘルパー関数
    pub mod debug_helpers {
        use tracing::{info, error, debug};
    use std::collections::HashMap;

    /// システム情報をログに出力
    pub fn log_system_info() {
        info!("=== システム情報 ===");
        info!("OS: {}", std::env::consts::OS);
        info!("アーキテクチャ: {}", std::env::consts::ARCH);
        info!("Rustバージョン: {}", env!("CARGO_PKG_RUST_VERSION"));
        info!("プロジェクトバージョン: {}", env!("CARGO_PKG_VERSION"));
        
        // メモリ使用量（利用可能な場合）
        if let Ok(memory) = get_memory_usage() {
            info!("メモリ使用量: {} MB", memory / 1024 / 1024);
        }
    }

    /// メモリ使用量を取得（Linux専用）
    fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error>> {
        #[cfg(target_os = "linux")]
        {
            let status = std::fs::read_to_string("/proc/self/status")?;
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let kb: u64 = parts[1].parse()?;
                        return Ok(kb * 1024);
                    }
                }
            }
        }
        Err("メモリ使用量を取得できませんでした".into())
    }

    /// エラーの詳細情報をログに出力
    pub fn log_error_details(error: &dyn std::error::Error, context: &str) {
        error!(
            context = context,
            error = %error,
            "エラーが発生しました"
        );

        // エラーチェーンをログに出力
        let mut source = error.source();
        let mut level = 1;
        while let Some(err) = source {
            error!(
                context = context,
                level = level,
                source_error = %err,
                "エラーの原因"
            );
            source = err.source();
            level += 1;
        }
    }

    /// デバッグ用の状態ダンプ
    pub fn dump_state<T: std::fmt::Debug>(name: &str, state: &T) {
        debug!(
            component = name,
            state = ?state,
            "状態ダンプ"
        );
    }

    /// パフォーマンス統計を収集
    pub struct PerformanceStats {
        operations: HashMap<String, Vec<u128>>,
    }

    impl PerformanceStats {
        pub fn new() -> Self {
            Self {
                operations: HashMap::new(),
            }
        }

        pub fn record_operation(&mut self, name: &str, duration_ms: u128) {
            self.operations
                .entry(name.to_string())
                .or_insert_with(Vec::new)
                .push(duration_ms);
        }

        pub fn log_summary(&self) {
            info!("=== パフォーマンス統計 ===");
            for (name, durations) in &self.operations {
                if !durations.is_empty() {
                    let avg = durations.iter().sum::<u128>() / durations.len() as u128;
                    let min = *durations.iter().min().unwrap();
                    let max = *durations.iter().max().unwrap();
                    
                    info!(
                        operation = name,
                        count = durations.len(),
                        avg_ms = avg,
                        min_ms = min,
                        max_ms = max,
                        "操作統計"
                    );
                }
            }
        }
    }

    impl Default for PerformanceStats {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// デバッグ用のテストヘルパー
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    use tracing_test::traced_test;

    /// テスト用のログ初期化
    pub fn init_test_logging() {
        let config = DebugConfig::test();
        let _ = init_logging(&config);
    }

    /// テスト用のパフォーマンス測定
    #[traced_test]
    pub fn test_performance_measurement() {
        let result = measure_time!("test_operation", {
            std::thread::sleep(std::time::Duration::from_millis(100));
            42
        });
        assert_eq!(result, 42);
    }
} 