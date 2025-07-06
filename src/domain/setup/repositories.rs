use super::entities::{BoardModel, SystemSetupStatus};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SetupError {
    #[error("Failed to detect board model: {0}")]
    BoardDetectionFailed(String),
    
    #[error("Failed to configure boot: {0}")]
    BootConfigurationFailed(String),
    
    #[error("Failed to manage systemd service: {0}")]
    SystemdServiceFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("File system error: {0}")]
    FileSystemError(#[from] std::io::Error),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub trait BoardDetector: Send + Sync {
    fn detect_board(&self) -> Result<BoardModel, SetupError>;
}

pub trait BootConfigurator: Send + Sync {
    fn configure_boot_for_otg(&self, board: &BoardModel) -> Result<(), SetupError>;
    fn is_boot_configured(&self, board: &BoardModel) -> Result<bool, SetupError>;
}

pub trait SystemdServiceManager: Send + Sync {
    fn create_gadget_service(&self) -> Result<(), SetupError>;
    fn enable_gadget_service(&self) -> Result<(), SetupError>;
    fn is_service_enabled(&self) -> Result<bool, SetupError>;
}

pub trait SystemSetupRepository: Send + Sync {
    fn get_setup_status(&self) -> Result<SystemSetupStatus, SetupError>;
}