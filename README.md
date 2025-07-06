# Splatoon3 Ghost Drawer

Nintendo Switch Pro Controllerをエミュレートして、Splatoon3の広場で画像を自動描画するシステムです。USB OTG機能を使用してSwitchに接続し、画像データを忠実に再現します。

## 主な機能

- 🎨 画像ファイルからSplatoon3用ドットデータへの自動変換
- 🎮 Nintendo Switch Pro Controllerの完全エミュレーション
- 🔌 USB OTG経由でのSwitch直接接続
- 🌐 Web UIによる直感的な操作とリアルタイム制御
- 📊 描画進捗のリアルタイム監視とログストリーミング
- 🚀 高速な画像処理と最適化されたドット配置

## 技術スタック

- **言語**: Rust 2024 Edition
- **アーキテクチャ**: Domain-Driven Design (DDD)
- **Webフレームワーク**: Axum
- **非同期ランタイム**: Tokio
- **対応プラットフォーム**: Linux (USB Gadget API対応)

## 対応ハードウェア

USB OTG (On-The-Go) 機能をサポートするLinuxボードが必要です：

- **Raspberry Pi Zero / Zero W / Zero 2W**
- **Orange Pi Zero 2W**
- その他のUSB Gadget API対応Linuxデバイス

## クイックスタート

### 1. システムセットアップ（初回のみ）

```bash
# USB Gadgetモードの設定とsystemdサービスの登録
sudo splatoon3-ghost-drawer setup
```

### 2. アプリケーションの起動

```bash
# Webサーバーを起動（デフォルト: 0.0.0.0:8080）
splatoon3-ghost-drawer run

# カスタムポートで起動
splatoon3-ghost-drawer run --port 3000

# ローカルホストのみで起動
splatoon3-ghost-drawer run --host 127.0.0.1
```

### 3. Web UIにアクセス

ブラウザで `http://[デバイスのIPアドレス]:8080` にアクセスして操作を開始します。

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

4. **ビルドと実行**
```bash
# デバッグビルド
cargo build

# リリースビルド（推奨）
cargo build --release

# セットアップ（初回のみ、要root権限）
sudo ./target/release/splatoon3-ghost-drawer setup

# アプリケーション起動
./target/release/splatoon3-ghost-drawer run
```

### 使用方法

#### CLIコマンド

```bash
# ヘルプの表示
splatoon3-ghost-drawer --help

# システムセットアップ（初回のみ）
sudo splatoon3-ghost-drawer setup

# 強制的に再セットアップ
sudo splatoon3-ghost-drawer setup --force

# Webサーバーの起動
splatoon3-ghost-drawer run

# カスタム設定でサーバー起動
splatoon3-ghost-drawer run --host 0.0.0.0 --port 8080
```

#### Web UIの使用

1. `splatoon3-ghost-drawer run` でサーバーを起動
2. ブラウザで `http://[IPアドレス]:8080` にアクセス
3. 画像をアップロードして変換・描画を実行

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
# USB Gadgetの状態確認
sudo systemctl status splatoon3-gadget.service

# カーネルモジュールの確認
lsmod | grep -E "dwc2|libcomposite"

# 手動でUSB Gadgetを設定
sudo splatoon3-ghost-drawer _internal_configure_gadget
```

### Web UIにアクセスできない

```bash
# サーバーが起動しているか確認
ps aux | grep splatoon3-ghost-drawer

# ポートが開いているか確認
sudo lsof -i :8080

# ファイアウォールの確認
sudo iptables -L -n | grep 8080
```

### Nintendo Switchで認識されない

```bash
# HIDデバイスの確認
ls /dev/hidg*

# USB Gadgetの状態確認
cat /sys/kernel/config/usb_gadget/g1/UDC

# dmesgでUSB関連のログを確認
dmesg | tail -50 | grep -i usb
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