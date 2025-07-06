use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BoardModel {
    OrangePiZero2W,
    RaspberryPiZero,
    RaspberryPiZero2W,
    Unknown(String),
}

impl BoardModel {
    pub fn otg_device_tree_overlay(&self) -> Option<&'static str> {
        match self {
            BoardModel::OrangePiZero2W => Some("sun50i-h616-usb-otg"),
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => None,
            BoardModel::Unknown(_) => None,
        }
    }

    pub fn requires_config_txt(&self) -> bool {
        matches!(
            self,
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W
        )
    }

    pub fn usb_device_path(&self) -> &'static str {
        match self {
            BoardModel::OrangePiZero2W => "/sys/kernel/config/usb_gadget/g1",
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => {
                "/sys/kernel/config/usb_gadget/g1"
            }
            BoardModel::Unknown(_) => "/sys/kernel/config/usb_gadget/g1",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemSetupStatus {
    pub boot_configured: bool,
    pub systemd_service_enabled: bool,
    pub usb_gadget_configured: bool,
}