use crate::domain::hardware::{Board, BoardModel, BoardRepository, HardwareError};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::process::Command;
use tracing::{debug, info, warn};
use std::os::unix::process::ExitStatusExt;

pub struct LinuxBoardDetector;

impl LinuxBoardDetector {
    pub fn new() -> Self {
        Self
    }

    async fn read_cpu_info(&self) -> Result<(String, String), HardwareError> {
        let cpu_info = fs::read_to_string("/proc/cpuinfo")
            .await
            .map_err(|e| HardwareError::FileOperationFailed(format!("Failed to read /proc/cpuinfo: {}", e)))?;

        let mut model = String::new();
        let mut hardware = String::new();

        for line in cpu_info.lines() {
            if line.starts_with("Model") || line.starts_with("model name") {
                model = line.split(':').nth(1).unwrap_or("").trim().to_string();
            } else if line.starts_with("Hardware") {
                hardware = line.split(':').nth(1).unwrap_or("").trim().to_string();
            }
        }

        Ok((model, hardware))
    }

    async fn check_module_loaded(&self, module_name: &str) -> bool {
        let output = Command::new("lsmod")
            .output()
            .await
            .unwrap_or_else(|_| std::process::Output {
                status: std::process::ExitStatus::from_raw(1),
                stdout: Vec::new(),
                stderr: Vec::new(),
            });

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.lines().any(|line| line.starts_with(module_name))
        } else {
            false
        }
    }

    async fn check_usb_otg_status(&self) -> bool {
        // Check if dwc2 module is loaded
        let dwc2_loaded = self.check_module_loaded("dwc2").await;
        
        // Check if /sys/kernel/config/usb_gadget exists
        let gadget_path = Path::new("/sys/kernel/config/usb_gadget");
        let gadget_exists = gadget_path.exists();

        // Check if any UDC is available
        let udc_path = Path::new("/sys/class/udc");
        let udc_available = if udc_path.exists() {
            match fs::read_dir(udc_path).await {
                Ok(mut entries) => {
                    if entries.next_entry().await.is_ok() {
                        true
                    } else {
                        false
                    }
                }
                Err(_) => false,
            }
        } else {
            false
        };

        dwc2_loaded && gadget_exists && udc_available
    }
}

#[async_trait]
impl BoardRepository for LinuxBoardDetector {
    async fn detect_board(&self) -> Result<Board, HardwareError> {
        let (model_info, hardware_info) = self.read_cpu_info().await?;
        debug!("CPU Model: {}, Hardware: {}", model_info, hardware_info);

        let board_model = BoardModel::from_cpu_info(&model_info, &hardware_info);
        
        if matches!(board_model, BoardModel::Unknown) {
            warn!("Unknown board model detected");
        }

        let mut board = Board::new(board_model);
        
        // Check USB OTG availability
        board.usb_otg_available = self.check_usb_otg_status().await;
        
        // Check kernel modules
        for module in &mut board.kernel_modules {
            module.loaded = self.check_module_loaded(&module.name).await;
        }

        Ok(board)
    }

    async fn check_kernel_modules(&self, board: &mut Board) -> Result<(), HardwareError> {
        for module in &mut board.kernel_modules {
            module.loaded = self.check_module_loaded(&module.name).await;
            debug!("Module {} loaded: {}", module.name, module.loaded);
        }
        Ok(())
    }

    async fn load_kernel_module(&self, module_name: &str) -> Result<(), HardwareError> {
        info!("Loading kernel module: {}", module_name);
        
        let output = Command::new("sudo")
            .args(&["modprobe", module_name])
            .output()
            .await
            .map_err(|e| HardwareError::SystemCommandFailed(format!("Failed to execute modprobe: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(HardwareError::KernelModuleNotLoaded(format!("{}: {}", module_name, stderr)));
        }

        Ok(())
    }

    async fn configure_boot_settings(&self, board: &Board) -> Result<(), HardwareError> {
        let config_path = board.model.config_file_path()
            .ok_or_else(|| HardwareError::BoardNotSupported("No config file path for this board".to_string()))?;

        let dtoverlay = board.model.required_dtoverlay()
            .ok_or_else(|| HardwareError::BoardNotSupported("No dtoverlay for this board".to_string()))?;

        // Check if running with sufficient privileges
        let euid = unsafe { libc::geteuid() };
        if euid != 0 {
            return Err(HardwareError::PermissionDenied);
        }

        // Read current config
        let config_content = fs::read_to_string(config_path)
            .await
            .map_err(|e| HardwareError::FileOperationFailed(format!("Failed to read {}: {}", config_path, e)))?;

        // Check if already configured
        if config_content.contains(dtoverlay) {
            info!("Boot configuration already contains {}", dtoverlay);
            return Ok(());
        }

        // Append configuration
        let new_content = format!("{}\n{}\n", config_content.trim_end(), dtoverlay);
        
        fs::write(config_path, &new_content)
            .await
            .map_err(|e| HardwareError::FileOperationFailed(format!("Failed to write {}: {}", config_path, e)))?;

        info!("Boot configuration updated. Reboot required to apply changes.");
        Ok(())
    }
}