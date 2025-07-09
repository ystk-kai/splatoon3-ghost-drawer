use crate::domain::controller::{
    ActionType, Button, ControllerCommand, ControllerEmulator, DPad,
};
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
            buttons: 0,
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

    /// HIDデバイスパスを検索
    fn find_hid_device(&self) -> Result<String, HardwareError> {
        let hid_paths = [
            "/dev/hidg0",
            "/dev/hidg1",
            "/dev/hidg2",
            "/dev/hidg3",
        ];

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
            
            // Pro Controller標準入力レポート (64バイト)
            let mut report = [0u8; 64];
            
            // レポートID
            report[0] = 0x30; // 標準入力レポート
            
            // タイマー (簡易実装)
            report[1] = 0x00;
            
            // バッテリー状態とコネクション情報
            report[2] = 0x91; // 充電中、フル充電
            
            // ボタン状態 (3バイト)
            report[3] = (state.buttons & 0xFF) as u8;
            report[4] = ((state.buttons >> 8) & 0xFF) as u8;
            report[5] = ((state.buttons >> 16) & 0xFF) as u8;
            
            // 左スティック
            report[6] = state.left_stick_x;
            report[7] = state.left_stick_y;
            report[8] = 0x00; // 左スティック上位ビット
            
            // 右スティック
            report[9] = state.right_stick_x;
            report[10] = state.right_stick_y;
            report[11] = 0x00; // 右スティック上位ビット
            
            // 振動データ（未実装）
            report[12] = 0x00;
            
            // 残りは0で埋める
            
            // HIDデバイスに書き込み（エラーハンドリング改善）
            match OpenOptions::new()
                .write(true)
                .open(path) 
            {
                Ok(mut file) => {
                    match file.write_all(&report) {
                        Ok(_) => {
                            debug!("Sent HID report");
                            Ok(())
                        }
                        Err(e) => {
                            if e.kind() == std::io::ErrorKind::BrokenPipe 
                                || e.raw_os_error() == Some(108) // ESHUTDOWN
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
        button.value() as u32
    }

    /// DPad値を計算
    fn dpad_to_bits(dpad: &DPad) -> u32 {
        match dpad.value() {
            0x00 => 0x00000,  // UP
            0x01 => 0x10000,  // UP_RIGHT
            0x02 => 0x20000,  // RIGHT
            0x03 => 0x30000,  // DOWN_RIGHT
            0x04 => 0x40000,  // DOWN
            0x05 => 0x50000,  // DOWN_LEFT
            0x06 => 0x60000,  // LEFT
            0x07 => 0x70000,  // UP_LEFT
            _ => 0x80000,     // NEUTRAL
        }
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
                "USB Gadget not configured".to_string()
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
                *self.device_path.lock().unwrap() = None;
                Err(e)
            }
        }
    }

    fn is_connected(&self) -> Result<bool, HardwareError> {
        let device_path = self.device_path.lock().unwrap();
        if let Some(path) = device_path.as_ref() {
            // デバイスファイルが存在し、書き込み可能かチェック
            if Path::new(path).exists() {
                // USB Gadgetの状態を確認
                let gadget_path = Path::new("/sys/kernel/config/usb_gadget/nintendo_controller/UDC");
                if gadget_path.exists() {
                    let udc_content = std::fs::read_to_string(gadget_path)
                        .map_err(|e| HardwareError::IoError(e))?;
                    Ok(!udc_content.trim().is_empty())
                } else {
                    Ok(false)
                }
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    fn execute_command(&self, command: &ControllerCommand) -> Result<(), HardwareError> {
        debug!("Executing controller command: {}", command.name);
        
        for action in &command.sequence {
            match &action.action_type {
                ActionType::PressButton(button) => {
                    let mut state = self.current_state.lock().unwrap();
                    state.buttons |= Self::button_to_bits(&button);
                    drop(state);
                    self.send_report()?;
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
                }
                ActionType::ReleaseButton(button) => {
                    let mut state = self.current_state.lock().unwrap();
                    state.buttons &= !Self::button_to_bits(&button);
                    drop(state);
                    self.send_report()?;
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
                }
                ActionType::SetDPad(dpad) => {
                    let mut state = self.current_state.lock().unwrap();
                    // DPadビットをクリアしてから設定
                    state.buttons &= 0xFFF0FFFF;
                    state.buttons |= Self::dpad_to_bits(&dpad);
                    drop(state);
                    self.send_report()?;
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
                }
                ActionType::MoveLeftStick(position) => {
                    let mut state = self.current_state.lock().unwrap();
                    state.left_stick_x = position.x;
                    state.left_stick_y = position.y;
                    drop(state);
                    self.send_report()?;
                    thread::sleep(Duration::from_millis(action.duration_ms as u64));
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