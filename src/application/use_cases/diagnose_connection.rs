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

        // 1. カーネルモジュールの確認
        self.check_kernel_modules()?;

        // 2. USB Gadgetの設定確認
        self.check_gadget_configuration()?;

        // 3. HIDデバイスの確認
        self.check_hid_devices()?;

        // 4. USB OTGモードの確認
        self.check_otg_mode()?;

        // 5. USB接続の確認
        self.check_usb_connection()?;

        // 5. dmesgログの確認
        self.check_dmesg_logs()?;

        // 6. 推奨される対処法
        self.show_recommendations();

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
            println!("   ❌ USB OTG mode file not found");
            println!("   This may indicate USB OTG is not enabled in Device Tree");
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
                println!("   ✅ Nintendo Pro Controller detected by host");
            } else {
                println!("   ❌ Pro Controller not detected by host");
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
