use thiserror::Error;

#[derive(Error, Debug)]
pub enum HardwareError {
    #[error("Board not supported for USB OTG: {0}")]
    BoardNotSupported(String),

    #[error("USB OTG not available on this board")]
    UsbOtgNotAvailable,

    #[error("Required kernel module not loaded: {0}")]
    KernelModuleNotLoaded(String),

    #[error("Failed to detect board model")]
    BoardDetectionFailed,

    #[error("USB gadget configuration failed: {0}")]
    GadgetConfigurationFailed(String),

    #[error("Systemd service operation failed: {0}")]
    SystemdServiceFailed(String),

    #[error("Permission denied. Root privileges required")]
    PermissionDenied,

    #[error("System command failed: {0}")]
    SystemCommandFailed(String),

    #[error("File operation failed: {0}")]
    FileOperationFailed(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl HardwareError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            HardwareError::KernelModuleNotLoaded(_) | HardwareError::SystemdServiceFailed(_)
        )
    }
}
