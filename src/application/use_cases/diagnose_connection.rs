use crate::domain::hardware::errors::HardwareError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

/// 接続問題を診断するユースケース
pub struct DiagnoseConnectionUseCase;

impl DiagnoseConnectionUseCase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiagnoseConnectionUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnoseConnectionUseCase {
    pub fn execute(&self) -> Result<(), HardwareError> {
        println!("🔍 Connection Diagnostics");
        println!("=======================\n");

        // 1. システム情報の確認
        self.check_system_info()?;

        // 2. ブート設定の確認
        self.check_boot_configuration()?;

        // 3. カーネルモジュールの確認
        self.check_kernel_modules()?;

        // 3.5. 競合の確認
        self.check_g_ether_conflict()?;

        // 4. UDC（USB Device Controller）の確認
        self.check_udc_status()?;

        // 5. USB Gadgetの設定確認
        self.check_gadget_configuration()?;

        // 6. HIDデバイスの確認
        self.check_hid_devices()?;

        // 7. USB OTGモードの確認
        self.check_otg_mode()?;

        // 8. サービス状態の確認
        self.check_service_status()?;

        // 9. USB接続の確認
        self.check_usb_connection()?;

        // 5. dmesgログの確認
        self.check_dmesg_logs()?;

        // 6. 推奨される対処法
        self.show_recommendations();

        Ok(())
    }

    fn check_g_ether_conflict(&self) -> Result<(), HardwareError> {
        println!("🚫 Checking for conflicts:");

        let output = Command::new("lsmod")
            .output()
            .map_err(|e| HardwareError::Unknown(format!("Failed to run lsmod: {e}")))?;
        let lsmod = String::from_utf8_lossy(&output.stdout);

        if lsmod.contains("g_ether") {
            println!("   ❌ g_ether module detected!");
            println!("      This module conflicts with the Nintendo Switch gadget.");
            println!("      Please remove 'modules-load=dwc2,g_ether' from /boot/cmdline.txt");
            println!("      or /boot/firmware/cmdline.txt and reboot.");
        } else {
            println!("   ✅ No g_ether conflict detected");
        }

        println!();
        Ok(())
    }

    fn check_kernel_modules(&self) -> Result<(), HardwareError> {
        println!("📦 Kernel Modules:");

        let required_modules = vec![
            ("libcomposite", "USB Gadget framework"),
            ("usb_f_hid", "HID function support"),
            ("dwc2", "Raspberry Pi USB driver"),
            ("musb_hdrc", "Orange Pi USB driver"),
            ("sunxi", "Allwinner platform support"),
        ];

        let lsmod_output = Command::new("lsmod")
            .output()
            .map_err(|e| HardwareError::Unknown(format!("Failed to run lsmod: {e}")))?;

        let lsmod_text = String::from_utf8_lossy(&lsmod_output.stdout);

        for (module, description) in required_modules {
            let is_loaded = lsmod_text.lines().any(|line| line.starts_with(module));
            println!(
                "   {} ({}): {}",
                module,
                description,
                if is_loaded {
                    "✅ Loaded"
                } else {
                    "❌ Not loaded"
                }
            );
        }

        println!();
        Ok(())
    }

    fn check_gadget_configuration(&self) -> Result<(), HardwareError> {
        println!("🔌 USB Gadget Configuration:");

        let gadget_path = "/sys/kernel/config/usb_gadget/nintendo_controller";

        if !Path::new(gadget_path).exists() {
            println!("   ❌ Gadget not configured");
            return Ok(());
        }

        println!("   ✅ Gadget directory exists");

        // UDCの確認
        let udc_path = format!("{gadget_path}/UDC");
        if let Ok(udc) = fs::read_to_string(&udc_path) {
            let udc = udc.trim();
            if udc.is_empty() {
                println!("   ❌ UDC not bound");
            } else {
                println!("   ✅ UDC bound to: {udc}");

                // UDCの詳細情報
                let udc_state_path = format!("/sys/class/udc/{udc}/state");
                if let Ok(state) = fs::read_to_string(&udc_state_path) {
                    println!("   📊 UDC state: {}", state.trim());
                }
            }
        } else {
            println!("   ❌ Cannot read UDC status");
        }

        // HID functionの確認
        let hid_path = format!("{gadget_path}/functions/hid.usb0");
        if Path::new(&hid_path).exists() {
            println!("   ✅ HID function configured");

            if let Ok(report_length) = fs::read_to_string(format!("{hid_path}/report_length")) {
                println!("   📏 Report length: {} bytes", report_length.trim());
            }
        } else {
            println!("   ❌ HID function not configured");
        }

        println!();
        Ok(())
    }

    fn check_hid_devices(&self) -> Result<(), HardwareError> {
        println!("🎮 HID Devices:");

        let hid_devices = vec!["/dev/hidg0", "/dev/hidg1", "/dev/hidg2", "/dev/hidg3"];

        for device in hid_devices {
            if Path::new(device).exists() {
                println!("   ✅ {device} exists");

                // 権限の確認
                if let Ok(metadata) = fs::metadata(device) {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    println!("      Permissions: {:o}", mode & 0o777);
                }

                // 書き込みテスト
                match fs::OpenOptions::new().write(true).open(device) {
                    Ok(mut file) => {
                        let test_data = [0u8; 64];
                        match file.write_all(&test_data) {
                            Ok(_) => println!("      ✅ Write test successful"),
                            Err(e) => {
                                if e.raw_os_error() == Some(108) {
                                    println!(
                                        "      ❌ Write test failed: Transport endpoint not connected"
                                    );
                                } else {
                                    println!("      ❌ Write test failed: {e}");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::PermissionDenied {
                            println!("      ❌ Permission denied (need sudo)");
                        } else {
                            println!("      ❌ Cannot open: {e}");
                        }
                    }
                }
            }
        }

        println!();
        Ok(())
    }

    fn check_otg_mode(&self) -> Result<(), HardwareError> {
        println!("🔄 USB OTG Mode:");

        // Find musb-hdrc mode files
        let musb_dirs = vec!["/sys/devices/platform/soc", "/sys/devices/platform"];

        let mut found_otg = false;

        for base_dir in musb_dirs {
            if let Ok(entries) = fs::read_dir(base_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();

                    if name.contains("usb") {
                        let mode_path = path.join("musb-hdrc.4.auto/mode");
                        if mode_path.exists() {
                            found_otg = true;
                            if let Ok(mode) = fs::read_to_string(&mode_path) {
                                let mode = mode.trim();
                                println!("   Mode: {mode}");

                                if mode == "peripheral" || mode == "b_peripheral" {
                                    println!("   ✅ USB OTG is in peripheral mode");
                                } else {
                                    println!(
                                        "   ⚠️  USB OTG is in {mode} mode (should be peripheral)"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        if !found_otg {
            // Check for Raspberry Pi dwc2
            let lsmod = Command::new("lsmod").output().ok();
            let is_dwc2_loaded = if let Some(output) = lsmod {
                String::from_utf8_lossy(&output.stdout).contains("dwc2")
            } else {
                false
            };

            if is_dwc2_loaded {
                println!("   ✅ dwc2 module loaded (Raspberry Pi)");
            } else {
                println!("   ❌ USB OTG mode file not found");
                println!("   This may indicate USB OTG is not enabled in Device Tree");
            }
        }

        println!();
        Ok(())
    }

    fn check_usb_connection(&self) -> Result<(), HardwareError> {
        println!("🔗 USB Connection Status:");

        // lsusbの出力を確認
        if let Ok(output) = Command::new("lsusb").output() {
            let lsusb = String::from_utf8_lossy(&output.stdout);
            if lsusb.contains("057e:2009") {
                println!("   ℹ️  Nintendo Pro Controller detected by host (Self-check)");
            } else {
                println!("   ℹ️  Pro Controller not detected by host (Normal for Gadget mode)");
            }
        }

        // USB gadgetの状態を確認
        let state_path = "/sys/kernel/config/usb_gadget/nintendo_controller/state";
        if let Ok(state) = fs::read_to_string(state_path) {
            println!("   📊 Gadget state: {}", state.trim());
        }

        println!();
        Ok(())
    }

    fn check_system_info(&self) -> Result<(), HardwareError> {
        println!("🖥️ System Information:");

        // Check board model
        if let Ok(model) = fs::read_to_string("/proc/device-tree/model") {
            println!("   Board Model: {}", model.trim_end_matches('\0'));
        }

        // Check kernel version
        if let Ok(version) = fs::read_to_string("/proc/version") {
            let kernel_line = version.lines().next().unwrap_or("Unknown");
            println!("   Kernel: {kernel_line}");
        }

        // Check if running as root
        let is_root = unsafe { libc::geteuid() == 0 };
        println!(
            "   Running as root: {}",
            if is_root { "✅ Yes" } else { "❌ No" }
        );

        println!();
        Ok(())
    }

    fn check_boot_configuration(&self) -> Result<(), HardwareError> {
        println!("🔧 Boot Configuration:");

        // Check config.txt files
        let config_files = vec!["/boot/firmware/config.txt", "/boot/config.txt"];
        let mut found_config = false;

        for config_file in &config_files {
            if Path::new(config_file).exists() {
                found_config = true;
                println!("   Config file: {config_file} ✅");

                if let Ok(content) = fs::read_to_string(config_file) {
                    let has_dwc2 = content.lines().any(|line| {
                        let trimmed = line.trim();
                        trimmed == "dtoverlay=dwc2" && !trimmed.starts_with('#')
                    });

                    println!(
                        "   dtoverlay=dwc2: {}",
                        if has_dwc2 { "✅ Found" } else { "❌ Missing" }
                    );

                    // Check for conflicting configurations
                    let has_dwc2_host = content.contains("dtoverlay=dwc2,dr_mode=host");
                    if has_dwc2_host {
                        println!("   ⚠️  Found conflicting dwc2 host mode configuration");
                    }
                }
                break;
            }
        }

        if !found_config {
            println!("   Config file: ❌ Not found");
        }

        // Check /etc/modules
        if Path::new("/etc/modules").exists()
            && let Ok(content) = fs::read_to_string("/etc/modules")
        {
            let has_dwc2 = content.lines().any(|line| line.trim() == "dwc2");
            let has_libcomposite = content.lines().any(|line| line.trim() == "libcomposite");

            println!(
                "   /etc/modules dwc2: {}",
                if has_dwc2 { "✅ Found" } else { "❌ Missing" }
            );
            println!(
                "   /etc/modules libcomposite: {}",
                if has_libcomposite {
                    "✅ Found"
                } else {
                    "❌ Missing"
                }
            );
        }

        // Check blacklist
        let blacklist_file = "/etc/modprobe.d/blacklist-dwc_otg.conf";
        let blacklist_exists = Path::new(blacklist_file).exists();
        println!(
            "   dwc_otg blacklisted: {}",
            if blacklist_exists {
                "✅ Yes"
            } else {
                "❌ No"
            }
        );

        println!();
        Ok(())
    }

    fn check_udc_status(&self) -> Result<(), HardwareError> {
        println!("🔌 USB Device Controller (UDC):");

        let udc_dir = "/sys/class/udc";
        if !Path::new(udc_dir).exists() {
            println!("   UDC directory: ❌ Not found");
            println!("   This indicates USB OTG is not enabled or dwc2 is not loaded");
            println!();
            return Ok(());
        }

        println!("   UDC directory: ✅ Found");

        // List available UDCs
        if let Ok(entries) = fs::read_dir(udc_dir) {
            let udcs: Vec<_> = entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.file_name().to_string_lossy().to_string())
                .collect();

            if udcs.is_empty() {
                println!("   Available UDCs: ❌ None found");
                println!("   Check if dwc2 module is loaded with correct parameters");
            } else {
                println!("   Available UDCs: ✅ {}", udcs.join(", "));
            }
        }

        println!();
        Ok(())
    }

    fn check_service_status(&self) -> Result<(), HardwareError> {
        println!("🔄 Service Status:");

        let services = vec![
            ("splatoon3-gadget.service", "USB Gadget Configuration"),
            ("splatoon3-ghost-drawer.service", "Web UI Service"),
        ];

        for (service_name, description) in services {
            if let Ok(output) = Command::new("systemctl")
                .arg("is-active")
                .arg(service_name)
                .output()
            {
                let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let status_icon = match status.as_str() {
                    "active" => "✅",
                    "inactive" => "⏸️",
                    "failed" => "❌",
                    _ => "❓",
                };

                println!("   {service_name} ({description}): {status_icon} {status}");

                // If failed, show recent logs
                if status == "failed"
                    && let Ok(log_output) = Command::new("journalctl")
                        .arg("-u")
                        .arg(service_name)
                        .arg("--no-pager")
                        .arg("-n")
                        .arg("3")
                        .output()
                {
                    let logs = String::from_utf8_lossy(&log_output.stdout);
                    if !logs.trim().is_empty() {
                        println!("     Recent logs:");
                        for line in logs.lines().take(3) {
                            println!("       {line}");
                        }
                    }
                }
            } else {
                println!("   {service_name} ({description}): ❓ Unknown");
            }
        }

        println!();
        Ok(())
    }

    fn check_dmesg_logs(&self) -> Result<(), HardwareError> {
        println!("📋 Recent USB/HID Messages:");

        if let Ok(output) = Command::new("dmesg")
            .args(["-t", "--level=err,warn"])
            .output()
        {
            let dmesg = String::from_utf8_lossy(&output.stdout);
            let relevant_lines: Vec<&str> = dmesg
                .lines()
                .rev()
                .filter(|line| {
                    line.contains("musb")
                        || line.contains("dwc2")
                        || line.contains("gadget")
                        || line.contains("hid")
                        || line.contains("USB")
                        || line.contains("nintendo")
                })
                .take(10)
                .collect();

            if relevant_lines.is_empty() {
                println!("   No recent USB/HID messages found");
            } else {
                for line in relevant_lines.iter().rev() {
                    println!("   - {line}");
                }
            }
        }

        println!();
        Ok(())
    }

    fn show_recommendations(&self) {
        println!("💡 Recommendations:");
        println!("   If connection fails on Orange Pi Zero 2W:");
        println!("   1. Ensure USB OTG is enabled in device tree");
        println!("   2. Try: sudo modprobe sunxi musb_hdrc");
        println!("   3. Restart gadget: sudo systemctl restart splatoon3-gadget.service");
        println!("   4. Check Nintendo Switch is on Home screen");
        println!("   5. Try reconnecting USB cable");
        println!();
        println!("   For detailed logs: sudo dmesg | grep -E '(musb|gadget|hid)'");
    }
}
