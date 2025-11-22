use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("Checking HID device /dev/hidg0...");

    if !Path::new("/dev/hidg0").exists() {
        println!("❌ /dev/hidg0 does not exist");
        return;
    }

    println!("✅ /dev/hidg0 exists");

    // Check UDC
    let udc_path = "/sys/kernel/config/usb_gadget/nintendo_controller/UDC";
    if Path::new(udc_path).exists() {
        match std::fs::read_to_string(udc_path) {
            Ok(content) => {
                if content.trim().is_empty() {
                    println!("❌ UDC is empty (not bound)");
                } else {
                    println!("✅ UDC is bound to: {}", content.trim());
                }
            }
            Err(e) => println!("❌ Failed to read UDC: {}", e),
        }
    } else {
        println!("❌ UDC path does not exist");
    }

    // Try to write
    println!("Attempting to write 8 bytes to /dev/hidg0...");
    match OpenOptions::new().write(true).open("/dev/hidg0") {
        Ok(mut file) => {
            let report = [0u8; 8];
            match file.write_all(&report) {
                Ok(_) => println!("✅ Write successful! Device is connected."),
                Err(e) => {
                    println!("❌ Write failed: {}", e);
                    if let Some(code) = e.raw_os_error() {
                        println!("   OS Error Code: {}", code);
                    }
                }
            }
        }
        Err(e) => println!("❌ Failed to open /dev/hidg0: {}", e),
    }
}
