use crate::domain::setup::repositories::{SetupError, SystemdServiceManager};
use std::fs;
use std::io::Write;
use std::path::Path;
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
        Self
    }

    fn get_executable_path() -> Result<String, SetupError> {
        // Get the path of the current executable
        std::env::current_exe()
            .map(|path| path.to_string_lossy().to_string())
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to get executable path: {e}"))
            })
    }

    fn create_splatoon3_user(&self) -> Result<(), SetupError> {
        info!("Creating dedicated splatoon3 user...");

        // Check if user already exists
        let check_output = Command::new("id")
            .arg("splatoon3")
            .output();

        if let Ok(output) = check_output {
            if output.status.success() {
                info!("User 'splatoon3' already exists");
                return Ok(());
            }
        }

        // Create system user
        let output = Command::new("useradd")
            .arg("--system")
            .arg("--no-create-home")
            .arg("--shell")
            .arg("/usr/sbin/nologin")
            .arg("--comment")
            .arg("Splatoon3 Ghost Drawer Service User")
            .arg("splatoon3")
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create user: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to create splatoon3 user: {stderr}"
            )));
        }

        // Add user to input group for HID device access
        let output = Command::new("usermod")
            .arg("-a")
            .arg("-G")
            .arg("input")
            .arg("splatoon3")
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to add user to input group: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to add splatoon3 user to input group: {stderr}"
            )));
        }

        info!("Created splatoon3 user and added to input group");
        Ok(())
    }

    fn setup_hid_device_permissions(&self) -> Result<(), SetupError> {
        info!("Setting up HID device permissions...");

        // Create udev rule for HID device permissions
        let udev_rule_content = r#"# Splatoon3 Ghost Drawer HID Device Permissions
# Give splatoon3 user access to HID gadget devices
SUBSYSTEM=="hidg", GROUP="splatoon3", MODE="0664"
KERNEL=="hidg*", GROUP="splatoon3", MODE="0664"

# Also ensure input group access
SUBSYSTEM=="input", GROUP="input", MODE="0664"
KERNEL=="event*", GROUP="input", MODE="0664"
"#;

        let udev_rule_path = "/etc/udev/rules.d/99-splatoon3-hid.rules";
        fs::write(udev_rule_path, udev_rule_content)
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to create udev rule: {e}")))?;

        info!("Created udev rule at {}", udev_rule_path);

        // Create systemd-tmpfiles rule for runtime permissions
        let tmpfiles_content = r#"# Splatoon3 Ghost Drawer Runtime Permissions
d /dev/hidg0 0664 root splatoon3 -
d /dev/hidg1 0664 root splatoon3 -
d /dev/hidg2 0664 root splatoon3 -
d /dev/hidg3 0664 root splatoon3 -
"#;

        let tmpfiles_path = "/etc/tmpfiles.d/splatoon3-hid.conf";
        fs::write(tmpfiles_path, tmpfiles_content)
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to create tmpfiles rule: {e}")))?;

        info!("Created tmpfiles rule at {}", tmpfiles_path);

        // Reload udev rules
        let output = Command::new("udevadm")
            .arg("control")
            .arg("--reload-rules")
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                info!("Reloaded udev rules");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!("Failed to reload udev rules (non-critical): {}", stderr);
            }
        }

        Ok(())
    }
}

impl SystemdServiceManager for LinuxSystemdManager {
    fn create_gadget_service(&self) -> Result<(), SetupError> {
        info!("Creating systemd service file...");

        // Get the current executable path dynamically
        let executable_path = Self::get_executable_path()?;

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer USB Gadget Configuration
After=sysinit.target
Before=basic.target
DefaultDependencies=no

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart={executable_path} _internal_configure_gadget
ExecStop=/bin/sh -c 'echo "" > /sys/kernel/config/usb_gadget/nintendo_controller/UDC || true'
StandardOutput=journal
StandardError=journal
TimeoutStartSec=30s

[Install]
WantedBy=sysinit.target
"#
        );

        // Write service file
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(GADGET_SERVICE_FILE)
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create service file: {e}"))
            })?;

        file.write_all(service_content.as_bytes()).map_err(|e| {
            SetupError::SystemdServiceFailed(format!("Failed to write service file: {e}"))
        })?;

        info!("Created systemd service file at {}", GADGET_SERVICE_FILE);

        // Reload systemd daemon
        let output = Command::new("systemctl")
            .arg("daemon-reload")
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl daemon-reload failed: {stderr}"
            )));
        }

        info!("Reloaded systemd daemon");

        Ok(())
    }

    fn enable_gadget_service(&self) -> Result<(), SetupError> {
        info!("Enabling systemd service...");

        let output = Command::new("systemctl")
            .args(["enable", &format!("{GADGET_SERVICE_NAME}.service")])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl enable failed: {stderr}"
            )));
        }

        info!("Enabled {} service", GADGET_SERVICE_NAME);

        Ok(())
    }

    fn is_service_enabled(&self) -> Result<bool, SetupError> {
        let output = Command::new("systemctl")
            .args(["is-enabled", &format!("{GADGET_SERVICE_NAME}.service")])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
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

        // Create dedicated user for web service
        self.create_splatoon3_user()?;

        // Setup HID device permissions
        self.setup_hid_device_permissions()?;

        // Get the current executable path dynamically
        let executable_path = Self::get_executable_path()?;

        let service_content = format!(
            r#"[Unit]
Description=Splatoon3 Ghost Drawer Web Service
After=network-online.target splatoon3-gadget.service
Wants=network-online.target
Requires=splatoon3-gadget.service

[Service]
Type=simple
ExecStart={executable_path} run
Restart=on-failure
RestartSec=10
User=splatoon3
Group=splatoon3
Environment="RUST_LOG=info"
StandardOutput=journal
StandardError=journal
TimeoutStartSec=60s
# Grant access to HID devices
SupplementaryGroups=input

[Install]
WantedBy=multi-user.target
"#
        );

        // Write service file
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(WEB_SERVICE_FILE)
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to create web service file: {e}"))
            })?;

        file.write_all(service_content.as_bytes()).map_err(|e| {
            SetupError::SystemdServiceFailed(format!("Failed to write web service file: {e}"))
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
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl daemon-reload failed: {stderr}"
            )));
        }

        info!("Reloaded systemd daemon for web service");

        Ok(())
    }

    fn enable_web_service(&self) -> Result<(), SetupError> {
        info!("Enabling web UI systemd service...");

        let output = Command::new("systemctl")
            .args(["enable", &format!("{WEB_SERVICE_NAME}.service")])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl enable failed: {stderr}"
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
                .args(["stop", &format!("{service_name}.service")])
                .output();

            // Disable service
            let _ = Command::new("systemctl")
                .args(["disable", &format!("{service_name}.service")])
                .output();
        }

        // Remove service files
        for service_file in [GADGET_SERVICE_FILE, WEB_SERVICE_FILE] {
            if std::path::Path::new(service_file).exists() {
                fs::remove_file(service_file).map_err(|e| {
                    SetupError::SystemdServiceFailed(format!(
                        "Failed to remove service file {service_file}: {e}"
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
                SetupError::SystemdServiceFailed(format!("Failed to run systemctl: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "systemctl daemon-reload failed: {stderr}"
            )));
        }

        info!("Removed all systemd services");

        Ok(())
    }

    fn setup_application_files(&self) -> Result<(), SetupError> {
        info!("Setting up application files...");

        let app_dir = "/opt/splatoon3-ghost-drawer";

        // Create application directory
        fs::create_dir_all(app_dir).map_err(SetupError::FileSystemError)?;

        // webディレクトリはバイナリに埋め込まれているため、コピーは不要
        info!("Web assets are embedded in the binary, no need to copy files");

        // Copy the binary itself to /opt for consistency
        let binary_dest = Path::new(app_dir).join("splatoon3-ghost-drawer");
        let binary_src = Self::get_executable_path()?;

        let output = Command::new("cp")
            .args([&binary_src, binary_dest.to_str().unwrap()])
            .output()
            .map_err(|e| SetupError::SystemdServiceFailed(format!("Failed to copy binary: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to copy binary: {stderr}"
            )));
        }

        // Make binary executable
        let output = Command::new("chmod")
            .args(["+x", binary_dest.to_str().unwrap()])
            .output()
            .map_err(|e| {
                SetupError::SystemdServiceFailed(format!("Failed to chmod binary: {e}"))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::SystemdServiceFailed(format!(
                "Failed to make binary executable: {stderr}"
            )));
        }

        info!("Application files setup completed");
        Ok(())
    }

    fn cleanup_application_files(&self) -> Result<(), SetupError> {
        info!("Cleaning up application files...");

        let app_dir = "/opt/splatoon3-ghost-drawer";

        if Path::new(app_dir).exists() {
            fs::remove_dir_all(app_dir).map_err(SetupError::FileSystemError)?;
            info!("Removed application directory: {}", app_dir);
        }

        Ok(())
    }
}
