# Splatoon3 Ghost Drawer - アーキテクチャ設計書

## 1. ハードウェア要件と対応ボード

### 1.1 USB OTG機能の必要性

本プロジェクトは Nintendo Switch Pro Controller として認識されるため、USB OTG（On-The-Go）機能を持つシングルボードコンピューターが必要です。

#### 1.1.1 技術要件
- **USB OTG サポート**: デバイスモードでの動作
- **dwc2 ドライバー対応**: USB Gadget制御
- **libcomposite モジュール**: HIDデバイスエミュレーション
- **十分なメモリ**: 画像処理とWebサーバーの同時実行

### 1.2 対応ハードウェア

#### 1.2.1 Orange Pi Zero 2W 2GB（最推奨）

**技術仕様**:
```
Orange Pi Zero 2W:
- Allwinner H618 SoC (ARM Cortex-A53 × 4, 1.5GHz)
- RAM: 2GB LPDDR4 @ 792MHz
- USB Type-C × 2 (OTG + Host)
- WiFi 802.11ac / Bluetooth 5.0
- dwc2 ドライバー対応
- libcomposite モジュール対応
```

**利点**:
- 十分なメモリ容量（2GB）
- 高性能CPU（1.5GHz）
- デュアルUSB Type-C（OTG対応）
- 優れた価格性能比

#### 1.2.2 Raspberry Pi Zero 2W

**技術仕様**:
```
Raspberry Pi Zero 2W:
- Broadcom BCM2710A1 SoC (ARM Cortex-A53 × 4, 1.0GHz)
- RAM: 512MB LPDDR2
- Micro USB OTG ポート
- WiFi 802.11n / Bluetooth 4.2
- dwc2 ドライバー対応
- g_hid カーネルモジュール対応
```

**利点**:
- 確実なUSB OTG対応
- 豊富な情報とコミュニティサポート
- 低価格・省電力

**制限**:
- メモリ制限（512MB）により大きな画像処理に制約

## 2. アーキテクチャ概要

本プロジェクトは Domain-Driven Design (DDD) 原則に基づいて設計されており、ハードウェア制約を考慮した柔軟なアーキテクチャを採用しています。

### 2.1 レイヤー構成

**Interface Layer（インターフェース層）**
- CLI インターフェース
- Web API インターフェース
- イベントハンドリング

**Application Layer（アプリケーション層）**
- アプリケーションサービス
- ユースケース
- コマンドオブジェクト

**Domain Layer（ドメイン層）**
- Artwork コンテキスト
- Controller コンテキスト
- Painting コンテキスト

**Infrastructure Layer（インフラストラクチャ層）**
- ハードウェア抽象化
- 画像処理
- Web インターフェース

### 2.2 ハードウェア抽象化戦略

```rust
// ハードウェア抽象化トレイト
trait HardwareController {
    async fn send_hid_report(&self, report: HidReport) -> Result<(), HardwareError>;
    async fn initialize(&self) -> Result<(), HardwareError>;
    fn get_hardware_type(&self) -> HardwareType;
}

// 実装の選択
enum HardwareType {
    OrangePiZero2W,         // USB OTG直接制御（推奨）
    RaspberryPiZero2W,      // USB OTG直接制御
    UsbHidConverter,        // USB-HID変換器
}
```

## 3. ドメイン層の設計

### 3.1 境界付けられたコンテキスト

#### 3.1.1 Artwork Context
**責務**: 画像データの管理、変換、検証

**主要エンティティ**:
- Artwork: アートワーク集約ルート
- Canvas: 描画キャンバス (320x120)
- Dot: 個別ドット

**主要値オブジェクト**:
- ArtworkId: アートワーク識別子
- ImageFormat: 画像形式
- Resolution: 解像度

#### 3.1.2 Controller Context
**責務**: Nintendo Switch Pro Controller の制御とシミュレーション

**主要エンティティ**:
- ProController: コントローラー集約ルート
- ButtonState: ボタン状態
- StickState: スティック状態

**主要値オブジェクト**:
- HidReport: HID レポートデータ
- ButtonInput: ボタン入力
- StickInput: スティック入力

**ハードウェア抽象化対応**:
```rust
// ハードウェア非依存の制御抽象化
impl ProController {
    async fn send_button_press(&self, button: Button) -> Result<(), ControllerError> {
        let report = self.create_hid_report(button);
        self.hardware.send_hid_report(report).await
    }
}
```

#### 3.1.3 Painting Context
**責務**: 実際の描画プロセスの管理と制御

**主要エンティティ**:
- PaintingSession: ペインティングセッション集約ルート
- BrushStroke: ブラシストローク
- Progress: 進捗状況

**主要値オブジェクト**:
- SessionId: セッション識別子
- PaintingSpeed: 描画速度
- PaintingStatus: 描画状態

### 3.2 共有カーネル

**共有値オブジェクト**:
- Coordinates: 2次元座標
- Color: 色情報
- Timestamp: タイムスタンプ

**共有イベント**:
- SystemEvent: システム全体のイベント
- ArtworkEvent: アートワーク関連イベント

## 4. アプリケーション層の設計

### 4.1 主要ユースケース

#### 4.1.1 ProcessImageUseCase
画像処理のユースケース
- 画像の検証
- 画像の変換（320x120、グレースケール、2値化）
- ドットデータの生成
- アートワークエンティティの作成
- 永続化

#### 4.1.2 StartPaintingUseCase
ペインティング開始のユースケース
- アートワークの取得
- ペインティングセッションの作成
- 描画パスの最適化
- コントローラー操作の開始

### 4.2 アプリケーションサービス

#### 4.2.1 ArtworkApplicationService
アートワーク関連のユースケースをオーケストレーション

#### 4.2.2 ControllerApplicationService
コントローラー制御のオーケストレーション

#### 4.2.3 PaintingApplicationService
ペインティングプロセスのオーケストレーション

## 5. インフラストラクチャ層の設計

### 5.1 ハードウェア抽象化実装

#### 5.1.1 Raspberry Pi Zero 2W実装
```rust
struct RaspberryPiZeroController {
    gadget_device: GadgetDevice,
}

impl HardwareController for RaspberryPiZeroController {
    async fn send_hid_report(&self, report: HidReport) -> Result<(), HardwareError> {
        // USB Gadget経由でHIDレポート送信
        self.gadget_device.write_hid_report(report).await
    }
}
```

#### 5.1.2 Arduino Pro Micro実装
```rust
struct ArduinoProMicroController {
    serial_port: SerialPort,
}

impl HardwareController for ArduinoProMicroController {
    async fn send_hid_report(&self, report: HidReport) -> Result<(), HardwareError> {
        // シリアル経由でコマンド送信
        let command = SerialCommand::HidReport(report);
        self.serial_port.send(command).await
    }
}
```

#### 5.1.3 USB-HID変換器実装
```rust
struct UsbHidConverterController {
    usb_device: UsbDevice,
}

impl HardwareController for UsbHidConverterController {
    async fn send_hid_report(&self, report: HidReport) -> Result<(), HardwareError> {
        // USB経由でHID変換器に送信
        self.usb_device.send_raw_data(report.to_bytes()).await
    }
}
```

### 5.2 HID レポート管理
Nintendo Switch Pro Controller プロトコルに従ったHIDレポートの生成と送信

### 5.3 画像処理実装

#### 5.3.1 画像変換パイプライン
- 画像処理ステップの抽象化
- 変換パイプラインの実装
- フォーマット変換

**メモリ制約対応（Raspberry Pi Zero 2W）**:
```rust
// ストリーミング処理による低メモリ実装
impl ImageProcessor {
    async fn process_streaming(&self, input: AsyncRead) -> Result<Canvas, ProcessError> {
        // チャンク単位での処理
        let mut canvas = Canvas::new();
        let mut chunk_processor = ChunkProcessor::new(1024); // 1KB chunks
        
        while let Some(chunk) = input.read_chunk().await? {
            let processed = chunk_processor.process(chunk).await?;
            canvas.merge(processed);
        }
        
        Ok(canvas)
    }
}
```

### 5.4 永続化実装

#### 5.4.1 リポジトリ実装
- インメモリ実装（開発・テスト用）
- ファイル実装（本番用）

## 6. 非同期処理とエラーハンドリング

### 6.1 非同期処理戦略

- tokio を使用した非同期ランタイム
- async/await による非同期処理
- Channel を使用したタスク間通信
- Stream を使用した進捗更新

### 6.2 エラーハンドリング戦略

#### 6.2.1 エラー分類
- ドメインエラー: ビジネスルール違反
- インフラストラクチャエラー: 外部システムエラー
- アプリケーションエラー: ユースケース実行エラー
- ハードウェアエラー: ハードウェア固有エラー

#### 6.2.2 エラー回復戦略
- Retry Policy: 指数バックオフによる再試行
- Circuit Breaker: 障害の拡散防止
- Fallback: 代替手段への切り替え
- Hardware Failover: ハードウェア障害時の代替制御

## 7. パフォーマンス最適化

### 7.1 描画パス最適化

描画順序の最適化アルゴリズム
- 最近傍法による最適化
- 巡回セールスマン問題による最適化

### 7.2 並行処理

- 画像処理: CPU集約的タスクの並列化
- I/O処理: 非同期I/Oによる効率化
- 進捗更新: バックグラウンドタスクでの定期更新

### 7.3 メモリ最適化（Raspberry Pi Zero 2W）

```rust
// メモリ効率的な実装
impl Canvas {
    // 遅延評価による描画データ生成
    fn lazy_dots(&self) -> impl Iterator<Item = Dot> + '_ {
        (0..self.height()).flat_map(move |y| {
            (0..self.width()).filter_map(move |x| {
                if self.has_dot_at(x, y) {
                    Some(Dot::new(Color::black(), 255))
                } else {
                    None
                }
            })
        })
    }
}
```

## 8. テスト戦略

### 8.1 テスト分類

- 単体テスト: ドメインロジックのテスト
- 統合テスト: コンポーネント間の連携テスト
- E2Eテスト: システム全体のテスト
- ハードウェアテスト: ハードウェア固有のテスト

### 8.2 テスト実装

モックを使用したテスト駆動開発
- ドメインロジックの単体テスト
- アプリケーションサービスの統合テスト
- インフラストラクチャの結合テスト

```rust
// ハードウェア抽象化のテスト
#[cfg(test)]
mod tests {
    use super::*;
    
    struct MockHardwareController {
        sent_reports: Arc<Mutex<Vec<HidReport>>>,
    }
    
    impl HardwareController for MockHardwareController {
        async fn send_hid_report(&self, report: HidReport) -> Result<(), HardwareError> {
            self.sent_reports.lock().await.push(report);
            Ok(())
        }
    }
}
```

## 9. 監視とロギング

### 9.1 ログ戦略

- 構造化ログ: JSON形式でのログ出力
- ログレベル: DEBUG, INFO, WARN, ERROR
- ログローテーション: サイズベースの自動ローテーション

### 9.2 メトリクス

- システムメトリクス: CPU、メモリ使用率
- アプリケーションメトリクス: 処理時間、エラー率
- ビジネスメトリクス: 描画成功率、平均描画時間
- ハードウェアメトリクス: HID送信成功率、レイテンシー

## 10. 実装ガイドライン

### 10.1 ハードウェア選択指針

1. **Raspberry Pi Zero 2W（推奨）**:
   - 単純な構成
   - 低コスト
   - USB OTG直接対応

2. **Raspberry Pi 4 + Arduino Pro Micro**:
   - 高性能画像処理
   - 分散アーキテクチャ
   - 拡張性

3. **USB-HID変換器**:
   - 商用製品の安定性
   - 追加開発不要
   - 高コスト

### 10.2 開発優先順位

1. ドメイン層の実装（ハードウェア非依存）
2. ハードウェア抽象化層の実装
3. 具体的なハードウェア実装
4. 統合テストとデバッグ

この設計により、保守性が高く、テスタブルで、拡張可能なSplatoon3自動描画システムを実現します。 