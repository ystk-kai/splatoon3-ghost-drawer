use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::sync::Arc;
use tracing::info;

pub struct FixPermissionsUseCase {
    usb_gadget_manager: Arc<dyn UsbGadgetManager>,
}

impl FixPermissionsUseCase {
    pub fn new(usb_gadget_manager: Arc<dyn UsbGadgetManager>) -> Self {
        Self { usb_gadget_manager }
    }

    pub fn execute(&self) -> Result<(), SetupError> {
        info!("Fixing HID device permissions...");

        // Check if gadget is configured
        if !self.usb_gadget_manager.is_gadget_configured()? {
            return Err(SetupError::Unknown(
                "USB Gadget is not configured. Please run 'fix-connection' first.".to_string(),
            ));
        }

        // Configure HID permissions
        self.configure_hid_permissions()?;

        info!("HID device permissions fixed successfully!");
        Ok(())
    }

    fn configure_hid_permissions(&self) -> Result<(), SetupError> {
        use std::path::Path;
        use std::process::Command;
        use tracing::warn;

        info!("Configuring HID device permissions...");

        // Check for HID devices
        for i in 0..4 {
            let hid_path = format!("/dev/hidg{i}");
            if Path::new(&hid_path).exists() {
                info!("Found HID device: {}", hid_path);

                // Change ownership to the original user if run with sudo
                if let Ok(uid) = std::env::var("SUDO_UID")
                    && let Ok(gid) = std::env::var("SUDO_GID")
                {
                    info!("Setting permissions for {} to {}:{}", hid_path, uid, gid);
                    if let Ok(uid_int) = uid.parse::<u32>()
                        && let Ok(gid_int) = gid.parse::<u32>()
                    {
                        use std::os::unix::fs::chown;
                        if let Err(e) = chown(&hid_path, Some(uid_int), Some(gid_int)) {
                            warn!("Failed to chown {}: {}", hid_path, e);
                        } else {
                            info!("Successfully changed ownership of {}", hid_path);
                        }
                    }
                }

                // Set permissions to read/write for owner and group
                let output = Command::new("chmod")
                    .arg("664")
                    .arg(&hid_path)
                    .output()
                    .map_err(|e| {
                        SetupError::Unknown(format!("Failed to change permissions: {e}"))
                    })?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to change permissions of {}: {}", hid_path, stderr);
                } else {
                    info!("Set permissions for {} to 664", hid_path);
                }
            }
        }

        Ok(())
    }
}
