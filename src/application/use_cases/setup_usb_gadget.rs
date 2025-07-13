use crate::domain::hardware::{
    BoardModel, BoardRepository, HardwareError, SystemdService, SystemdServiceRepository,
    UsbGadget, UsbGadgetRepository,
};
use tracing::{debug, info, warn};

pub struct SetupUsbGadgetUseCase<BR, UR, SR> {
    board_repo: BR,
    gadget_repo: UR,
    service_repo: SR,
}

impl<BR, UR, SR> SetupUsbGadgetUseCase<BR, UR, SR>
where
    BR: BoardRepository,
    UR: UsbGadgetRepository,
    SR: SystemdServiceRepository,
{
    pub fn new(board_repo: BR, gadget_repo: UR, service_repo: SR) -> Self {
        Self {
            board_repo,
            gadget_repo,
            service_repo,
        }
    }

    pub async fn execute(&self, force: bool) -> Result<SetupResult, HardwareError> {
        info!("USB Gadgetセットアップを開始します");

        // 1. ボードの検出
        info!("ボードモデルを検出中...");
        let mut board = self.board_repo.detect_board().await?;
        info!("検出されたボード: {}", board.model);

        // 2. ボードがUSB OTGをサポートしているか確認
        if !board.model.supports_usb_otg() {
            return Err(HardwareError::BoardNotSupported(board.model.to_string()));
        }

        // 3. カーネルモジュールの確認とロード
        info!("カーネルモジュールを確認中...");
        self.board_repo.check_kernel_modules(&mut board).await?;

        let required_modules = board.required_modules();
        if !required_modules.is_empty() {
            info!("必要なカーネルモジュールをロード中...");
            for module in required_modules {
                info!("モジュール {} をロード中...", module.name);
                self.board_repo.load_kernel_module(&module.name).await?;
            }
        }

        // 4. ブート設定の更新
        if force || !board.usb_otg_available {
            info!("ブート設定を更新中...");
            self.board_repo.configure_boot_settings(&board).await?;
        }

        // 5. USB Gadgetの作成と設定
        info!("USB Gadgetを作成中...");
        let mut gadget = UsbGadget::nintendo_controller();

        // 既存のGadgetがあるか確認
        match self.gadget_repo.get_gadget_state(&gadget.id).await {
            Ok(existing) if existing.is_active() && !force => {
                warn!("USB Gadgetは既に設定されています。--forceオプションで再設定できます。");
                return Ok(SetupResult {
                    board_model: board.model,
                    gadget_created: false,
                    service_installed: false,
                    reboot_required: false,
                    warnings: vec!["USB Gadgetは既に設定されています".to_string()],
                });
            }
            Ok(existing) => {
                if existing.is_active() {
                    info!("既存のGadgetを非アクティブ化中...");
                    self.gadget_repo.deactivate_gadget(&mut gadget).await?;
                }
            }
            Err(_) => {
                debug!("既存のGadgetは見つかりませんでした");
            }
        }

        // Gadgetを作成
        self.gadget_repo.create_gadget(&gadget).await?;
        self.gadget_repo.configure_gadget(&gadget).await?;
        self.gadget_repo.activate_gadget(&mut gadget).await?;
        info!("USB Gadgetが正常に作成されました");

        // 6. systemdサービスの設定
        info!("systemdサービスを設定中...");
        let mut service = SystemdService::nintendo_controller_service();

        self.service_repo.create_service(&service).await?;
        self.service_repo.enable_service(&mut service).await?;
        self.service_repo.reload_daemon().await?;

        if force {
            self.service_repo.start_service(&mut service).await?;
            info!("systemdサービスが開始されました");
        }

        // 7. 結果のまとめ
        let reboot_required = !board.usb_otg_available;
        let mut warnings = Vec::new();

        if reboot_required {
            warnings.push("USB OTGを有効にするには再起動が必要です".to_string());
        }

        Ok(SetupResult {
            board_model: board.model,
            gadget_created: true,
            service_installed: true,
            reboot_required,
            warnings,
        })
    }
}

#[derive(Debug, Clone)]
pub struct SetupResult {
    pub board_model: BoardModel,
    pub gadget_created: bool,
    pub service_installed: bool,
    pub reboot_required: bool,
    pub warnings: Vec<String>,
}

impl SetupResult {
    pub fn summary(&self) -> String {
        let mut lines = vec![
            format!("ボードモデル: {}", self.board_model),
            format!(
                "USB Gadget作成: {}",
                if self.gadget_created {
                    "成功"
                } else {
                    "スキップ"
                }
            ),
            format!(
                "サービス設定: {}",
                if self.service_installed {
                    "成功"
                } else {
                    "スキップ"
                }
            ),
        ];

        if self.reboot_required {
            lines.push("⚠️  再起動が必要です".to_string());
        }

        for warning in &self.warnings {
            lines.push(format!("⚠️  {warning}"));
        }

        lines.join("\n")
    }
}
