use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tracing::{debug, error, info};

const GADGET_PATH: &str = "/sys/kernel/config/usb_gadget/g1";
const VID: &str = "0x057e"; // Nintendo
const PID: &str = "0x2009"; // Pro Controller

pub struct LinuxUsbGadgetManager;

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
            .map_err(|e| {
                SetupError::Unknown(format!("Failed to run modprobe: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to load libcomposite: {}", stderr);
        }

        Ok(())
    }

    fn get_udc_name(&self) -> Result<String, SetupError> {
        let udc_dir = "/sys/class/udc";
        
        let entries = fs::read_dir(udc_dir).map_err(|e| {
            SetupError::Unknown(format!("Failed to read UDC directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                SetupError::Unknown(format!("Failed to read UDC entry: {}", e))
            })?;
            
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.is_empty() {
                info!("Found UDC: {}", name);
                return Ok(name);
            }
        }

        Err(SetupError::Unknown("No UDC found".to_string()))
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
                .args(&["-t", "configfs", "none", "/sys/kernel/config"])
                .output()
                .map_err(|e| {
                    SetupError::Unknown(format!("Failed to mount configfs: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.contains("already mounted") {
                    return Err(SetupError::Unknown(format!(
                        "Failed to mount configfs: {}",
                        stderr
                    )));
                }
            }
        }

        // Create gadget directory
        self.create_directory(GADGET_PATH)?;

        // Set vendor and product IDs
        self.write_file(&format!("{}/idVendor", GADGET_PATH), VID)?;
        self.write_file(&format!("{}/idProduct", GADGET_PATH), PID)?;

        // Set USB version
        self.write_file(&format!("{}/bcdUSB", GADGET_PATH), "0x0200")?; // USB 2.0
        self.write_file(&format!("{}/bcdDevice", GADGET_PATH), "0x0100")?;

        // Set device class
        self.write_file(&format!("{}/bDeviceClass", GADGET_PATH), "0x00")?;
        self.write_file(&format!("{}/bDeviceSubClass", GADGET_PATH), "0x00")?;
        self.write_file(&format!("{}/bDeviceProtocol", GADGET_PATH), "0x00")?;

        // Set strings
        let strings_dir = format!("{}/strings/0x409", GADGET_PATH);
        self.create_directory(&strings_dir)?;
        self.write_file(&format!("{}/serialnumber", strings_dir), "000000000001")?;
        self.write_file(&format!("{}/manufacturer", strings_dir), "Nintendo")?;
        self.write_file(&format!("{}/product", strings_dir), "Pro Controller")?;

        // Create configuration
        let config_dir = format!("{}/configs/c.1", GADGET_PATH);
        self.create_directory(&config_dir)?;
        self.write_file(&format!("{}/MaxPower", config_dir), "500")?;

        let config_strings_dir = format!("{}/strings/0x409", config_dir);
        self.create_directory(&config_strings_dir)?;
        self.write_file(&format!("{}/configuration", config_strings_dir), "Pro Controller")?;

        // Create HID function
        let hid_dir = format!("{}/functions/hid.usb0", GADGET_PATH);
        self.create_directory(&hid_dir)?;
        self.write_file(&format!("{}/protocol", hid_dir), "0")?;
        self.write_file(&format!("{}/subclass", hid_dir), "0")?;
        self.write_file(&format!("{}/report_length", hid_dir), "64")?;

        // Write HID report descriptor
        let report_desc = include_bytes!("../../domain/hardware/pro_controller_descriptor.bin");
        let report_desc_path = format!("{}/report_desc", hid_dir);
        
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&report_desc_path)
            .map_err(|e| {
                error!("Failed to open report descriptor file: {}", e);
                SetupError::FileSystemError(e)
            })?;
        
        file.write_all(report_desc).map_err(|e| {
            error!("Failed to write report descriptor: {}", e);
            SetupError::FileSystemError(e)
        })?;

        info!("Wrote HID report descriptor");

        // Link function to configuration
        let function_link = format!("{}/hid.usb0", config_dir);
        if !Path::new(&function_link).exists() {
            std::os::unix::fs::symlink(&hid_dir, &function_link).map_err(|e| {
                error!("Failed to create symlink: {}", e);
                SetupError::FileSystemError(e)
            })?;
            debug!("Linked HID function to configuration");
        }

        // Enable the gadget
        let udc_name = self.get_udc_name()?;
        self.write_file(&format!("{}/UDC", GADGET_PATH), &udc_name)?;

        info!("USB Gadget configured successfully!");

        Ok(())
    }

    fn is_gadget_configured(&self) -> Result<bool, SetupError> {
        // Check if gadget path exists
        if !Path::new(GADGET_PATH).exists() {
            return Ok(false);
        }

        // Check if UDC is set (gadget is active)
        let udc_path = format!("{}/UDC", GADGET_PATH);
        if !Path::new(&udc_path).exists() {
            return Ok(false);
        }

        let udc_content = fs::read_to_string(&udc_path)?;
        Ok(!udc_content.trim().is_empty())
    }
}