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

        // Setup application files
        info!("Setting up application files...");
        self.systemd_manager.setup_application_files()?;
        info!("Application files setup completed.");

        // Create and enable web UI service
        info!("Creating web UI systemd service...");
        self.systemd_manager.create_web_service()?;

        info!("Enabling web UI systemd service...");
        self.systemd_manager.enable_web_service()?;
        info!("Web UI systemd service enabled.");

        // Try to start services immediately for testing
        info!("Attempting to start services for immediate testing...");
        if let Err(e) = self.try_start_services() {
            info!(
                "Could not start services immediately (this is normal): {}",
                e
            );
            info!("Services will start automatically after reboot.");
        } else {
            info!("Services started successfully! You can test the system without rebooting.");
        }

        info!("System setup completed successfully!");
        info!("For full functionality, please reboot the device: sudo reboot");

        Ok(())
    }

    fn try_start_services(&self) -> Result<(), SetupError> {
        // Try to start the gadget service
        let output = std::process::Command::new("systemctl")
            .arg("start")
            .arg("splatoon3-gadget.service")
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to start gadget service: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::Unknown(format!(
                "Gadget service failed to start: {stderr}"
            )));
        }

        // Try to start the web service
        let output = std::process::Command::new("systemctl")
            .arg("start")
            .arg("splatoon3-ghost-drawer.service")
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to start web service: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::Unknown(format!(
                "Web service failed to start: {stderr}"
            )));
        }

        info!("Both services started successfully!");
        Ok(())
    }
}

fn is_running_as_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
