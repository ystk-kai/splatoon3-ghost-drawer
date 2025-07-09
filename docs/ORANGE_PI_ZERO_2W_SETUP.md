# Orange Pi Zero 2W USB OTG セットアップガイド

Orange Pi Zero 2WでSplatoon3 Ghost Drawerを使用するための詳細な設定手順です。

## 問題の症状

- `UDC state: not attached`
- `/dev/hidg0`への書き込みで「Transport endpoint not connected」エラー
- Nintendo SwitchがPro Controllerを認識しない

## 1. Device Tree Overlayの設定

Orange Pi Zero 2WではUSB OTGがデフォルトで無効になっている場合があります。

### 設定ファイルの確認

```bash
# 設定ファイルを確認
cat /boot/orangepiEnv.txt
```

### USB OTGの有効化

1. 設定ファイルを編集：
```bash
sudo nano /boot/orangepiEnv.txt
```

2. 以下の行を追加または修正：
```
overlays=usb-otg
```

複数のoverlayがある場合：
```
overlays=usb-otg i2c3 spi-spidev
```

3. 再起動：
```bash
sudo reboot
```

## 2. USB OTG動作モードの確認

### 現在のUSBモードを確認

```bash
# USB OTGの状態を確認
cat /sys/devices/platform/soc/*.usb/musb-hdrc.*.auto/mode

# 期待される出力: "b_peripheral" または "peripheral"
```

### OTGモードの手動設定

```bash
# peripheralモードに設定
echo "peripheral" | sudo tee /sys/devices/platform/soc/*.usb/musb-hdrc.*.auto/mode
```

## 3. 電源供給の確認

Orange Pi Zero 2Wは電源不足でUSB OTGが不安定になることがあります。

### 推奨事項

- **電源**: 5V/3A以上の電源アダプターを使用
- **USBケーブル**: 高品質なUSB-Cケーブル（データ転送対応）を使用
- **接続方法**: USB-CポートをNintendo Switchに接続

## 4. トラブルシューティング手順

### 手順1: 既存のGadgetをクリーンアップ

```bash
sudo splatoon3-ghost-drawer cleanup --gadget-only
```

### 手順2: カーネルモジュールの再ロード

```bash
# モジュールをアンロード
sudo modprobe -r g_hid
sudo modprobe -r usb_f_hid
sudo modprobe -r libcomposite

# モジュールをロード
sudo modprobe libcomposite
sudo modprobe usb_f_hid
```

### 手順3: USB Gadgetサービスの再起動

```bash
sudo systemctl restart splatoon3-gadget.service
```

### 手順4: 詳細情報の確認

```bash
# システム情報を詳細表示
sudo splatoon3-ghost-drawer info --verbose

# 診断を実行
sudo splatoon3-ghost-drawer diagnose
```

### 手順5: dmesgログの確認

```bash
# USB関連のカーネルログを確認
sudo dmesg | tail -50 | grep -E "(musb|otg|gadget|usb)"
```

## 5. 代替設定方法

もし上記の方法で解決しない場合：

### USB OTGドライバーの確認

```bash
# 利用可能なUDCを確認
ls -la /sys/class/udc/

# musb-hdrcドライバーの状態を確認
lsmod | grep musb
```

### Device Treeの直接編集（上級者向け）

```bash
# Device Tree Overlayを手動で適用
sudo mkdir -p /sys/kernel/config/device-tree/overlays/usb-otg
echo "usb-otg.dtbo" | sudo tee /sys/kernel/config/device-tree/overlays/usb-otg/path
```

## 6. 動作確認

1. Nintendo Switchをホーム画面にする
2. Orange Pi Zero 2WとSwitchをUSB-Cケーブルで接続
3. 以下のコマンドでテスト：

```bash
sudo splatoon3-ghost-drawer test
```

## 7. よくある問題と解決策

### 問題: UDC state: not attached

**解決策**:
- Device Tree Overlayでusb-otgが有効になっているか確認
- 電源供給が十分か確認
- USBケーブルを交換してみる

### 問題: Permission denied

**解決策**:
```bash
# HIDデバイスの権限を確認
ls -la /dev/hidg0

# 必要に応じて権限を変更（一時的）
sudo chmod 666 /dev/hidg0
```

### 問題: musb_hdrc not loaded

**解決策**:
```bash
# カーネルにビルトインされている可能性があるため確認
grep CONFIG_USB_MUSB /boot/config-$(uname -r)
```

## 8. 参考情報

- [Orange Pi Zero 2W Wiki](http://www.orangepi.org/html/hardWare/computerAndMicrocontrollers/details/Orange-Pi-Zero-2W.html)
- [Linux USB Gadget API Documentation](https://www.kernel.org/doc/html/latest/usb/gadget.html)
- [Allwinner H616 Documentation](https://linux-sunxi.org/H616)

## サポート

問題が解決しない場合は、以下の情報を含めてIssueを作成してください：

- `sudo splatoon3-ghost-drawer info --verbose`の出力
- `sudo splatoon3-ghost-drawer diagnose`の出力
- `dmesg | grep -E "(musb|otg|gadget|usb)"`の出力
- `/boot/orangepiEnv.txt`の内容