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
        Self
    }

    fn configure_armbian_env(&self, board: &BoardModel) -> Result<(), SetupError> {
        // Orange Pi Zero 2W uses orangepiEnv.txt, other boards might use armbianEnv.txt
        let env_files = vec!["/boot/orangepiEnv.txt", "/boot/armbianEnv.txt"];
        let mut env_file_path = None;

        for file in &env_files {
            if Path::new(file).exists() {
                env_file_path = Some(*file);
                break;
            }
        }

        let env_file = env_file_path.ok_or_else(|| {
            SetupError::BootConfigurationFailed(
                "Neither orangepiEnv.txt nor armbianEnv.txt found".to_string(),
            )
        })?;

        info!("Using boot environment file: {}", env_file);

        // Read existing configuration
        let content = fs::read_to_string(env_file)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Check and update overlays
        // For Orange Pi Zero 2W, we use "usb-otg" instead of the full overlay name
        let overlay_to_add = if matches!(board, BoardModel::OrangePiZero2W) {
            "usb-otg"
        } else if let Some(overlay) = board.otg_device_tree_overlay() {
            overlay
        } else {
            return Ok(()); // No overlay needed
        };

        let mut found = false;
        for line in &mut lines {
            if line.starts_with("overlays=") {
                let existing_overlays = line.split('=').nth(1).unwrap_or("");
                if !existing_overlays.contains(overlay_to_add) {
                    *line = format!("overlays={} {}", existing_overlays.trim(), overlay_to_add)
                        .trim()
                        .to_string();
                    info!(
                        "Updated overlays in {} (added {})",
                        env_file, overlay_to_add
                    );
                }
                found = true;
                break;
            }
        }
        if !found {
            lines.push(format!("overlays={overlay_to_add}"));
            info!("Added overlays={} to {}", overlay_to_add, env_file);
        }

        // Add USB OTG mode parameter for Orange Pi Zero 2W
        if matches!(board, BoardModel::OrangePiZero2W) {
            // Check for param_dwc2_dr_mode
            let mut found_dr_mode = false;
            for line in &mut lines {
                if line.starts_with("param_dwc2_dr_mode=") {
                    if line != "param_dwc2_dr_mode=otg" {
                        *line = "param_dwc2_dr_mode=otg".to_string();
                        info!("Updated param_dwc2_dr_mode to otg");
                    }
                    found_dr_mode = true;
                    break;
                }
            }
            if !found_dr_mode {
                lines.push("param_dwc2_dr_mode=otg".to_string());
                info!("Added param_dwc2_dr_mode=otg to {}", env_file);
            }
        }

        // Write back the configuration
        let mut file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(env_file)?;

        for line in &lines {
            writeln!(file, "{line}")?;
        }

        Ok(())
    }

    fn configure_raspberry_pi(&self, _board: &BoardModel) -> Result<(), SetupError> {
        // Check both possible locations for config.txt
        let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];
        let mut config_file = None;
        
        for file in &config_files {
            if Path::new(file).exists() {
                config_file = Some(*file);
                break;
            }
        }
        
        let config_path = config_file.ok_or_else(|| {
            SetupError::BootConfigurationFailed("config.txt not found in /boot or /boot/firmware".to_string())
        })?;

        // Read existing configuration
        let content = fs::read_to_string(config_path)?;
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Check if dtoverlay=dwc2 is already present
        let has_dwc2 = lines.iter().any(|line| line.trim() == "dtoverlay=dwc2");

        if !has_dwc2 {
            // Append the configuration
            let mut file = fs::OpenOptions::new().append(true).open(config_path)?;

            writeln!(file, "\n# Enable USB OTG mode for gadget")?;
            writeln!(file, "dtoverlay=dwc2")?;
            info!("Added dtoverlay=dwc2 to {}", config_path);
        }

        // Add dwc2 to /etc/modules
        let modules_file = "/etc/modules";
        if Path::new(modules_file).exists() {
            let modules_content = fs::read_to_string(modules_file)?;
            if !modules_content.lines().any(|line| line.trim() == "dwc2") {
                let mut file = fs::OpenOptions::new().append(true).open(modules_file)?;
                writeln!(file, "dwc2")?;
                info!("Added dwc2 to /etc/modules");
            }
        }

        // Blacklist dwc_otg to prevent conflicts
        let blacklist_file = "/etc/modprobe.d/blacklist-dwc_otg.conf";
        if !Path::new(blacklist_file).exists() {
            fs::write(blacklist_file, "blacklist dwc_otg\n")?;
            info!("Created {} to blacklist dwc_otg", blacklist_file);
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
                "Unknown board model: {name}"
            ))),
        }
    }

    fn is_boot_configured(&self, board: &BoardModel) -> Result<bool, SetupError> {
        match board {
            BoardModel::OrangePiZero2W => {
                // Check both possible env files
                let env_files = vec!["/boot/orangepiEnv.txt", "/boot/armbianEnv.txt"];

                for env_file in env_files {
                    if Path::new(env_file).exists() {
                        let content = fs::read_to_string(env_file)?;
                        // Check for "usb-otg" which is what we actually add
                        return Ok(content.contains("usb-otg"));
                    }
                }

                Ok(false)
            }
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => {
                // Check both possible locations for config.txt
                let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];
                
                for config_file in config_files {
                    if Path::new(config_file).exists() {
                        let content = fs::read_to_string(config_file)?;
                        if content.contains("dtoverlay=dwc2") {
                            // Also check if dwc_otg is blacklisted
                            let blacklist_exists = Path::new("/etc/modprobe.d/blacklist-dwc_otg.conf").exists();
                            return Ok(blacklist_exists);
                        }
                    }
                }
                
                Ok(false)
            }
            BoardModel::Unknown(_) => Ok(false),
        }
    }

    fn remove_boot_configuration(&self, board: &BoardModel) -> Result<(), SetupError> {
        info!("Removing boot configuration for board: {:?}", board);

        match board {
            BoardModel::OrangePiZero2W => {
                let env_files = vec!["/boot/orangepiEnv.txt", "/boot/armbianEnv.txt"];

                for env_file in env_files {
                    if !Path::new(env_file).exists() {
                        continue;
                    }

                    let content = fs::read_to_string(env_file)?;
                    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    let mut modified = false;

                    // Remove "usb-otg" overlay
                    for line in &mut lines {
                        if line.starts_with("overlays=") && line.contains("usb-otg") {
                            // Remove the overlay from the line
                            let overlays: Vec<&str> = line[9..]
                                .split(' ')
                                .filter(|s| !s.contains("usb-otg"))
                                .collect();

                            if overlays.is_empty() {
                                *line = String::new();
                            } else {
                                *line = format!("overlays={}", overlays.join(" "));
                            }
                            modified = true;
                            break;
                        }
                    }

                    // Also remove param_dwc2_dr_mode line
                    lines.retain(|line| !line.starts_with("param_dwc2_dr_mode="));
                    if lines.len() != content.lines().count() {
                        modified = true;
                    }

                    if modified {
                        // Remove empty lines
                        lines.retain(|line| !line.is_empty());

                        let mut file = fs::OpenOptions::new()
                            .write(true)
                            .truncate(true)
                            .open(env_file)?;

                        for line in &lines {
                            writeln!(file, "{line}")?;
                        }
                        info!("Removed USB OTG configuration from {}", env_file);
                    }
                }

                Ok(())
            }
            BoardModel::RaspberryPiZero | BoardModel::RaspberryPiZero2W => {
                // Check both possible locations for config.txt
                let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];
                
                for config_file in config_files {
                    if !Path::new(config_file).exists() {
                        continue;
                    }

                    let content = fs::read_to_string(config_file)?;
                    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                    let mut new_lines = Vec::new();
                    let mut skip_next = false;

                    for line in lines {
                        if skip_next && line.trim().is_empty() {
                            skip_next = false;
                            continue;
                        }
                        skip_next = false;

                        if line.trim() == "dtoverlay=dwc2" {
                            skip_next = true;
                            continue;
                        }
                        if line.contains("Enable USB OTG mode for gadget") {
                            continue;
                        }
                        new_lines.push(line);
                    }

                    fs::write(config_file, new_lines.join("\n"))?;
                    info!("Removed dtoverlay=dwc2 from {}", config_file);
                }

                // Remove dwc2 from /etc/modules
                let modules_file = "/etc/modules";
                if Path::new(modules_file).exists() {
                    let content = fs::read_to_string(modules_file)?;
                    let lines: Vec<&str> = content.lines()
                        .filter(|line| line.trim() != "dwc2")
                        .collect();
                    fs::write(modules_file, lines.join("\n"))?;
                    info!("Removed dwc2 from /etc/modules");
                }

                // Remove blacklist file
                let blacklist_file = "/etc/modprobe.d/blacklist-dwc_otg.conf";
                if Path::new(blacklist_file).exists() {
                    fs::remove_file(blacklist_file)?;
                    info!("Removed {}", blacklist_file);
                }

                Ok(())
            }
            BoardModel::Unknown(name) => {
                info!(
                    "No boot configuration to remove for unknown board: {}",
                    name
                );
                Ok(())
            }
        }
    }
}
