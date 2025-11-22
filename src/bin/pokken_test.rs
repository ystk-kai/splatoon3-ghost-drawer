use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Starting Pokken Controller Test (L Button Spam)...");
    let device_path = "/dev/hidg0";

    // Pokken Controller Report Structure (8 bytes)
    // Byte 0: Buttons Low
    // Byte 1: Buttons High
    // Byte 2: HAT (0-7, 8=Neutral)
    // Byte 3: LX (0-255, 128=Center)
    // Byte 4: LY (0-255, 128=Center)
    // Byte 5: RX (0-255, 128=Center)
    // Byte 6: RY (0-255, 128=Center)
    // Byte 7: Vendor (0)

    // Button mapping for Pokken (based on standard mapping)
    // L is often mapped to bit 4 (0x10) or bit 6 (0x40) depending on the specific controller mapping.
    // Let's try standard Switch Pro Controller mapping first, but mapped to Pokken report.
    // In Pokken report:
    // Bit 0: Y
    // Bit 1: B
    // Bit 2: A
    // Bit 3: X
    // Bit 4: L
    // Bit 5: R
    // Bit 6: ZL
    // Bit 7: ZR
    // Bit 8: Minus
    // Bit 9: Plus
    // Bit 10: LStick Click
    // Bit 11: RStick Click
    // Bit 12: Home
    // Bit 13: Capture

    let l_button_mask = 0x10; // Bit 4

    loop {
        // Press L
        println!("Pressing L...");
        send_report(device_path, l_button_mask, 0x08); // 0x08 is HAT Neutral
        thread::sleep(Duration::from_millis(100));

        // Release
        println!("Releasing...");
        send_report(device_path, 0, 0x08);
        thread::sleep(Duration::from_millis(100));
        
        // Press A (Bit 2 -> 0x04) just in case L is not visible
        println!("Pressing A...");
        send_report(device_path, 0x04, 0x08);
        thread::sleep(Duration::from_millis(100));

        // Release
        println!("Releasing...");
        send_report(device_path, 0, 0x08);
        thread::sleep(Duration::from_millis(500));
    }
}

fn send_report(path: &str, buttons: u16, hat: u8) {
    let mut report = [0u8; 8];
    
    // Buttons (Little Endian)
    report[0] = (buttons & 0xFF) as u8;
    report[1] = ((buttons >> 8) & 0xFF) as u8;
    
    // HAT
    report[2] = hat;
    
    // Sticks (Center)
    report[3] = 128;
    report[4] = 128;
    report[5] = 128;
    report[6] = 128;
    
    // Vendor
    report[7] = 0;

    match OpenOptions::new().write(true).open(path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&report) {
                eprintln!("Failed to write report: {}", e);
            }
        }
        Err(e) => eprintln!("Failed to open device: {}", e),
    }
}
