use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: String,
    pub rust_version: String,
    pub os: String,
    pub arch: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub nintendo_switch_connected: bool,
    pub usb_otg_available: bool,
    pub hid_device_available: bool,
    pub last_check: String,
    pub details: HardwareDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDetails {
    pub board_model: Option<String>,
    pub usb_gadget_configured: bool,
    pub hid_device_path: Option<String>,
    pub kernel_modules_loaded: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationRequest {
    pub press_ms: u32,
    pub release_ms: u32,
    pub wait_ms: u32,
    #[serde(default)]
    pub skip_initialization: bool,
}

impl Default for CalibrationRequest {
    fn default() -> Self {
        Self {
            press_ms: 50,
            release_ms: 30,
            wait_ms: 20,
            skip_initialization: false,
        }
    }
}


