use crate::domain::setup::repositories::{SetupError, SystemdServiceManager};
use std::fs;
use std::io::Write;
use std::process::Command;
use tracing::{debug, info};

const SERVICE_NAME: &str = "splatoon3-gadget";
const SERVICE_FILE: &str = "/etc/systemd/system/splatoon3-gadget.service";

pub struct LinuxSystemdManager;

impl Default for LinuxSystemdManager {
    fn default() -> Self {
        Self
    }
}

impl LinuxSystemdManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_executable_path() -> Result<String, SetupError> {
        // Get the path of the current executable
        std::env::current_exe()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to get executable path: {}", e)))
    }
}

impl SystemdServiceManager for LinuxSystemdManager {
    fn create_gadget_service(&self) -> Result<(), SetupError> {
        info!("Creating systemd service file...");

        let executable_path = Self::get_executable_path()?;

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer USB Gadget Configuration
After=multi-user.target
Before=network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart={} _internal_configure_gadget
ExecStop=/bin/sh -c 'echo "" > /sys/kernel/config/usb_gadget/g1/UDC || true'
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#,
            executable_path
        );

        // Write service file
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SERVICE_FILE)
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create service file: {}", e))
            })?;

        file.write_all(service_content.as_bytes())
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to write service file: {}", e)))?;

        info!("Created systemd service file at {}", SERVICE_FILE);

        // Reload systemd daemon
        let output = Command::new("systemctl")
            .arg("daemon-reload")
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl daemon-reload failed: {}",
                stderr
            )));
        }

        info!("Reloaded systemd daemon");

        Ok(())
    }

    fn enable_gadget_service(&self) -> Result<(), SetupError> {
        info!("Enabling systemd service...");

        let output = Command::new("systemctl")
            .args(["enable", &format!("{}.service", SERVICE_NAME)])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl enable failed: {}",
                stderr
            )));
        }

        info!("Enabled {} service", SERVICE_NAME);

        Ok(())
    }

    fn is_service_enabled(&self) -> Result<bool, SetupError> {
        let output = Command::new("systemctl")
            .args(["is-enabled", &format!("{}.service", SERVICE_NAME)])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {}", e))
            })?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Service {} is-enabled result: {}", SERVICE_NAME, result);

        Ok(result == "enabled")
    }
}