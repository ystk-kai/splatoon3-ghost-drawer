use crate::domain::setup::repositories::{
    BoardDetector, BootConfigurator, SetupError, SystemdServiceManager,
};
use std::sync::Arc;
use tracing::info;

pub struct SetupSystemUseCase {
    board_detector: Arc<dyn BoardDetector>,
    boot_configurator: Arc<dyn BootConfigurator>,
    systemd_manager: Arc<dyn SystemdServiceManager>,
}

impl SetupSystemUseCase {
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

    pub fn execute(&self, force: bool) -> Result<(), SetupError> {
        info!("Starting system setup...");

        // Check if running as root
        if !is_running_as_root() {
            return Err(SetupError::PermissionDenied(
                "This command requires root privileges. Please run with sudo.".to_string(),
            ));
        }

        // Detect board model
        info!("Detecting board model...");
        let board = self.board_detector.detect_board()?;
        info!("Detected board: {:?}", board);

        // Check if already configured
        if !force && self.boot_configurator.is_boot_configured(&board)? {
            info!("Boot configuration already set up. Use --force to reconfigure.");
        } else {
            // Configure boot settings for USB OTG
            info!("Configuring boot settings for USB OTG...");
            self.boot_configurator.configure_boot_for_otg(&board)?;
            info!("Boot configuration completed.");
        }

        // Check if systemd service exists and is enabled
        if !force && self.systemd_manager.is_service_enabled()? {
            info!("Systemd gadget service already enabled. Use --force to recreate.");
        } else {
            // Create gadget systemd service
            info!("Creating gadget systemd service...");
            self.systemd_manager.create_gadget_service()?;

            // Enable gadget systemd service
            info!("Enabling gadget systemd service...");
            self.systemd_manager.enable_gadget_service()?;
            info!("Gadget systemd service enabled.");
        }

        // Create and enable web UI service
        info!("Creating web UI systemd service...");
        self.systemd_manager.create_web_service()?;
        
        info!("Enabling web UI systemd service...");
        self.systemd_manager.enable_web_service()?;
        info!("Web UI systemd service enabled.");

        info!("System setup completed successfully!");
        info!("Please reboot the device for changes to take effect.");

        Ok(())
    }
}

fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}