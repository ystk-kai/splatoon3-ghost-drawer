use crate::domain::hardware::repositories::UsbGadgetManager;
use crate::domain::setup::repositories::SetupError;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::error;

/// 接続問題を修正するユースケース（主にOrange Pi Zero 2W向け）
pub struct FixConnectionUseCase<G: UsbGadgetManager> {
    gadget_manager: Arc<G>,
}

impl<G: UsbGadgetManager> FixConnectionUseCase<G> {
    pub fn new(gadget_manager: Arc<G>) -> Self {
        Self { gadget_manager }
    }

    pub fn execute(&self) -> Result<(), SetupError> {
        println!("🔧 USB Gadget Connection Fix");
        println!("============================\n");

        // 1. 必要なカーネルモジュールをロード
        self.load_kernel_modules()?;
        
        // 2. USB Gadgetサービスを停止
        self.stop_gadget_service()?;
        
        // 3. USB Gadgetをリセット
        self.reset_usb_gadget()?;
        
        // 4. USB Gadgetサービスを再起動
        self.start_gadget_service()?;
        
        // 5. 接続状態を確認
        self.check_connection_status()?;
        
        // 6. 推奨事項を表示
        self.show_recommendations();
        
        Ok(())
    }
    
    fn load_kernel_modules(&self) -> Result<(), SetupError> {
        println!("📦 Loading kernel modules...");
        
        let modules = vec![
            ("sunxi", "Allwinner platform support"),
            ("musb_hdrc", "MUSB HDRC driver"),
            ("usb_f_hid", "USB HID function"),
            ("libcomposite", "USB Gadget framework"),
        ];
        
        for (module, description) in modules {
            print!("   {} ({}): ", module, description);
            
            let output = Command::new("modprobe")
                .arg(module)
                .output()
                .map_err(|e| SetupError::Unknown(format!("Failed to run modprobe: {}", e)))?;
            
            if output.status.success() {
                println!("✅ Loaded");
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Module") && stderr.contains("not found") {
                    println!("⚠️  Module not available");
                } else if stderr.is_empty() {
                    // モジュールが既にロードされている場合
                    println!("✅ Already loaded");
                } else {
                    println!("⚠️  Failed: {}", stderr.trim());
                }
            }
        }
        
        println!();
        Ok(())
    }
    
    fn stop_gadget_service(&self) -> Result<(), SetupError> {
        println!("⏹️  Stopping USB Gadget service...");
        
        let output = Command::new("systemctl")
            .args(["stop", "splatoon3-gadget.service"])
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to stop service: {}", e)))?;
        
        if output.status.success() {
            println!("   ✅ Service stopped");
        } else {
            println!("   ⚠️  Service may not be running");
        }
        
        // 少し待機
        thread::sleep(Duration::from_millis(500));
        
        Ok(())
    }
    
    fn reset_usb_gadget(&self) -> Result<(), SetupError> {
        println!("🔄 Resetting USB Gadget...");
        
        let udc_path = "/sys/kernel/config/usb_gadget/nintendo_controller/UDC";
        
        if Path::new(udc_path).exists() {
            // UDCをアンバインド
            fs::write(udc_path, "").map_err(|e| {
                error!("Failed to unbind UDC: {}", e);
                SetupError::FileSystemError(e)
            })?;
            println!("   ✅ UDC unbound");
            
            // 少し待機
            thread::sleep(Duration::from_millis(500));
        }
        
        Ok(())
    }
    
    fn start_gadget_service(&self) -> Result<(), SetupError> {
        println!("▶️  Starting USB Gadget service...");
        
        let output = Command::new("systemctl")
            .args(["start", "splatoon3-gadget.service"])
            .output()
            .map_err(|e| SetupError::Unknown(format!("Failed to start service: {}", e)))?;
        
        if output.status.success() {
            println!("   ✅ Service started");
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(SetupError::Unknown(format!("Failed to start service: {}", stderr)));
        }
        
        // サービスが完全に起動するまで待機
        println!("   ⏳ Waiting for service to initialize...");
        thread::sleep(Duration::from_secs(2));
        
        Ok(())
    }
    
    fn check_connection_status(&self) -> Result<(), SetupError> {
        println!("\n🔍 Checking connection status...");
        
        // USB Gadgetの設定確認
        if self.gadget_manager.is_gadget_configured()? {
            println!("   ✅ USB Gadget configured");
            
            // UDCの状態確認
            let udc_path = "/sys/kernel/config/usb_gadget/nintendo_controller/UDC";
            if let Ok(udc) = fs::read_to_string(udc_path) {
                let udc = udc.trim();
                if !udc.is_empty() {
                    println!("   ✅ UDC bound to: {}", udc);
                } else {
                    println!("   ❌ UDC not bound");
                }
            }
        } else {
            println!("   ❌ USB Gadget not configured");
        }
        
        // HIDデバイスの確認
        if Path::new("/dev/hidg0").exists() {
            println!("   ✅ HID device /dev/hidg0 exists");
            
            // 書き込みテスト
            match fs::OpenOptions::new()
                .write(true)
                .open("/dev/hidg0")
            {
                Ok(mut file) => {
                    let test_data = vec![0u8; 64];
                    match file.write_all(&test_data) {
                        Ok(_) => println!("   ✅ HID device is writable"),
                        Err(e) => {
                            if e.raw_os_error() == Some(108) {
                                println!("   ⚠️  HID device not ready (Nintendo Switch may not be connected)");
                            } else {
                                println!("   ❌ HID device write test failed: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    println!("   ❌ Cannot open HID device: {}", e);
                }
            }
        } else {
            println!("   ❌ HID device not found");
        }
        
        println!();
        Ok(())
    }
    
    fn show_recommendations(&self) {
        println!("💡 Next steps:");
        println!("   1. Ensure Nintendo Switch is on the Home screen");
        println!("   2. Connect your device to Nintendo Switch via USB-C");
        println!("   3. Run: sudo splatoon3-ghost-drawer test");
        println!();
        println!("   If still having issues:");
        println!("   - Run: sudo splatoon3-ghost-drawer diagnose");
        println!("   - Check dmesg: sudo dmesg | grep -E '(musb|gadget|hid)'");
        println!("   - Try rebooting your device");
    }
}