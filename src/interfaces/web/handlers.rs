use super::artwork_handlers::ArtworkState;
use super::log_streamer::stream_logs;
use super::models::{HardwareDetails, HardwareStatus, SystemInfo};
use axum::{
    Json,
    extract::{State, ws::WebSocketUpgrade},
    response::Response,
};
use std::path::Path;
use std::sync::Arc;

/// Get system information
pub async fn get_system_info() -> Json<SystemInfo> {
    Json(SystemInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        rust_version: "1.85.0".to_string(), // Since CARGO_PKG_RUST_VERSION is not available
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        uptime_seconds: get_system_uptime(),
    })
}

/// Get hardware status
pub async fn get_hardware_status(State(state): State<Arc<ArtworkState>>) -> Json<HardwareStatus> {
    // Use the controller abstraction to check connection status
    // This allows MockController to report "connected" even if physical hardware is missing
    let nintendo_switch_connected = state.controller.is_connected().unwrap_or(false);

    let usb_otg_available = check_usb_otg_availability();
    let hid_device_available = check_hid_device_availability();

    Json(HardwareStatus {
        nintendo_switch_connected,
        usb_otg_available,
        hid_device_available,
        last_check: chrono::Utc::now().to_rfc3339(),
        details: get_hardware_details(),
    })
}

/// WebSocket handler for log streaming
pub async fn websocket_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(stream_logs)
}

// Helper functions

fn get_system_uptime() -> u64 {
    // Try to read system uptime from /proc/uptime
    if let Ok(contents) = std::fs::read_to_string("/proc/uptime")
        && let Some(uptime_str) = contents.split_whitespace().next()
        && let Ok(uptime) = uptime_str.parse::<f64>()
    {
        return uptime as u64;
    }
    0
}

#[allow(dead_code)]
async fn check_nintendo_switch_connection() -> bool {
    use tracing::debug;

    // Check if HID device is available and if gadget is active
    if !check_hid_device_availability() {
        debug!("HID device not available");
        return false;
    }

    // Check USB Gadget state - UDC should contain the USB controller name when connected
    let gadget_udc_path = "/sys/kernel/config/usb_gadget/nintendo_controller/UDC";
    match std::fs::read_to_string(gadget_udc_path) {
        Ok(udc_content) => {
            let udc_trimmed = udc_content.trim();
            debug!("UDC content: '{}'", udc_trimmed);

            // UDCが空でない場合は接続中
            if !udc_trimmed.is_empty() {
                // 追加チェック: HIDデバイスの状態を確認
                if let Ok(file) = std::fs::OpenOptions::new().write(true).open("/dev/hidg0") {
                    drop(file); // ファイルをすぐに閉じる

                    // USB gadgetの状態を確認
                    let state_path = "/sys/kernel/config/usb_gadget/nintendo_controller/state";
                    if let Ok(state) = std::fs::read_to_string(state_path) {
                        debug!("USB gadget state: {}", state.trim());
                    }

                    return true;
                }
            }
        }
        Err(e) => {
            debug!("Failed to read UDC file: {}", e);
        }
    }

    false
}

fn check_usb_otg_availability() -> bool {
    // Check if USB OTG is configured
    Path::new("/sys/kernel/config/usb_gadget").exists()
}

fn check_hid_device_availability() -> bool {
    // Check if HID device exists
    let hid_path = "/dev/hidg0";
    Path::new(hid_path).exists()
}

fn get_hardware_details() -> HardwareDetails {
    let mut details = HardwareDetails {
        board_model: None,
        usb_gadget_configured: false,
        hid_device_path: None,
        kernel_modules_loaded: Vec::new(),
    };

    // Get board model
    if let Ok(model) = std::fs::read_to_string("/proc/device-tree/model") {
        details.board_model = Some(model.trim_end_matches('\0').to_string());
    }

    // Check USB gadget configuration
    details.usb_gadget_configured =
        Path::new("/sys/kernel/config/usb_gadget/nintendo_controller").exists();

    // Check HID device
    if Path::new("/dev/hidg0").exists() {
        details.hid_device_path = Some("/dev/hidg0".to_string());
    }

    // Check loaded kernel modules
    if let Ok(modules) = std::fs::read_to_string("/proc/modules") {
        for line in modules.lines() {
            if let Some(module_name) = line.split_whitespace().next()
                && (module_name.contains("libcomposite") || module_name.contains("dwc2"))
            {
                details.kernel_modules_loaded.push(module_name.to_string());
            }
        }
    }

    details
}
