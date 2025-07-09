use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::entities::BoardModel;
use crate::domain::setup::repositories::{BoardDetector, SetupError};
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// システム情報を表示するユースケース
pub struct ShowSystemInfoUseCase<D: BoardDetector, G: UsbGadgetManager> {
    board_detector: Arc<D>,
    gadget_manager: Arc<G>,
}

impl<D: BoardDetector, G: UsbGadgetManager> ShowSystemInfoUseCase<D, G> {
    pub fn new(board_detector: Arc<D>, gadget_manager: Arc<G>) -> Self {
        Self {
            board_detector,
            gadget_manager,
        }
    }

    pub fn execute(&self, verbose: bool) -> Result<(), SetupError> {
        println!("🔍 System Information");
        println!("====================");
        
        // ボード情報
        self.show_board_info(verbose)?;
        
        // USB Gadget情報
        self.show_usb_gadget_info(verbose)?;
        
        // HIDデバイス情報
        self.show_hid_device_info(verbose)?;
        
        // systemdサービス情報
        self.show_systemd_service_info(verbose)?;
        
        if verbose {
            // カーネルモジュール情報
            self.show_kernel_module_info()?;
            
            // USB関連の詳細情報
            self.show_usb_detail_info()?;
            
            // USB Gadgetのデバッグ情報
            self.show_gadget_debug_info()?;
        }
        
        Ok(())
    }
    
    fn show_board_info(&self, verbose: bool) -> Result<(), SetupError> {
        println!("\n📋 Board Information:");
        
        match self.board_detector.detect_board() {
            Ok(board) => {
                let model_str = match &board {
                    BoardModel::OrangePiZero2W => "Orange Pi Zero 2W",
                    BoardModel::RaspberryPiZero => "Raspberry Pi Zero",
                    BoardModel::RaspberryPiZero2W => "Raspberry Pi Zero 2W",
                    BoardModel::Unknown(s) => s,
                };
                println!("   Model: {}", model_str);
                
                // All supported boards have USB OTG
                let has_otg = !matches!(board, BoardModel::Unknown(_));
                println!("   USB OTG Support: {}", if has_otg { "✅ Yes" } else { "❌ No" });
                
                if verbose {
                    println!("   Details:");
                    println!("      - Device tree overlay: {}", board.otg_device_tree_overlay().unwrap_or("None"));
                    println!("      - Requires config.txt: {}", if board.requires_config_txt() { "Yes" } else { "No" });
                    println!("      - USB device path: {}", board.usb_device_path());
                }
            }
            Err(e) => {
                println!("   ❌ Failed to detect board: {}", e);
            }
        }
        
        Ok(())
    }
    
    fn show_usb_gadget_info(&self, verbose: bool) -> Result<(), SetupError> {
        println!("\n🔌 USB Gadget Status:");
        
        // Gadget設定確認
        match self.gadget_manager.is_gadget_configured() {
            Ok(configured) => {
                if configured {
                    println!("   Configuration: ✅ Configured");
                    
                    // UDC状態確認
                    let udc_path = Path::new("/sys/kernel/config/usb_gadget/nintendo_controller/UDC");
                    if udc_path.exists() {
                        match fs::read_to_string(udc_path) {
                            Ok(udc) => {
                                let udc = udc.trim();
                                if udc.is_empty() {
                                    println!("   Connection: ❌ Not connected (UDC not bound)");
                                } else {
                                    println!("   Connection: ✅ Connected (UDC: {})", udc);
                                }
                            }
                            Err(e) => {
                                println!("   Connection: ⚠️  Unknown (Failed to read UDC: {})", e);
                            }
                        }
                    }
                } else {
                    println!("   Configuration: ❌ Not configured");
                    println!("   Connection: ❌ Not connected");
                }
            }
            Err(e) => {
                println!("   Status: ❌ Error checking gadget: {}", e);
            }
        }
        
        if verbose {
            // 詳細なGadget情報
            let gadget_path = Path::new("/sys/kernel/config/usb_gadget/nintendo_controller");
            if gadget_path.exists() {
                println!("\n   Gadget Details:");
                
                // Vendor/Product ID
                if let Ok(vendor_id) = fs::read_to_string(gadget_path.join("idVendor")) {
                    println!("      - Vendor ID: {}", vendor_id.trim());
                }
                if let Ok(product_id) = fs::read_to_string(gadget_path.join("idProduct")) {
                    println!("      - Product ID: {}", product_id.trim());
                }
                
                // Strings
                let strings_path = gadget_path.join("strings/0x409");
                if let Ok(manufacturer) = fs::read_to_string(strings_path.join("manufacturer")) {
                    println!("      - Manufacturer: {}", manufacturer.trim());
                }
                if let Ok(product) = fs::read_to_string(strings_path.join("product")) {
                    println!("      - Product: {}", product.trim());
                }
            }
        }
        
        Ok(())
    }
    
    fn show_hid_device_info(&self, verbose: bool) -> Result<(), SetupError> {
        println!("\n🎮 HID Device Status:");
        
        let hid_devices = vec!["/dev/hidg0", "/dev/hidg1", "/dev/hidg2", "/dev/hidg3"];
        let mut found_devices = Vec::new();
        
        for device in &hid_devices {
            if Path::new(device).exists() {
                found_devices.push(*device);
            }
        }
        
        if found_devices.is_empty() {
            println!("   Devices: ❌ No HID gadget devices found");
        } else {
            println!("   Devices: ✅ Found {} device(s)", found_devices.len());
            for device in &found_devices {
                println!("      - {}", device);
                
                if verbose {
                    // デバイスの権限情報
                    if let Ok(metadata) = fs::metadata(device) {
                        use std::os::unix::fs::PermissionsExt;
                        let mode = metadata.permissions().mode();
                        println!("        Permissions: {:o}", mode & 0o777);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn show_systemd_service_info(&self, verbose: bool) -> Result<(), SetupError> {
        println!("\n⚙️  Systemd Services:");
        
        let services = vec![
            ("splatoon3-gadget.service", "USB Gadget Service"),
            ("splatoon3-ghost-drawer.service", "Web UI Service"),
        ];
        
        for (service_name, description) in services {
            print!("   {}: ", description);
            
            // systemctl is-enabled
            let enabled_output = std::process::Command::new("systemctl")
                .args(["is-enabled", service_name])
                .output();
                
            let is_enabled = enabled_output
                .map(|o| o.status.success())
                .unwrap_or(false);
                
            // systemctl is-active
            let active_output = std::process::Command::new("systemctl")
                .args(["is-active", service_name])
                .output();
                
            let is_active = active_output
                .map(|o| o.status.success())
                .unwrap_or(false);
                
            if is_enabled && is_active {
                println!("✅ Enabled & Active");
            } else if is_enabled && !is_active {
                println!("⚠️  Enabled but Inactive");
            } else if !is_enabled && is_active {
                println!("⚠️  Active but not Enabled");
            } else {
                println!("❌ Disabled & Inactive");
            }
            
            if verbose && (is_enabled || is_active) {
                // サービスの詳細状態
                if let Ok(output) = std::process::Command::new("systemctl")
                    .args(["status", service_name, "--no-pager", "-n", "3"])
                    .output()
                {
                    let status = String::from_utf8_lossy(&output.stdout);
                    for line in status.lines().skip(1).take(3) {
                        println!("      {}", line.trim());
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn show_kernel_module_info(&self) -> Result<(), SetupError> {
        println!("\n🔧 Kernel Modules:");
        
        let modules = vec!["dwc2", "libcomposite"];
        
        for module in modules {
            print!("   {}: ", module);
            
            let output = std::process::Command::new("lsmod")
                .output()
                .map_err(|e| SetupError::Unknown(format!("Failed to run lsmod: {}", e)))?;
                
            let lsmod_output = String::from_utf8_lossy(&output.stdout);
            if lsmod_output.lines().any(|line| line.starts_with(module)) {
                println!("✅ Loaded");
            } else {
                println!("❌ Not loaded");
            }
        }
        
        Ok(())
    }
    
    fn show_usb_detail_info(&self) -> Result<(), SetupError> {
        println!("\n🔍 USB Details:");
        
        // USB Device情報
        let usb_device_path = Path::new("/sys/bus/usb/devices");
        if usb_device_path.exists() {
            println!("   USB Devices:");
            
            // dmesgから最近のUSB関連メッセージを取得
            if let Ok(output) = std::process::Command::new("dmesg")
                .args(["-t"])
                .output()
            {
                let dmesg = String::from_utf8_lossy(&output.stdout);
                let usb_lines: Vec<&str> = dmesg
                    .lines()
                    .rev()
                    .filter(|line| line.contains("dwc2") || line.contains("gadget") || line.contains("Nintendo"))
                    .take(5)
                    .collect();
                    
                if !usb_lines.is_empty() {
                    println!("   Recent USB Messages:");
                    for line in usb_lines.iter().rev() {
                        println!("      - {}", line);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn show_gadget_debug_info(&self) -> Result<(), SetupError> {
        println!("\n🐛 USB Gadget Debug Information:");
        println!("   ================================");
        
        // ConfigFSのマウント状態
        println!("\n   📁 ConfigFS Mount:");
        let output = std::process::Command::new("mount")
            .arg("-t")
            .arg("configfs")
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to check mount: {}", e)))?;
            
        let mount_info = String::from_utf8_lossy(&output.stdout);
        if mount_info.is_empty() {
            println!("      ❌ ConfigFS is not mounted");
        } else {
            for line in mount_info.lines() {
                println!("      ✅ {}", line);
            }
        }
        
        // UDCデバイスの詳細
        println!("\n   🔌 UDC Devices:");
        let udc_dir = "/sys/class/udc";
        
        if !Path::new(udc_dir).exists() {
            println!("      ❌ UDC directory does not exist");
        } else if let Ok(entries) = fs::read_dir(udc_dir) {
            let mut found_udc = false;
            
            for entry in entries {
                if let Ok(entry) = entry {
                    found_udc = true;
                    let udc_name = entry.file_name();
                    println!("      🟢 {}", udc_name.to_string_lossy());
                    
                    let udc_path = entry.path();
                    
                    // state
                    if let Ok(state) = fs::read_to_string(udc_path.join("state")) {
                        println!("         State: {}", state.trim());
                    }
                    
                    // current_speed
                    if let Ok(speed) = fs::read_to_string(udc_path.join("current_speed")) {
                        println!("         Speed: {}", speed.trim());
                    }
                    
                    // is_otg
                    if let Ok(is_otg) = fs::read_to_string(udc_path.join("is_otg")) {
                        println!("         OTG: {}", is_otg.trim());
                    }
                }
            }
            
            if !found_udc {
                println!("      ❌ No UDC devices found");
                println!("      💡 Try: sudo modprobe musb_hdrc");
            }
        }
        
        // Gadgetディレクトリの詳細
        println!("\n   📂 Gadget Directory:");
        let gadget_path = "/sys/kernel/config/usb_gadget/nintendo_controller";
        
        if Path::new(gadget_path).exists() {
            println!("      ✅ {} exists", gadget_path);
            
            // ディレクトリ内の主要ファイル
            let important_files = vec![
                "idVendor",
                "idProduct",
                "UDC",
                "functions/hid.usb0/protocol",
                "functions/hid.usb0/report_length",
            ];
            
            for file in important_files {
                let file_path = format!("{}/{}", gadget_path, file);
                if let Ok(content) = fs::read_to_string(&file_path) {
                    println!("      📄 {}: {}", file, content.trim());
                }
            }
        } else {
            println!("      ❌ Gadget directory does not exist");
        }
        
        // 権限チェック
        println!("\n   🔐 Permissions:");
        let paths_to_check = vec![
            "/sys/kernel/config",
            "/sys/kernel/config/usb_gadget",
            gadget_path,
            "/dev/hidg0",
        ];
        
        for path in paths_to_check {
            if Path::new(path).exists() {
                if let Ok(metadata) = fs::metadata(path) {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    let perms = format!("{:o}", mode & 0o777);
                    println!("      {} ({})", path, perms);
                }
            }
        }
        
        // 関連カーネルログ
        println!("\n   📃 Recent Gadget-related Kernel Messages:");
        if let Ok(output) = std::process::Command::new("dmesg")
            .args(["-t"])
            .output()
        {
            let dmesg = String::from_utf8_lossy(&output.stdout);
            let gadget_lines: Vec<&str> = dmesg
                .lines()
                .rev()
                .filter(|line| {
                    line.contains("musb") || 
                    line.contains("gadget") ||
                    line.contains("configfs") ||
                    line.contains("UDC") ||
                    line.contains("nintendo")
                })
                .take(10)
                .collect();
                
            if !gadget_lines.is_empty() {
                for line in gadget_lines.iter().rev() {
                    println!("      {}", line);
                }
            } else {
                println!("      No gadget-related messages found");
            }
        }
        
        Ok(())
    }
}