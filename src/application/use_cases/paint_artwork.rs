use crate::domain::artwork::entities::Artwork;
use crate::domain::controller::{
    ControllerError, ControllerRepository, ControllerSession, ControllerSessionRepository,
    HidDeviceRepository, ProController,
};
use crate::domain::painting::{
    ArtworkToCommandConverter, DrawingCanvasConfig, DrawingStrategy,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::info;
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

    pub async fn execute(&self, artwork: &Artwork, config: PaintConfig) -> Result<PaintResult, ControllerError> {
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

        // 6. セッションの作成
        let session_id = Uuid::new_v4().to_string();
        let mut session = ControllerSession::new(&session_id, &controller.id);
        
        // コマンドをキューに追加
        for command in commands {
            session.queue_command(command);
        }
        
        self.session_repo.create_session(&session).await?;

        // 7. セッションの実行
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

                // HIDレポートを送信
                let report = controller.current_state;
                self.hid_repo.write_report(device_path, &report).await?;

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

        // 8. クリーンアップ
        controller.reset_state();
        self.controller_repo.update_controller(&controller).await?;
        
        // 最終レポートを送信（すべてニュートラル）
        self.hid_repo.write_report(device_path, &controller.current_state).await?;

        // セッションを終了
        session.stop();
        self.session_repo.update_session(&session).await?;

        // 9. 結果をまとめる
        let drawable_dots = artwork.canvas.drawable_dots().len();
        let result = PaintResult {
            success: true,
            dots_painted: drawable_dots,
            commands_executed: executed_commands,
            duration_ms: session.started_at.map(|start| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                now - start
            }).unwrap_or(0),
            device_used: device_path.clone(),
        };

        info!("アートワークの描画が完了しました: {:?}", result);
        Ok(result)
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
            cursor_speed_ms: 50,
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