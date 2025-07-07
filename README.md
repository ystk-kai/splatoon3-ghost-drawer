# Splatoon3 Ghost Drawer

Nintendo Switch Pro Controllerをエミュレートして、Splatoon3の広場で画像を自動描画するシステムです。  
USB OTG機能を使用してSwitchに接続し、画像データを忠実に再現します。

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

### 1. インストール

```bash
# Rustツールチェーンのインストール（未インストールの場合）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# このプロジェクトをインストール
git clone https://github.com/yourusername/splatoon3-ghost-drawer.git
cd splatoon3-ghost-drawer
cargo install --path .
```

### 2. システムセットアップ（初回のみ）

```bash
# USB Gadgetモードの設定とsystemdサービスの登録
sudo splatoon3-ghost-drawer setup
```

### 3. アプリケーションの起動

```bash
# Webサーバーを起動（デフォルト: 0.0.0.0:8080）
splatoon3-ghost-drawer run

# カスタムポートで起動
splatoon3-ghost-drawer run --port 3000

# ローカルホストのみで起動
splatoon3-ghost-drawer run --host 127.0.0.1
```

### 4. Web UIにアクセス

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

4. **インストール**
```bash
# リリースビルドしてシステムにインストール（推奨）
cargo install --path .
# → ~/.cargo/bin/splatoon3-ghost-drawer にインストールされます
# → PATHが通っているため、どこからでも実行可能

# または手動でビルドして実行
cargo build --release
# → ./target/release/splatoon3-ghost-drawer を直接実行
```

5. **初期セットアップと実行**
```bash
# システムセットアップ（初回のみ、要root権限）
sudo splatoon3-ghost-drawer setup

# アプリケーション起動
splatoon3-ghost-drawer run
```

### 使用方法

#### CLIコマンド

このアプリケーションは3つのコマンドをサポートしています：

##### `setup` - システムセットアップ
```bash
# USB Gadgetモードの設定とsystemdサービスの登録（要root権限）
sudo splatoon3-ghost-drawer setup

# 強制的に再セットアップ（既存の設定を上書き）
sudo splatoon3-ghost-drawer setup --force
```

##### `run` - アプリケーション実行
```bash
# Webサーバーの起動（デフォルト: 0.0.0.0:8080）
splatoon3-ghost-drawer run

# カスタムホストとポートで起動
splatoon3-ghost-drawer run --host 127.0.0.1 --port 3000

# すべてのインターフェースで特定のポートで起動
splatoon3-ghost-drawer run --port 8888
```

##### ヘルプとバージョン
```bash
# ヘルプの表示
splatoon3-ghost-drawer --help
splatoon3-ghost-drawer <command> --help

# バージョンの表示
splatoon3-ghost-drawer --version
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

# 手動でUSB Gadgetを設定（通常はsystemdが自動実行）
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
