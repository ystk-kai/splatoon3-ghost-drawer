//! セットアップインフラストラクチャの実装
//! 
//! ドメイン層で定義されたセットアップ関連のトレイトの具体的な実装を提供します。
//! Linux環境でのボード検出、ブート設定、systemdサービス管理の実装が含まれます。

mod linux_board_detector;
mod linux_boot_configurator;
mod linux_systemd_manager;

// 公開APIの再エクスポート
pub use linux_board_detector::LinuxBoardDetector;
pub use linux_boot_configurator::LinuxBootConfigurator;
pub use linux_systemd_manager::LinuxSystemdManager;