use crate::domain::hardware::{HardwareError, UsbGadget, UsbGadgetRepository, UsbGadgetState};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;
use tracing::info;

pub struct LinuxUsbGadgetManager {
    configfs_path: String,
}

impl LinuxUsbGadgetManager {
    pub fn new() -> Self {
        Self {
            configfs_path: "/sys/kernel/config/usb_gadget".to_string(),
        }
    }
}

impl Default for LinuxUsbGadgetManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxUsbGadgetManager {
    async fn ensure_configfs_mounted(&self) -> Result<(), HardwareError> {
        let path = Path::new(&self.configfs_path);
        if !path.exists() {
            // Try to mount configfs
            let output = Command::new("sudo")
                .args(["mount", "-t", "configfs", "none", "/sys/kernel/config"])
                .output()
                .await
                .map_err(|e| {
                    HardwareError::SystemCommandFailed(format!("Failed to mount configfs: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(HardwareError::GadgetConfigurationFailed(format!(
                    "Failed to mount configfs: {}",
                    stderr
                )));
            }
        }
        Ok(())
    }

    async fn get_available_udc(&self) -> Result<String, HardwareError> {
        let udc_path = "/sys/class/udc";
        let mut entries = fs::read_dir(udc_path).await.map_err(|e| {
            HardwareError::FileOperationFailed(format!("Failed to read UDC directory: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            HardwareError::FileOperationFailed(format!("Failed to read UDC entry: {}", e))
        })? {
            if let Ok(name) = entry.file_name().into_string() {
                return Ok(name);
            }
        }

        Err(HardwareError::GadgetConfigurationFailed(
            "No UDC available".to_string(),
        ))
    }

    async fn write_file(&self, path: &str, content: &str) -> Result<(), HardwareError> {
        fs::write(path, content).await.map_err(|e| {
            HardwareError::FileOperationFailed(format!("Failed to write {}: {}", path, e))
        })?;
        Ok(())
    }

    async fn create_directory(&self, path: &str) -> Result<(), HardwareError> {
        fs::create_dir_all(path).await.map_err(|e| {
            HardwareError::FileOperationFailed(format!(
                "Failed to create directory {}: {}",
                path, e
            ))
        })?;
        Ok(())
    }
}

#[async_trait]
impl UsbGadgetRepository for LinuxUsbGadgetManager {
    async fn create_gadget(&self, gadget: &UsbGadget) -> Result<(), HardwareError> {
        self.ensure_configfs_mounted().await?;

        let gadget_path = gadget.full_path();

        // Create gadget directory
        self.create_directory(&gadget_path).await?;

        // Set USB device descriptor values
        self.write_file(
            &format!("{}/idVendor", gadget_path),
            &format!("0x{:04x}", gadget.descriptor.vendor_id),
        )
        .await?;
        self.write_file(
            &format!("{}/idProduct", gadget_path),
            &format!("0x{:04x}", gadget.descriptor.product_id),
        )
        .await?;
        self.write_file(
            &format!("{}/bcdDevice", gadget_path),
            &format!("0x{:04x}", gadget.descriptor.device_version),
        )
        .await?;
        self.write_file(
            &format!("{}/bcdUSB", gadget_path),
            &format!("0x{:04x}", gadget.descriptor.usb_version),
        )
        .await?;

        // Create strings directory
        let strings_path = format!("{}/strings/0x409", gadget_path);
        self.create_directory(&strings_path).await?;

        // Set string descriptors
        self.write_file(
            &format!("{}/serialnumber", strings_path),
            &gadget.descriptor.serial_number,
        )
        .await?;
        self.write_file(
            &format!("{}/manufacturer", strings_path),
            &gadget.descriptor.manufacturer,
        )
        .await?;
        self.write_file(
            &format!("{}/product", strings_path),
            &gadget.descriptor.product,
        )
        .await?;

        info!("USB gadget created at {}", gadget_path);
        Ok(())
    }

    async fn configure_gadget(&self, gadget: &UsbGadget) -> Result<(), HardwareError> {
        let gadget_path = gadget.full_path();

        // Create configuration
        let config_path = format!("{}/configs/c.1", gadget_path);
        self.create_directory(&config_path).await?;

        // Set configuration descriptor
        self.write_file(&format!("{}/MaxPower", config_path), "500")
            .await?;

        // Create configuration strings
        let config_strings_path = format!("{}/strings/0x409", config_path);
        self.create_directory(&config_strings_path).await?;
        self.write_file(
            &format!("{}/configuration", config_strings_path),
            "Nintendo Switch Pro Controller",
        )
        .await?;

        // Create HID function
        let function_path = format!("{}/functions/hid.usb0", gadget_path);
        self.create_directory(&function_path).await?;

        // Configure HID function
        self.write_file(&format!("{}/protocol", function_path), "0")
            .await?;
        self.write_file(&format!("{}/subclass", function_path), "0")
            .await?;
        self.write_file(&format!("{}/report_length", function_path), "64")
            .await?;

        // Write HID report descriptor for Nintendo Pro Controller
        let report_desc = vec![
            0x05, 0x01, // Usage Page (Generic Desktop)
            0x09, 0x05, // Usage (Game Pad)
            0xA1, 0x01, // Collection (Application)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x01, //   Usage Minimum (Button 1)
            0x29, 0x10, //   Usage Maximum (Button 16)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x10, //   Report Count (16)
            0x81, 0x02, //   Input (Data,Var,Abs)
            0x05, 0x01, //   Usage Page (Generic Desktop)
            0x09, 0x39, //   Usage (Hat switch)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x07, //   Logical Maximum (7)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x81, 0x42, //   Input (Data,Var,Abs,Null)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x81, 0x01, //   Input (Const,Array,Abs)
            0x05, 0x01, //   Usage Page (Generic Desktop)
            0x09, 0x30, //   Usage (X)
            0x09, 0x31, //   Usage (Y)
            0x09, 0x33, //   Usage (Rx)
            0x09, 0x34, //   Usage (Ry)
            0x15, 0x00, //   Logical Minimum (0)
            0x26, 0xFF, 0x00, //   Logical Maximum (255)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x04, //   Report Count (4)
            0x81, 0x02, //   Input (Data,Var,Abs)
            0xC0, // End Collection
        ];

        let report_desc_path = format!("{}/report_desc", function_path);
        fs::write(&report_desc_path, &report_desc)
            .await
            .map_err(|e| {
                HardwareError::FileOperationFailed(format!(
                    "Failed to write report descriptor: {}",
                    e
                ))
            })?;

        // Link function to configuration
        let symlink_path = format!("{}/hid.usb0", config_path);
        if !Path::new(&symlink_path).exists() {
            std::os::unix::fs::symlink(&function_path, &symlink_path).map_err(|e| {
                HardwareError::FileOperationFailed(format!("Failed to create symlink: {}", e))
            })?;
        }

        info!("USB gadget configured");
        Ok(())
    }

    async fn activate_gadget(&self, gadget: &mut UsbGadget) -> Result<(), HardwareError> {
        let udc = self.get_available_udc().await?;
        let udc_path = format!("{}/UDC", gadget.full_path());

        self.write_file(&udc_path, &udc).await?;

        gadget.udc_name = Some(udc.clone());
        gadget.state = UsbGadgetState::Active;

        info!("USB gadget activated with UDC: {}", udc);
        Ok(())
    }

    async fn deactivate_gadget(&self, gadget: &mut UsbGadget) -> Result<(), HardwareError> {
        let udc_path = format!("{}/UDC", gadget.full_path());

        // Write empty string to deactivate
        self.write_file(&udc_path, "").await?;

        gadget.udc_name = None;
        gadget.state = UsbGadgetState::Configured;

        info!("USB gadget deactivated");
        Ok(())
    }

    async fn get_gadget_state(&self, gadget_id: &str) -> Result<UsbGadget, HardwareError> {
        let gadget_path = format!("{}/{}", self.configfs_path, gadget_id);
        let path = Path::new(&gadget_path);

        if !path.exists() {
            return Err(HardwareError::GadgetConfigurationFailed(format!(
                "Gadget {} not found",
                gadget_id
            )));
        }

        let udc_path = format!("{}/UDC", gadget_path);
        let udc_content = fs::read_to_string(&udc_path).await.unwrap_or_default();
        let udc_name = udc_content.trim();

        let state = if udc_name.is_empty() {
            UsbGadgetState::Configured
        } else {
            UsbGadgetState::Active
        };

        let mut gadget = UsbGadget::new(gadget_id);
        gadget.state = state;
        if !udc_name.is_empty() {
            gadget.udc_name = Some(udc_name.to_string());
        }

        Ok(gadget)
    }

    async fn remove_gadget(&self, gadget_id: &str) -> Result<(), HardwareError> {
        let gadget_path = format!("{}/{}", self.configfs_path, gadget_id);

        // First deactivate if active
        if let Ok(mut gadget) = self.get_gadget_state(gadget_id).await {
            if gadget.is_active() {
                self.deactivate_gadget(&mut gadget).await?;
            }
        }

        // Remove symlinks
        let config_path = format!("{}/configs/c.1", gadget_path);
        let symlink_path = format!("{}/hid.usb0", config_path);
        if Path::new(&symlink_path).exists() {
            fs::remove_file(&symlink_path).await.map_err(|e| {
                HardwareError::FileOperationFailed(format!("Failed to remove symlink: {}", e))
            })?;
        }

        // Remove directories in reverse order
        let dirs_to_remove = vec![
            format!("{}/functions/hid.usb0", gadget_path),
            format!("{}/configs/c.1/strings/0x409", gadget_path),
            format!("{}/configs/c.1", gadget_path),
            format!("{}/strings/0x409", gadget_path),
            gadget_path.clone(),
        ];

        for dir in dirs_to_remove {
            if Path::new(&dir).exists() {
                fs::remove_dir(&dir).await.map_err(|e| {
                    HardwareError::FileOperationFailed(format!("Failed to remove {}: {}", dir, e))
                })?;
            }
        }

        info!("USB gadget {} removed", gadget_id);
        Ok(())
    }
}
