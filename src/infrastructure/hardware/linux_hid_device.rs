use crate::domain::controller::{ControllerError, HidDeviceRepository, HidReport};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info};

pub struct LinuxHidDeviceRepository {
    default_device_path: String,
}

impl Default for LinuxHidDeviceRepository {
    fn default() -> Self {
        Self {
            default_device_path: "/dev/hidg0".to_string(),
        }
    }
}

impl LinuxHidDeviceRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_device_path(device_path: impl Into<String>) -> Self {
        Self {
            default_device_path: device_path.into(),
        }
    }
}

#[async_trait]
impl HidDeviceRepository for LinuxHidDeviceRepository {
    async fn write_report(
        &self,
        device_path: &str,
        report: &HidReport,
    ) -> Result<(), ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        debug!("Writing HID report to {}", path);

        // HIDレポートをバイト配列に変換
        let report_bytes = report.to_bytes();

        // デバイスファイルを開く
        let mut file = OpenOptions::new()
            .write(true)
            .open(path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ControllerError::PermissionDenied
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ControllerError::DevicePathNotAvailable(path.to_string())
                } else {
                    ControllerError::HidWriteFailed(format!("Failed to open {path}: {e}"))
                }
            })?;

        // レポートを書き込む
        file.write_all(&report_bytes).await.map_err(|e| {
            ControllerError::HidWriteFailed(format!("Failed to write report: {e}"))
        })?;

        // 確実にフラッシュする
        file.flush()
            .await
            .map_err(|e| ControllerError::HidWriteFailed(format!("Failed to flush: {e}")))?;

        debug!("HID report written successfully: {:?}", report_bytes);
        Ok(())
    }

    async fn read_report(&self, device_path: &str) -> Result<HidReport, ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        debug!("Reading HID report from {}", path);

        // デバイスファイルを開く
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ControllerError::PermissionDenied
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ControllerError::DevicePathNotAvailable(path.to_string())
                } else {
                    ControllerError::HidReadFailed(format!("Failed to open {path}: {e}"))
                }
            })?;

        // 8バイトのバッファを準備
        let mut buffer = vec![0u8; 8];

        // レポートを読み込む
        use tokio::io::AsyncReadExt;
        file.read_exact(&mut buffer)
            .await
            .map_err(|e| ControllerError::HidReadFailed(format!("Failed to read report: {e}")))?;

        // バイト配列からHidReportに変換
        HidReport::from_bytes(&buffer).map_err(|_e| ControllerError::InvalidHidReport)
    }

    async fn open_device(&self, device_path: &str) -> Result<(), ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        // デバイスが存在するか確認
        if !Path::new(path).exists() {
            return Err(ControllerError::DevicePathNotAvailable(path.to_string()));
        }

        // 権限チェック（書き込み可能か）
        match OpenOptions::new().write(true).open(path).await {
            Ok(_) => {
                info!("HID device {} is accessible", path);
                Ok(())
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    Err(ControllerError::PermissionDenied)
                } else {
                    Err(ControllerError::DeviceInitFailed(format!(
                        "Cannot open {path}: {e}"
                    )))
                }
            }
        }
    }

    async fn close_device(&self, _device_path: &str) -> Result<(), ControllerError> {
        // Linuxでは特別なクローズ処理は不要
        // ファイルハンドルは自動的にクローズされる
        Ok(())
    }

    async fn list_devices(&self) -> Result<Vec<String>, ControllerError> {
        let mut devices = Vec::new();

        // /dev/hidg* パターンでデバイスを探す
        let dev_path = Path::new("/dev");

        let mut entries = tokio::fs::read_dir(dev_path)
            .await
            .map_err(|e| ControllerError::IoError(format!("Failed to read /dev: {e}")))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| ControllerError::IoError(format!("Failed to read entry: {e}")))?
        {
            if let Ok(name) = entry.file_name().into_string() {
                if name.starts_with("hidg") {
                    devices.push(format!("/dev/{name}"));
                }
            }
        }

        info!("Found HID devices: {:?}", devices);
        Ok(devices)
    }

    async fn write_pro_controller_report(
        &self,
        device_path: &str,
        report: &[u8; 64],
    ) -> Result<(), ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        debug!("Writing Pro Controller report to {}", path);

        // デバイスファイルを開く
        let mut file = OpenOptions::new()
            .write(true)
            .open(path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ControllerError::PermissionDenied
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ControllerError::DevicePathNotAvailable(path.to_string())
                } else {
                    ControllerError::HidWriteFailed(format!("Failed to open {path}: {e}"))
                }
            })?;

        // レポートを書き込む
        file.write_all(report).await.map_err(|e| {
            ControllerError::HidWriteFailed(format!("Failed to write report: {e}"))
        })?;

        // 確実にフラッシュする
        file.flush()
            .await
            .map_err(|e| ControllerError::HidWriteFailed(format!("Failed to flush: {e}")))?;

        debug!("Pro Controller report written successfully");
        Ok(())
    }

    async fn read_usb_command(&self, device_path: &str) -> Result<Vec<u8>, ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        debug!("Reading USB command from {}", path);

        // デバイスファイルを開く
        let mut file = OpenOptions::new()
            .read(true)
            .open(path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ControllerError::PermissionDenied
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ControllerError::DevicePathNotAvailable(path.to_string())
                } else {
                    ControllerError::HidReadFailed(format!("Failed to open {path}: {e}"))
                }
            })?;

        // 64バイトのバッファを準備
        let mut buffer = vec![0u8; 64];

        // コマンドを読み込む
        match file.read(&mut buffer).await {
            Ok(n) => {
                buffer.truncate(n);
                debug!("Read {} bytes from USB", n);
                Ok(buffer)
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    // データがない場合は空のベクタを返す
                    Ok(Vec::new())
                } else {
                    Err(ControllerError::HidReadFailed(format!(
                        "Failed to read command: {e}"
                    )))
                }
            }
        }
    }

    async fn write_usb_response(
        &self,
        device_path: &str,
        response: &[u8],
    ) -> Result<(), ControllerError> {
        let path = if device_path.is_empty() {
            &self.default_device_path
        } else {
            device_path
        };

        debug!(
            "Writing USB response to {} ({} bytes)",
            path,
            response.len()
        );

        // デバイスファイルを開く
        let mut file = OpenOptions::new()
            .write(true)
            .open(path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    ControllerError::PermissionDenied
                } else if e.kind() == std::io::ErrorKind::NotFound {
                    ControllerError::DevicePathNotAvailable(path.to_string())
                } else {
                    ControllerError::HidWriteFailed(format!("Failed to open {path}: {e}"))
                }
            })?;

        // レスポンスを書き込む
        file.write_all(response).await.map_err(|e| {
            ControllerError::HidWriteFailed(format!("Failed to write response: {e}"))
        })?;

        file.flush()
            .await
            .map_err(|e| ControllerError::HidWriteFailed(format!("Failed to flush: {e}")))?;

        debug!("USB response written successfully");
        Ok(())
    }
}
