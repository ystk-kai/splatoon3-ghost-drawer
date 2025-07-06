# Splatoon3 Ghost Drawer - デバッグガイド

このドキュメントは、Splatoon3 Ghost Drawerプロジェクトの効率的なデバッグ方法について説明します。

## 🔧 デバッグツールの概要

### 1. 構造化ログシステム

プロジェクトは`tracing`クレートを使用した構造化ログシステムを採用しています。

#### ログレベル
- **ERROR**: 致命的なエラー
- **WARN**: 警告（処理は継続可能）
- **INFO**: 一般的な情報
- **DEBUG**: デバッグ情報
- **TRACE**: 詳細なトレース情報

#### ログ設定
```bash
# 環境変数でログレベルを設定
export RUST_LOG=debug
export RUST_BACKTRACE=1

# 実行
cargo run
```

### 2. デバッグスクリプト

`scripts/debug.sh`を使用して様々なデバッグ機能を利用できます。

```bash
# デバッグスクリプトの使用方法
./scripts/debug.sh help

# 主要コマンド
./scripts/debug.sh build-debug      # デバッグビルド
./scripts/debug.sh run-debug        # デバッグ実行
./scripts/debug.sh test-debug       # デバッグテスト
./scripts/debug.sh analyze-logs     # ログ分析
./scripts/debug.sh monitor-memory   # メモリ監視
./scripts/debug.sh check-usb        # USB OTG確認
```

## 🐛 一般的なデバッグ手順

### 1. 基本的なデバッグフロー

```bash
# 1. デバッグビルド
./scripts/debug.sh build-debug

# 2. デバッグ実行
./scripts/debug.sh run-debug test

# 3. ログ分析
./scripts/debug.sh analyze-logs
```

### 2. エラーが発生した場合

```bash
# 1. 詳細なエラー情報を取得
export RUST_BACKTRACE=full
export RUST_LOG=debug

# 2. エラーシミュレーション
./scripts/debug.sh simulate-error

# 3. システム情報確認
./scripts/debug.sh system-info
```

## 🔍 特定の問題のデバッグ

### USB OTG関連の問題

```bash
# USB OTGの状態確認
./scripts/debug.sh check-usb

# 手動でのUSB Gadget確認
ls -la /sys/kernel/config/usb_gadget/
cat /sys/kernel/config/usb_gadget/*/UDC

# HIDデバイスの確認
ls -la /dev/hidg*
```

### メモリ関連の問題

```bash
# メモリ使用量の監視
./scripts/debug.sh monitor-memory

# Raspberry Pi Zero 2Wでのメモリ制限対策
# 1. 画像サイズを事前に縮小
# 2. バッチ処理を避ける
# 3. 不要なデータを早期に解放
```

### 画像処理の問題

```bash
# パフォーマンステスト
./scripts/debug.sh performance-test

# 画像処理のデバッグログを有効化
export RUST_LOG=splatoon3_ghost_drawer::domain::artwork=debug
```

### Web UI関連の問題

```bash
# ブラウザの開発者ツールを使用
# 1. F12でデベロッパーツールを開く
# 2. Consoleタブでエラーを確認
# 3. Networkタブで通信を確認

# サーバーサイドのデバッグ
export RUST_LOG=debug
cargo run -- --web-ui
```

## 📊 パフォーマンス測定

### 1. 基本的なパフォーマンス測定

```rust
use splatoon3_ghost_drawer::measure_time;

// 操作の実行時間を測定
let result = measure_time!("image_conversion", {
    // 画像変換処理
    convert_image(input, output)
});
```

### 2. 詳細なパフォーマンス分析

```rust
use splatoon3_ghost_drawer::debug::debug_helpers::PerformanceStats;

let mut stats = PerformanceStats::new();

// 操作時間を記録
let start = std::time::Instant::now();
perform_operation();
let duration = start.elapsed();
stats.record_operation("operation_name", duration.as_millis());

// 統計を出力
stats.log_summary();
```

## 🧪 テストデバッグ

### 1. 単体テストのデバッグ

```bash
# 詳細なテスト出力
./scripts/debug.sh test-debug

# 特定のテストのみ実行
cargo test test_name -- --nocapture

# テストの並列実行を無効化
export RUST_TEST_THREADS=1
cargo test
```

### 2. 統合テストのデバッグ

```bash
# 統合テストの実行
cargo test --test integration_tests

# 特定の統合テスト
cargo test --test integration_tests specific_test
```

## 🚨 トラブルシューティング

### よくある問題と解決方法

#### 1. ビルドエラー

```bash
# 依存関係の問題
cargo clean
cargo build

# Rustのバージョン確認
rustc --version
rustup update
```

#### 2. USB OTGが認識されない

```bash
# カーネルモジュールの確認
lsmod | grep dwc2

# USB Gadgetの設定確認
sudo systemctl status nintendo-controller.service

# 権限の確認
sudo chmod 666 /dev/hidg*
```

#### 3. メモリ不足エラー

```bash
# スワップファイルの作成（Raspberry Pi）
sudo dphys-swapfile swapoff
sudo nano /etc/dphys-swapfile  # CONF_SWAPSIZE=1024
sudo dphys-swapfile setup
sudo dphys-swapfile swapon
```

#### 4. 画像処理が遅い

```bash
# 画像サイズの確認
identify input.png

# 事前リサイズ
convert input.png -resize 320x240 resized.png

# 並列処理の無効化
export RAYON_NUM_THREADS=1
```

## 📝 ログファイルの分析

### ログファイルの場所

```
logs/
├── splatoon3-ghost-drawer.log      # 日次ローテーション
├── splatoon3-ghost-drawer.log.1    # 前日のログ
└── ...
```

### ログ分析コマンド

```bash
# エラーのみ抽出
grep '"level":"ERROR"' logs/splatoon3-ghost-drawer.log

# 特定の時間範囲
grep '"timestamp":"2024-01-01T12:' logs/splatoon3-ghost-drawer.log

# パフォーマンス統計
grep '"operation":' logs/splatoon3-ghost-drawer.log | jq '.duration_ms'
```

## 🔄 継続的なデバッグ

### 1. 自動ログ監視

```bash
# リアルタイムログ監視
tail -f logs/splatoon3-ghost-drawer.log

# エラーのみ監視
tail -f logs/splatoon3-ghost-drawer.log | grep ERROR
```

### 2. 定期的なヘルスチェック

```bash
# crontabに追加
*/5 * * * * /path/to/scripts/debug.sh check-usb >> /var/log/health-check.log
```

## 🎯 効果的なデバッグのベストプラクティス

### 1. 段階的なデバッグ

1. **再現可能な最小ケースを作成**
2. **ログレベルを段階的に上げる**
3. **一つずつ問題を切り分ける**

### 2. ログの活用

```rust
// 構造化ログの活用例
use tracing::{info, warn, error, debug};

debug!(
    artwork_id = %artwork.id(),
    canvas_size = ?artwork.canvas().size(),
    "アートワーク処理開始"
);

// エラーの詳細ログ
if let Err(e) = process_artwork(&artwork) {
    error!(
        artwork_id = %artwork.id(),
        error = %e,
        "アートワーク処理に失敗"
    );
}
```

### 3. テスト駆動デバッグ

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::test_helpers::init_test_logging;

    #[test]
    fn test_artwork_creation() {
        init_test_logging();
        
        // テストコード
        let artwork = create_test_artwork();
        assert!(artwork.is_valid());
    }
}
```

## 📚 参考リソース

- [Rust tracing documentation](https://docs.rs/tracing/)
- [USB OTG on Raspberry Pi](https://www.raspberrypi.org/documentation/hardware/computemodule/cm-otg.md)
- [Rust performance book](https://nnethercote.github.io/perf-book/)

このガイドを参考に、効率的なデバッグを行ってください。問題が解決しない場合は、ログファイルと共にIssueを作成してください。 