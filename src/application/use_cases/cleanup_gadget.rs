use crate::domain::setup::repositories::SetupError;
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

/// USB Gadgetの設定をクリーンアップするユースケース
pub struct CleanupGadgetUseCase;

impl CleanupGadgetUseCase {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CleanupGadgetUseCase {
    fn default() -> Self {
        Self::new()
    }
}

impl CleanupGadgetUseCase {
    pub fn execute(&self) -> Result<(), SetupError> {
        println!("🧹 Cleaning up USB Gadget configuration...");
        println!("=====================================\n");

        let gadget_path = "/sys/kernel/config/usb_gadget/nintendo_controller";

        if !Path::new(gadget_path).exists() {
            println!("✅ No gadget configuration found (already clean)");
            return Ok(());
        }

        // 1. UDCをアンバインド
        self.unbind_udc(gadget_path)?;

        // 2. 設定からfunctionのリンクを削除
        self.remove_function_links(gadget_path)?;

        // 3. stringsディレクトリを削除
        self.remove_strings_directories(gadget_path)?;

        // 4. 設定ディレクトリを削除
        self.remove_config_directories(gadget_path)?;

        // 5. functionディレクトリを削除
        self.remove_function_directories(gadget_path)?;

        // 6. gadget本体を削除
        self.remove_gadget_directory(gadget_path)?;

        // 7. 他のGadgetも確認
        self.check_other_gadgets()?;

        println!("\n✅ Cleanup completed successfully!");
        println!("\n💡 Next steps:");
        println!("   1. Run: sudo splatoon3-ghost-drawer debug-gadget");
        println!("   2. Run: sudo systemctl restart splatoon3-gadget.service");
        println!("   3. Run: sudo splatoon3-ghost-drawer test");

        Ok(())
    }

    fn unbind_udc(&self, gadget_path: &str) -> Result<(), SetupError> {
        let udc_path = format!("{}/UDC", gadget_path);

        if Path::new(&udc_path).exists() {
            println!("📌 Unbinding UDC...");

            // Read current UDC
            if let Ok(current_udc) = fs::read_to_string(&udc_path) {
                let current_udc = current_udc.trim();
                if !current_udc.is_empty() {
                    println!("   Current UDC: {}", current_udc);
                }
            }

            // Unbind
            match fs::write(&udc_path, "") {
                Ok(_) => {
                    println!("   ✅ UDC unbound");
                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => {
                    println!("   ⚠️  Failed to unbind UDC: {}", e);
                }
            }
        } else {
            println!("   ℹ️  No UDC binding found");
        }

        Ok(())
    }

    fn remove_function_links(&self, gadget_path: &str) -> Result<(), SetupError> {
        let link_path = format!("{}/configs/c.1/hid.usb0", gadget_path);

        if Path::new(&link_path).exists() {
            println!("🔗 Removing function links...");

            match fs::remove_file(&link_path) {
                Ok(_) => println!("   ✅ Removed hid.usb0 link"),
                Err(e) => println!("   ⚠️  Failed to remove link: {}", e),
            }
        }

        Ok(())
    }

    fn remove_strings_directories(&self, gadget_path: &str) -> Result<(), SetupError> {
        println!("📝 Removing strings directories...");

        let strings_dirs = vec![
            format!("{}/configs/c.1/strings/0x409", gadget_path),
            format!("{}/strings/0x409", gadget_path),
        ];

        for dir in strings_dirs {
            if Path::new(&dir).exists() {
                match fs::remove_dir(&dir) {
                    Ok(_) => println!("   ✅ Removed {}", dir),
                    Err(e) => println!("   ⚠️  Failed to remove {}: {}", dir, e),
                }
            }
        }

        Ok(())
    }

    fn remove_config_directories(&self, gadget_path: &str) -> Result<(), SetupError> {
        println!("⚙️  Removing config directories...");

        let config_dirs = vec![
            format!("{}/configs/c.1/strings", gadget_path),
            format!("{}/configs/c.1", gadget_path),
            format!("{}/configs", gadget_path),
        ];

        for dir in config_dirs {
            if Path::new(&dir).exists() {
                match fs::remove_dir(&dir) {
                    Ok(_) => println!("   ✅ Removed {}", dir),
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
                            println!("   ⚠️  Directory not empty: {}", dir);
                            // List contents for debugging
                            if let Ok(entries) = fs::read_dir(&dir) {
                                for entry in entries.flatten() {
                                    println!("      - {}", entry.file_name().to_string_lossy());
                                }
                            }
                        } else {
                            println!("   ⚠️  Failed to remove {}: {}", dir, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn remove_function_directories(&self, gadget_path: &str) -> Result<(), SetupError> {
        println!("🎮 Removing function directories...");

        let function_dirs = vec![
            format!("{}/functions/hid.usb0", gadget_path),
            format!("{}/functions", gadget_path),
        ];

        for dir in function_dirs {
            if Path::new(&dir).exists() {
                match fs::remove_dir(&dir) {
                    Ok(_) => println!("   ✅ Removed {}", dir),
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
                            println!("   ⚠️  Directory not empty: {}", dir);
                        } else {
                            println!("   ⚠️  Failed to remove {}: {}", dir, e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn remove_gadget_directory(&self, gadget_path: &str) -> Result<(), SetupError> {
        println!("📦 Removing gadget directory...");

        // Remove strings directory first
        let strings_dir = format!("{}/strings", gadget_path);
        if Path::new(&strings_dir).exists() {
            match fs::remove_dir(&strings_dir) {
                Ok(_) => println!("   ✅ Removed strings directory"),
                Err(e) => println!("   ⚠️  Failed to remove strings: {}", e),
            }
        }

        // Remove main gadget directory
        if Path::new(gadget_path).exists() {
            match fs::remove_dir(gadget_path) {
                Ok(_) => println!("   ✅ Removed gadget directory"),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::DirectoryNotEmpty {
                        println!("   ⚠️  Gadget directory not empty");

                        // List remaining contents
                        println!("   📋 Remaining contents:");
                        if let Ok(entries) = fs::read_dir(gadget_path) {
                            for entry in entries.flatten() {
                                let name = entry.file_name();
                                let path = entry.path();
                                if path.is_dir() {
                                    println!("      - {}/ (directory)", name.to_string_lossy());
                                } else {
                                    println!("      - {} (file)", name.to_string_lossy());
                                }
                            }
                        }
                    } else {
                        println!("   ⚠️  Failed to remove gadget: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    fn check_other_gadgets(&self) -> Result<(), SetupError> {
        println!("\n🔍 Checking for other gadgets...");

        let gadget_base = "/sys/kernel/config/usb_gadget";

        if let Ok(entries) = fs::read_dir(gadget_base) {
            let mut found_other = false;

            for entry in entries.flatten() {
                found_other = true;
                let name = entry.file_name();
                println!("   ⚠️  Found gadget: {}", name.to_string_lossy());

                // Check if it's bound
                let udc_path = entry.path().join("UDC");
                if let Ok(udc) = fs::read_to_string(&udc_path) {
                    let udc = udc.trim();
                    if !udc.is_empty() {
                        println!("      Bound to: {}", udc);
                    }
                }
            }

            if !found_other {
                println!("   ✅ No other gadgets found");
            }
        } else {
            println!("   ℹ️  Gadget directory not accessible");
        }

        Ok(())
    }
}
