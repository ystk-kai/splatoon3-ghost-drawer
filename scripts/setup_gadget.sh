#!/bin/bash

# Splatoon3 Ghost Drawer - USB Gadget Setup Script
# USB OTG対応ボード用USB Gadgetモード設定スクリプト

set -e

# 色付きメッセージ用の関数
print_error() {
    echo -e "\033[31m[ERROR]\033[0m $1" >&2
}

print_warning() {
    echo -e "\033[33m[WARNING]\033[0m $1"
}

print_info() {
    echo -e "\033[32m[INFO]\033[0m $1"
}

print_success() {
    echo -e "\033[32m[SUCCESS]\033[0m $1"
}

# ボードモデルの検出
detect_board_model() {
    local model_info=$(cat /proc/cpuinfo | grep "Model" | head -1)
    local hardware_info=$(cat /proc/cpuinfo | grep "Hardware" | head -1)
    
    if [[ "$model_info" == *"Orange Pi Zero 2W"* ]] || [[ "$hardware_info" == *"sun50iw9"* ]]; then
        echo "orange_pi_zero_2w"
    elif [[ "$model_info" == *"Raspberry Pi Zero 2"* ]]; then
        echo "raspberry_pi_zero_2w"
    elif [[ "$model_info" == *"Raspberry Pi Zero"* ]]; then
        echo "raspberry_pi_zero"
    else
        echo "unknown"
    fi
}

# USB OTG機能のチェック
check_usb_otg_support() {
    local board_model=$1
    
    case $board_model in
        "orange_pi_zero_2w")
            # Orange Pi Zero 2W: H618チップのUSB OTG確認
            if lsmod | grep -q "dwc2" || modinfo dwc2 >/dev/null 2>&1; then
                return 0
            else
                print_error "Orange Pi Zero 2W: dwc2ドライバーが見つかりません"
                return 1
            fi
            ;;
        "raspberry_pi_zero_2w"|"raspberry_pi_zero")
            # Raspberry Pi Zero 2W/Zero: dwc2ドライバー確認
            if lsmod | grep -q "dwc2" || modinfo dwc2 >/dev/null 2>&1; then
                return 0
            else
                print_error "Raspberry Pi Zero: dwc2ドライバーが見つかりません"
                return 1
            fi
            ;;
        *)
            print_error "USB OTG機能を持たないボードです: $board_model"
            print_info "対応ボード: Orange Pi Zero 2W, Raspberry Pi Zero 2W"
            return 1
            ;;
    esac
}

# USB Gadgetモジュールの設定
setup_gadget_modules() {
    local board_model=$1
    
    print_info "USB Gadgetモジュールを設定中..."
    
    # 共通モジュール
    echo "dwc2" | sudo tee -a /etc/modules >/dev/null 2>&1 || true
    echo "libcomposite" | sudo tee -a /etc/modules >/dev/null 2>&1 || true
    
    case $board_model in
        "orange_pi_zero_2w")
            # Orange Pi Zero 2W固有の設定
            if ! grep -q "dtoverlay=dwc2" /boot/armbianEnv.txt 2>/dev/null; then
                echo "overlays=usbhost2 usbhost3" | sudo tee -a /boot/armbianEnv.txt >/dev/null
                echo "param_dwc2_dr_mode=otg" | sudo tee -a /boot/armbianEnv.txt >/dev/null
                print_info "Orange Pi Zero 2W: USB OTG設定を追加しました"
            fi
            ;;
        "raspberry_pi_zero_2w"|"raspberry_pi_zero")
            # Raspberry Pi Zero固有の設定
            # 新しいファイル構造に対応
            local config_file="/boot/config.txt"
            local firmware_config="/boot/firmware/config.txt"
            local cmdline_file="/boot/firmware/cmdline.txt"
            
            # /boot/firmware/config.txtが存在する場合はそちらを使用
            if [ -f "$firmware_config" ]; then
                config_file="$firmware_config"
            fi
            
            # dtoverlay=dwc2の設定
            if ! grep -q "dtoverlay=dwc2" "$config_file"; then
                echo -e "\n# Enable USB gadget mode\ndtoverlay=dwc2" | sudo tee -a "$config_file" >/dev/null
                print_info "Raspberry Pi Zero: USB OTG設定を追加しました ($config_file)"
            fi
            
            # dwc_otgをブラックリストに追加
            if [ ! -f "/etc/modprobe.d/blacklist-dwc_otg.conf" ]; then
                echo "blacklist dwc_otg" | sudo tee /etc/modprobe.d/blacklist-dwc_otg.conf >/dev/null
                print_info "dwc_otgをブラックリストに追加しました"
            fi
            
            # /etc/modulesにdwc2が含まれていることを確認
            if ! grep -q "^dwc2$" /etc/modules; then
                echo "dwc2" | sudo tee -a /etc/modules >/dev/null
                print_info "/etc/modulesにdwc2を追加しました"
            fi
            ;;
    esac
}

# Nintendo Switch Pro Controllerエミュレーション設定
setup_nintendo_controller() {
    print_info "Nintendo Switch Pro Controllerエミュレーションを設定中..."
    
    # USB Gadget設定スクリプト作成
    cat << 'EOF' | sudo tee /usr/local/bin/setup-nintendo-controller.sh >/dev/null
#!/bin/bash

# Nintendo Switch Pro Controller USB Gadget設定

cd /sys/kernel/config/usb_gadget/
mkdir -p nintendo_controller
cd nintendo_controller

# デバイス記述子
echo 0x057e > idVendor    # Nintendo
echo 0x2009 > idProduct   # Pro Controller
echo 0x0100 > bcdDevice   # v1.0.0
echo 0x0200 > bcdUSB      # USB2

# デバイス情報
mkdir -p strings/0x409
echo "Nintendo Co., Ltd." > strings/0x409/manufacturer
echo "Pro Controller" > strings/0x409/product
echo "000000000001" > strings/0x409/serialnumber

# 設定記述子
mkdir -p configs/c.1/strings/0x409
echo "Config 1: ECM network" > configs/c.1/strings/0x409/configuration
echo 250 > configs/c.1/MaxPower

# HID機能
mkdir -p functions/hid.usb0
echo 1 > functions/hid.usb0/protocol
echo 1 > functions/hid.usb0/subclass
echo 64 > functions/hid.usb0/report_length

# HIDレポート記述子（Nintendo Switch Pro Controller用）
echo -ne \\x05\\x01\\x09\\x05\\xa1\\x01\\x06\\x01\\xff\\x85\\x21\\x09\\x21\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x30\\x09\\x30\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x31\\x09\\x31\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x32\\x09\\x32\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x33\\x09\\x33\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x3f\\x05\\x09\\x19\\x01\\x29\\x10\\x15\\x00\\x25\\x01\\x75\\x01\\x95\\x10\\x81\\x02\\x05\\x01\\x09\\x39\\x15\\x00\\x25\\x07\\x75\\x04\\x95\\x01\\x81\\x42\\x05\\x09\\x75\\x04\\x95\\x01\\x81\\x01\\x05\\x01\\x09\\x30\\x09\\x31\\x09\\x33\\x09\\x34\\x15\\x00\\x27\\xff\\xff\\x00\\x00\\x75\\x10\\x95\\x04\\x81\\x02\\x06\\x01\\xff\\x85\\x01\\x09\\x01\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x02\\x09\\x02\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x03\\x09\\x03\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x04\\x09\\x04\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x05\\x09\\x05\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x06\\x09\\x06\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x07\\x09\\x07\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x08\\x09\\x08\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x09\\x09\\x09\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0a\\x09\\x0a\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0b\\x09\\x0b\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0c\\x09\\x0c\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0d\\x09\\x0d\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0e\\x09\\x0e\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x0f\\x09\\x0f\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x10\\x09\\x10\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x11\\x09\\x11\\x75\\x08\\x95\\x30\\x81\\x02\\x85\\x12\\x09\\x12\\x75\\x08\\x95\\x30\\x81\\x02\\xc0 > functions/hid.usb0/report_desc

# 機能を設定にリンク
ln -s functions/hid.usb0 configs/c.1/

# UDCを有効化
ls /sys/class/udc > UDC

echo "Nintendo Switch Pro Controllerエミュレーション設定完了"
EOF

    sudo chmod +x /usr/local/bin/setup-nintendo-controller.sh
    
    # systemdサービス作成
    cat << 'EOF' | sudo tee /etc/systemd/system/nintendo-controller.service >/dev/null
[Unit]
Description=Nintendo Switch Pro Controller USB Gadget
After=multi-user.target
Wants=multi-user.target

[Service]
Type=oneshot
ExecStart=/usr/local/bin/setup-nintendo-controller.sh
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
EOF

    sudo systemctl daemon-reload
    sudo systemctl enable nintendo-controller.service
    
    print_success "Nintendo Switch Pro Controllerエミュレーション設定完了"
}

# メイン処理
main() {
    print_info "Splatoon3 Ghost Drawer USB Gadget設定を開始します..."
    
    # ルート権限チェック
    if [[ $EUID -ne 0 ]]; then
        print_error "このスクリプトはroot権限で実行してください"
        print_info "使用方法: sudo $0"
        exit 1
    fi
    
    # ボードモデル検出
    local board_model=$(detect_board_model)
    print_info "検出されたボード: $board_model"
    
    # USB OTG対応チェック
    if ! check_usb_otg_support "$board_model"; then
        print_error "USB OTG機能が利用できません"
        print_info ""
        print_info "対応ボード:"
        print_info "  - Orange Pi Zero 2W 2GB (推奨)"
        print_info "  - Raspberry Pi Zero 2W"
        print_info "  - Raspberry Pi Zero W"
        print_info ""
        print_info "非対応ボード:"
        print_info "  - Raspberry Pi 4 Model B (USB OTG機能なし)"
        print_info "  - Raspberry Pi 3 シリーズ"
        exit 1
    fi
    
    # USB Gadgetモジュール設定
    setup_gadget_modules "$board_model"
    
    # Nintendo Controllerエミュレーション設定
    setup_nintendo_controller
    
    print_success "USB Gadget設定が完了しました！"
    print_info ""
    print_info "次の手順:"
    print_info "1. システムを再起動してください: sudo reboot"
    print_info "2. 再起動後、USB OTGケーブルでNintendo Switchに接続"
    print_info "3. Splatoon3 Ghost Drawerアプリケーションを起動"
    print_info ""
    print_warning "再起動が必要です。今すぐ再起動しますか？ (y/N)"
    
    read -r response
    case $response in
        [yY][eE][sS]|[yY])
            print_info "システムを再起動します..."
            sudo reboot
            ;;
        *)
            print_info "手動で再起動してください: sudo reboot"
            ;;
    esac
}

# スクリプト実行
main "$@" 