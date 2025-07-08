use crate::domain::setup::repositories::{
    BoardDetector, BootConfigurator, SetupError, SystemdServiceManager,
};
use std::sync::Arc;
use tracing::info;

pub struct CleanupSystemUseCase {
    board_detector: Arc<dyn BoardDetector>,
    boot_configurator: Arc<dyn BootConfigurator>,
    systemd_manager: Arc<dyn SystemdServiceManager>,
}

impl CleanupSystemUseCase {
    pub fn new(
        board_detector: Arc<dyn BoardDetector>,
        boot_configurator: Arc<dyn BootConfigurator>,
        systemd_manager: Arc<dyn SystemdServiceManager>,
    ) -> Self {
        Self {
            board_detector,
            boot_configurator,
            systemd_manager,
        }
    }

    pub fn execute(&self) -> Result<(), SetupError> {
        info!("Starting system cleanup...");

        // Check if running as root
        if !is_running_as_root() {
            return Err(SetupError::PermissionDenied(
                "This command requires root privileges. Please run with sudo.".to_string(),
            ));
        }

        // Disable and remove systemd services
        info!("Disabling and removing systemd services...");
        self.systemd_manager.disable_and_remove_services()?;
        info!("Systemd services removed.");

        // Detect board model
        info!("Detecting board model...");
        let board = self.board_detector.detect_board()?;
        info!("Detected board: {:?}", board);

        // Remove boot configuration
        info!("Removing boot configuration...");
        self.boot_configurator.remove_boot_configuration(&board)?;
        info!("Boot configuration removed.");

        // Remove USB gadget configuration if exists
        info!("Cleaning up USB gadget configuration...");
        cleanup_usb_gadget()?;

        info!("System cleanup completed successfully!");
        info!("Please reboot the device for changes to take effect.");

        Ok(())
    }
}

fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn cleanup_usb_gadget() -> Result<(), SetupError> {
    use std::process::Command;
    
    // Remove USB gadget configuration if it exists
    let gadget_path = "/sys/kernel/config/usb_gadget/g1";
    
    if std::path::Path::new(gadget_path).exists() {
        // First, unbind the gadget
        let _ = Command::new("sh")
            .arg("-c")
            .arg("echo '' > /sys/kernel/config/usb_gadget/g1/UDC")
            .output();
        
        // Remove configurations
        let _ = Command::new("rm")
            .arg("-rf")
            .arg("/sys/kernel/config/usb_gadget/g1/configs/c.1/hid.usb0")
            .output();
        
        let _ = Command::new("rm")
            .arg("-rf")
            .arg("/sys/kernel/config/usb_gadget/g1/functions/hid.usb0")
            .output();
        
        let _ = Command::new("rmdir")
            .arg("/sys/kernel/config/usb_gadget/g1/configs/c.1/strings/0x409")
            .output();
        
        let _ = Command::new("rmdir")
            .arg("/sys/kernel/config/usb_gadget/g1/configs/c.1")
            .output();
        
        let _ = Command::new("rmdir")
            .arg("/sys/kernel/config/usb_gadget/g1/strings/0x409")
            .output();
        
        let _ = Command::new("rmdir")
            .arg("/sys/kernel/config/usb_gadget/g1")
            .output();
        
        info!("USB gadget configuration removed.");
    }
    
    Ok(())
}