use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoardModel {
    OrangePiZero2W,
    RaspberryPiZero2W,
    RaspberryPiZeroW,
    Unknown,
}

impl BoardModel {
    pub fn from_cpu_info(model_info: &str, hardware_info: &str) -> Self {
        if model_info.contains("Orange Pi Zero 2W") || hardware_info.contains("sun50iw9") {
            BoardModel::OrangePiZero2W
        } else if model_info.contains("Raspberry Pi Zero 2") {
            BoardModel::RaspberryPiZero2W
        } else if model_info.contains("Raspberry Pi Zero") {
            BoardModel::RaspberryPiZeroW
        } else {
            BoardModel::Unknown
        }
    }

    pub fn supports_usb_otg(&self) -> bool {
        matches!(
            self,
            BoardModel::OrangePiZero2W
                | BoardModel::RaspberryPiZero2W
                | BoardModel::RaspberryPiZeroW
        )
    }

    pub fn config_file_path(&self) -> Option<&'static str> {
        match self {
            BoardModel::OrangePiZero2W => Some("/boot/armbianEnv.txt"),
            BoardModel::RaspberryPiZero2W | BoardModel::RaspberryPiZeroW => {
                Some("/boot/config.txt")
            }
            BoardModel::Unknown => None,
        }
    }

    pub fn required_dtoverlay(&self) -> Option<&'static str> {
        match self {
            BoardModel::OrangePiZero2W => Some("param_dwc2_dr_mode=otg"),
            BoardModel::RaspberryPiZero2W | BoardModel::RaspberryPiZeroW => Some("dtoverlay=dwc2"),
            BoardModel::Unknown => None,
        }
    }
}

impl fmt::Display for BoardModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BoardModel::OrangePiZero2W => write!(f, "Orange Pi Zero 2W"),
            BoardModel::RaspberryPiZero2W => write!(f, "Raspberry Pi Zero 2W"),
            BoardModel::RaspberryPiZeroW => write!(f, "Raspberry Pi Zero W"),
            BoardModel::Unknown => write!(f, "Unknown Board"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UsbGadgetState {
    NotConfigured,
    Configured,
    Active,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UsbDeviceDescriptor {
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_version: u16,
    pub usb_version: u16,
    pub manufacturer: String,
    pub product: String,
    pub serial_number: String,
}

impl Default for UsbDeviceDescriptor {
    fn default() -> Self {
        Self {
            vendor_id: 0x057e,      // Nintendo
            product_id: 0x2009,     // Pro Controller
            device_version: 0x0100, // v1.0.0
            usb_version: 0x0200,    // USB 2.0
            manufacturer: "Nintendo Co., Ltd.".to_string(),
            product: "Pro Controller".to_string(),
            serial_number: "000000000001".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SystemdServiceState {
    NotInstalled,
    Installed,
    Enabled,
    Running,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KernelModule {
    pub name: String,
    pub loaded: bool,
}

impl KernelModule {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            loaded: false,
        }
    }

    pub fn dwc2() -> Self {
        Self::new("dwc2")
    }

    pub fn libcomposite() -> Self {
        Self::new("libcomposite")
    }
}
