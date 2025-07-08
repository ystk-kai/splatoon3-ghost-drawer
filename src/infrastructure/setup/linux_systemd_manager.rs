use crate::domain::setup::repositories::{SetupError, SystemdServiceManager};
use std::fs;
use std::io::Write;
use std::process::Command;
use tracing::{debug, info};

const GADGET_SERVICE_NAME: &str = "splatoon3-gadget";
const GADGET_SERVICE_FILE: &str = "/etc/systemd/system/splatoon3-gadget.service";
const WEB_SERVICE_NAME: &str = "splatoon3-ghost-drawer";
const WEB_SERVICE_FILE: &str = "/etc/systemd/system/splatoon3-ghost-drawer.service";

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
            .open(GADGET_SERVICE_FILE)
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create service file: {}", e))
            })?;

        file.write_all(service_content.as_bytes())
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to write service file: {}", e)))?;

        info!("Created systemd service file at {}", GADGET_SERVICE_FILE);

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
            .args(["enable", &format!("{}.service", GADGET_SERVICE_NAME)])
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

        info!("Enabled {} service", GADGET_SERVICE_NAME);

        Ok(())
    }

    fn is_service_enabled(&self) -> Result<bool, SetupError> {
        let output = Command::new("systemctl")
            .args(["is-enabled", &format!("{}.service", GADGET_SERVICE_NAME)])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {}", e))
            })?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Service {} is-enabled result: {}", GADGET_SERVICE_NAME, result);

        Ok(result == "enabled")
    }

    fn create_web_service(&self) -> Result<(), SetupError> {
        info!("Creating web UI systemd service file...");

        let executable_path = Self::get_executable_path()?;

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer Web Service
After=network.target splatoon3-gadget.service
Requires=splatoon3-gadget.service

[Service]
Type=simple
ExecStart={} run
Restart=always
RestartSec=10
User=root
Environment="RUST_LOG=info"
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
            .open(WEB_SERVICE_FILE)
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create web service file: {}", e))
            })?;

        file.write_all(service_content.as_bytes())
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to write web service file: {}", e)))?;

        info!("Created web UI systemd service file at {}", WEB_SERVICE_FILE);

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

        info!("Reloaded systemd daemon for web service");

        Ok(())
    }

    fn enable_web_service(&self) -> Result<(), SetupError> {
        info!("Enabling web UI systemd service...");

        let output = Command::new("systemctl")
            .args(["enable", &format!("{}.service", WEB_SERVICE_NAME)])
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

        info!("Enabled {} service", WEB_SERVICE_NAME);

        Ok(())
    }

    fn disable_and_remove_services(&self) -> Result<(), SetupError> {
        info!("Disabling and removing systemd services...");

        // Stop and disable both services
        for service_name in [GADGET_SERVICE_NAME, WEB_SERVICE_NAME] {
            // Stop service
            let _ = Command::new("systemctl")
                .args(["stop", &format!("{}.service", service_name)])
                .output();

            // Disable service
            let _ = Command::new("systemctl")
                .args(["disable", &format!("{}.service", service_name)])
                .output();
        }

        // Remove service files
        for service_file in [GADGET_SERVICE_FILE, WEB_SERVICE_FILE] {
            if std::path::Path::new(service_file).exists() {
                fs::remove_file(service_file)
                    .map_err(|e| SetupError::SystemdServiceFailed(
                        format!("Failed to remove service file {}: {}", service_file, e)
                    ))?;
                info!("Removed service file: {}", service_file);
            }
        }

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

        info!("Removed all systemd services");

        Ok(())
    }
}