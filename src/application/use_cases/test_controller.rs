use crate::domain::controller::{Button, ControllerAction, ControllerCommand, ControllerEmulator};
use crate::domain::hardware::errors::HardwareError;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

/// コントローラーのテストと動作確認を行うユースケース
pub struct TestControllerUseCase<E: ControllerEmulator> {
    emulator: Arc<E>,
}

impl<E: ControllerEmulator> TestControllerUseCase<E> {
    pub fn new(emulator: Arc<E>) -> Self {
        Self { emulator }
    }

    pub async fn execute(&self, duration: u16, mode: &str) -> Result<(), HardwareError> {
        info!(
            "Starting controller test (mode: {}, duration: {}s)",
            mode, duration
        );

        // 初期化
        self.emulator.initialize()?;
        info!("Controller initialized successfully");

        // 接続確認
        if !self.emulator.is_connected()? {
            warn!("Controller is not connected to Nintendo Switch");
            println!("⚠️  Controller is not connected to Nintendo Switch");
            println!("   Please ensure:");
            println!("   1. USB cable is connected");
            println!("   2. Nintendo Switch is powered on");
            println!("   3. USB gadget is properly configured");
            return Err(HardwareError::NotConnected);
        }

        info!("Controller is connected to Nintendo Switch");
        println!("✅ Controller connected to Nintendo Switch!");

        match mode {
            "basic" => self.run_basic_test(duration).await?,
            "buttons" => self.run_button_test(duration).await?,
            "sticks" => self.run_stick_test(duration).await?,
            "interactive" => self.run_interactive_test().await?,
            _ => {
                error!("Unknown test mode: {}", mode);
                return Err(HardwareError::InvalidParameter(format!(
                    "Unknown test mode: {mode}"
                )));
            }
        }

        info!("Controller test completed");
        Ok(())
    }

    /// 基本的な接続テスト
    async fn run_basic_test(&self, duration: u16) -> Result<(), HardwareError> {
        println!("\n🎮 Running basic controller test...");
        println!("   This test will:");
        println!("   - Press A button every 2 seconds");
        println!("   - Move left stick in a circle");

        let start_time = std::time::Instant::now();
        let test_duration = if duration == 0 {
            Duration::from_secs(10)
        } else {
            Duration::from_secs(duration as u64)
        };

        while start_time.elapsed() < test_duration {
            // Aボタンを押す
            println!("   Pressing A button...");
            let mut command = ControllerCommand::new("Test A button");
            command = command
                .add_action(ControllerAction::press_button(Button::A, 100))
                .add_action(ControllerAction::release_button(Button::A, 100))
                .add_action(ControllerAction::wait(1000));

            self.emulator.execute_command(&command)?;

            // 左スティックを円を描くように動かす
            println!("   Moving left stick in circle...");
            let stick_positions = vec![
                (0, 127),     // Up
                (127, 127),   // Up-Right
                (127, 0),     // Right
                (127, -127),  // Down-Right
                (0, -127),    // Down
                (-127, -127), // Down-Left
                (-127, 0),    // Left
                (-127, 127),  // Up-Left
                (0, 0),       // Center
            ];

            for (x, y) in stick_positions {
                let mut command = ControllerCommand::new("Test stick movement");
                command = command
                    .add_action(ControllerAction::move_left_stick(
                        crate::domain::controller::StickPosition::new(
                            (x as i16 + 128) as u8,
                            (y as i16 + 128) as u8,
                        ),
                        200,
                    ))
                    .add_action(ControllerAction::wait(200));

                self.emulator.execute_command(&command)?;
            }

            sleep(Duration::from_millis(500)).await;
        }

        println!("✅ Basic test completed!");
        Ok(())
    }

    /// ボタンテスト
    async fn run_button_test(&self, duration: u16) -> Result<(), HardwareError> {
        println!("\n🎮 Running button test...");
        println!("   Testing all buttons sequentially:");

        let buttons = vec![
            (Button::A, "A"),
            (Button::B, "B"),
            (Button::X, "X"),
            (Button::Y, "Y"),
            (Button::L, "L"),
            (Button::R, "R"),
            (Button::ZL, "ZL"),
            (Button::ZR, "ZR"),
            (Button::PLUS, "Plus"),
            (Button::MINUS, "Minus"),
            (Button::HOME, "Home"),
            (Button::CAPTURE, "Capture"),
            (Button::L_STICK, "L Stick"),
            (Button::R_STICK, "R Stick"),
        ];

        let start_time = std::time::Instant::now();
        let test_duration = if duration == 0 {
            Duration::from_secs(buttons.len() as u64 * 2)
        } else {
            Duration::from_secs(duration as u64)
        };

        let mut button_index = 0;
        while start_time.elapsed() < test_duration && button_index < buttons.len() {
            let (button, name) = &buttons[button_index];
            println!("   Testing {name} button...");

            let mut command = ControllerCommand::new(format!("Test {name} button"));
            command = command
                .add_action(ControllerAction::press_button(*button, 200))
                .add_action(ControllerAction::release_button(*button, 200))
                .add_action(ControllerAction::wait(1000));

            self.emulator.execute_command(&command)?;

            button_index = (button_index + 1) % buttons.len();
            sleep(Duration::from_millis(500)).await;
        }

        println!("✅ Button test completed!");
        Ok(())
    }

    /// スティックテスト
    async fn run_stick_test(&self, duration: u16) -> Result<(), HardwareError> {
        println!("\n🎮 Running stick test...");
        println!("   Testing both analog sticks:");

        let start_time = std::time::Instant::now();
        let test_duration = if duration == 0 {
            Duration::from_secs(20)
        } else {
            Duration::from_secs(duration as u64)
        };

        while start_time.elapsed() < test_duration {
            // 左スティックテスト
            println!("   Testing left stick...");
            for angle in (0..360).step_by(30) {
                let radians = (angle as f64).to_radians();
                let x = (127.0 * radians.cos()) as i8;
                let y = (127.0 * radians.sin()) as i8;

                let mut command = ControllerCommand::new("Test left stick");
                command = command.add_action(ControllerAction::move_left_stick(
                    crate::domain::controller::StickPosition::new(
                        (x as i16 + 128) as u8,
                        (y as i16 + 128) as u8,
                    ),
                    100,
                ));

                self.emulator.execute_command(&command)?;
                sleep(Duration::from_millis(100)).await;
            }

            // センターに戻す
            let mut command = ControllerCommand::new("Center left stick");
            command = command.add_action(ControllerAction::move_left_stick(
                crate::domain::controller::StickPosition::new(128, 128),
                100,
            ));
            self.emulator.execute_command(&command)?;

            sleep(Duration::from_millis(500)).await;

            // 右スティックテスト
            println!("   Testing right stick...");
            for angle in (0..360).step_by(30) {
                let radians = (angle as f64).to_radians();
                let x = (127.0 * radians.cos()) as i8;
                let y = (127.0 * radians.sin()) as i8;

                let mut command = ControllerCommand::new("Test right stick");
                command = command.add_action(ControllerAction::move_right_stick(
                    crate::domain::controller::StickPosition::new(
                        (x as i16 + 128) as u8,
                        (y as i16 + 128) as u8,
                    ),
                    100,
                ));

                self.emulator.execute_command(&command)?;
                sleep(Duration::from_millis(100)).await;
            }

            // センターに戻す
            let mut command = ControllerCommand::new("Center right stick");
            command = command.add_action(ControllerAction::move_right_stick(
                crate::domain::controller::StickPosition::new(128, 128),
                100,
            ));
            self.emulator.execute_command(&command)?;

            sleep(Duration::from_millis(1000)).await;
        }

        println!("✅ Stick test completed!");
        Ok(())
    }

    /// インタラクティブテスト（将来の実装用）
    async fn run_interactive_test(&self) -> Result<(), HardwareError> {
        println!("\n🎮 Interactive test mode");
        println!("   This mode is not yet implemented.");
        println!("   In the future, it will allow manual control via keyboard input.");

        // TODO: キーボード入力を受け付けて、リアルタイムでコントローラーを操作

        Ok(())
    }
}
