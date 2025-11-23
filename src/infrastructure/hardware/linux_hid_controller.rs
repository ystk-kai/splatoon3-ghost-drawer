use crate::domain::controller::{ActionType, Button, ControllerCommand, ControllerEmulator, DPad};
use crate::domain::hardware::errors::HardwareError;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Linux HIDデバイスを使用したコントローラーエミュレーター
pub struct LinuxHidController {
    device_path: Mutex<Option<String>>,
    current_state: Mutex<ProControllerState>,
}

#[derive(Clone, Copy, Debug)]
struct ProControllerState {
    buttons: u32,
    left_stick_x: u8,
    left_stick_y: u8,
    right_stick_x: u8,
    right_stick_y: u8,
}

impl Default for ProControllerState {
    fn default() -> Self {
        Self {
            // Initialize buttons with DPad Neutral (0x08 shifted by 16)
            buttons: (DPad::NEUTRAL.value() as u32) << 16,
            left_stick_x: 0x80,  // 中央値 (128)
            left_stick_y: 0x80,  // 中央値 (128)
            right_stick_x: 0x80, // 中央値 (128)
            right_stick_y: 0x80, // 中央値 (128)
        }
    }
}

impl LinuxHidController {
    pub fn new() -> Self {
        Self {
            device_path: Mutex::new(None),
            current_state: Mutex::new(ProControllerState::default()),
        }
    }
}

impl Default for LinuxHidController {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxHidController {
    /// HIDデバイスパスを検索
    fn find_hid_device(&self) -> Result<String, HardwareError> {
        let hid_paths = ["/dev/hidg0", "/dev/hidg1", "/dev/hidg2", "/dev/hidg3"];

        for path in &hid_paths {
            if Path::new(path).exists() {
                info!("Found HID device at: {}", path);
                return Ok(path.to_string());
            }
        }

        Err(HardwareError::DeviceNotFound(
            "No HID gadget device found".to_string(),
        ))
    }

    /// 現在の状態をHIDレポートとして送信
    fn send_report(&self) -> Result<(), HardwareError> {
        let device_path = self.device_path.lock().unwrap();
        if let Some(path) = device_path.as_ref() {
            let state = self.current_state.lock().unwrap();

            // Pokken Controller Report (8 bytes)
            let mut report = [0u8; 8];

            // Byte 0-1: Buttons (Little Endian)
            report[0] = (state.buttons & 0xFF) as u8;
            report[1] = ((state.buttons >> 8) & 0xFF) as u8;

            // Byte 2: HAT (Lower 4 bits)
            // HAT is stored in bits 16-19 of state.buttons in our internal representation for DPad
            // But here we need to extract it or recalculate it.
            // Actually, let's look at how dpad_to_bits stores it.
            // It stores it as 0x00000 to 0x80000.
            // Let's assume state.buttons bits 16-19 hold the HAT value if we change dpad_to_bits.
            // For now, let's decode the HAT from the upper bits of state.buttons if we use that storage,
            // OR better, let's just use a separate field for HAT in ProControllerState if we want to be clean.
            // However, to minimize changes, let's look at dpad_to_bits below.
            // It currently returns 0x00000 etc.
            // Let's change dpad_to_bits to return the 4-bit value and store it in a specific place,
            // or just extract it here.
            // Wait, the current implementation of dpad_to_bits returns a bitmask for Pro Controller?
            // No, Pro Controller HAT is 0-7.
            // The current dpad_to_bits returns 0x00000, 0x10000... which looks like it's trying to map to specific bits?
            // Actually, looking at the previous code:
            // 0x00 => 0x00000 (UP)
            // ...
            // _ => 0x80000 (NEUTRAL)
            // This suggests the HAT value was being stored in the 3rd byte (bits 16-23) of the buttons field?
            // In the previous send_report:
            // report[5] = ((state.buttons >> 16) & 0xFF) as u8;
            // So yes, it was stored in byte 5.
            // For Pokken, HAT is in Byte 2.
            // Let's extract it.
            let hat_value = (state.buttons >> 16) & 0x0F;
            report[2] = hat_value as u8;

            // Byte 3: LX
            report[3] = state.left_stick_x;
            // Byte 4: LY
            report[4] = state.left_stick_y;
            // Byte 5: RX
            report[5] = state.right_stick_x;
            // Byte 6: RY
            report[6] = state.right_stick_y;
            // Byte 7: Vendor
            report[7] = 0x00;

            // HIDデバイスに書き込み（エラーハンドリング改善）
            match OpenOptions::new().write(true).open(path) {
                Ok(mut file) => {
                    match file.write_all(&report) {
                        Ok(_) => {
                            info!("HID Report: Btn={:04X} HAT={:02X} L=({},{}) R=({},{}) Raw=[{:02X},{:02X},{:02X},{:02X},{:02X},{:02X},{:02X},{:02X}]",
                                  (report[1] as u16) << 8 | report[0] as u16, report[2],
                                  report[3], report[4], report[5], report[6],
                                  report[0], report[1], report[2], report[3], report[4], report[5], report[6], report[7]);
                            Ok(())
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::BrokenPipe
                                || e.raw_os_error() == Some(108)
                            // ESHUTDOWN
                            {
                                warn!("HID device disconnected: {}", e);
                                Err(HardwareError::NotConnected)
                            } else {
                                error!("Failed to write HID report: {}", e);
                                Err(HardwareError::IoError(e))
                            }
                        }
                    }
                }
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::PermissionDenied {
                        error!("Permission denied accessing HID device: {}", path);
                        Err(HardwareError::PermissionDenied)
                    } else {
                        error!("Failed to open HID device {}: {}", path, e);
                        Err(HardwareError::IoError(e))
                    }
                }
            }
        } else {
            Err(HardwareError::NotInitialized)
        }
    }

    /// ボタン値を計算
    fn button_to_bits(button: &Button) -> u32 {
        // Pokken Controller Mapping based on standard Switch Pro Controller
        // We need to map our internal Button constants to the Pokken report format
        // Pokken Report (Little Endian):
        // Byte 0: Y(1), B(2), A(4), X(8), L(10), R(20), ZL(40), ZR(80)
        // Byte 1: Minus(1), Plus(2), LStick(4), RStick(8), Home(10), Capture(20)
        
        let val = button.value();
        let mut mapped = 0u32;
        
        // Byte 0 mappings
        if val & Button::Y.value() != 0 { mapped |= 0x0001; }
        if val & Button::B.value() != 0 { mapped |= 0x0002; }
        if val & Button::A.value() != 0 { mapped |= 0x0004; }
        if val & Button::X.value() != 0 { mapped |= 0x0008; }
        if val & Button::L.value() != 0 { mapped |= 0x0010; }
        if val & Button::R.value() != 0 { mapped |= 0x0020; }
        if val & Button::ZL.value() != 0 { mapped |= 0x0040; }
        if val & Button::ZR.value() != 0 { mapped |= 0x0080; }
        
        // Byte 1 mappings (shifted by 8)
        if val & Button::MINUS.value() != 0 { mapped |= 0x0100; }
        if val & Button::PLUS.value() != 0 { mapped |= 0x0200; }
        if val & Button::L_STICK.value() != 0 { mapped |= 0x0400; }
        if val & Button::R_STICK.value() != 0 { mapped |= 0x0800; }
        if val & Button::HOME.value() != 0 { mapped |= 0x1000; }
        if val & Button::CAPTURE.value() != 0 { mapped |= 0x2000; }
        
        mapped
    }

    /// DPad値を計算
    fn dpad_to_bits(dpad: &DPad) -> u32 {
        // Shifted by 16 bits to be stored in the upper part of buttons
        // This will be extracted as Byte 2 in send_report
        (dpad.value() as u32) << 16
    }
}

impl ControllerEmulator for LinuxHidController {
    fn initialize(&self) -> Result<(), HardwareError> {
        info!("Initializing Linux HID controller...");

        // USB Gadgetが設定されているか確認
        let gadget_path = Path::new("/sys/kernel/config/usb_gadget/nintendo_controller");
        if !gadget_path.exists() {
            error!("USB Gadget not configured. Run 'sudo splatoon3-ghost-drawer setup' first.");
            return Err(HardwareError::GadgetConfigurationFailed(
                "USB Gadget not configured".to_string(),
            ));
        }

        // HIDデバイスを検索
        let device_path = self.find_hid_device()?;
        info!("Found HID device at: {}", device_path);

        // デバイスの権限を確認
        if let Err(e) = std::fs::metadata(&device_path) {
            error!("Cannot access HID device {}: {}", device_path, e);
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                return Err(HardwareError::PermissionDenied);
            }
            return Err(HardwareError::IoError(e));
        }

        // デバイスパスを保存
        *self.device_path.lock().unwrap() = Some(device_path.clone());

        // 初期状態を送信（エラーの場合は詳細情報を提供）
        match self.send_report() {
            Ok(_) => {
                info!("Linux HID controller initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to send initial report: {}", e);

                // エラーの種類に応じて詳細情報を提供
                match &e {
                    HardwareError::NotConnected => {
                        error!("HID device appears to be disconnected. This can happen if:");
                        error!("1. Nintendo Switch is not ready to receive input");
                        error!("2. USB cable is not properly connected");
                        error!("3. USB Gadget needs to be reset");
                        println!("\n❌ HID device is not connected properly.");
                        println!("   Try the following:");
                        println!("   1. Ensure Nintendo Switch is on the Home screen");
                        println!("   2. Reconnect the USB cable");
                        println!("   3. Run 'sudo systemctl restart splatoon3-gadget.service'");
                    }
                    HardwareError::PermissionDenied => {
                        error!("Permission denied accessing HID device");
                        println!("\n❌ Permission denied accessing HID device.");
                        println!("   This command must be run with sudo.");
                    }
                    HardwareError::IoError(io_err)
                        if io_err.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        error!("Broken pipe when writing to HID device");
                        println!("\n❌ HID device connection was broken.");
                        println!("   The USB connection may have been interrupted.");
                    }
                    _ => {
                        error!("Unexpected error during initialization");
                    }
                }

                *self.device_path.lock().unwrap() = None;
                Err(e)
            }
        }
    }

    fn is_connected(&self) -> Result<bool, HardwareError> {
        let device_path = self.device_path.lock().unwrap();
        if let Some(path) = device_path.as_ref() {
            // デバイスファイルが存在し、書き込み可能かチェック
            if !Path::new(path).exists() {
                warn!("HID device {} does not exist", path);
                return Ok(false);
            }

            // USB Gadgetの状態を確認
            let gadget_path = Path::new("/sys/kernel/config/usb_gadget/nintendo_controller/UDC");
            if !gadget_path.exists() {
                warn!("USB Gadget UDC path does not exist");
                return Ok(false);
            }

            // UDCの状態確認（権限エラーはエラーとして扱う＝厳格なチェック）
            let udc_content = std::fs::read_to_string(gadget_path).map_err(|e| {
                error!("Failed to read UDC status: {}", e);
                HardwareError::IoError(e)
            })?;

            let is_connected = !udc_content.trim().is_empty();
            if !is_connected {
                warn!("UDC is not bound (empty UDC file)");
                return Ok(false);
            } else {
                debug!("UDC is bound to: {}", udc_content.trim());
            }

            // 実際にHIDデバイスに書き込めるかテスト（接続状態の確認）
            // O_NONBLOCKを使用して、ブロッキングを防ぎつつ厳格にチェックする
            use std::os::unix::fs::OpenOptionsExt;
            match OpenOptions::new()
                .write(true)
                .custom_flags(libc::O_NONBLOCK) // ノンブロッキングモード
                .open(path) 
            {
                Ok(mut file) => {
                    // NEUTRAL状態のレポートを送信してテスト
                    let test_report = [0x00, 0x00, 0x08, 0x80, 0x80, 0x80, 0x80, 0x00];
                    match file.write_all(&test_report) {
                        Ok(_) => {
                            debug!("HID device is writable and connected (Non-blocking check passed)");
                            Ok(true)
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::WouldBlock {
                                // ブロックされる＝ホストがポーリングしていない可能性があるが、
                                // デバイスファイルが開けている以上、物理的には接続されているとみなす。
                                // ここでfalseを返すと、ポーリング間隔のタイミング次第で「未接続」と判定されてしまう。
                                debug!("HID device write would block (Buffer full or Host not polling). Assuming connected.");
                                Ok(true)
                            } else if e.kind() == std::io::ErrorKind::BrokenPipe
                                || e.raw_os_error() == Some(108) // ESHUTDOWN
                            {
                                warn!("HID device not ready: {}", e);
                                Ok(false)
                            } else {
                                error!("Failed to test HID device: {}", e);
                                Err(HardwareError::IoError(e))
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Cannot open HID device for testing: {}", e);
                    Ok(false)
                }
            }
        } else {
            warn!("No HID device path configured");
            Ok(false)
        }
    }

    fn execute_command(&self, command: &ControllerCommand) -> Result<(), HardwareError> {
        debug!("Executing controller command: {}", command.name);

        for action in &command.sequence {
            match &action.action_type {
                ActionType::PressButton(button) => {
                    info!("PressButton: {:?}, bits: 0x{:04X}", button, Self::button_to_bits(button));
                    let mut state = self.current_state.lock().unwrap();
                    // CRITICAL: D-padビットをクリアしてからボタンを押す
                    // これにより、ボタン押下中にD-pad入力が残らない
                    state.buttons &= 0xFFF0FFFF; // D-padビット（16-19）をクリア
                    state.buttons |= (DPad::NEUTRAL.value() as u32) << 16; // NEUTRAL状態を設定
                    state.buttons |= Self::button_to_bits(button);
                    info!("State buttons after press: 0x{:08X}", state.buttons);
                    // スティックの値は変更しない（現在の値を維持）
                    // これにより、意図しないスティック入力を防ぐ
                    drop(state);
                    // 押下中は継続的にレポートを送信（8ms間隔 = 125Hz）
                    let start_time = std::time::Instant::now();
                    let duration = Duration::from_millis(action.duration_ms as u64);
                    let report_interval = Duration::from_millis(8);
                    
                    while start_time.elapsed() < duration {
                        self.send_report()?;
                        thread::sleep(report_interval);
                    }
                }
                ActionType::ReleaseButton(button) => {
                    info!("ReleaseButton: {:?}, bits: 0x{:04X}", button, Self::button_to_bits(button));
                    let mut state = self.current_state.lock().unwrap();
                    state.buttons &= !Self::button_to_bits(button);
                    // D-padビットもクリアしてNEUTRALに設定
                    state.buttons &= 0xFFF0FFFF; // D-padビット（16-19）をクリア
                    state.buttons |= (DPad::NEUTRAL.value() as u32) << 16; // NEUTRAL状態を設定
                    info!("State buttons after release: 0x{:08X}", state.buttons);
                    // スティックの値は変更しない（現在の値を維持）
                    drop(state);
                    // リリース中も継続的にレポートを送信（8ms間隔 = 125Hz）
                    let start_time = std::time::Instant::now();
                    let duration = Duration::from_millis(action.duration_ms as u64);
                    let report_interval = Duration::from_millis(8);
                    
                    while start_time.elapsed() < duration {
                        self.send_report()?;
                        thread::sleep(report_interval);
                    }
                }
                ActionType::SetDPad(dpad) => {
                    info!("SetDPad: {:?}, bits: 0x{:08X}", dpad, Self::dpad_to_bits(dpad));
                    let mut state = self.current_state.lock().unwrap();
                    // DPadビットをクリアしてから設定
                    state.buttons &= 0xFFF0FFFF;
                    state.buttons |= Self::dpad_to_bits(dpad);
                    info!("State buttons after DPad: 0x{:08X}", state.buttons);
                    // スティックの値は変更しない（現在の値を維持）
                    // これにより、D-pad使用時にスティックからの意図しない入力を防ぐ
                    drop(state);
                    // DPad入力中も継続的にレポートを送信（8ms間隔 = 125Hz）
                    let start_time = std::time::Instant::now();
                    let duration = Duration::from_millis(action.duration_ms as u64);
                    let report_interval = Duration::from_millis(8);
                    
                    while start_time.elapsed() < duration {
                        self.send_report()?;
                        thread::sleep(report_interval);
                    }
                }
                ActionType::MoveLeftStick(position) => {
                    let mut state = self.current_state.lock().unwrap();
                    state.left_stick_x = position.x;
                    state.left_stick_y = position.y;
                    drop(state);
                    // 左スティック入力中も継続的にレポートを送信（8ms間隔 = 125Hz）
                    let start_time = std::time::Instant::now();
                    let duration = Duration::from_millis(action.duration_ms as u64);
                    let report_interval = Duration::from_millis(8);
                    
                    while start_time.elapsed() < duration {
                        self.send_report()?;
                        thread::sleep(report_interval);
                    }
                    // スティック移動後、自動的に中央に戻す
                    // CENTER (128, 128) でない場合のみリセット
                    if position.x != 128 || position.y != 128 {
                        let mut state = self.current_state.lock().unwrap();
                        state.left_stick_x = 128;
                        state.left_stick_y = 128;
                        drop(state);
                        // ニュートラル状態を確実に送信
                        for _ in 0..5 {
                            self.send_report()?;
                            thread::sleep(report_interval);
                        }
                    }
                }
                ActionType::MoveRightStick(position) => {
                    let mut state = self.current_state.lock().unwrap();
                    state.right_stick_x = position.x;
                    state.right_stick_y = position.y;
                    drop(state);
                    self.send_report()?;
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
                }
                ActionType::Wait => {
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
                }
                ActionType::SetReport(_) => {
                    // Not implemented for this use case
                }
            }
        }

        Ok(())
    }

    fn shutdown(&self) -> Result<(), HardwareError> {
        info!("Shutting down Linux HID controller...");

        // ニュートラル状態に戻す
        *self.current_state.lock().unwrap() = ProControllerState::default();
        self.send_report()?;

        // デバイスパスをクリア
        *self.device_path.lock().unwrap() = None;

        info!("Linux HID controller shut down successfully");
        Ok(())
    }
}
