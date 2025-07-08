use crate::domain::artwork::entities::Artwork;
use crate::domain::controller::{
    ButtonState, ControllerError, ControllerRepository, ControllerSession,
    ControllerSessionRepository, DPad, HidDeviceRepository, ProController, StickPosition,
};
use crate::domain::painting::{ArtworkToCommandConverter, DrawingCanvasConfig, DrawingStrategy};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};
use uuid::Uuid;

pub struct PaintArtworkUseCase<CR, HR, SR> {
    controller_repo: CR,
    hid_repo: HR,
    session_repo: SR,
}

impl<CR, HR, SR> PaintArtworkUseCase<CR, HR, SR>
where
    CR: ControllerRepository,
    HR: HidDeviceRepository,
    SR: ControllerSessionRepository,
{
    pub fn new(controller_repo: CR, hid_repo: HR, session_repo: SR) -> Self {
        Self {
            controller_repo,
            hid_repo,
            session_repo,
        }
    }

    pub async fn execute(
        &self,
        artwork: &Artwork,
        config: PaintConfig,
    ) -> Result<PaintResult, ControllerError> {
        info!("アートワークの描画を開始: {}", artwork.metadata.name);

        // 1. HIDデバイスの確認
        let devices = self.hid_repo.list_devices().await?;
        if devices.is_empty() {
            return Err(ControllerError::DevicePathNotAvailable(
                "No HID devices found. Is USB gadget active?".to_string(),
            ));
        }
        let device_path = &devices[0];
        info!("HIDデバイスを使用: {}", device_path);

        // 2. デバイスのオープン
        self.hid_repo.open_device(device_path).await?;

        // 3. コントローラーの作成または取得
        let controller_id = "paint_controller";
        let mut controller = match self.controller_repo.get_controller(controller_id).await {
            Ok(c) => c,
            Err(_) => {
                let c = ProController::new(controller_id);
                self.controller_repo.create_controller(&c).await?;
                c
            }
        };

        // 4. コントローラーを接続
        if !controller.is_connected {
            controller.connect(device_path);
            self.controller_repo.update_controller(&controller).await?;
        }

        // 5. アートワークをコマンドに変換
        let drawing_config = DrawingCanvasConfig {
            cursor_speed_ms: config.cursor_speed_ms,
            dot_draw_delay_ms: config.dot_draw_delay_ms,
            ..Default::default()
        };

        let converter = ArtworkToCommandConverter::new(drawing_config, config.strategy);
        let commands = converter.convert(artwork);
        info!("{}個のコマンドを生成しました", commands.len());

        // 6. 接続確認のためのダミーレポート送信
        info!("接続確認のためのダミーレポートを送信");
        let mut neutral_report = controller.current_state;
        // ニュートラル状態を確実に設定
        neutral_report.dpad = DPad::NEUTRAL;
        neutral_report.left_stick = StickPosition::CENTER;
        neutral_report.right_stick = StickPosition::CENTER;
        neutral_report.buttons = ButtonState::new();

        // Pro Controllerの初期化シーケンス
        info!("初期化シーケンスを開始");

        // 最初にニュートラルレポートを送信
        let pro_report = neutral_report.to_pro_controller_bytes();
        for i in 0..10 {
            self.hid_repo
                .write_pro_controller_report(device_path, &pro_report)
                .await?;
            if i < 5 {
                sleep(Duration::from_millis(100)).await;
            } else {
                sleep(Duration::from_millis(50)).await;
            }
        }

        // USBコマンドをチェックして応答（非ブロッキング）
        self.handle_usb_commands(device_path).await?;

        // 7. セッションの作成
        let session_id = Uuid::new_v4().to_string();
        let mut session = ControllerSession::new(&session_id, &controller.id);

        // コマンドをキューに追加
        for command in commands {
            session.queue_command(command);
        }

        self.session_repo.create_session(&session).await?;

        // 8. セッションの実行
        session.start();
        self.session_repo.update_session(&session).await?;

        let total_commands = session.remaining_commands();
        let mut executed_commands = 0;
        let mut last_progress = 0;

        while !session.is_completed() && session.is_active {
            if let Some(current_action) = session.current_action() {
                // アクションを適用
                controller.apply_action(current_action);
                self.controller_repo.update_controller(&controller).await?;

                // Pro Controller形式のHIDレポートを送信
                let report = controller.current_state.to_pro_controller_bytes();
                self.hid_repo
                    .write_pro_controller_report(device_path, &report)
                    .await?;

                // アクションの継続時間だけ待機
                sleep(Duration::from_millis(current_action.duration_ms as u64)).await;

                // 次のアクションへ
                if !session.advance_action() {
                    executed_commands += 1;

                    // 進捗を報告
                    let progress = (executed_commands * 100) / total_commands;
                    if progress > last_progress + 10 {
                        info!("描画進捗: {}%", progress);
                        last_progress = progress;
                    }
                }

                self.session_repo.update_session(&session).await?;
            }

            // 中断チェック
            if config.check_interrupt {
                // TODO: 中断シグナルのチェック
            }
        }

        // 9. クリーンアップ
        controller.reset_state();
        self.controller_repo.update_controller(&controller).await?;

        // 最終レポートを送信（すべてニュートラル）
        let final_report = controller.current_state.to_pro_controller_bytes();
        self.hid_repo
            .write_pro_controller_report(device_path, &final_report)
            .await?;

        // セッションを終了
        session.stop();
        self.session_repo.update_session(&session).await?;

        // 10. 結果をまとめる
        let drawable_dots = artwork.canvas.drawable_dots().len();
        let result = PaintResult {
            success: true,
            dots_painted: drawable_dots,
            commands_executed: executed_commands,
            duration_ms: session
                .started_at
                .map(|start| {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64;
                    now - start
                })
                .unwrap_or(0),
            device_used: device_path.clone(),
        };

        info!("アートワークの描画が完了しました: {:?}", result);
        Ok(result)
    }

    /// USBコマンドを処理して応答
    async fn handle_usb_commands(&self, device_path: &str) -> Result<(), ControllerError> {
        use tokio::time::timeout;

        // 非ブロッキングでコマンドをチェック
        match timeout(
            Duration::from_millis(100),
            self.hid_repo.read_usb_command(device_path),
        )
        .await
        {
            Ok(Ok(command)) if !command.is_empty() => {
                info!(
                    "受信したUSBコマンド: {:02x?}",
                    &command[..command.len().min(16)]
                );

                // コマンドに応じて応答
                if command.len() >= 2 {
                    match (command[0], command[1]) {
                        (0x80, 0x01) => {
                            // Status request
                            info!("Status request received");
                            let response = vec![0x81, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                            self.hid_repo
                                .write_usb_response(device_path, &response)
                                .await?;
                        }
                        (0x80, 0x02) => {
                            // Handshake
                            info!("Handshake request received");
                            let response = vec![0x81, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                            self.hid_repo
                                .write_usb_response(device_path, &response)
                                .await?;
                        }
                        (0x80, 0x03) => {
                            // Force USB mode (no timeout)
                            info!("Force USB mode request received");
                            let response = vec![0x81, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                            self.hid_repo
                                .write_usb_response(device_path, &response)
                                .await?;
                        }
                        (0x80, 0x04) => {
                            // Enable vibration
                            info!("Enable vibration request received");
                            let response = vec![0x81, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
                            self.hid_repo
                                .write_usb_response(device_path, &response)
                                .await?;
                        }
                        _ => {
                            info!("未知のコマンド: 0x{:02x} 0x{:02x}", command[0], command[1]);
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                // エラーの場合はログに記録して続行
                debug!("コマンド読み取りエラー: {}", e);
            }
            Err(_) => {
                // タイムアウトは正常（コマンドがない）
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PaintConfig {
    /// カーソル移動速度（ミリ秒/ピクセル）
    pub cursor_speed_ms: u32,
    /// ドット描画待機時間（ミリ秒）
    pub dot_draw_delay_ms: u32,
    /// 描画戦略
    pub strategy: DrawingStrategy,
    /// 中断チェックを行うか
    pub check_interrupt: bool,
}

impl Default for PaintConfig {
    fn default() -> Self {
        Self {
            cursor_speed_ms: 100,
            dot_draw_delay_ms: 100,
            strategy: DrawingStrategy::ZigZag,
            check_interrupt: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PaintResult {
    pub success: bool,
    pub dots_painted: usize,
    pub commands_executed: usize,
    pub duration_ms: u64,
    pub device_used: String,
}

impl PaintResult {
    pub fn summary(&self) -> String {
        if self.success {
            format!(
                "描画成功:\n  描画ドット数: {}\n  実行コマンド数: {}\n  所要時間: {:.1}秒\n  使用デバイス: {}",
                self.dots_painted,
                self.commands_executed,
                self.duration_ms as f64 / 1000.0,
                self.device_used
            )
        } else {
            "描画失敗".to_string()
        }
    }
}
