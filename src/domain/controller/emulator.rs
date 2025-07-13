use super::ControllerCommand;
use crate::domain::hardware::errors::HardwareError;

/// コントローラーエミュレーターのトレイト
pub trait ControllerEmulator: Send + Sync {
    /// エミュレーターを初期化
    fn initialize(&self) -> Result<(), HardwareError>;

    /// Nintendo Switchに接続されているか確認
    fn is_connected(&self) -> Result<bool, HardwareError>;

    /// コントローラーコマンドを実行
    fn execute_command(&self, command: &ControllerCommand) -> Result<(), HardwareError>;

    /// エミュレーターをシャットダウン
    fn shutdown(&self) -> Result<(), HardwareError>;
}
