#!/bin/bash

# Splatoon3 Ghost Drawer デバッグスクリプト
# このスクリプトは様々なデバッグ機能を提供します

set -e

# 色付きの出力
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ログ関数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# プロジェクトルートの確認
if [ ! -f "Cargo.toml" ]; then
    log_error "プロジェクトルートで実行してください"
    exit 1
fi

# 使用方法を表示
show_usage() {
    echo "Splatoon3 Ghost Drawer デバッグスクリプト"
    echo ""
    echo "使用方法:"
    echo "  $0 [コマンド] [オプション]"
    echo ""
    echo "コマンド:"
    echo "  build-debug        デバッグビルドを実行"
    echo "  run-debug          デバッグモードで実行"
    echo "  test-debug         デバッグ情報付きでテストを実行"
    echo "  analyze-logs       ログファイルを分析"
    echo "  monitor-memory     メモリ使用量を監視"
    echo "  check-usb          USB OTGの状態を確認"
    echo "  simulate-error     エラーシミュレーション"
    echo "  performance-test   パフォーマンステスト"
    echo "  clean-logs         ログファイルをクリア"
    echo "  system-info        システム情報を表示"
    echo "  help               このヘルプを表示"
    echo ""
    echo "オプション:"
    echo "  --verbose          詳細出力"
    echo "  --trace            トレースレベルのログ"
    echo "  --no-color         色付き出力を無効化"
    echo ""
}

# デバッグビルド
build_debug() {
    log_info "デバッグビルドを開始します..."
    
    # 環境変数を設定
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    
    # ビルド実行
    cargo build --verbose
    
    if [ $? -eq 0 ]; then
        log_info "デバッグビルドが完了しました"
    else
        log_error "デバッグビルドに失敗しました"
        exit 1
    fi
}

# デバッグ実行
run_debug() {
    log_info "デバッグモードで実行します..."
    
    # 環境変数を設定
    export RUST_LOG=debug
    export RUST_BACKTRACE=full
    export RUST_ENV=development
    
    # 引数を処理
    local args=""
    if [ "$1" = "--trace" ]; then
        export RUST_LOG=trace
        shift
    fi
    
    # 実行
    cargo run -- "$@"
}

# テストをデバッグモードで実行
test_debug() {
    log_info "デバッグ情報付きでテストを実行します..."
    
    # 環境変数を設定
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    export RUST_TEST_THREADS=1
    
    # テスト実行
    cargo test --verbose -- --nocapture
}

# ログファイルを分析
analyze_logs() {
    log_info "ログファイルを分析します..."
    
    local log_dir="logs"
    if [ ! -d "$log_dir" ]; then
        log_warn "ログディレクトリが見つかりません: $log_dir"
        return 1
    fi
    
    # 最新のログファイルを探す
    local latest_log=$(find "$log_dir" -name "*.log" -type f -printf '%T@ %p\n' | sort -n | tail -1 | cut -d' ' -f2-)
    
    if [ -z "$latest_log" ]; then
        log_warn "ログファイルが見つかりません"
        return 1
    fi
    
    log_info "最新のログファイル: $latest_log"
    
    # ログ統計を表示
    echo ""
    echo "=== ログ統計 ==="
    echo "総行数: $(wc -l < "$latest_log")"
    echo "エラー数: $(grep -c '"level":"ERROR"' "$latest_log" 2>/dev/null || echo 0)"
    echo "警告数: $(grep -c '"level":"WARN"' "$latest_log" 2>/dev/null || echo 0)"
    echo "情報数: $(grep -c '"level":"INFO"' "$latest_log" 2>/dev/null || echo 0)"
    
    # 最新のエラーを表示
    echo ""
    echo "=== 最新のエラー（最大5件） ==="
    grep '"level":"ERROR"' "$latest_log" | tail -5 | while read line; do
        echo "$line" | jq -r '"\(.timestamp) - \(.message)"' 2>/dev/null || echo "$line"
    done
}

# メモリ使用量を監視
monitor_memory() {
    log_info "メモリ使用量の監視を開始します..."
    log_info "Ctrl+C で終了します"
    
    while true; do
        # プロセスを探す
        local pid=$(pgrep -f "splatoon3-ghost-drawer" | head -1)
        
        if [ -n "$pid" ]; then
            # メモリ使用量を取得
            local memory=$(ps -p "$pid" -o rss= 2>/dev/null)
            if [ -n "$memory" ]; then
                local memory_mb=$((memory / 1024))
                echo "$(date '+%Y-%m-%d %H:%M:%S') - PID: $pid, メモリ使用量: ${memory_mb}MB"
            fi
        else
            echo "$(date '+%Y-%m-%d %H:%M:%S') - プロセスが見つかりません"
        fi
        
        sleep 5
    done
}

# USB OTGの状態を確認
check_usb() {
    log_info "USB OTGの状態を確認します..."
    
    # USB Gadgetの状態を確認
    if [ -d "/sys/kernel/config/usb_gadget" ]; then
        log_info "USB Gadget設定が利用可能です"
        
        # 設定されているGadgetを列挙
        for gadget in /sys/kernel/config/usb_gadget/*; do
            if [ -d "$gadget" ]; then
                local name=$(basename "$gadget")
                log_info "Gadget: $name"
                
                # UDCの状態を確認
                if [ -f "$gadget/UDC" ]; then
                    local udc=$(cat "$gadget/UDC")
                    if [ -n "$udc" ]; then
                        log_info "  UDC: $udc (アクティブ)"
                    else
                        log_warn "  UDC: 未設定"
                    fi
                fi
            fi
        done
    else
        log_warn "USB Gadget設定が利用できません"
    fi
    
    # HIDデバイスを確認
    if ls /dev/hidg* >/dev/null 2>&1; then
        log_info "HIDデバイスが見つかりました:"
        ls -la /dev/hidg*
    else
        log_warn "HIDデバイスが見つかりません"
    fi
}

# エラーシミュレーション
simulate_error() {
    log_info "エラーシミュレーションを実行します..."
    
    # 一時的なテストファイルを作成
    local test_file="/tmp/splatoon3_error_test.txt"
    echo "test" > "$test_file"
    
    # 権限を削除してアクセスエラーを発生させる
    chmod 000 "$test_file"
    
    # アプリケーションを実行してエラーを発生させる
    export RUST_LOG=debug
    export RUST_BACKTRACE=1
    
    log_info "アクセス権限エラーをシミュレートします..."
    cargo run -- info "$test_file" || true
    
    # クリーンアップ
    chmod 644 "$test_file"
    rm -f "$test_file"
    
    log_info "エラーシミュレーションが完了しました"
}

# パフォーマンステスト
performance_test() {
    log_info "パフォーマンステストを実行します..."
    
    # テスト用の画像を作成（ImageMagickが必要）
    local test_image="/tmp/test_image.png"
    if command -v convert >/dev/null 2>&1; then
        convert -size 320x240 xc:white "$test_image"
        log_info "テスト画像を作成しました: $test_image"
    else
        log_warn "ImageMagickが見つかりません。テスト画像をスキップします"
        return 1
    fi
    
    # パフォーマンス測定
    export RUST_LOG=info
    export RUST_BACKTRACE=1
    
    log_info "画像変換のパフォーマンスを測定します..."
    time cargo run -- convert "$test_image" --format png --resolution 320x120
    
    # クリーンアップ
    rm -f "$test_image"
    
    log_info "パフォーマンステストが完了しました"
}

# ログファイルをクリア
clean_logs() {
    log_info "ログファイルをクリアします..."
    
    local log_dir="logs"
    if [ -d "$log_dir" ]; then
        rm -rf "$log_dir"/*
        log_info "ログファイルをクリアしました"
    else
        log_warn "ログディレクトリが見つかりません: $log_dir"
    fi
}

# システム情報を表示
system_info() {
    log_info "システム情報を表示します..."
    
    echo ""
    echo "=== システム情報 ==="
    echo "OS: $(uname -s)"
    echo "カーネル: $(uname -r)"
    echo "アーキテクチャ: $(uname -m)"
    echo "ホスト名: $(hostname)"
    echo ""
    
    echo "=== ハードウェア情報 ==="
    echo "CPU: $(nproc) コア"
    if [ -f /proc/cpuinfo ]; then
        echo "CPU詳細: $(grep 'model name' /proc/cpuinfo | head -1 | cut -d':' -f2 | xargs)"
    fi
    if [ -f /proc/meminfo ]; then
        echo "メモリ: $(grep MemTotal /proc/meminfo | awk '{print $2/1024/1024 " GB"}')"
    fi
    echo ""
    
    echo "=== Rust情報 ==="
    echo "Rustcバージョン: $(rustc --version)"
    echo "Cargoバージョン: $(cargo --version)"
    echo ""
    
    echo "=== USB情報 ==="
    if command -v lsusb >/dev/null 2>&1; then
        echo "USB デバイス:"
        lsusb
    else
        echo "lsusb コマンドが見つかりません"
    fi
}

# メイン処理
main() {
    case "${1:-help}" in
        "build-debug")
            build_debug
            ;;
        "run-debug")
            shift
            run_debug "$@"
            ;;
        "test-debug")
            test_debug
            ;;
        "analyze-logs")
            analyze_logs
            ;;
        "monitor-memory")
            monitor_memory
            ;;
        "check-usb")
            check_usb
            ;;
        "simulate-error")
            simulate_error
            ;;
        "performance-test")
            performance_test
            ;;
        "clean-logs")
            clean_logs
            ;;
        "system-info")
            system_info
            ;;
        "help"|"--help"|"-h")
            show_usage
            ;;
        *)
            log_error "不明なコマンド: $1"
            show_usage
            exit 1
            ;;
    esac
}

# スクリプトを実行
main "$@" 