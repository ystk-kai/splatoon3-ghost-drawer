use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Controller not found: {0}")]
    ControllerNotFound(String),

    #[error("Controller already connected")]
    AlreadyConnected,

    #[error("Controller not connected")]
    NotConnected,

    #[error("Invalid HID report data")]
    InvalidHidReport,

    #[error("Device path not available: {0}")]
    DevicePathNotAvailable(String),

    #[error("Failed to write HID report: {0}")]
    HidWriteFailed(String),

    #[error("Failed to read HID report: {0}")]
    HidReadFailed(String),

    #[error("Controller session not found: {0}")]
    SessionNotFound(String),

    #[error("Controller session already active")]
    SessionAlreadyActive,

    #[error("Controller session not active")]
    SessionNotActive,

    #[error("Invalid controller command: {0}")]
    InvalidCommand(String),

    #[error("Controller mapping not found: {0}")]
    MappingNotFound(String),

    #[error("Invalid stick position: x={0}, y={1}")]
    InvalidStickPosition(u8, u8),

    #[error("Invalid button value: {0}")]
    InvalidButtonValue(u16),

    #[error("Invalid DPad value: {0}")]
    InvalidDPadValue(u8),

    #[error("Command queue is full")]
    CommandQueueFull,

    #[error("No commands in queue")]
    NoCommandsInQueue,

    #[error("Device initialization failed: {0}")]
    DeviceInitFailed(String),

    #[error("Permission denied. Root privileges may be required")]
    PermissionDenied,

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl ControllerError {
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ControllerError::NotConnected
                | ControllerError::SessionNotActive
                | ControllerError::NoCommandsInQueue
        )
    }
}