use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::sync::Arc;
use tracing::info;

pub struct ConfigureUsbGadgetUseCase {
    usb_gadget_manager: Arc<dyn UsbGadgetManager>,
}

impl ConfigureUsbGadgetUseCase {
    pub fn new(usb_gadget_manager: Arc<dyn UsbGadgetManager>) -> Self {
        Self { usb_gadget_manager }
    }

    pub fn execute(&self) -> Result<(), SetupError> {
        info!("Configuring USB Gadget as Nintendo Switch Pro Controller...");

        // Check if running as root
        if !is_running_as_root() {
            return Err(SetupError::PermissionDenied(
                "USB Gadget configuration requires root privileges.".to_string(),
            ));
        }

        // Check if already configured
        if self.usb_gadget_manager.is_gadget_configured()? {
            info!("USB Gadget is already configured.");
            return Ok(());
        }

        // Configure USB Gadget
        self.usb_gadget_manager.configure_as_pro_controller()?;

        info!("USB Gadget configured successfully!");
        Ok(())
    }
}

fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}