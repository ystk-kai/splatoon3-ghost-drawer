# Splatoon3 Ghost Drawer

このプロジェクトは Domain-Driven Design (DDD) 原則に基づいて設計されており、Rust 2024 Edition を使用して実装されています。

## 主な機能

- 画像ファイルからSplatoon3用ドットデータへの変換
- Nintendo Switch Pro Controllerエミュレーション
- USB OTG経由での自動描画実行
- Web UIによる直感的な操作
- リアルタイム進捗監視

## 技術スタック

- **言語**: Rust 2024 Edition
- **アーキテクチャ**: Domain-Driven Design (DDD)
- **非同期ランタイム**: tokio
- **プラットフォーム**: Orange Pi Zero 2W (推奨) / Raspberry Pi Zero 2W

## 推奨ハードウェア

### 🥇 Orange Pi Zero 2W 2GB（最推奨）

**技術仕様**:
- **CPU**: Allwinner H618 (ARM Cortex-A53 × 4, 1.5GHz)
- **RAM**: 2GB LPDDR4 @ 792MHz
- **USB**: Type-C × 2 (OTG + Host)
- **WiFi**: 802.11ac デュアルバンド
- **Bluetooth**: 5.0
- **価格**: 約$20-25

**利点**:
- 十分なメモリ容量（2GB）
- 高性能CPU（1.5GHz）
- デュアルUSB Type-C（OTG対応）
- 優れた価格性能比

### 🥈 Raspberry Pi Zero 2W

**技術仕様**:
- **CPU**: Broadcom BCM2710A1 (ARM Cortex-A53 × 4, 1.0GHz)
- **RAM**: 512MB LPDDR2
- **USB**: Micro USB OTG
- **WiFi**: 802.11n
- **Bluetooth**: 4.2
- **価格**: 約$15

**利点**:
- 確実なUSB OTG対応
- 豊富な情報とコミュニティサポート
- 低価格・省電力

**制限**:
- メモリ制限（512MB）により大きな画像処理に制約
- 処理速度がOrange Pi Zero 2Wより劣る

## 推奨OS

### Orange Pi Zero 2W: Armbian Noble Server (Ubuntu 24.04)
```bash
# ダウンロード
https://www.armbian.com/orange-pi-zero-2w/

# 推奨イメージ
Armbian_community_24.5.1_Orangepizero2w_noble_current_6.12.y_server.img.xz
```

### Raspberry Pi Zero 2W: Raspberry Pi OS Lite
```bash
# ダウンロード
https://www.raspberrypi.com/software/operating-systems/

# 推奨イメージ
Raspberry Pi OS Lite (64-bit)
```

## 画像処理性能

| 処理内容 | Orange Pi Zero 2W | Raspberry Pi Zero 2W |
|---------|------------------|---------------------|
| **10MB→50KB変換** | 3-8秒 | 15-30秒 |
| **複数画像同時処理** | ✅ 快適 | ⚠️ 制限あり |
| **Webサーバー+変換** | ✅ 快適 | ⚠️ 重い |
| **メモリ使用量** | 余裕あり | ほぼ限界 |

### 推奨用途

- **Orange Pi Zero 2W**: 本格運用、複数画像処理、快適なWeb UI
- **Raspberry Pi Zero 2W**: 学習用、単発処理、軽量運用

## 開発

### 前提条件

- Rust 2024 Edition
- USB OTG対応シングルボードコンピューター
- 十分な電源供給（5V/2A以上推奨）

### セットアップ

1. **リポジトリのクローン**
```bash
git clone https://github.com/ystk-kai/splatoon3-ghost-drawer.git
cd splatoon3-ghost-drawer
```

2. **依存関係のインストール**
```bash
# Orange Pi Zero 2W (Armbian)
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev

# Raspberry Pi Zero 2W (Raspberry Pi OS)
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

3. **Rustのインストール**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup default stable
```

4. **USB Gadgetモードの設定**
```bash
# セットアップスクリプトの実行
sudo ./scripts/setup_gadget.sh
```

5. **ビルドと実行**
```bash
# デバッグビルド
cargo build

# リリースビルド
cargo build --release

# 実行（推奨）
cargo run -- help

# Web UIサーバー起動
cargo run -- serve

# 直接実行（必要に応じて）
sudo ./target/release/splatoon3-ghost-drawer help
```

### 使用方法

#### CLIインターフェース

```bash
# 画像変換
cargo run -- convert input.png --output artwork.json

# 描画実行
cargo run -- paint artwork.json --speed normal

# 設定表示
cargo run -- config

# テストモード
cargo run -- test

# Web UIサーバー起動
cargo run -- serve --port 8080
```

#### Web UI

```bash
# Webサーバー起動
cargo run -- serve

# カスタムポート・ホストで起動
cargo run -- serve --port 8080 --host 0.0.0.0

# ブラウザでアクセス
http://localhost:8080
```

## アーキテクチャ

詳細なアーキテクチャ設計については [docs/architecture.md](docs/architecture.md) を参照してください。

### 主要コンポーネント

- **Domain Layer**: アートワーク、コントローラー、ペインティングのドメインロジック
- **Application Layer**: ユースケースとアプリケーションサービス
- **Infrastructure Layer**: ハードウェア抽象化、USB OTG制御、画像処理
- **Interface Layer**: CLI、Web UI、イベントハンドリング

## 制限事項

1. **ハードウェア制約**: USB OTG対応ボードが必要
2. **性能制約**: 
   - Raspberry Pi Zero 2W: メモリ制限により大きな画像処理に時間がかかる
   - Orange Pi Zero 2W: ほとんどの用途で問題なし
3. **互換性**: Nintendo Switch本体のファームウェアバージョンによる制約
4. **法的制約**: 自動化ツールの使用は利用規約を確認してください

## トラブルシューティング

### USB OTG が認識されない

```bash
# dwc2ドライバーの確認
lsmod | grep dwc2

# USB Gadgetの状態確認
sudo systemctl status nintendo-controller.service

# 手動でのGadget設定
sudo /usr/local/bin/setup-nintendo-controller.sh
```

### 画像変換が遅い

```bash
# メモリ使用量確認
free -h

# CPU使用率確認
htop

# 画像サイズの事前縮小
# クライアント側でのリサイズ機能を利用
```

### Nintendo Switchで認識されない

```bash
# HIDデバイスの確認
ls /dev/hidg*

# USB接続の確認
dmesg | grep -i usb

# 電源供給の確認（5V/2A以上）
```

## 貢献

1. このリポジトリをフォーク
2. 機能ブランチを作成 (`git checkout -b feature/amazing-feature`)
3. 変更をコミット (`git commit -m 'Add amazing feature'`)
4. ブランチにプッシュ (`git push origin feature/amazing-feature`)
5. プルリクエストを作成

## ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。詳細は [LICENSE](LICENSE) ファイルを参照してください。

## 免責事項

このプロジェクトは教育・研究目的で開発されています。Nintendo Switch や Splatoon3 の利用規約を遵守してご使用ください。自動化ツールの使用により発生する問題について、開発者は責任を負いません。

## 謝辞

- Rust コミュニティ
- Armbian プロジェクト
- Orange Pi / Raspberry Pi コミュニティ
- Domain-Driven Design コミュニティ 