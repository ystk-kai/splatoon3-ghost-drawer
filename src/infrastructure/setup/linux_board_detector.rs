use crate::domain::setup::entities::BoardModel;
use crate::domain::setup::repositories::{BoardDetector, SetupError};
use std::fs;
use tracing::{debug, error};

pub struct LinuxBoardDetector;

impl Default for LinuxBoardDetector {
    fn default() -> Self {
        Self
    }
}

impl LinuxBoardDetector {
    pub fn new() -> Self {
        Self::default()
    }
}

impl BoardDetector for LinuxBoardDetector {
    fn detect_board(&self) -> Result<BoardModel, SetupError> {
        // Try to read /proc/cpuinfo
        let cpuinfo = fs::read_to_string("/proc/cpuinfo").map_err(|e| {
            error!("Failed to read /proc/cpuinfo: {}", e);
            SetupError::BoardDetectionFailed(format!("Cannot read /proc/cpuinfo: {}", e))
        })?;

        debug!("CPU info:\n{}", cpuinfo);

        // Check for specific board identifiers
        if cpuinfo.contains("Allwinner sun50iw9") || cpuinfo.contains("sun50i-h616") {
            return Ok(BoardModel::OrangePiZero2W);
        }

        // Try to read /proc/device-tree/model
        if let Ok(model) = fs::read_to_string("/proc/device-tree/model") {
            let model = model.trim_end_matches('\0'); // Remove null terminator
            debug!("Device tree model: {}", model);

            if model.contains("Raspberry Pi Zero W") {
                return Ok(BoardModel::RaspberryPiZero);
            } else if model.contains("Raspberry Pi Zero 2 W") {
                return Ok(BoardModel::RaspberryPiZero2W);
            } else if model.contains("OrangePi Zero2W") || model.contains("OrangePi Zero 2W") {
                return Ok(BoardModel::OrangePiZero2W);
            }
        }

        // Try additional detection methods
        if let Ok(compatible) = fs::read_to_string("/proc/device-tree/compatible") {
            let compatible = compatible.replace('\0', " ");
            debug!("Device tree compatible: {}", compatible);

            if compatible.contains("raspberrypi") {
                if compatible.contains("zero-w") {
                    return Ok(BoardModel::RaspberryPiZero);
                } else if compatible.contains("zero-2-w") {
                    return Ok(BoardModel::RaspberryPiZero2W);
                }
            } else if compatible.contains("allwinner") && compatible.contains("h616") {
                return Ok(BoardModel::OrangePiZero2W);
            }
        }

        Err(SetupError::BoardDetectionFailed(
            "Could not identify board model".to_string(),
        ))
    }
}
