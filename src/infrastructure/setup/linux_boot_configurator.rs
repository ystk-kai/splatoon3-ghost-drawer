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
        info!("Configuring Raspberry Pi for USB gadget mode...");

        // Step 1: Handle config.txt configuration with comprehensive conflict resolution
        self.configure_config_txt()?;

        // Step 2: Configure kernel modules
        self.configure_kernel_modules()?;

        // Step 3: Handle dwc_otg conflicts
        self.handle_dwc_otg_conflicts()?;

        // Step 4: Force immediate module loading for testing
        self.force_load_modules()?;

        info!("Raspberry Pi USB gadget configuration completed");
        Ok(())
    }

    fn configure_config_txt(&self) -> Result<(), SetupError> {
        // Check both possible locations for config.txt
        let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];
        let mut config_path = None;

        for file in &config_files {
            if Path::new(file).exists() {
                config_path = Some(*file);
                break;
            }
        }

        let config_file = config_path.ok_or_else(|| {
            SetupError::BootConfigurationFailed(
                "config.txt not found in /boot or /boot/firmware".to_string(),
            )
        })?;

        info!("Configuring {} for USB OTG", config_file);

        // Create backup before modification
        self.create_config_backup(config_file)?;

        // Read and parse existing configuration
        let content = fs::read_to_string(config_file)?;
        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

        // Check if configuration is already correct
        if self.is_config_already_correct(&lines)? {
            info!("Configuration is already correct, no changes needed");
            return Ok(());
        }

        // Remove any conflicting configurations first
        self.remove_conflicting_config(&mut lines)?;

        // Add our configuration in the [all] section
        self.add_gadget_config(&mut lines)?;

        // Write back the modified configuration
        fs::write(config_file, lines.join("\n"))?;
        info!("Updated {} with USB gadget configuration", config_file);

        Ok(())
    }

    fn create_config_backup(&self, config_file: &str) -> Result<(), SetupError> {
        let backup_file = format!("{config_file}.splatoon3-backup");

        // Only create backup if it doesn't exist
        if !Path::new(&backup_file).exists() {
            fs::copy(config_file, &backup_file)?;
            info!("Created backup at {}", backup_file);
        }

        Ok(())
    }

    fn is_config_already_correct(&self, lines: &[String]) -> Result<bool, SetupError> {
        let mut in_all_section = false;
        let mut found_dwc2 = false;
        let mut found_our_comment = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed == "[all]" {
                in_all_section = true;
            } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                in_all_section = false;
            } else if in_all_section {
                if trimmed == "dtoverlay=dwc2" {
                    found_dwc2 = true;
                } else if trimmed == "# Splatoon3 Ghost Drawer USB Gadget Configuration" {
                    found_our_comment = true;
                }
            }
        }

        Ok(found_dwc2 && found_our_comment)
    }

    fn restore_config_backup(&self) -> Result<(), SetupError> {
        let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];

        for config_file in config_files {
            if Path::new(config_file).exists() {
                let backup_file = format!("{config_file}.splatoon3-backup");

                if Path::new(&backup_file).exists() {
                    fs::copy(&backup_file, config_file)?;
                    info!("Restored {} from backup", config_file);
                    return Ok(());
                }
            }
        }

        Err(SetupError::BootConfigurationFailed(
            "No backup found".to_string(),
        ))
    }

    fn remove_conflicting_config(&self, lines: &mut Vec<String>) -> Result<(), SetupError> {
        let mut i = 0;
        let mut in_cm4_section = false;
        let mut in_cm5_section = false;

        while i < lines.len() {
            let line = lines[i].trim().to_string();

            // Track which section we're in
            if line == "[cm4]" {
                in_cm4_section = true;
                in_cm5_section = false;
            } else if line == "[cm5]" {
                in_cm4_section = false;
                in_cm5_section = true;
            } else if line == "[all]" || (line.starts_with('[') && line.ends_with(']')) {
                in_cm4_section = false;
                in_cm5_section = false;
            }

            // Remove conflicting dwc2 configurations
            if line.contains("dtoverlay=dwc2") {
                if in_cm5_section && line.contains("dr_mode=host") {
                    // Keep CM5 host mode but add a comment
                    let commented = format!("# {}", lines[i]);
                    info!("Commented out conflicting CM5 host mode: {}", line);
                    lines[i] = commented;
                } else if !in_cm4_section && !in_cm5_section {
                    // Remove any standalone dwc2 overlays outside sections
                    info!("Removed conflicting dtoverlay: {}", line);
                    lines.remove(i);
                    continue; // Don't increment i since we removed a line
                }
            }

            // Remove duplicate comments and empty lines from previous runs
            if line.contains("Enable USB OTG mode")
                || line.contains("Enable USB gadget mode")
                || line.contains("Splatoon3 Ghost Drawer USB Gadget Configuration")
            {
                lines.remove(i);
                continue;
            }

            i += 1;
        }

        Ok(())
    }

    fn add_gadget_config(&self, lines: &mut Vec<String>) -> Result<(), SetupError> {
        // Find the [all] section or add it
        let mut all_section_index = None;

        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "[all]" {
                all_section_index = Some(i);
                break;
            }
        }

        let insert_index = if let Some(index) = all_section_index {
            // Find the end of the [all] section
            let mut end_index = lines.len();
            for (i, line) in lines.iter().enumerate().skip(index + 1) {
                if line.trim().starts_with('[') && line.trim().ends_with(']') {
                    end_index = i;
                    break;
                }
            }
            end_index
        } else {
            // Add [all] section at the end
            lines.push("".to_string());
            lines.push("[all]".to_string());
            lines.len()
        };

        // Check if our configuration already exists in the [all] section
        let mut has_gadget_config = false;
        if let Some(all_idx) = all_section_index {
            for i in (all_idx + 1)..insert_index {
                if i < lines.len() && lines[i].trim() == "dtoverlay=dwc2" {
                    has_gadget_config = true;
                    break;
                }
            }
        }

        if !has_gadget_config {
            // Remove any trailing empty lines before adding our config
            while insert_index > 0
                && lines
                    .get(insert_index - 1)
                    .is_some_and(|l| l.trim().is_empty())
            {
                lines.remove(insert_index - 1);
            }

            let final_insert_index = if let Some(all_idx) = all_section_index {
                // Find the actual end of [all] section after cleanup
                let mut end_idx = lines.len();
                for (i, line) in lines.iter().enumerate().skip(all_idx + 1) {
                    if line.trim().starts_with('[') && line.trim().ends_with(']') {
                        end_idx = i;
                        break;
                    }
                }
                end_idx
            } else {
                lines.len()
            };

            lines.insert(final_insert_index, "".to_string());
            lines.insert(
                final_insert_index + 1,
                "# Splatoon3 Ghost Drawer USB Gadget Configuration".to_string(),
            );
            lines.insert(final_insert_index + 2, "dtoverlay=dwc2".to_string());
            info!("Added USB gadget configuration to [all] section");
        } else {
            info!("USB gadget configuration already exists in [all] section");
        }

        Ok(())
    }

    fn configure_kernel_modules(&self) -> Result<(), SetupError> {
        info!("Configuring kernel modules for USB gadget");

        let modules_file = "/etc/modules";
        let required_modules = vec!["dwc2", "libcomposite"];

        // Create /etc/modules if it doesn't exist
        if !Path::new(modules_file).exists() {
            fs::write(modules_file, "")?;
            info!("Created {}", modules_file);
        }

        let mut content = fs::read_to_string(modules_file)?;
        let mut modified = false;

        for module in &required_modules {
            if !content.lines().any(|line| line.trim() == *module) {
                if !content.is_empty() && !content.ends_with('\n') {
                    content.push('\n');
                }
                content.push_str(&format!("{module}\n"));
                modified = true;
                info!("Added {} to {}", module, modules_file);
            }
        }

        if modified {
            fs::write(modules_file, content)?;
        }

        Ok(())
    }

    fn handle_dwc_otg_conflicts(&self) -> Result<(), SetupError> {
        info!("Handling dwc_otg conflicts");

        // Create modprobe.d directory if it doesn't exist
        let modprobe_dir = "/etc/modprobe.d";
        if !Path::new(modprobe_dir).exists() {
            fs::create_dir_all(modprobe_dir)?;
            info!("Created {}", modprobe_dir);
        }

        // Blacklist dwc_otg
        let blacklist_file = "/etc/modprobe.d/blacklist-dwc_otg.conf";
        let blacklist_content = "# Splatoon3 Ghost Drawer: Blacklist dwc_otg to prevent conflicts with dwc2 gadget mode\nblacklist dwc_otg\n";

        if !Path::new(blacklist_file).exists()
            || fs::read_to_string(blacklist_file).unwrap_or_default() != blacklist_content
        {
            fs::write(blacklist_file, blacklist_content)?;
            info!("Created/updated {}", blacklist_file);
        }

        // Also create a prefer-dwc2 configuration
        let prefer_file = "/etc/modprobe.d/prefer-dwc2.conf";
        let prefer_content = "# Splatoon3 Ghost Drawer: Prefer dwc2 over dwc_otg\ninstall dwc_otg /bin/true\nalias usb-otg dwc2\n";

        if !Path::new(prefer_file).exists() {
            fs::write(prefer_file, prefer_content)?;
            info!("Created {}", prefer_file);
        }

        Ok(())
    }

    fn force_load_modules(&self) -> Result<(), SetupError> {
        info!("Force loading required modules for immediate testing");

        let modules = vec!["dwc2", "libcomposite", "usb_f_hid"];

        for module in &modules {
            // Try to unload dwc_otg first if it's loaded
            if *module == "dwc2" {
                let _ = std::process::Command::new("modprobe")
                    .arg("-r")
                    .arg("dwc_otg")
                    .output();
            }

            let output = std::process::Command::new("modprobe")
                .arg(module)
                .output()
                .map_err(|e| SetupError::Unknown(format!("Failed to execute modprobe: {e}")))?;

            if output.status.success() {
                info!("Successfully loaded module: {}", module);
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!(
                    "Module {} may already be loaded or unavailable: {}",
                    module, stderr
                );
            }
        }

        Ok(())
    }

    fn check_raspberry_pi_configuration(&self) -> Result<bool, SetupError> {
        // Check 1: config.txt has dtoverlay=dwc2 in [all] section
        let config_ok = self.check_config_txt_configuration()?;

        // Check 2: Required modules in /etc/modules
        let modules_ok = self.check_modules_configuration()?;

        // Check 3: dwc_otg conflicts handled
        let conflicts_ok = self.check_conflict_resolution()?;

        Ok(config_ok && modules_ok && conflicts_ok)
    }

    fn check_config_txt_configuration(&self) -> Result<bool, SetupError> {
        let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];

        for config_file in config_files {
            if !Path::new(config_file).exists() {
                continue;
            }

            let content = fs::read_to_string(config_file)?;
            let lines: Vec<&str> = content.lines().collect();

            let mut in_all_section = false;
            let mut found_gadget_config = false;

            for line in lines {
                let trimmed = line.trim();

                if trimmed == "[all]" {
                    in_all_section = true;
                } else if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    in_all_section = false;
                } else if in_all_section && trimmed == "dtoverlay=dwc2" {
                    found_gadget_config = true;
                    break;
                }
            }

            return Ok(found_gadget_config);
        }

        Ok(false)
    }

    fn check_modules_configuration(&self) -> Result<bool, SetupError> {
        let modules_file = "/etc/modules";
        if !Path::new(modules_file).exists() {
            return Ok(false);
        }

        let content = fs::read_to_string(modules_file)?;
        let required_modules = vec!["dwc2", "libcomposite"];

        for module in required_modules {
            if !content.lines().any(|line| line.trim() == module) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn check_conflict_resolution(&self) -> Result<bool, SetupError> {
        // Check if blacklist file exists and has correct content
        let blacklist_file = "/etc/modprobe.d/blacklist-dwc_otg.conf";
        let prefer_file = "/etc/modprobe.d/prefer-dwc2.conf";

        let blacklist_ok = Path::new(blacklist_file).exists()
            && fs::read_to_string(blacklist_file)
                .unwrap_or_default()
                .contains("blacklist dwc_otg");

        let prefer_ok = Path::new(prefer_file).exists();

        Ok(blacklist_ok && prefer_ok)
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
                // Check comprehensive configuration
                self.check_raspberry_pi_configuration()
            }
            BoardModel::Unknown(_) => Ok(false),
        }
    }

    fn remove_boot_configuration(&self, board: &BoardModel) -> Result<(), SetupError> {
        info!("Removing boot configuration for board: {:?}", board);

        // Try to restore from backup first
        if let Ok(()) = self.restore_config_backup() {
            info!("Successfully restored configuration from backup");
        } else {
            info!("No backup found, manually removing configuration");
        }

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
                    let lines: Vec<&str> = content
                        .lines()
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
