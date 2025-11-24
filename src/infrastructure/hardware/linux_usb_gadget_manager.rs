use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info, warn};

const GADGET_PATH: &str = "/sys/kernel/config/usb_gadget/nintendo_controller";
const VID: &str = "0x0f0d"; // HORI CO., LTD.
const PID: &str = "0x0092"; // Pokken Tournament DX Pro Pad

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

                if let Ok(output) = output
                    && !output.status.success()
                {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    warn!("Failed to load module {}: {}", module, stderr);
                }
            }

            // Raspberry Pi Zero 2W specific: check device tree overlay is loaded
            info!("Checking device tree overlays for Raspberry Pi Zero 2W...");
            let overlay_cmd = Command::new("ls").arg("/boot/firmware/overlays/").output();

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

                // Change ownership to splatoon3 user/group
                // Since this runs as root (systemd service), we can't rely on SUDO_UID
                // We'll try to find the splatoon3 user/group ID

                let uid_output = Command::new("id").args(["-u", "splatoon3"]).output();
                let gid_output = Command::new("id").args(["-g", "splatoon3"]).output();

                if let (Ok(uid_out), Ok(gid_out)) = (uid_output, gid_output) {
                    if uid_out.status.success() && gid_out.status.success() {
                        let uid = String::from_utf8_lossy(&uid_out.stdout).trim().to_string();
                        let gid = String::from_utf8_lossy(&gid_out.stdout).trim().to_string();

                        info!("Setting permissions for {} to {}:{}", hid_path, uid, gid);

                        let output = Command::new("chown")
                            .arg(format!("{}:{}", uid, gid))
                            .arg(&hid_path)
                            .output()
                            .map_err(|e| {
                                SetupError::Unknown(format!("Failed to change ownership: {e}"))
                            })?;

                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            warn!("Failed to change ownership of {}: {}", hid_path, stderr);
                        }
                    } else {
                        warn!("Could not find splatoon3 user/group IDs");
                    }
                } else {
                    warn!("Failed to execute id command");
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
        self.write_file(&format!("{hid_dir}/report_length"), "8")?;

        // Write HID report descriptor for Nintendo Pro Controller
        // This is the actual descriptor used by the Pro Controller

        // ... (inside configure_as_pro_controller)

        // Set USB version
        self.write_file(&format!("{GADGET_PATH}/bcdUSB"), "0x0200")?; // USB 2.0
        self.write_file(&format!("{GADGET_PATH}/bcdDevice"), "0x0100")?;

        // ...

        // Write HID report descriptor for Pokken Tournament DX Pro Pad
        let report_desc = vec![
            0x05, 0x01, // Usage Page (Generic Desktop Ctrls)
            0x09, 0x05, // Usage (Game Pad)
            0xA1, 0x01, // Collection (Application)
            0x15, 0x00, //   Logical Minimum (0)
            0x25, 0x01, //   Logical Maximum (1)
            0x35, 0x00, //   Physical Minimum (0)
            0x45, 0x01, //   Physical Maximum (1)
            0x75, 0x01, //   Report Size (1)
            0x95, 0x10, //   Report Count (16)
            0x05, 0x09, //   Usage Page (Button)
            0x19, 0x01, //   Usage Minimum (0x01)
            0x29, 0x10, //   Usage Maximum (0x10)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x05, 0x01, //   Usage Page (Generic Desktop Ctrls)
            0x25, 0x07, //   Logical Maximum (7)
            0x46, 0x3B, 0x01, //   Physical Maximum (315)
            0x75, 0x04, //   Report Size (4)
            0x95, 0x01, //   Report Count (1)
            0x65, 0x14, //   Unit (System: English Rotation, Length: Centimeter)
            0x09, 0x39, //   Usage (Hat Switch)
            0x81, 0x42, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,Null State)
            0x65, 0x00, //   Unit (None)
            0x95, 0x01, //   Report Count (1)
            0x81,
            0x01, //   Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x26, 0xFF, 0x00, //   Logical Maximum (255)
            0x46, 0xFF, 0x00, //   Physical Maximum (255)
            0x09, 0x30, //   Usage (X)
            0x09, 0x31, //   Usage (Y)
            0x09, 0x32, //   Usage (Z)
            0x09, 0x35, //   Usage (Rz)
            0x75, 0x08, //   Report Size (8)
            0x95, 0x04, //   Report Count (4)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x06, 0x00, 0xFF, //   Usage Page (Vendor Defined 0xFF00)
            0x09, 0x20, //   Usage (0x20)
            0x95, 0x01, //   Report Count (1)
            0x81,
            0x02, //   Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
            0x0A, 0x21, 0x26, //   Usage (0x2621)
            0x95, 0x08, //   Report Count (8)
            0x91,
            0x02, //   Output (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
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

        // Robustly ensure /dev/hidg0 exists and is a character device
        let hidg0_path = Path::new("/dev/hidg0");
        let needs_recreation = if !hidg0_path.exists() {
            true
        } else {
            // Check if it is a character device
            match fs::metadata(hidg0_path) {
                Ok(metadata) => {
                    use std::os::unix::fs::FileTypeExt;
                    !metadata.file_type().is_char_device()
                }
                Err(_) => true,
            }
        };

        if needs_recreation {
            warn!("/dev/hidg0 missing or not a character device. Attempting manual creation...");

            // Clean up if it exists (as directory or file)
            if hidg0_path.exists() {
                if hidg0_path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(hidg0_path) {
                        error!("Failed to remove directory /dev/hidg0: {}", e);
                        // Try with command as fallback
                        let _ = Command::new("rm").args(["-rf", "/dev/hidg0"]).output();
                    }
                } else {
                    let _ = fs::remove_file("/dev/hidg0");
                }
            }

            // Create device node manually: mknod /dev/hidg0 c 236 0
            // Note: Major number 236 is typical for HID gadget, but dynamic.
            // Ideally we should read it from /sys/kernel/config/usb_gadget/nintendo_controller/functions/hid.usb0/dev
            // Format is "Major:Minor" e.g. "236:0"

            let dev_path = format!("{hid_dir}/dev");
            let (major, minor) = if let Ok(dev_content) = fs::read_to_string(&dev_path) {
                let parts: Vec<&str> = dev_content.trim().split(':').collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    error!("Invalid format in {}: {}", dev_path, dev_content);
                    ("236".to_string(), "0".to_string())
                }
            } else {
                error!("Could not read device number from {}", dev_path);
                ("236".to_string(), "0".to_string())
            };

            info!(
                "Creating /dev/hidg0 with major {} and minor {}",
                major, minor
            );

            let output = Command::new("mknod")
                .args(["/dev/hidg0", "c", &major, &minor])
                .output()
                .map_err(|e| SetupError::Unknown(format!("Failed to run mknod: {e}")))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("Failed to create /dev/hidg0: {}", stderr);
            }
        }

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
