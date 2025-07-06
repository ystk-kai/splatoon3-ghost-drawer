use crate::domain::setup::entities::BoardModel;
use crate::domain::setup::repositories::{BootConfigurator, SetupError};
use std::fs;
use std::io::Write;
use std::path::Path;
use tracing::info;

pub struct LinuxBootConfigurator;

impl Default for LinuxBootConfigurator {
    fn default() -> Self {
        Self
    }
}

impl LinuxBootConfigurator {
    pub fn new() -> Self {
        Self::default()
    }

    fn configure_armbian_env(&self, board: &BoardModel) -> Result<(), SetupError> {
        let env_file = "/boot/armbianEnv.txt";
        
        if !Path::new(env_file).exists() {
            return Err(SetupError::BootConfigurationFailed(
                "ArmbianEnv.txt not found".to_string(),
            ));
        }

        // Read existing configuration
        let content = fs::read_to_string(env_file)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Check and update overlays
        let overlay_line = board
            .otg_device_tree_overlay()
            .map(|overlay| format!("overlays={}", overlay));

        if let Some(ref new_overlay_line) = overlay_line {
            let mut found = false;
            for line in &mut lines {
                if line.starts_with("overlays=") {
                    if !line.contains(board.otg_device_tree_overlay().unwrap()) {
                        *line = new_overlay_line.clone();
                        info!("Updated overlays in {}", env_file);
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                lines.push(new_overlay_line.clone());
                info!("Added overlays to {}", env_file);
            }
        }

        // Write back the configuration
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(env_file)?;
        
        for line in &lines {
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    fn configure_raspberry_pi(&self, _board: &BoardModel) -> Result<(), SetupError> {
        let config_file = "/boot/config.txt";
        
        if !Path::new(config_file).exists() {
            return Err(SetupError::BootConfigurationFailed(
                "config.txt not found".to_string(),
            ));
        }

        // Read existing configuration
        let content = fs::read_to_string(config_file)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Check if dtoverlay=dwc2 is already present
        let has_dwc2 = lines.iter().any(|line| line.trim() == "dtoverlay=dwc2");

        if !has_dwc2 {
            // Append the configuration
            let mut file = fs::OpenOptions::new()
                .append(true)
                .open(config_file)?;
            
            writeln!(file, "\n# Enable USB OTG mode for gadget")?;
            writeln!(file, "dtoverlay=dwc2")?;
            info!("Added dtoverlay=dwc2 to {}", config_file);
        }

        // Check /boot/cmdline.txt for modules-load=dwc2
        let cmdline_file = "/boot/cmdline.txt";
        if Path::new(cmdline_file).exists() {
            let cmdline = fs::read_to_string(cmdline_file)?;
            
            if !cmdline.contains("modules-load=dwc2") {
                // Insert after rootwait
                let new_cmdline = if cmdline.contains("rootwait") {
                    cmdline.replace("rootwait", "rootwait modules-load=dwc2")
                } else {
                    format!("{} modules-load=dwc2", cmdline.trim())
                };
                
                fs::write(cmdline_file, new_cmdline)?;
                info!("Added modules-load=dwc2 to {}", cmdline_file);
            }
        }

        Ok(())
    }
}

impl BootConfigurator for LinuxBootConfigurator {
    fn configure_boot_for_otg(&self, board: &BoardModel) -> Result<(), SetupError> {
        info!("Configuring boot settings for board: {:?}", board);

        match board {
            BoardModel::OrangePiZero2W => self.configure_armbian_env(board),
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => {
                self.configure_raspberry_pi(board)
            }
            BoardModel::Unknown(name) => Err(SetupError::BootConfigurationFailed(format!(
                "Unknown board model: {}",
                name
            ))),
        }
    }

    fn is_boot_configured(&self, board: &BoardModel) -> Result<bool, SetupError> {
        match board {
            BoardModel::OrangePiZero2W => {
                let env_file = "/boot/armbianEnv.txt";
                if !Path::new(env_file).exists() {
                    return Ok(false);
                }
                
                let content = fs::read_to_string(env_file)?;
                if let Some(overlay) = board.otg_device_tree_overlay() {
                    Ok(content.contains(overlay))
                } else {
                    Ok(true)
                }
            }
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => {
                let config_file = "/boot/config.txt";
                if !Path::new(config_file).exists() {
                    return Ok(false);
                }
                
                let content = fs::read_to_string(config_file)?;
                Ok(content.contains("dtoverlay=dwc2"))
            }
            BoardModel::Unknown(_) => Ok(false),
        }
    }
}