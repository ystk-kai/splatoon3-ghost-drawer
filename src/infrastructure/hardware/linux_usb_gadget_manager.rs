use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

const GADGET_PATH: &str = "/sys/kernel/config/usb_gadget/nintendo_controller";
const VID: &str = "0x057e"; // Nintendo
const PID: &str = "0x2009"; // Pro Controller

pub struct LinuxUsbGadgetManager;

impl Default for LinuxUsbGadgetManager {
    fn default() -> Self {
        Self
    }
}

impl LinuxUsbGadgetManager {
    pub fn new() -> Self {
        Self
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), SetupError> {
        fs::write(path, content).map_err(|e| {
            error!("Failed to write to {}: {}", path, e);
            SetupError::FileSystemError(e)
        })?;
        debug!("Wrote '{}' to {}", content.trim(), path);
        Ok(())
    }

    fn create_directory(&self, path: &str) -> Result<(), SetupError> {
        if !Path::new(path).exists() {
            fs::create_dir_all(path).map_err(|e| {
                error!("Failed to create directory {}: {}", path, e);
                SetupError::FileSystemError(e)
            })?;
            debug!("Created directory {}", path);
        }
        Ok(())
    }

    fn load_kernel_modules(&self) -> Result<(), SetupError> {
        info!("Loading kernel modules...");

        // Load libcomposite module
        let output = Command::new("modprobe")
            .arg("libcomposite")
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to run modprobe: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to load libcomposite: {}", stderr);
        }

        Ok(())
    }

    fn get_udc_name(&self) -> Result<String, SetupError> {
        let udc_dir = "/sys/class/udc";

        // First check if the directory exists
        if !Path::new(udc_dir).exists() {
            error!("UDC directory does not exist: {}", udc_dir);

            // Try to load the necessary modules for Raspberry Pi Zero 2W
            info!("Attempting to load USB gadget modules...");

            // For Raspberry Pi Zero 2W with BCM2710A1
            let modules = vec!["dwc2", "libcomposite", "usb_f_hid"];
            for module in modules {
                info!("Loading module: {}", module);
                let output = Command::new("modprobe").arg(module).output();

                if let Ok(output) = output {
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        warn!("Failed to load module {}: {}", module, stderr);
                    }
                }
            }

            // Raspberry Pi Zero 2W specific: check device tree overlay is loaded
            info!("Checking device tree overlays for Raspberry Pi Zero 2W...");
            let overlay_cmd = Command::new("ls")
                .arg("/boot/firmware/overlays/")
                .output();

            if let Ok(output) = overlay_cmd {
                let overlays = String::from_utf8_lossy(&output.stdout);
                if overlays.contains("dwc2") {
                    info!("dwc2 overlay available");
                }
            }

            // Wait a bit for modules to initialize
            std::thread::sleep(std::time::Duration::from_millis(1000));

            // Check again after loading modules
            if !Path::new(udc_dir).exists() {
                error!("UDC directory still not found. This may indicate:");
                error!("1. USB OTG is not enabled in device tree");
                error!("2. The musb driver is not compatible with your kernel");
                error!("3. Hardware does not support USB OTG");
                return Err(SetupError::Unknown(
                    "UDC directory not found after loading modules".to_string(),
                ));
            }
        }

        let entries = fs::read_dir(udc_dir)
            .map_err(|e| SetupError::Unknown(format!("Failed to read UDC directory: {e}")))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| SetupError::Unknown(format!("Failed to read UDC entry: {e}")))?;

            let name = entry.file_name().to_string_lossy().to_string();
            if !name.is_empty() {
                info!("Found UDC: {}", name);

                // For Raspberry Pi Zero 2W, the UDC is typically dwc2 based
                if name.contains("dwc2") || name.contains("fe980000.usb") {
                    info!("Using Raspberry Pi Zero 2W UDC: {}", name);
                    return Ok(name);
                }

                return Ok(name);
            }
        }

        Err(SetupError::Unknown("No UDC found".to_string()))
    }

    fn configure_hid_permissions(&self) -> Result<(), SetupError> {
        info!("Configuring HID device permissions...");

        // Check for HID devices
        for i in 0..4 {
            let hid_path = format!("/dev/hidg{i}");
            if Path::new(&hid_path).exists() {
                info!("Found HID device: {}", hid_path);

                // Change ownership to current user
                if let Ok(uid) = std::env::var("SUDO_UID") {
                    if let Ok(gid) = std::env::var("SUDO_GID") {
                        info!("Setting permissions for {} to {}:{}", hid_path, uid, gid);

                        let output = Command::new("chown")
                            .arg(format!("{uid}:{gid}"))
                            .arg(&hid_path)
                            .output()
                            .map_err(|e| {
                                SetupError::Unknown(format!("Failed to change ownership: {e}"))
                            })?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            warn!("Failed to change ownership of {}: {}", hid_path, stderr);
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

impl UsbGadgetManager for LinuxUsbGadgetManager {
    fn configure_as_pro_controller(&self) -> Result<(), SetupError> {
        info!("Configuring USB Gadget as Nintendo Switch Pro Controller...");

        // Load kernel modules
        self.load_kernel_modules()?;

        // Check if configfs is mounted
        if !Path::new("/sys/kernel/config/usb_gadget").exists() {
            info!("Mounting configfs...");
            let output = Command::new("mount")
                .args(["-t", "configfs", "none", "/sys/kernel/config"])
                .output()
                .map_err(|e| SetupError::Unknown(format!("Failed to mount configfs: {e}")))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("already mounted") {
                    return Err(SetupError::Unknown(format!(
                        "Failed to mount configfs: {stderr}"
                    )));
                }
            }
        }

        // If gadget already exists, try to clean it up first
        if Path::new(GADGET_PATH).exists() {
            info!("Cleaning up existing gadget configuration...");

            // Unbind UDC if bound
            let udc_path = format!("{GADGET_PATH}/UDC");
            if Path::new(&udc_path).exists() {
                let _ = fs::write(&udc_path, "");
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            // Remove symlinks from configs
            let config_path = format!("{GADGET_PATH}/configs/c.1/hid.usb0");
            if Path::new(&config_path).exists() {
                let _ = fs::remove_file(&config_path);
            }

            // Remove directories in reverse order
            let dirs_to_remove = vec![
                format!("{}/configs/c.1/strings/0x409", GADGET_PATH),
                format!("{}/configs/c.1", GADGET_PATH),
                format!("{}/configs", GADGET_PATH),
                format!("{}/functions/hid.usb0", GADGET_PATH),
                format!("{}/functions", GADGET_PATH),
                format!("{}/strings/0x409", GADGET_PATH),
                format!("{}/strings", GADGET_PATH),
                GADGET_PATH.to_string(),
            ];

            for dir in dirs_to_remove {
                if Path::new(&dir).exists() {
                    let _ = fs::remove_dir(&dir);
                }
            }

            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // Create gadget directory
        self.create_directory(GADGET_PATH)?;

        // Set vendor and product IDs
        self.write_file(&format!("{GADGET_PATH}/idVendor"), VID)?;
        self.write_file(&format!("{GADGET_PATH}/idProduct"), PID)?;

        // Set USB version
        self.write_file(&format!("{GADGET_PATH}/bcdUSB"), "0x0200")?; // USB 2.0
        self.write_file(&format!("{GADGET_PATH}/bcdDevice"), "0x0100")?;

        // Set device class
        self.write_file(&format!("{GADGET_PATH}/bDeviceClass"), "0x00")?;
        self.write_file(&format!("{GADGET_PATH}/bDeviceSubClass"), "0x00")?;
        self.write_file(&format!("{GADGET_PATH}/bDeviceProtocol"), "0x00")?;

        // Set strings
        let strings_dir = format!("{GADGET_PATH}/strings/0x409");
        self.create_directory(&strings_dir)?;
        self.write_file(&format!("{strings_dir}/serialnumber"), "000000000001")?;
        self.write_file(&format!("{strings_dir}/manufacturer"), "Nintendo")?;
        self.write_file(&format!("{strings_dir}/product"), "Pro Controller")?;

        // Create configuration
        let config_dir = format!("{GADGET_PATH}/configs/c.1");
        self.create_directory(&config_dir)?;
        self.write_file(&format!("{config_dir}/MaxPower"), "500")?;

        let config_strings_dir = format!("{config_dir}/strings/0x409");
        self.create_directory(&config_strings_dir)?;
        self.write_file(
            &format!("{config_strings_dir}/configuration"),
            "Pro Controller",
        )?;

        // Create HID function
        let hid_dir = format!("{GADGET_PATH}/functions/hid.usb0");
        self.create_directory(&hid_dir)?;
        self.write_file(&format!("{hid_dir}/protocol"), "0")?;
        self.write_file(&format!("{hid_dir}/subclass"), "0")?;
        self.write_file(&format!("{hid_dir}/report_length"), "64")?;

        // Write HID report descriptor for Nintendo Pro Controller
        // This is the actual descriptor used by the Pro Controller
        let report_desc = vec![
            0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
            0x15, 0x00, // Logical Minimum (0)
            0x09, 0x04, // Usage (Joystick)
            0xA1, 0x01, // Collection (Application)
            0x85, 0x30, //   Report ID (48)
            0x05, 0x01, //   Usage Page (Generic Desktop Ctrls)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x01, //   Usage Minimum (0x01)
            0x29, 0x0A, //   Usage Maximum (0x0A)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x0A, //   Report Count (10)
            0x55, 0x00, //   Unit Exponent (0)
            0x65, 0x00, //   Unit (None)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x0B, //   Usage Minimum (0x0B)
            0x29, 0x0E, //   Usage Maximum (0x0E)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x04, //   Report Count (4)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x02, //   Report Count (2)
            0x81,
            0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x0B, 0x01, 0x00, 0x01, 0x00, //   Usage (0x010001)
            0xA1, 0x00, //   Collection (Physical)
            0x0B, 0x30, 0x00, 0x01, 0x00, //     Usage (0x010030)
            0x0B, 0x31, 0x00, 0x01, 0x00, //     Usage (0x010031)
            0x0B, 0x32, 0x00, 0x01, 0x00, //     Usage (0x010032)
            0x0B, 0x35, 0x00, 0x01, 0x00, //     Usage (0x010035)
            0x15, 0x00, //     Logical Minimum (0)
            0x27, 0xFF, 0xFF, 0x00, 0x00, //     Logical Maximum (65534)
            0x75, 0x10, //     Report Size (16)
            0x95, 0x04, //     Report Count (4)
            0x81,
            0x02, //     Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0xC0, //   End Collection
            0x0B, 0x39, 0x00, 0x01, 0x00, //   Usage (0x010039)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x07, //   Logical Maximum (7)
            0x35, 0x00, //   Physical Minimum (0)
            0x46, 0x3B, 0x01, //   Physical Maximum (315)
            0x65, 0x14, //   Unit (System: English Rotation, Length: Centimeter)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x0F, //   Usage Minimum (0x0F)
            0x29, 0x12, //   Usage Maximum (0x12)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x04, //   Report Count (4)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x34, //   Report Count (52)
            0x81,
            0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x06, 0x00, 0xFF, //   Usage Page (Vendor Defined 0xFF00)
            0x85, 0x21, //   Report ID (33)
            0x09, 0x01, //   Usage (0x01)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x81,
            0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x85, 0x81, //   Report ID (-127)
            0x09, 0x02, //   Usage (0x02)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x81,
            0x03, //   Input (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x85, 0x01, //   Report ID (1)
            0x09, 0x03, //   Usage (0x03)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x91,
            0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
            0x85, 0x10, //   Report ID (16)
            0x09, 0x04, //   Usage (0x04)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x91,
            0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
            0x85, 0x80, //   Report ID (-128)
            0x09, 0x05, //   Usage (0x05)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x91,
            0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
            0x85, 0x82, //   Report ID (-126)
            0x09, 0x06, //   Usage (0x06)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x3F, //   Report Count (63)
            0x91,
            0x83, //   Output (Const,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Volatile)
            0xC0, // End Collection
        ];

        let report_desc_path = format!("{hid_dir}/report_desc");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&report_desc_path)
            .map_err(|e| {
                error!("Failed to open report descriptor file: {}", e);
                SetupError::FileSystemError(e)
            })?;

        file.write_all(&report_desc).map_err(|e| {
            error!("Failed to write report descriptor: {}", e);
            SetupError::FileSystemError(e)
        })?;

        info!("Wrote HID report descriptor");

        // Link function to configuration
        let function_link = format!("{config_dir}/hid.usb0");
        if !Path::new(&function_link).exists() {
            std::os::unix::fs::symlink(&hid_dir, &function_link).map_err(|e| {
                error!("Failed to create symlink: {}", e);
                SetupError::FileSystemError(e)
            })?;
            debug!("Linked HID function to configuration");
        }

        // Enable the gadget
        let udc_name = self.get_udc_name()?;
        self.write_file(&format!("{GADGET_PATH}/UDC"), &udc_name)?;

        // Wait for HID device to be created
        std::thread::sleep(std::time::Duration::from_millis(1000));

        // Set appropriate permissions for HID device
        self.configure_hid_permissions()?;

        info!("USB Gadget configured successfully!");

        Ok(())
    }

    fn is_gadget_configured(&self) -> Result<bool, SetupError> {
        // Check if gadget path exists
        if !Path::new(GADGET_PATH).exists() {
            return Ok(false);
        }

        // Check if UDC is set (gadget is active)
        let udc_path = format!("{GADGET_PATH}/UDC");
        if !Path::new(&udc_path).exists() {
            return Ok(false);
        }

        let udc_content = fs::read_to_string(&udc_path)?;
        Ok(!udc_content.trim().is_empty())
    }

    fn reconnect_gadget(&self) -> Result<(), SetupError> {
        info!("Reconnecting USB Gadget...");

        // Get the current UDC name
        let udc_path = format!("{GADGET_PATH}/UDC");
        let udc_name = if Path::new(&udc_path).exists() {
            fs::read_to_string(&udc_path)
                .ok()
                .and_then(|s| {
                    let trimmed = s.trim();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed.to_string())
                    }
                })
                .or_else(|| {
                    // Try to get UDC name if not set
                    self.get_udc_name().ok()
                })
        } else {
            Some(self.get_udc_name()?)
        };

        // Disconnect the gadget
        info!("Disconnecting gadget...");
        fs::write(&udc_path, "").map_err(|e| {
            error!("Failed to disconnect gadget: {}", e);
            SetupError::FileSystemError(e)
        })?;

        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Reconnect the gadget
        info!(
            "Reconnecting gadget with UDC: {}",
            udc_name.as_ref().unwrap_or(&"auto".to_string())
        );
        if let Some(udc) = udc_name {
            fs::write(&udc_path, &udc).map_err(|e| {
                error!("Failed to reconnect gadget: {}", e);
                SetupError::FileSystemError(e)
            })?;
        } else {
            return Err(SetupError::Unknown(
                "No UDC available for reconnection".to_string(),
            ));
        }

        info!("USB Gadget reconnected successfully!");
        Ok(())
    }
}
