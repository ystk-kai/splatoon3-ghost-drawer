//! セットアップドメイン
//! 
//! システムの初期設定とUSBガジェット構成に関するドメインモデルを提供します。
//! このモジュールはボード検出、ブート設定、systemdサービス管理などの
//! セットアップ関連の概念を表現します。

pub mod entities;
pub mod repositories;

// 主要な型の再エクスポート
pub use entities::{BoardModel, SystemSetupStatus};
pub use repositories::{
    BoardDetector, BootConfigurator, SetupError, SystemSetupRepository, SystemdServiceManager,
};