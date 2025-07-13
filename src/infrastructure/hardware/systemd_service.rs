use crate::domain::hardware::{
    HardwareError, SystemdService, SystemdServiceRepository, SystemdServiceState,
};
use async_trait::async_trait;
use tokio::fs;
use tokio::process::Command;
use tracing::info;

pub struct SystemdServiceManager;

impl Default for SystemdServiceManager {
    fn default() -> Self {
        Self
    }
}

impl SystemdServiceManager {
    pub fn new() -> Self {
        Self
    }

    async fn run_systemctl(&self, args: &[&str]) -> Result<std::process::Output, HardwareError> {
        let output = Command::new("sudo")
            .arg("systemctl")
            .args(args)
            .output()
            .await
            .map_err(|e| {
                HardwareError::SystemCommandFailed(format!("Failed to run systemctl: {}", e))
            })?;

        Ok(output)
    }

    fn create_service_content(service: &SystemdService) -> String {
        format!(
            r#"[Unit]
Description=Nintendo Switch Pro Controller USB Gadget
After=network.target

[Service]
Type=oneshot
RemainAfterExit=yes
ExecStart={}
ExecStop=/usr/local/bin/remove-nintendo-controller.sh
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
"#,
            service.exec_start
        )
    }
}

#[async_trait]
impl SystemdServiceRepository for SystemdServiceManager {
    async fn create_service(&self, service: &SystemdService) -> Result<(), HardwareError> {
        let content = Self::create_service_content(service);

        // Write service file
        fs::write(&service.unit_file_path, &content)
            .await
            .map_err(|e| {
                HardwareError::FileOperationFailed(format!(
                    "Failed to write service file {}: {}",
                    service.unit_file_path, e
                ))
            })?;

        // Create the setup script
        let setup_script = r#"#!/bin/bash
# Nintendo Controller USB Gadget Setup Script

set -e

GADGET_PATH="/sys/kernel/config/usb_gadget/nintendo_controller"

# Load necessary modules
modprobe libcomposite || true

# Check if gadget already exists
if [ -d "$GADGET_PATH" ]; then
    echo "Gadget already exists, skipping creation"
    exit 0
fi

# Create gadget
mkdir -p $GADGET_PATH
cd $GADGET_PATH

# Set USB IDs
echo 0x057e > idVendor          # Nintendo
echo 0x2009 > idProduct         # Pro Controller
echo 0x0100 > bcdDevice
echo 0x0200 > bcdUSB

# Set strings
mkdir -p strings/0x409
echo "000000000001" > strings/0x409/serialnumber
echo "Nintendo Co., Ltd." > strings/0x409/manufacturer
echo "Pro Controller" > strings/0x409/product

# Create configuration
mkdir -p configs/c.1/strings/0x409
echo "Nintendo Switch Pro Controller" > configs/c.1/strings/0x409/configuration
echo 500 > configs/c.1/MaxPower

# Create HID function
mkdir -p functions/hid.usb0
echo 0 > functions/hid.usb0/protocol
echo 0 > functions/hid.usb0/subclass
echo 8 > functions/hid.usb0/report_length

# Set report descriptor
echo -ne '\x05\x01\x09\x05\xA1\x01\x05\x09\x19\x01\x29\x10\x15\x00\x25\x01\x75\x01\x95\x10\x81\x02\x05\x01\x09\x39\x15\x00\x25\x07\x75\x04\x95\x01\x81\x42\x75\x04\x95\x01\x81\x01\x05\x01\x09\x30\x09\x31\x09\x33\x09\x34\x15\x00\x26\xFF\x00\x75\x08\x95\x04\x81\x02\xC0' > functions/hid.usb0/report_desc

# Link function to configuration
ln -s functions/hid.usb0 configs/c.1/

# Activate gadget
UDC=$(ls /sys/class/udc | head -n1)
if [ -n "$UDC" ]; then
    echo $UDC > UDC
    echo "USB Gadget activated with UDC: $UDC"
else
    echo "No UDC found!"
    exit 1
fi
"#;

        let setup_script_path = "/usr/local/bin/setup-nintendo-controller.sh";
        fs::write(setup_script_path, setup_script)
            .await
            .map_err(|e| {
                HardwareError::FileOperationFailed(format!("Failed to write setup script: {}", e))
            })?;

        // Make script executable
        let output = Command::new("sudo")
            .args(["chmod", "+x", setup_script_path])
            .output()
            .await
            .map_err(|e| {
                HardwareError::SystemCommandFailed(format!(
                    "Failed to make script executable: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(HardwareError::SystemdServiceFailed(
                "Failed to make setup script executable".to_string(),
            ));
        }

        // Create removal script
        let remove_script = r#"#!/bin/bash
# Nintendo Controller USB Gadget Removal Script

GADGET_PATH="/sys/kernel/config/usb_gadget/nintendo_controller"

if [ ! -d "$GADGET_PATH" ]; then
    echo "Gadget does not exist"
    exit 0
fi

cd $GADGET_PATH

# Deactivate gadget
echo "" > UDC || true

# Remove symlinks
rm -f configs/c.1/hid.usb0

# Remove directories
rmdir functions/hid.usb0
rmdir configs/c.1/strings/0x409
rmdir configs/c.1
rmdir strings/0x409
cd ..
rmdir nintendo_controller

echo "USB Gadget removed"
"#;

        let remove_script_path = "/usr/local/bin/remove-nintendo-controller.sh";
        fs::write(remove_script_path, remove_script)
            .await
            .map_err(|e| {
                HardwareError::FileOperationFailed(format!("Failed to write removal script: {}", e))
            })?;

        // Make removal script executable
        Command::new("sudo")
            .args(["chmod", "+x", remove_script_path])
            .output()
            .await
            .map_err(|e| {
                HardwareError::SystemCommandFailed(format!(
                    "Failed to make removal script executable: {}",
                    e
                ))
            })?;

        info!("Systemd service created: {}", service.name);
        Ok(())
    }

    async fn enable_service(&self, service: &mut SystemdService) -> Result<(), HardwareError> {
        let output = self.run_systemctl(&["enable", &service.name]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HardwareError::SystemdServiceFailed(format!(
                "Failed to enable service: {}",
                stderr
            )));
        }

        service.state = SystemdServiceState::Enabled;
        info!("Systemd service enabled: {}", service.name);
        Ok(())
    }

    async fn start_service(&self, service: &mut SystemdService) -> Result<(), HardwareError> {
        let output = self.run_systemctl(&["start", &service.name]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HardwareError::SystemdServiceFailed(format!(
                "Failed to start service: {}",
                stderr
            )));
        }

        service.state = SystemdServiceState::Running;
        info!("Systemd service started: {}", service.name);
        Ok(())
    }

    async fn stop_service(&self, service: &mut SystemdService) -> Result<(), HardwareError> {
        let output = self.run_systemctl(&["stop", &service.name]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HardwareError::SystemdServiceFailed(format!(
                "Failed to stop service: {}",
                stderr
            )));
        }

        service.state = SystemdServiceState::Installed;
        info!("Systemd service stopped: {}", service.name);
        Ok(())
    }

    async fn get_service_state(&self, service_name: &str) -> Result<SystemdService, HardwareError> {
        let output = self.run_systemctl(&["is-active", service_name]).await?;
        let is_active = output.status.success();

        let output = self.run_systemctl(&["is-enabled", service_name]).await?;
        let is_enabled = output.status.success();

        let state = match (is_active, is_enabled) {
            (true, _) => SystemdServiceState::Running,
            (false, true) => SystemdServiceState::Enabled,
            (false, false) => {
                // Check if service file exists
                let service_path = format!("/etc/systemd/system/{}.service", service_name);
                if std::path::Path::new(&service_path).exists() {
                    SystemdServiceState::Installed
                } else {
                    SystemdServiceState::NotInstalled
                }
            }
        };

        let mut service = SystemdService::new(service_name);
        service.state = state;
        Ok(service)
    }

    async fn reload_daemon(&self) -> Result<(), HardwareError> {
        let output = self.run_systemctl(&["daemon-reload"]).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HardwareError::SystemdServiceFailed(format!(
                "Failed to reload systemd daemon: {}",
                stderr
            )));
        }

        info!("Systemd daemon reloaded");
        Ok(())
    }
}
