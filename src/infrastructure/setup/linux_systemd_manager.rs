use crate::domain::setup::repositories::{SetupError, SystemdServiceManager};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{debug, info, warn};

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
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to get executable path: {}", e))
            })
    }
}

impl SystemdServiceManager for LinuxSystemdManager {
    fn create_gadget_service(&self) -> Result<(), SetupError> {
        info!("Creating systemd service file...");

        // Use the path where the binary will be installed
        let executable_path = "/opt/splatoon3-ghost-drawer/splatoon3-ghost-drawer".to_string();

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer USB Gadget Configuration
After=sysinit.target
Before=basic.target
DefaultDependencies=no

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart={} _internal_configure_gadget
ExecStop=/bin/sh -c 'echo "" > /sys/kernel/config/usb_gadget/g1/UDC || true'
StandardOutput=journal
StandardError=journal
TimeoutStartSec=30s

[Install]
WantedBy=sysinit.target
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

        file.write_all(service_content.as_bytes()).map_err(|e| {
            SetupError::SystemdServiceFailed(format!("Failed to write service file: {}", e))
        })?;

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
        debug!(
            "Service {} is-enabled result: {}",
            GADGET_SERVICE_NAME, result
        );

        Ok(result == "enabled")
    }

    fn create_web_service(&self) -> Result<(), SetupError> {
        info!("Creating web UI systemd service file...");

        // Use the path where the binary will be installed
        let executable_path = "/opt/splatoon3-ghost-drawer/splatoon3-ghost-drawer".to_string();

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer Web Service
After=network-online.target splatoon3-gadget.service
Wants=network-online.target
Requires=splatoon3-gadget.service

[Service]
Type=simple
WorkingDirectory=/opt/splatoon3-ghost-drawer
ExecStart={} run
Restart=on-failure
RestartSec=10
User=root
Environment="RUST_LOG=info"
StandardOutput=journal
StandardError=journal
TimeoutStartSec=60s

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
                SetupError::SystemdServiceFailed(format!(
                    "Failed to create web service file: {}",
                    e
                ))
            })?;

        file.write_all(service_content.as_bytes()).map_err(|e| {
            SetupError::SystemdServiceFailed(format!("Failed to write web service file: {}", e))
        })?;

        info!(
            "Created web UI systemd service file at {}",
            WEB_SERVICE_FILE
        );

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
                fs::remove_file(service_file).map_err(|e| {
                    SetupError::SystemdServiceFailed(format!(
                        "Failed to remove service file {}: {}",
                        service_file, e
                    ))
                })?;
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

    fn setup_application_files(&self) -> Result<(), SetupError> {
        info!("Setting up application files...");

        let app_dir = "/opt/splatoon3-ghost-drawer";

        // Create application directory
        fs::create_dir_all(app_dir).map_err(|e| SetupError::FileSystemError(e))?;

        // Get source directory (where the binary was executed from)
        let exe_path = std::env::current_exe().map_err(|e| {
            SetupError::SystemdServiceFailed(format!("Failed to get executable path: {}", e))
        })?;

        let src_dir = exe_path.parent().and_then(|p| p.parent()).ok_or_else(|| {
            SetupError::SystemdServiceFailed("Failed to determine source directory".to_string())
        })?;

        // Look for web directory in common locations
        let web_dirs = [
            src_dir.join("web"),
            Path::new("/home/ystk/projects/splatoon3-ghost-drawer/web").to_path_buf(),
            Path::new("./web").to_path_buf(),
        ];

        let mut web_src_found = false;
        for web_src in &web_dirs {
            if web_src.exists() {
                info!("Found web directory at: {:?}", web_src);

                // Copy web directory
                let web_dest = Path::new(app_dir).join("web");

                // Remove existing web directory if it exists
                if web_dest.exists() {
                    fs::remove_dir_all(&web_dest).map_err(|e| SetupError::FileSystemError(e))?;
                }

                // Copy directory recursively
                let output = Command::new("cp")
                    .args([
                        "-r",
                        &web_src.to_string_lossy(),
                        &web_dest.to_string_lossy(),
                    ])
                    .output()
                    .map_err(|e| {
                        SetupError::SystemdServiceFailed(format!(
                            "Failed to copy web directory: {}",
                            e
                        ))
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(SetupError::SystemdServiceFailed(format!(
                        "Failed to copy web directory: {}",
                        stderr
                    )));
                }

                info!("Copied web directory to {}", web_dest.display());
                web_src_found = true;
                break;
            }
        }

        if !web_src_found {
            warn!(
                "Web directory not found in any expected location. Web UI may not work properly."
            );
        }

        // Copy the binary itself to /opt for consistency
        let binary_dest = Path::new(app_dir).join("splatoon3-ghost-drawer");
        let binary_src = Self::get_executable_path()?;

        let output = Command::new("cp")
            .args([&binary_src, binary_dest.to_str().unwrap()])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to copy binary: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to copy binary: {}",
                stderr
            )));
        }

        // Make binary executable
        let output = Command::new("chmod")
            .args(["+x", binary_dest.to_str().unwrap()])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to chmod binary: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to make binary executable: {}",
                stderr
            )));
        }

        info!("Application files setup completed");
        Ok(())
    }

    fn cleanup_application_files(&self) -> Result<(), SetupError> {
        info!("Cleaning up application files...");

        let app_dir = "/opt/splatoon3-ghost-drawer";

        if Path::new(app_dir).exists() {
            fs::remove_dir_all(app_dir).map_err(|e| SetupError::FileSystemError(e))?;
            info!("Removed application directory: {}", app_dir);
        }

        Ok(())
    }
}
