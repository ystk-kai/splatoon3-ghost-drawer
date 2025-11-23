// Splatoon3 Ghost Drawer - Web UI
// 公式Splatoonフォント対応とUI改善

class GhostDrawerApp {
    constructor() {
        this.currentFile = null;
        this.currentArtworkId = null;
        this.currentBinaryData = null;
        this.isProcessing = false;
        this.isServerConnected = false;
        this.isHardwareConnected = false;
        this.connectionCheckInterval = null;
        this.abortController = null;
        this.imageProcessor = new ImageProcessor();
        this.threshold = 128;
        this.brightness = 0;
        this.contrast = 0;
        this.gamma = 1.0;
        this.exposure = 0.0;
        this.highlights = 0;
        this.shadows = 0;
        this.blackPoint = 0;
        this.whitePoint = 255;
        this.previewMode = false; // 2値化前プレビューモード
        this.previewTimeout = null;
        this.cropMode = false;
        this.cropArea = null;
        this.cropSelected = false;
        this.isDragging = false;
        this.dragStart = null;
        this.resizing = null;
        this.moving = false;
        this.moveStart = null;
        this.paintingSpeed = 2.0;
        this.isPainting = false;
        this.isPaused = false;
        this.paintingInterval = null;
        this.paintingStartTime = null;
        this.currentDotIndex = 0;
        this.paintedDots = [];
        this.simulationMultiplier = 1;
        this.penState = 'up'; // up or down
        this.currentPosition = { x: 0, y: 0 };
        this.currentOperationIndex = 0;
        this.operationStartTime = null;
        this.dpadCount = 0;
        this.aButtonCount = 0;
        this.currentDpadCount = 0;
        this.currentAButtonCount = 0;
        
        this.init();
    }

    init() {
        this.setupEventListeners();
        this.setupDragAndDrop();
        this.startConnectionCheck();
        this.addLog('システムを初期化しています...', 'info');
        this.addLog('Webサーバーが起動しました', 'success');
    }

    setupEventListeners() {
        // ファイル選択
        document.getElementById('uploadButton').addEventListener('click', () => {
            document.getElementById('fileInput').click();
        });

        document.getElementById('fileInput').addEventListener('change', (e) => {
            if (e.target.files.length > 0) {
                this.handleFileSelect(e.target.files[0]);
            }
        });

        // 描画準備モーダル内のキャリブレーションボタン
    document.getElementById('openCalibrationFromPrepareButton')?.addEventListener('click', () => {
        if (window.calibrationManager) {
            window.calibrationManager.openModal('device');
        }
    });

    // アクションボタン
        document.getElementById('paintDeviceButton').addEventListener('click', () => {
            this.showPaintPrepareModal(true);
        });

        document.getElementById('paintSimulationButton').addEventListener('click', () => {
            this.showPaintPrepareModal(false);
        });

        // 描画準備モーダル
        document.getElementById('closePaintPrepareButton')?.addEventListener('click', () => {
            this.closePaintPrepareModal();
        });

        document.getElementById('cancelPaintPrepareButton')?.addEventListener('click', () => {
            this.closePaintPrepareModal();
        });

        document.getElementById('openCalibrationFromPaintButton')?.addEventListener('click', () => {
            // 描画準備モーダルを閉じてキャリブレーションモーダルを開く
            this.closePaintPrepareModal();
            if (window.calibrationManager) {
                window.calibrationManager.openModal();
            }
        });

        document.getElementById('startPaintingButton')?.addEventListener('click', () => {
            const useDevice = this.pendingPaintUseDevice;
            this.closePaintPrepareModal();
            this.executePainting(useDevice);
        });

        document.getElementById('downloadButton').addEventListener('click', () => {
            this.downloadResult();
        });

        // クリアボタン
        document.getElementById('clearButton').addEventListener('click', () => {
            this.clearAll();
        });

        // 画像変更ボタン
        document.getElementById('changeImageButton').addEventListener('click', () => {
            document.getElementById('fileInput').click();
        });
        
        // 変換ボタン
        const convertButton = document.getElementById('convertButton');
        if (convertButton) {
            convertButton.addEventListener('click', () => {
                this.convertImage();
            });
        }

        // 切り取りボタン
        document.getElementById('cropButton').addEventListener('click', () => {
            this.toggleCropMode();
        });

        // 切り取り適用ボタン
        document.getElementById('applyCropButton').addEventListener('click', () => {
            this.applyCrop();
        });

        // ログコントロール
        document.getElementById('clearLogButton').addEventListener('click', () => {
            this.clearLog();
        });

        document.getElementById('downloadLogButton').addEventListener('click', () => {
            this.downloadLog();
        });

        // 調整スライダーの設定
        this.setupAdjustmentSliders();
        
        // 描画コントロールの設定
        this.setupPaintingControls();
    }

    setupAdjustmentSliders() {
        try {
            // 閾値スライダー
            const thresholdSlider = document.getElementById('thresholdSlider');
            const thresholdValue = document.getElementById('thresholdValue');
            
            if (thresholdSlider && thresholdValue) {
                thresholdSlider.addEventListener('input', (e) => {
                    this.threshold = parseInt(e.target.value);
                    thresholdValue.textContent = this.threshold;
                    
                    // グラデーションを更新
                    const percentage = (this.threshold / 255) * 100;
                    e.target.style.background = `linear-gradient(to right, #000 0%, #000 ${percentage}%, #fff ${percentage}%, #fff 100%)`;
                    
                    this.debouncedUpdatePreview();
                });
            }
        } catch (error) {
            console.error('Error in setupAdjustmentSliders:', error);
            console.error('Error at:', error.stack);
        }

        // 明るさスライダー
        const brightnessSlider = document.getElementById('brightnessSlider');
        const brightnessValue = document.getElementById('brightnessValue');
        
        if (brightnessSlider && brightnessValue) {
            brightnessSlider.addEventListener('input', (e) => {
                this.brightness = parseInt(e.target.value);
                brightnessValue.textContent = this.brightness > 0 ? `+${this.brightness}` : this.brightness;
                this.debouncedUpdatePreview();
            });
        }

        // コントラストスライダー
        const contrastSlider = document.getElementById('contrastSlider');
        const contrastValue = document.getElementById('contrastValue');
        
        if (contrastSlider && contrastValue) {
            contrastSlider.addEventListener('input', (e) => {
                this.contrast = parseInt(e.target.value);
                contrastValue.textContent = this.contrast > 0 ? `+${this.contrast}` : this.contrast;
                this.debouncedUpdatePreview();
            });
        }

        // ガンマスライダー
        const gammaSlider = document.getElementById('gammaSlider');
        const gammaValue = document.getElementById('gammaValue');
        
        if (gammaSlider && gammaValue) {
            gammaSlider.addEventListener('input', (e) => {
                this.gamma = parseFloat(e.target.value);
                gammaValue.textContent = this.gamma.toFixed(1);
                this.debouncedUpdatePreview();
            });
        }

        // 露出スライダー
        const exposureSlider = document.getElementById('exposureSlider');
        const exposureValue = document.getElementById('exposureValue');
        
        if (exposureSlider && exposureValue) {
            exposureSlider.addEventListener('input', (e) => {
                this.exposure = parseFloat(e.target.value);
                exposureValue.textContent = this.exposure.toFixed(1);
                this.debouncedUpdatePreview();
            });
        }

        // ハイライトスライダー
        const highlightsSlider = document.getElementById('highlightsSlider');
        const highlightsValue = document.getElementById('highlightsValue');
        
        if (highlightsSlider && highlightsValue) {
            highlightsSlider.addEventListener('input', (e) => {
                this.highlights = parseInt(e.target.value);
                highlightsValue.textContent = this.highlights > 0 ? `+${this.highlights}` : this.highlights;
                this.debouncedUpdatePreview();
            });
        }

        // シャドウスライダー
        const shadowsSlider = document.getElementById('shadowsSlider');
        const shadowsValue = document.getElementById('shadowsValue');
        
        if (shadowsSlider && shadowsValue) {
            shadowsSlider.addEventListener('input', (e) => {
                this.shadows = parseInt(e.target.value);
                shadowsValue.textContent = this.shadows > 0 ? `+${this.shadows}` : this.shadows;
                this.debouncedUpdatePreview();
            });
        }

        // ブラックポイントスライダー
        const blackPointSlider = document.getElementById('blackPointSlider');
        const blackPointValue = document.getElementById('blackPointValue');
        
        if (blackPointSlider && blackPointValue) {
            blackPointSlider.addEventListener('input', (e) => {
                this.blackPoint = parseInt(e.target.value);
                blackPointValue.textContent = this.blackPoint;
                this.debouncedUpdatePreview();
            });
        }

        // ホワイトポイントスライダー
        const whitePointSlider = document.getElementById('whitePointSlider');
        const whitePointValue = document.getElementById('whitePointValue');
        
        if (whitePointSlider && whitePointValue) {
            whitePointSlider.addEventListener('input', (e) => {
                this.whitePoint = parseInt(e.target.value);
                whitePointValue.textContent = this.whitePoint;
                this.debouncedUpdatePreview();
            });
        }

        // プレビューモードトグル
        const previewModeToggle = document.getElementById('previewModeToggle');
        
        if (previewModeToggle) {
            previewModeToggle.addEventListener('change', (e) => {
                this.previewMode = e.target.checked;
                if (this.previewMode) {
                    this.addLog('2値化前プレビューモードを有効にしました', 'info');
                } else {
                    this.addLog('2値化プレビューモードに戻しました', 'info');
                }
                this.debouncedUpdatePreview();
            });
        }

        // リセットボタン
        const resetAdjustmentsButton = document.getElementById('resetAdjustmentsButton');
        if (resetAdjustmentsButton) {
            resetAdjustmentsButton.addEventListener('click', () => {
                this.resetAdjustments();
            });
        }
    }

    debouncedUpdatePreview() {
        if (this.currentFile && this.currentArtworkId) {
            clearTimeout(this.previewTimeout);
            this.previewTimeout = setTimeout(() => {
                this.updatePreview();
            }, 300); // 300ms のデバウンス
        }
    }
    
    setupPaintingControls() {
        // キー操作速度スライダー
        const operationSpeedSlider = document.getElementById('operationSpeedSlider');
        const operationSpeedValue = document.getElementById('operationSpeedValue');
        
        if (operationSpeedSlider && operationSpeedValue) {
            operationSpeedSlider.addEventListener('input', (e) => {
                this.paintingSpeed = parseFloat(e.target.value);
                operationSpeedValue.textContent = this.paintingSpeed.toFixed(1);
            
                // 描画中の場合は推定時間を再計算
                if (this.isPainting && this.paintingOperations) {
                    const estimatedSeconds = this.calculateRealPaintingTime();
                    document.getElementById('estimatedTime').textContent = this.formatTime(estimatedSeconds);
                }
            });
        }
        
        // 進捗スライダー（シミュレーション時のみ）
        const progressSlider = document.getElementById('progressSlider');
        const progressSliderValue = document.getElementById('progressSliderValue');
        
        if (progressSlider && progressSliderValue) {
            progressSlider.addEventListener('input', (e) => {
                if (!this.isDevicePainting && this.paintingPath && this.paintingPath.length > 0) {
                    const progress = parseFloat(e.target.value) / 100;
                    progressSliderValue.textContent = `${e.target.value}%`;
                    this.jumpToProgress(progress);
                }
            });
        }
        
        // 一時停止ボタン
        const pausePaintingButton = document.getElementById('pausePaintingButton');
        if (pausePaintingButton) {
            pausePaintingButton.addEventListener('click', () => {
                this.togglePausePainting();
            });
        }
        
        // 停止ボタン
        const stopPaintingButton = document.getElementById('stopPaintingButton');
        if (stopPaintingButton) {
            stopPaintingButton.addEventListener('click', () => {
                this.stopPainting();
            });
        }
        
        // シミュレーション倍速ボタン
        const speedButtons = document.querySelectorAll('.speed-multiplier-btn');
        if (speedButtons.length > 0) {
            speedButtons.forEach(btn => {
                btn.addEventListener('click', (e) => {
                    // すべてのボタンからアクティブクラスを削除
                    document.querySelectorAll('.speed-multiplier-btn').forEach(b => {
                        b.classList.remove('active');
                    });
                    
                    // クリックされたボタンにアクティブクラスを追加
                    e.target.classList.add('active');
                    
                    // 倍速を設定
                    this.simulationMultiplier = parseInt(e.target.dataset.speed);
                    this.addLog(`シミュレーション速度を${this.simulationMultiplier}倍に変更しました`, 'info');
                });
            });
            
            // デフォルトで1xをアクティブに
            const defaultSpeedBtn = document.querySelector('.speed-multiplier-btn[data-speed="1"]');
            if (defaultSpeedBtn) {
                defaultSpeedBtn.classList.add('active');
            }
        }

        // シミュレーション画面の速度スライダー（キャリブレーション同期）
        const simSlider = document.getElementById('simulationSpeedSlider');
        if (simSlider) {
            simSlider.addEventListener('input', (e) => {
                if (window.calibrationManager && window.calibrationManager.speedInput) {
                    window.calibrationManager.speedInput.value = e.target.value;
                    window.calibrationManager.updateValues();
                }
            });
        }


        // 描画戦略の変更監視
        const strategySelect = document.getElementById('paint-strategy');
        if (strategySelect) {
            strategySelect.addEventListener('change', (e) => {
                this.selectedStrategy = e.target.value;
                this.renderStrategyStats();
            });
        }

        // 描画回数の変更監視
        const repeatsInput = document.getElementById('paint-repeats');
        if (repeatsInput) {
            repeatsInput.addEventListener('input', () => {
                this.renderStrategyStats();
            });
        }

        // リアルタイム描画回数の変更監視
        const liveRepeatsInput = document.getElementById('liveRepeatsInput');
        if (liveRepeatsInput) {
            liveRepeatsInput.addEventListener('change', async (e) => {
                const repeats = parseInt(e.target.value, 10) || 1;
                this.currentRepeats = repeats;
                if (this.isPainting) {
                    try {
                        const response = await fetch('/api/painting/repeats', {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body: JSON.stringify({ repeats: repeats })
                        });
                        
                        if (response.ok) {
                            this.addLog(`描画回数を ${repeats}回 に変更しました`, 'info');
                            this.updateEstimatedTime();
                        } else {
                            throw new Error('Failed to update repeats');
                        }
                    } catch (error) {
                        console.error('Error updating repeats:', error);
                        this.addLog('描画回数の更新に失敗しました', 'error');
                    }
                }
            });
        }
    }

    setupDragAndDrop() {
        const uploadArea = document.getElementById('uploadArea');

        uploadArea.addEventListener('dragover', (e) => {
            e.preventDefault();
            uploadArea.classList.add('dragover');
        });

        uploadArea.addEventListener('dragleave', (e) => {
            e.preventDefault();
            uploadArea.classList.remove('dragover');
        });

        uploadArea.addEventListener('drop', (e) => {
            e.preventDefault();
            uploadArea.classList.remove('dragover');
            
            const files = e.dataTransfer.files;
            if (files.length > 0) {
                this.handleFileSelect(files[0]);
            }
        });
    }


    startConnectionCheck() {
        this.checkConnection();
        this.connectionCheckInterval = setInterval(() => {
            this.checkConnection();
        }, 1000); // 1秒ごとにチェック
    }

    async checkConnection() {
        try {
            // サーバー接続確認
            if (this.abortController) {
                this.abortController.abort();
            }
            this.abortController = new AbortController();
            
            const response = await fetch('/api/system/info', {
                signal: this.abortController.signal,
                timeout: 3000
            });

            if (response.ok) {
                this.isServerConnected = true;
                const data = await response.json();
                this.updateSystemStatus(data);
            } else {
                this.isServerConnected = false;
                this.updateConnectionStatus();
            }
        } catch (error) {
            this.isServerConnected = false;
            this.updateConnectionStatus();
            console.log('Connection check failed:', error.message);
        }

        // ハードウェア接続確認
        try {
            const hardwareResponse = await fetch('/api/hardware/status', {
                signal: this.abortController.signal,
                timeout: 3000
            });

            if (hardwareResponse.ok) {
                const hardwareData = await hardwareResponse.json();
                this.isHardwareConnected = hardwareData.nintendo_switch_connected;
                this.updateHardwareStatus(hardwareData);
            }
        } catch (error) {
            this.isHardwareConnected = false;
            console.log('Hardware check failed:', error.message);
        }

        this.updateConnectionStatus();
    }

    updateConnectionStatus() {
        const statusElement = document.getElementById('connectionStatus');
        const textElement = document.getElementById('connectionText');
        const indicatorElement = document.getElementById('statusIndicator');
        
        // すべてのクラスをクリア
        statusElement.classList.remove('connected', 'disconnected');
        indicatorElement.classList.remove('bg-green-500', 'bg-red-500', 'bg-yellow-500');
        
        if (this.isHardwareConnected) {
            statusElement.classList.add('connected');
            indicatorElement.classList.add('bg-green-500');
            textElement.textContent = 'Nintendo Switch接続済み';
        } else if (this.isServerConnected) {
            statusElement.classList.add('connected');
            indicatorElement.classList.add('bg-yellow-500');
            textElement.textContent = 'サーバー接続済み（機器未接続）';
        } else {
            statusElement.classList.add('disconnected');
            indicatorElement.classList.add('bg-red-500');
            textElement.textContent = '未接続';
        }
    }

    updateSystemStatus(data) {
        const serverStatus = document.getElementById('serverStatus');
        serverStatus.textContent = '接続済み';
        serverStatus.className = 'text-sm font-semibold status-connected';
    }

    updateHardwareStatus(data) {
        // Nintendo Switch
        const switchStatus = document.getElementById('switchStatus');
        if (data.nintendo_switch_connected) {
            switchStatus.textContent = '接続済み';
            switchStatus.className = 'text-sm font-semibold status-connected';
        } else {
            switchStatus.textContent = '未接続';
            switchStatus.className = 'text-sm font-semibold status-disconnected';
        }

        // USB OTG
        const usbStatus = document.getElementById('usbStatus');
        if (data.usb_otg_available) {
            usbStatus.textContent = '利用可能';
            usbStatus.className = 'text-sm font-semibold status-connected';
        } else {
            usbStatus.textContent = '利用不可';
            usbStatus.className = 'text-sm font-semibold status-error';
        }

        // HIDデバイス
        const hidStatus = document.getElementById('hidStatus');
        if (data.hid_device_available) {
            hidStatus.textContent = '利用可能';
            hidStatus.className = 'text-sm font-semibold status-connected';
        } else {
            hidStatus.textContent = '利用不可';
            hidStatus.className = 'text-sm font-semibold status-error';
        }
    }

    handleFileSelect(file) {
        if (!this.validateFile(file)) {
            return;
        }

        this.currentFile = file;
        this.displayOriginalImage(file);
        this.updateButtonStates();
        this.addLog(`ファイル選択: ${file.name} (${this.formatFileSize(file.size)})`, 'info');
        
        // 画像選択時に自動変換を実行
        this.addLog(`サーバー接続状態: ${this.isServerConnected ? '接続済み' : '未接続'}`, 'info');
        if (this.isServerConnected) {
            this.addLog('画像選択を検出しました。自動変換を開始します...', 'info');
            setTimeout(() => {
                this.convertImage();
            }, 500); // 少し遅延を入れてプレビューが表示されてから実行
        } else {
            this.addLog('サーバーが未接続のため、自動変換をスキップしました', 'warning');
            // サーバー接続がなくても手動で変換を実行
            this.addLog('手動で変換を実行してください', 'info');
        }
    }

    validateFile(file) {
        const maxSize = 10 * 1024 * 1024; // 10MB
        const allowedTypes = ['image/png', 'image/jpeg', 'image/jpg', 'image/gif', 'image/bmp'];

        if (file.size > maxSize) {
            this.addLog(`エラー: ファイルサイズが大きすぎます (最大10MB)`, 'error');
            return false;
        }

        if (!allowedTypes.includes(file.type)) {
            this.addLog(`エラー: サポートされていないファイル形式です`, 'error');
            return false;
        }

        return true;
    }

    displayOriginalImage(file) {
        const reader = new FileReader();
        reader.onload = (e) => {
            const uploadArea = document.getElementById('uploadArea');
            const originalImageArea = document.getElementById('originalImageArea');
            const originalImage = document.getElementById('originalImage');
            const imageDetails = document.getElementById('originalImageDetails');

            originalImage.src = e.target.result;
            uploadArea.classList.add('hidden');
            originalImageArea.classList.remove('hidden');
            
            // 調整パネルを表示
            const adjustmentPanel = document.getElementById('adjustmentPanel');
            if (adjustmentPanel) {
                adjustmentPanel.classList.remove('hidden');
            }

            // 画像情報を表示
            const img = new Image();
            img.onload = () => {
                imageDetails.textContent = `${img.width} × ${img.height} px, ${this.formatFileSize(file.size)}`;
            };
            img.src = e.target.result;
        };
        reader.readAsDataURL(file);
    }

    displayConvertedImage(artwork) {
        const convertedArea = document.getElementById('convertedArea');
        const convertedImageArea = document.getElementById('convertedImageArea');
        const convertedCanvas = document.getElementById('convertedCanvas');
        const convertedDetails = document.getElementById('convertedImageDetails');

        // キャンバスのサイズを設定
        convertedCanvas.width = artwork.canvas.width;
        convertedCanvas.height = artwork.canvas.height;

        const ctx = convertedCanvas.getContext('2d');
        ctx.fillStyle = '#ffffff';
        ctx.fillRect(0, 0, convertedCanvas.width, convertedCanvas.height);

        // ドットを描画（簡易版）
        const dotSize = Math.max(1, Math.min(convertedCanvas.width / artwork.canvas.width, convertedCanvas.height / artwork.canvas.height));
        
        // サンプルパターンを描画
        for (let y = 0; y < artwork.canvas.height; y++) {
            for (let x = 0; x < artwork.canvas.width; x++) {
                const isDark = (x + y) % 2 === 0;
                ctx.fillStyle = isDark ? '#000000' : '#ffffff';
                ctx.fillRect(x * dotSize, y * dotSize, dotSize, dotSize);
            }
        }

        // 表示を切り替え
        convertedArea.classList.add('hidden');
        convertedImageArea.classList.remove('hidden');

        // 詳細情報を表示
        convertedDetails.textContent = `${artwork.canvas.width} × ${artwork.canvas.height} px, ${artwork.total_dots || 0} ドット`;
    }

    displayProcessedCanvas(canvas) {
        const convertedArea = document.getElementById('convertedArea');
        const convertedImageArea = document.getElementById('convertedImageArea');
        const convertedCanvas = document.getElementById('convertedCanvas');
        const convertedDetails = document.getElementById('convertedImageDetails');

        // プレビュー用に拡大表示
        const scaledCanvas = this.imageProcessor.createScaledPreview(canvas, 2);
        
        // 既存のキャンバスサイズを更新
        convertedCanvas.width = scaledCanvas.width;
        convertedCanvas.height = scaledCanvas.height;
        
        // 拡大したキャンバスをコピー
        const ctx = convertedCanvas.getContext('2d');
        ctx.drawImage(scaledCanvas, 0, 0);

        // 表示を切り替え
        convertedArea.classList.add('hidden');
        convertedImageArea.classList.remove('hidden');

        // 詳細情報を表示
        const dotCount = this.currentBinaryData ? this.currentBinaryData.filter(d => d).length : 0;
        convertedDetails.textContent = `${canvas.width} × ${canvas.height} px, ${dotCount} ドット`;
    }

    updateButtonStates() {
        const hasFile = this.currentFile !== null;
        const paintDeviceButton = document.getElementById('paintDeviceButton');
        const paintSimulationButton = document.getElementById('paintSimulationButton');

        // 画像がある場合は両方のボタンを有効化
        paintDeviceButton.disabled = !hasFile || this.isProcessing;
        paintSimulationButton.disabled = !hasFile || this.isProcessing;
        
        // 接続状態に応じて実機描画ボタンの表示を変更
        if (!this.isHardwareConnected && hasFile) {
            // 未接続時は実機描画ボタンを半透明に
            paintDeviceButton.style.opacity = '0.6';
            paintDeviceButton.title = 'Nintendo Switchと接続してください';
        } else {
            paintDeviceButton.style.opacity = '1';
            paintDeviceButton.title = '';
        }
        
        document.getElementById('downloadButton').disabled = !hasFile || this.isProcessing;
    }

    async convertImage() {
        if (!this.currentFile || this.isProcessing) return;

        this.isProcessing = true;
        this.updateButtonStates();
        this.showProgress();

        try {
            this.addLog('画像変換を開始します...', 'info');
            this.updateProgress(10, '画像を読み込み中...');

            // ブラウザ側で画像処理
            const adjustments = {
                brightness: this.brightness,
                contrast: this.contrast,
                gamma: this.gamma,
                exposure: this.exposure,
                highlights: this.highlights,
                shadows: this.shadows,
                blackPoint: this.blackPoint,
                whitePoint: this.whitePoint,
                previewMode: this.previewMode
            };
            
            // 切り取り範囲がある場合は、画像の表示サイズ情報を追加
            let cropAreaWithImageInfo = null;
            if (this.cropArea) {
                const originalImage = document.getElementById('originalImage');
                cropAreaWithImageInfo = {
                    ...this.cropArea,
                    originalImage: {
                        width: originalImage.width,
                        height: originalImage.height
                    }
                };
            }
            
            const processedData = await this.imageProcessor.processImage(
                this.currentFile, 
                this.threshold, 
                adjustments,
                cropAreaWithImageInfo
            );
            
            this.updateProgress(30, '画像をリサイズ中...');
            this.addLog(`画像をリサイズしました: ${processedData.width}x${processedData.height}`, 'info');
            
            this.updateProgress(50, '2値化処理中...');
            this.currentBinaryData = processedData.binaryData;
            
            // ドットデータに変換
            const dots = this.imageProcessor.convertToDotData(
                processedData.binaryData,
                processedData.width,
                processedData.height
            );
            
            if (dots.length === 0) {
                this.addLog('ドットが検出されませんでした。閾値を調整してください。', 'warning');
                this.displayProcessedCanvas(processedData.canvas);
                this.hideProgress();
                return;
            }
            
            this.addLog(`2値化完了: ${dots.length}個の描画ドット`, 'info');
            
            // 変換結果をサーバーに送信
            this.updateProgress(70, 'サーバーにデータを送信中...');
            
            const requestData = {
                name: this.currentFile.name.replace(/\.[^/.]+$/, '') || 'Untitled',
                width: processedData.width,
                height: processedData.height,
                dots: dots
            };
            
            // デバッグ用にリクエストデータをログ出力
            console.log('Sending artwork data:', {
                name: requestData.name,
                width: requestData.width,
                height: requestData.height,
                dotsCount: requestData.dots.length,
                firstDot: requestData.dots[0] || null
            });
            
            const response = await fetch('/api/artworks', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(requestData)
            });

            if (!response.ok) {
                let errorMessage = `サーバーエラー: ${response.status}`;
                try {
                    const errorData = await response.json();
                    errorMessage = errorData.message || errorData.error || errorMessage;
                    console.error('Server error response:', errorData);
                } catch (e) {
                    // JSONパースエラーの場合は元のメッセージを使用
                }
                throw new Error(errorMessage);
            }

            const result = await response.json();
            this.currentArtworkId = result.id;
            
            this.updateProgress(100, '変換完了');
            this.addLog('画像変換が完了しました', 'success');
            this.addLog(`アートワークID: ${result.id}`, 'info');
            
            // 変換後の画像を表示
            this.displayProcessedCanvas(processedData.canvas);
            
            // 調整パネルを表示（既に表示されているはず）
            const adjustmentPanel = document.getElementById('adjustmentPanel');
            if (adjustmentPanel) {
                adjustmentPanel.classList.remove('hidden');
            }
            
            // USB OTG接続時でも自動描画は行わない
            if (this.isHardwareConnected) {
                this.addLog('USB OTG接続を検出しました。「🎮 実機に描画」ボタンを押して描画を開始してください。', 'info');
            } else {
                this.addLog('ハードウェアが接続されていません。「💻 シミュレーション」で動作を確認できます。', 'info');
            }
            
            setTimeout(() => {
                this.hideProgress();
            }, 1000);

        } catch (error) {
            this.addLog(`変換エラー: ${error.message}`, 'error');
            this.hideProgress();
        } finally {
            this.isProcessing = false;
            this.updateButtonStates();
        }
    }


    async preparePaintingData() {
        if (!this.currentArtworkId) return;

        try {
            const strategySelect = document.getElementById('paint-strategy');
            const strategy = strategySelect.value;
            const strategyText = strategySelect.options[strategySelect.selectedIndex].text;
            
            const response = await fetch(`/api/artworks/${this.currentArtworkId}/path?strategy=${strategy}`);
            if (!response.ok) throw new Error('Failed to fetch drawing path');
            
            const data = await response.json();
            const path = data.path; // Array of {x, y}
            
            this.paintingPath = path;
            this.paintingOperations = [];
            
            let currentX = 0;
            let currentY = 0;
            
            // Generate operations from path
            for (const point of path) {
                const targetX = point.x;
                const targetY = point.y;
                
                const dx = targetX - currentX;
                const dy = targetY - currentY;
                const moveDuration = 1.0 / (this.paintingSpeed || 100); // Default to 100 if undefined
                
                // Move X
                if (dx > 0) {
                    for (let i = 0; i < dx; i++) {
                        this.paintingOperations.push({ 
                            type: 'move', 
                            direction: 'right',
                            from: { x: currentX, y: currentY },
                            to: { x: currentX + 1, y: currentY },
                            duration: moveDuration,
                            isDpadMove: true
                        });
                        currentX++;
                    }
                } else if (dx < 0) {
                    for (let i = 0; i < Math.abs(dx); i++) {
                        this.paintingOperations.push({ 
                            type: 'move', 
                            direction: 'left',
                            from: { x: currentX, y: currentY },
                            to: { x: currentX - 1, y: currentY },
                            duration: moveDuration,
                            isDpadMove: true
                        });
                        currentX--;
                    }
                }
                
                // Move Y
                if (dy > 0) {
                    for (let i = 0; i < dy; i++) {
                        this.paintingOperations.push({ 
                            type: 'move', 
                            direction: 'down',
                            from: { x: currentX, y: currentY },
                            to: { x: currentX, y: currentY + 1 },
                            duration: moveDuration,
                            isDpadMove: true
                        });
                        currentY++;
                    }
                } else if (dy < 0) {
                    for (let i = 0; i < Math.abs(dy); i++) {
                        this.paintingOperations.push({ 
                            type: 'move', 
                            direction: 'up',
                            from: { x: currentX, y: currentY },
                            to: { x: currentX, y: currentY - 1 },
                            duration: moveDuration,
                            isDpadMove: true
                        });
                        currentY--;
                    }
                }
                
                // Paint
                this.paintingOperations.push({ 
                    type: 'draw', 
                    x: targetX, 
                    y: targetY,
                    position: { x: targetX, y: targetY }
                });
                
                currentX = targetX;
                currentY = targetY;
            }
            
            this.totalOperations = this.paintingOperations.length;
            this.currentOperationIndex = 0;
            
            // Update UI with estimated time
            const estimatedTime = data.estimated_time_sec;
            this.totalEstimatedTime = estimatedTime; // Store total estimated time for progress update
            
            document.getElementById('estimatedTime').textContent = this.formatTime(estimatedTime);
            document.getElementById('totalDots').textContent = path.length.toLocaleString();
            document.getElementById('skippedDots').textContent = (320 * 120 - path.length).toLocaleString();
            document.getElementById('displayStrategy').textContent = strategyText;

            // Calculate dpad and A button counts based on new operations
            this.dpadCount = 0;
            this.baseAButtonCount = 0; // Store base count for 1 repeat
            for (const op of this.paintingOperations) {
                if (op.type === 'move') {
                    if (op.isDpadMove) {
                        const distance = Math.abs(op.to.x - op.from.x) + Math.abs(op.to.y - op.from.y);
                        this.dpadCount += distance;
                    }
                } else if (op.type === 'draw') {
                    this.baseAButtonCount++;
                } else if (op.type === 'pen_up' || op.type === 'pen_down') {
                    this.dpadCount++;
                }
            }
            
            // Initial A button count calculation
            const repeatsInput = document.getElementById('paint-repeats');
            const initialRepeats = repeatsInput ? parseInt(repeatsInput.value, 10) || 1 : 1;
            this.aButtonCount = this.baseAButtonCount * initialRepeats;

            // Calculate base movement time for dynamic estimation
            // API returns estimated time for repeats=1
            const timing = window.calibrationManager ? window.calibrationManager.getTimingValues() : { pressMs: 100, releaseMs: 60, waitMs: 40 };
            const dotCycleSec = (timing.pressMs + timing.releaseMs + timing.waitMs) / 1000;
            const basePaintTime = path.length * dotCycleSec;
            this.totalMovementTime = Math.max(0, estimatedTime - basePaintTime);
            this.totalDotsCount = path.length;
            document.getElementById('dpadOperations').textContent = `0/${this.dpadCount.toLocaleString()}回`;
            document.getElementById('aButtonPresses').textContent = `0/${this.aButtonCount.toLocaleString()}回`;

            // 実機想定時間（APIから取得）
            if (data.estimated_time_sec) {
                this.totalEstimatedTime = data.estimated_time_sec;
            } else {
                // フォールバック: (Aボタン数 * 200ms + 十字キー数 * 150ms) / 1000
                this.totalEstimatedTime = (this.aButtonCount * 0.2) + (this.dpadCount * 0.15);
            }

            // クライアント側で再計算して最新のキャリブレーション値を反映
            this.updateEstimatedTime();

            this.addLog(`描画ドット数: ${path.length}個`, 'info');
            this.addLog(`推定描画時間: ${this.formatTime(this.totalEstimatedTime)}`, 'info');
            this.addLog(`操作回数 - 十字キー: ${this.dpadCount}回、Aボタン: ${this.aButtonCount}回`, 'info');
            
        } catch (error) {
            console.error('Error preparing painting data:', error);
            alert(`描画データの準備に失敗しました: ${error.message}`);
        }
    }
    
    generateOperations() {
        // This method is no longer used as operations are generated from API path
        // Keeping it for now, but it will be removed or refactored if not needed elsewhere.
        return [];
    }
    
    calculateMoveDuration(from, to) {
        // This method is no longer used directly for operation generation
        // but might be used for simulation timing.
        // Real device timing is handled by the server and calibration.
        const baseDuration = Math.max(50 / this.paintingSpeed, 30); // ms
        const dx = Math.abs(to.x - from.x);
        const dy = Math.abs(to.y - from.y);
        const totalPixels = dx + dy;
        return (totalPixels * baseDuration * 2) / 1000; // 秒
    }
    
    calculateRealPaintingTime() {
        // This method is no longer used as estimated time comes from API
        return 0; 
    }
    
    calculateTotalEstimatedTime() {
        if (!this.paintingOperations) return 0;
        
        let total = 0;
        const timing = window.calibrationManager ? window.calibrationManager.getTimingValues() : { pressMs: 100, releaseMs: 60, waitMs: 40 };
        
        // 秒単位に変換
        const pressSec = timing.pressMs / 1000;
        const releaseSec = timing.releaseMs / 1000;
        const waitSec = timing.waitMs / 1000;

        // 描画回数を取得 (リアルタイム設定があればそちらを優先)
        let repeats = this.currentRepeats || 1;
        
        // まだcurrentRepeatsが設定されていない場合（準備画面など）
        if (!this.isPainting) {
            const prepareRepeatsInput = document.getElementById('paint-repeats');
            if (prepareRepeatsInput) {
                repeats = parseInt(prepareRepeatsInput.value, 10) || 1;
            }
        }
        console.log(`Calculating time with repeats: ${repeats}`);

        for (const op of this.paintingOperations) {
            switch (op.type) {
                case 'pen_up':
                    total += releaseSec;
                    break;
                case 'pen_down':
                    total += pressSec;
                    break;
                case 'draw':
                    // 描画はキャリブレーション値（押下+解放+待機）× 回数
                    total += (pressSec + releaseSec + waitSec) * repeats;
                    break;
                case 'move':
                    // 移動もキャリブレーション値（押下+解放+待機）を使用
                    total += (pressSec + releaseSec + waitSec);
                    break;
            }
        }
        return total;
    }

    updateEstimatedTime() {
        if (!this.paintingOperations || this.paintingOperations.length === 0) return;
        
        this.totalEstimatedTime = this.calculateTotalEstimatedTime();
        document.getElementById('estimatedTime').textContent = this.formatTime(this.totalEstimatedTime);
    }

    initializePaintingUI() {
        this.currentDotIndex = 0;
        this.currentOperationIndex = 0;
        this.currentDpadCount = 0;
        this.currentAButtonCount = 0;
        this.paintingStartTime = Date.now();
        this.operationStartTime = Date.now();
        this.penState = 'up';
        this.currentPosition = { x: 0, y: 0 };

        // Estimated time now comes from API, so preCalculatedEstimate should be set after preparePaintingData
        // For now, set to 0 or a placeholder until preparePaintingData is called.
        this.preCalculatedEstimate = 0; 
        
        // 描画キャンバスを初期化
        const paintingCanvas = document.getElementById('paintingCanvas');
        const ctx = paintingCanvas.getContext('2d');
        paintingCanvas.width = 320;
        paintingCanvas.height = 120;
        
        // 背景を白で塗りつぶし
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, 320, 120);
        
        // カーソルを初期位置に表示
        const cursor = document.getElementById('paintingCursor');
        cursor.style.left = '0px';
        cursor.style.top = '0px';
        cursor.classList.remove('hidden');
        cursor.classList.add('pen-up');
        cursor.classList.remove('pen-down');
        // 実機描画時のスムーズな移動のためにトランジションを追加
        cursor.style.transition = 'left 0.1s linear, top 0.1s linear';
    }

    updatePaintingProgress(data) {
        // ステータスメッセージの処理
        if (data.status_message) {
            const progressText = document.getElementById('progressText');
            if (progressText) {
                progressText.textContent = `準備中... (${data.status_message})`;
                // Ensure progress container is visible
                const progressContainer = document.getElementById('progressContainer');
                if (progressContainer) progressContainer.classList.remove('hidden');
            }
            return;
        }

        if (!this.isPainting) return;

        const { current, total, x, y, dpad_operations, a_button_presses, is_paint } = data;

        // is_paintがtrueの場合のみキャンバスにドットを描画
        if (is_paint !== false) {
            const paintingCanvas = document.getElementById('paintingCanvas');
            const ctx = paintingCanvas.getContext('2d');
            ctx.fillStyle = '#000000';
            ctx.fillRect(x, y, 1, 1);
        }

        // カーソルを移動（座標がある場合のみ更新）
        if (x !== undefined && y !== undefined) {
            const cursor = document.getElementById('paintingCursor');
            const rect = document.getElementById('paintingCanvas').getBoundingClientRect();
            const scaleX = rect.width / 320;
            const scaleY = rect.height / 120;
            
            // 位置を保存
            this.currentPosition = { x, y };

            cursor.style.left = `${x * scaleX}px`;
            cursor.style.top = `${y * scaleY}px`;
        }

        // 描画済みドット数を更新（is_paintがtrueの場合のみ）
        if (is_paint !== false) {
            document.getElementById('paintedDots').textContent = current.toLocaleString();
        }

        // 十字キー操作とAボタン押下回数を更新（常に更新）
        if (dpad_operations !== undefined) {
            document.getElementById('dpadOperations').textContent = `${dpad_operations.toLocaleString()}/${this.dpadCount.toLocaleString()}回`;
        }
        if (a_button_presses !== undefined) {
            document.getElementById('aButtonPresses').textContent = `${a_button_presses.toLocaleString()}/${this.aButtonCount.toLocaleString()}回`;
        }

        // 経過時間と残り時間を計算（常に更新）
        const elapsed = (Date.now() - this.paintingStartTime) / 1000;
        document.getElementById('elapsedTime').textContent = this.formatTime(elapsed);

        // プログレスバーと残り時間を更新（is_paintがtrueの場合のみ）
        if (is_paint !== false && current > 0) {
            const percentage = Math.min((current / total) * 100, 100);
            const progressFill = document.getElementById('progressFill');
            const progressText = document.getElementById('progressText');

            if (progressFill) {
                progressFill.style.width = `${percentage}%`;
            }

            // ハイブリッド時間推定：事前計算とリアルタイムをブレンド
            const remainingDots = total - current;
            let estimatedRemaining;

            // リアルタイム設定を取得
            const liveInput = document.getElementById('liveRepeatsInput');
            const currentRepeats = liveInput ? parseInt(liveInput.value, 10) || 1 : this.currentRepeats;

            // Aボタン回数を動的に更新 (現在の実績 + 残りドット * 現在のリピート数)
            if (a_button_presses !== undefined) {
                this.aButtonCount = a_button_presses + (remainingDots * currentRepeats);
                document.getElementById('aButtonPresses').textContent = `${a_button_presses.toLocaleString()}/${this.aButtonCount.toLocaleString()}回`;
            }

            // 残り時間を動的に計算
            if (this.totalMovementTime !== undefined && this.totalDotsCount > 0) {
                // 残りの移動時間（移動時間はリピート回数に依存しないと仮定）
                // 進捗率に基づいて残りの移動時間を按分
                const remainingMovementTime = (this.totalMovementTime / this.totalDotsCount) * remainingDots;
                
                // 残りの描画時間（リピート回数を考慮）
                const timing = window.calibrationManager ? window.calibrationManager.getTimingValues() : { pressMs: 100, releaseMs: 60, waitMs: 40 };
                const dotCycleSec = (timing.pressMs + timing.releaseMs + timing.waitMs) / 1000;
                const remainingPaintTime = remainingDots * currentRepeats * dotCycleSec;

                estimatedRemaining = remainingMovementTime + remainingPaintTime;
            } else if (this.totalEstimatedTime) {
                // フォールバック: 単純な減算 (リピート変更を考慮できないため精度は低い)
                estimatedRemaining = Math.max(0, this.totalEstimatedTime - elapsed);
            } else {
                // フォールバック
                const averageTimePerDot = elapsed / current;
                estimatedRemaining = remainingDots * averageTimePerDot;
            }

            // 推定完了時刻
            const estimatedCompletion = new Date(Date.now() + estimatedRemaining * 1000);
            const completionTimeStr = estimatedCompletion.toLocaleTimeString('ja-JP', {
                hour: '2-digit',
                minute: '2-digit'
            });

            // プログレステキストと残り時間表示を更新
            if (progressText) {
                progressText.textContent = `${percentage.toFixed(1)}% 完了 (${current}/${total}) - 残り ${this.formatTime(estimatedRemaining)} (完了予定: ${completionTimeStr})`;
            }

            // 推定時間表示も更新（全体の推定時間）
            const totalEstimatedTime = elapsed + estimatedRemaining;
            document.getElementById('estimatedTime').textContent = this.formatTime(totalEstimatedTime);
        }
    }

    showPaintPrepareModal(useDevice) {
        if (!this.currentBinaryData) return;

        const isConnected = this.isServerConnected && this.isHardwareConnected;

        if (useDevice && !isConnected) {
            alert('実機が接続されていません。');
            return;
        }

        if (!useDevice && !isConnected) {
            if (!confirm('実機が接続されていません。シミュレーションモードで開始しますか？')) {
                return;
            }
        }

        // useDeviceを保存
        this.pendingPaintUseDevice = useDevice;

        // キャリブレーション設定を表示
        this.updatePaintPrepareModalValues();

        // シミュレーションモードの場合はキャリブレーション関連を非表示にする
        const calibrationContainer = document.getElementById('calibrationSettingsContainer');
        const calibrationButton = document.getElementById('openCalibrationFromPaintButton');
        
        if (!useDevice) {
            calibrationContainer?.classList.add('hidden');
            calibrationButton?.classList.add('hidden');
        } else {
            calibrationContainer?.classList.remove('hidden');
            calibrationButton?.classList.remove('hidden');
        }

        // モーダルを表示
        const modal = document.getElementById('paintPrepareModal');
        modal?.classList.remove('hidden');
        modal?.classList.add('flex');

        // 戦略比較データを取得・表示
        this.fetchStrategyStats();
    }

    async fetchStrategyStats() {
        if (!this.currentArtworkId) return;

        const tbody = document.getElementById('strategyComparisonBody');
        tbody.innerHTML = '<tr class="bg-gray-800 border-b border-gray-700"><td colspan="4" class="px-3 py-2 text-center">読み込み中...</td></tr>';

        try {
            const response = await fetch(`/api/artworks/${this.currentArtworkId}/strategies`);
            if (!response.ok) throw new Error('Failed to fetch strategy stats');
            
            this.strategyData = await response.json();
            this.renderStrategyStats();

        } catch (error) {
            console.error('Error fetching strategy stats:', error);
            tbody.innerHTML = '<tr class="bg-gray-800 border-b border-gray-700"><td colspan="4" class="px-3 py-2 text-center text-red-400">読み込みエラー</td></tr>';
        }
    }

    renderStrategyStats() {
        if (!this.strategyData || !this.strategyData.strategies) return;

        const tbody = document.getElementById('strategyComparisonBody');
        if (!tbody) return;
        
        // 描画回数を取得
        const repeatsInput = document.getElementById('paint-repeats');
        const repeats = repeatsInput ? parseInt(repeatsInput.value, 10) || 1 : 1;

        // キャリブレーション値を取得
        const timing = window.calibrationManager ? window.calibrationManager.getTimingValues() : { pressMs: 100, releaseMs: 60, waitMs: 40 };
        const cycleSec = (timing.pressMs + timing.releaseMs + timing.waitMs) / 1000;

        // 最も推定時間が短い戦略を見つける
        let minTime = Infinity;
        let bestStrategy = null;

        this.strategyData.strategies.forEach(stat => {
            // 時間を計算 (キャリブレーション値に基づく)
            // Aボタン回数 * サイクル時間 * リピート回数 + 十字キー回数 * サイクル時間
            const estimatedTime = (stat.a_button_presses * cycleSec * repeats) + (stat.dpad_operations * cycleSec);
            
            if (estimatedTime < minTime) {
                minTime = estimatedTime;
                bestStrategy = stat.strategy;
            }
        });

        // 初回表示時（または戦略が未選択時）に最適な戦略を自動選択
        if (bestStrategy && (!this.selectedStrategy || !this.strategyData.strategies.find(s => s.strategy === this.selectedStrategy))) {
            this.selectedStrategy = bestStrategy;
            const select = document.getElementById('paint-strategy');
            if (select) select.value = bestStrategy;
        }

        const currentStrategy = this.selectedStrategy || (this.strategyData.strategies[0] ? this.strategyData.strategies[0].strategy : 'GreedyTwoOpt');
        
        tbody.innerHTML = '';

        // 戦略名の日本語マッピング
        const strategyNames = {
            'GreedyTwoOpt': 'Greedy + 2-opt',
            'NearestNeighbor': '最近傍法',
            'ZigZag': '牛耕式 (ジグザグ)',
            'RasterScan': 'ラスタースキャン'
        };

        this.strategyData.strategies.forEach(stat => {
            const tr = document.createElement('tr');
            tr.className = 'bg-gray-800 border-b border-gray-700 hover:bg-gray-700 transition-colors';
            if (stat.strategy === currentStrategy) {
                tr.classList.add('bg-gray-700', 'font-semibold');
            }

            // 時間を計算 (キャリブレーション値に基づく)
            // Aボタン回数 * サイクル時間 * リピート回数 + 十字キー回数 * サイクル時間
            const estimatedTime = (stat.a_button_presses * cycleSec * repeats) + (stat.dpad_operations * cycleSec);
            
            // 時間をフォーマット (分:秒)
            const minutes = Math.floor(estimatedTime / 60);
            const seconds = Math.floor(estimatedTime % 60);
            const timeStr = `${minutes}分${seconds.toString().padStart(2, '0')}秒`;

            tr.innerHTML = `
                <td class="px-3 py-2">${strategyNames[stat.strategy] || stat.strategy}</td>
                <td class="px-3 py-2 text-right">${stat.dpad_operations.toLocaleString()}</td>
                <td class="px-3 py-2 text-right">${stat.a_button_presses.toLocaleString()}</td>
                <td class="px-3 py-2 text-right">${timeStr}</td>
            `;
            
            // 行クリックで戦略を選択
            tr.addEventListener('click', () => {
                this.selectedStrategy = stat.strategy;
                const select = document.getElementById('paint-strategy');
                if (select) select.value = stat.strategy;
                // ハイライト更新
                this.renderStrategyStats();
            });

            tbody.appendChild(tr);
        });
        

    }

    closePaintPrepareModal() {
        const modal = document.getElementById('paintPrepareModal');
        modal?.classList.add('hidden');
        modal?.classList.remove('flex');
        this.pendingPaintUseDevice = null;
    }

    updatePaintPrepareModalValues() {
        // CalibrationManagerから現在の値を取得
        if (window.calibrationManager) {
            const speedValue = parseInt(window.calibrationManager.speedInput?.value || 100);
            const displaySpeed = speedValue / 100.0;
            const timing = window.calibrationManager.getTimingValues();

            document.getElementById('currentCalibrationSpeed').textContent = `${displaySpeed.toFixed(1)}x`;
            document.getElementById('currentCalibrationPress').textContent = `${timing.pressMs}ms`;
            document.getElementById('currentCalibrationRelease').textContent = `${timing.releaseMs}ms`;
            document.getElementById('currentCalibrationWait').textContent = `${timing.waitMs}ms`;
        }
    }

    async executePainting(useDevice) {
        if (!this.currentBinaryData) return;

        const isConnected = this.isServerConnected && this.isHardwareConnected;
        const isDevicePainting = useDevice !== null ? useDevice : isConnected;
    this.isDevicePainting = isDevicePainting;

    if (isDevicePainting && !isConnected) {
            alert('実機が接続されていません。');
            return;
        }

        if (!isDevicePainting && useDevice === null && !confirm('実機が接続されていません。シミュレーションモードで開始しますか？')) {
            return;
        }

        this.isPainting = true;
        this.updateButtonStates();

        // リアルタイム設定の初期値を設定
        const liveRepeatsInput = document.getElementById('liveRepeatsInput');
        const prepareRepeatsInput = document.getElementById('paint-repeats');
        if (liveRepeatsInput && prepareRepeatsInput) {
            liveRepeatsInput.value = prepareRepeatsInput.value;
            this.currentRepeats = parseInt(prepareRepeatsInput.value, 10) || 1;
        } else {
            this.currentRepeats = 1;
        }

        // 描画データを準備
        await this.preparePaintingData(); // Await the async call

        // 描画進捗エリアを表示
        document.getElementById('paintingProgress').classList.remove('hidden');
        this.hideProgress();

        // 進捗バーを表示
        const progressContainer = document.getElementById('progressContainer');
        if (progressContainer) {
            progressContainer.classList.remove('hidden');
        }

        // シミュレーションの場合は倍速コントロールと進捗スライダーを表示
        if (!isDevicePainting) {
            document.getElementById('simulationSpeedControl').classList.remove('hidden');
            document.getElementById('progressSliderControl').classList.remove('hidden');
            // 進捗スライダーをリセット
            document.getElementById('progressSlider').value = 0;
            document.getElementById('progressSliderValue').textContent = '0%';
        } else {
            document.getElementById('simulationSpeedControl').classList.add('hidden');
            document.getElementById('progressSliderControl').classList.add('hidden');
        }

        try {
            if (isDevicePainting) {
                // 実際の描画 - タイミング値を使用
                const timing = window.calibrationManager ? window.calibrationManager.getTimingValues() : {
                    pressMs: 100,
                    releaseMs: 60,
                    waitMs: 40
                };

                this.addLog(`Nintendo Switchで描画を開始します... (設定: ${timing.pressMs}+${timing.releaseMs}+${timing.waitMs}ms/px)`, 'info');

                // UI初期化
                this.initializePaintingUI();

                const strategy = document.getElementById('paint-strategy').value;
            const repeatsInput = document.getElementById('paint-repeats');
            const repeats = repeatsInput ? parseInt(repeatsInput.value, 10) || 1 : 1;

            const response = await fetch(`/api/artworks/${this.currentArtworkId}/paint`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    press_ms: timing.pressMs,
                    release_ms: timing.releaseMs,
                    wait_ms: timing.waitMs,
                    preview: false,
                    strategy: strategy,
                    repeats: repeats
                })
            });

                if (!response.ok) {
                    throw new Error(`描画エラー: ${response.status}`);
                }

                // WebSocketで進捗を監視するため、ここでは何もしない
                // updatePaintingProgress が呼ばれるのを待つ
            } else {
                // シミュレーション
                this.addLog(`描画シミュレーションを開始します...`, 'info');
                this.startPaintingVisualization();
            }

        } catch (error) {
            this.addLog(`描画エラー: ${error.message}`, 'error');
            this.stopPainting();
        }
    }

    startPaintingVisualization() {
        this.initializePaintingUI();

        // 描画を開始
        this.executeNextOperation();
    }

    async startPainting(useDevice = null) {
        if (!this.currentBinaryData) return;
        
        const isConnected = this.isServerConnected && this.isHardwareConnected;
                            
        let isDevicePainting;
        if (useDevice !== null) {
            isDevicePainting = useDevice;
        } else {
            isDevicePainting = isConnected;
        }
        
        if (isDevicePainting && !isConnected) {
            alert('実機が接続されていません。');
            return;
        }
        
        if (!isDevicePainting && useDevice === null && !confirm('実機が接続されていません。シミュレーションモードで開始しますか？')) {
            return;
        }
        
        this.isPainting = true;
        this.updateButtonStates();
        
        // 描画データを準備
        await this.preparePaintingData();
        
        // 描画進捗エリアを表示
        document.getElementById('paintingProgress').classList.remove('hidden');
        this.hideProgress();
        
        // シミュレーションの場合は倍速コントロールと進捗スライダーを表示
        if (!isDevicePainting) {
            document.getElementById('simulationSpeedControl').classList.remove('hidden');
            document.getElementById('progressSliderControl').classList.remove('hidden');
            // 進捗スライダーをリセット
            document.getElementById('progressSlider').value = 0;
            document.getElementById('progressSliderValue').textContent = '0%';
        } else {
            document.getElementById('simulationSpeedControl').classList.add('hidden');
            document.getElementById('progressSliderControl').classList.add('hidden');
        }

        try {
            if (isDevicePainting) {
                // 実際の描画
                this.addLog(`Nintendo Switchで描画を開始します... (速度: ${this.paintingSpeed.toFixed(1)}ドット/秒)`, 'info');
                
                // UI初期化
                this.initializePaintingUI();

                const response = await fetch(`/api/artworks/${this.currentArtworkId}/paint`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        speed: this.paintingSpeed,
                        preview: false
                    })
                });

                if (!response.ok) {
                    throw new Error(`描画エラー: ${response.status}`);
                }
                
                // WebSocketで進捗を監視するため、ここでは何もしない
                // updatePaintingProgress が呼ばれるのを待つ
            } else {
                // シミュレーション
                this.addLog(`描画シミュレーションを開始します... (速度: ${this.paintingSpeed.toFixed(1)}ドット/秒)`, 'info');
                this.startPaintingVisualization();
            }

        } catch (error) {
            this.addLog(`描画エラー: ${error.message}`, 'error');
            this.stopPainting();
        }
    }
    
    executeNextOperation() {
        if (!this.isPainting || this.isPaused) return;
        if (this.currentOperationIndex >= this.paintingOperations.length) {
            this.completePainting();
            return;
        }
        
        const operation = this.paintingOperations[this.currentOperationIndex];
        const cursor = document.getElementById('paintingCursor');
        const paintingCanvas = document.getElementById('paintingCanvas');
        const rect = paintingCanvas.getBoundingClientRect();
        const scaleX = rect.width / 320;
        const scaleY = rect.height / 120;
        
        // キャリブレーション値を取得
        let pressMs = 100;
        let releaseMs = 60;
        let waitMs = 40;
        
        if (window.calibrationManager) {
            const timing = window.calibrationManager.getTimingValues();
            pressMs = timing.pressMs;
            releaseMs = timing.releaseMs;
            waitMs = timing.waitMs;
        }

        switch (operation.type) {
            case 'pen_up':
                this.penState = 'up';
                cursor.classList.add('pen-up');
                cursor.classList.remove('pen-down');
                this.currentDpadCount++;
                // ペンを上げる操作の時間（キャリブレーション値を使用）
                const penUpTime = releaseMs / this.simulationMultiplier;
                setTimeout(() => {
                    this.currentOperationIndex++;
                    this.executeNextOperation();
                }, penUpTime);
                break;
                
            case 'pen_down':
                this.penState = 'down';
                cursor.classList.add('pen-down');
                cursor.classList.remove('pen-up');
                this.currentDpadCount++;
                // ペンを下げる操作の時間（キャリブレーション値を使用）
                const penDownTime = pressMs / this.simulationMultiplier;
                setTimeout(() => {
                    this.currentOperationIndex++;
                    this.executeNextOperation();
                }, penDownTime);
                break;
                
            case 'move':
                // 移動アニメーション
                if (operation.isDpadMove) {
                    // 十字キー移動の場合、移動距離分をカウント
                    const distance = Math.abs(operation.to.x - operation.from.x) + Math.abs(operation.to.y - operation.from.y);
                    this.currentDpadCount += distance;
                }
                this.animateMove(operation.from, operation.to, operation.duration, () => {
                    this.currentOperationIndex++;
                    this.executeNextOperation();
                });
                break;
                
            case 'draw':
                // ドットを描画
                const ctx = paintingCanvas.getContext('2d');
                ctx.fillStyle = '#000000';
                ctx.fillRect(operation.position.x, operation.position.y, 1, 1);
                
                this.paintedDots.push(operation.position);
                this.currentDotIndex++;
                this.currentAButtonCount++;
                
                // 描画操作の時間（キャリブレーション値を使用: 押下+解放+待機）
                const drawTime = (pressMs + releaseMs + waitMs) / this.simulationMultiplier;
                setTimeout(() => {
                    this.currentOperationIndex++;
                    this.executeNextOperation();
                }, drawTime);
                break;
        }
        
        // 進捗情報を更新
        this.updatePaintingStats();
        
        // プログレスバーを表示
        document.getElementById('progressContainer').classList.remove('hidden');
    }
    
    animateMove(from, to, duration, callback) {
        const cursor = document.getElementById('paintingCursor');
        const paintingCanvas = document.getElementById('paintingCanvas');
        const rect = paintingCanvas.getBoundingClientRect();
        const scaleX = rect.width / 320;
        const scaleY = rect.height / 120;

        const startTime = Date.now();
        const animationDuration = (duration * 1000) / this.simulationMultiplier;

        // 前回のフレームの位置を記録（ペンが下がっている場合の軌跡描画用）
        let lastX = from.x;
        let lastY = from.y;

        const animate = () => {
            if (!this.isPainting || this.isPaused) return;

            const elapsed = Date.now() - startTime;
            const progress = Math.min(elapsed / animationDuration, 1);

            // 線形補間
            const currentX = from.x + (to.x - from.x) * progress;
            const currentY = from.y + (to.y - from.y) * progress;

            // ペンが下がっている場合は移動軌跡を描画
            if (this.penState === 'down') {
                const ctx = paintingCanvas.getContext('2d');
                ctx.strokeStyle = '#000000';
                ctx.lineWidth = 1;
                ctx.beginPath();
                ctx.moveTo(lastX, lastY);
                ctx.lineTo(currentX, currentY);
                ctx.stroke();
            }

            // 次のフレーム用に位置を更新
            lastX = currentX;
            lastY = currentY;

            // カーソル位置を更新
            cursor.style.left = `${currentX * scaleX}px`;
            cursor.style.top = `${currentY * scaleY}px`;

            // 現在位置を更新
            this.currentPosition = { x: currentX, y: currentY };

            if (progress < 1) {
                requestAnimationFrame(animate);
            } else {
                this.currentPosition = to;
                callback();
            }
        };

        animate();
    }
    
    updatePaintingStats() {
        const elapsed = (Date.now() - this.paintingStartTime) / 1000;
        // 進捗は操作数ベースで計算（移動も含めるため）
        const progress = this.paintingOperations.length > 0 ? this.currentOperationIndex / this.paintingOperations.length : 0;
        
        // 残り時間の計算（シミュレーション用）
        // 推定時間は固定（totalEstimatedTime）とし、ここでの更新は行わない
        
        document.getElementById('paintedDots').textContent = this.currentDotIndex.toLocaleString();
        
        // 経過時間と残り時間を更新
        // シミュレーション時は「経過時間」は不要（ユーザー要望）
        if (this.isDevicePainting) {
            document.getElementById('elapsedTime').textContent = this.formatTime(elapsed);
            // 実機描画時は残り時間を表示したいかもしれないが、ユーザー要望により「推定時間」は静的にする
            // もし残り時間を表示したい場合は別の要素が必要だが、ここでは推定時間を上書きしないようにする
        } else {
            // シミュレーション時
            document.getElementById('elapsedTime').textContent = '-'; // 不要
            // 推定時間は静的に表示するため、ここでは更新しない
        }
        
        // ボタン操作の進捗を更新
        document.getElementById('dpadOperations').textContent = `${this.currentDpadCount.toLocaleString()}/${this.dpadCount.toLocaleString()}回`;
        document.getElementById('aButtonPresses').textContent = `${this.currentAButtonCount.toLocaleString()}/${this.aButtonCount.toLocaleString()}回`;
        
        // プログレスバーも更新
        const progressPercent = Math.round(progress * 100);
        document.getElementById('progressFill').style.width = `${progressPercent}%`;
        document.getElementById('progressText').textContent = `描画中... ${progressPercent}%`;
        
        // 進捗スライダーも更新（シミュレーション時のみ）
        if (!this.isDevicePainting) {
            document.getElementById('progressSlider').value = progress * 100;
            document.getElementById('progressSliderValue').textContent = `${progressPercent}%`;
        }
    }
    
    completePainting() {
        this.isPainting = false;
        this.isProcessing = false;
        this.updateButtonStates();

        const totalTime = (Date.now() - this.paintingStartTime) / 1000;
        this.addLog(`描画が完了しました（実行時間: ${this.formatTime(totalTime)}）`, 'success');


        // 描画進捗エリアと進捗バーを少し表示してから隠す（実機描画時のみ）


        // シミュレーション時は確認のために残す
        if (this.isDevicePainting) {
            setTimeout(() => {
                document.getElementById('paintingProgress').classList.add('hidden');
                const progressContainer = document.getElementById('progressContainer');
                if (progressContainer) {
                    progressContainer.classList.add('hidden');
                }
            }, 3000);
        }
    }
    
    async togglePausePainting() {
        if (!this.isPainting) return;
        
        this.isPaused = !this.isPaused;

        // Call backend to pause/resume if connected to hardware
        if (this.isHardwareConnected) {
            try {
                await fetch('/api/painting/pause', { method: 'POST' });
            } catch (e) {
                console.error('Failed to pause/resume painting on backend:', e);
                this.addLog(`バックエンドの一時停止/再開に失敗しました: ${e.message}`, 'error');
            }
        }

        const pauseButton = document.getElementById('pausePaintingButton');
        
        if (this.isPaused) {
            pauseButton.innerHTML = `
                <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                再開
            `;
            this.addLog('描画を一時停止しました', 'info');
        } else {
            pauseButton.innerHTML = `
                <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                一時停止
            `;
            this.addLog('描画を再開しました', 'info');
            // 現在の操作から再開
            if (this.currentOperationIndex < this.paintingOperations.length) {
                this.executeNextOperation();
            }
        }
    }
    
    async stopPainting() {
        if (!this.isPainting) return;
        
        // Call backend to stop if connected to hardware
        if (this.isHardwareConnected) {
            try {
                await fetch('/api/painting/stop', { method: 'POST' });
            } catch (e) {
                console.error('Failed to stop painting on backend:', e);
                this.addLog(`バックエンドの停止に失敗しました: ${e.message}`, 'error');
            }
        }

        this.isPainting = false;
        this.isPaused = false;
        this.isProcessing = false;
        this.currentDotIndex = 0;
        this.currentOperationIndex = 0;
        this.currentDpadCount = 0;
        this.currentAButtonCount = 0;

        // UIをリセット
        document.getElementById('paintingProgress').classList.add('hidden');
        document.getElementById('paintingCursor').classList.add('hidden');
        const progressContainer = document.getElementById('progressContainer');
        if (progressContainer) {
            progressContainer.classList.add('hidden');
        }
        this.updateButtonStates();
        
        const pauseButton = document.getElementById('pausePaintingButton');
        pauseButton.innerHTML = `
            <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                    d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            一時停止
        `;
        
        this.addLog('描画を停止しました', 'warning');
    }
    
    
    formatTime(seconds) {
        if (seconds < 60) {
            return `${Math.round(seconds)}秒`;
        } else {
            const minutes = Math.floor(seconds / 60);
            const secs = Math.round(seconds % 60);
            return `${minutes}分${secs}秒`;
        }
    }
    
    jumpToProgress(progress) {
        if (!this.paintingPath || this.paintingPath.length === 0) return;
        
        // シミュレーションを一時停止
        if (this.isPainting && !this.isPaused) {
            this.isPaused = true;
            // 一時停止ボタンの表示を更新
            const pauseButton = document.getElementById('pausePaintingButton');
            pauseButton.innerHTML = `
                <svg class="mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                        d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                再開
            `;
            this.addLog('進捗スライダー操作のため一時停止しました', 'info');
        }
        
        // 目標の操作インデックスを計算
        const targetOperationIndex = Math.floor(this.paintingOperations.length * progress);
        
        // キャンバスを再描画
        const paintingCanvas = document.getElementById('paintingCanvas');
        const ctx = paintingCanvas.getContext('2d');
        
        // 背景を白で塗りつぶし
        ctx.clearRect(0, 0, paintingCanvas.width, paintingCanvas.height);
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, paintingCanvas.width, paintingCanvas.height);
        
        // 描画済みドットの配列をリセット
        this.paintedDots = [];
        
        // 最初からターゲット位置まで再描画
        let currentDotIndex = 0;
        let dpadCount = 0;
        let aButtonCount = 0;
        
        // 描画操作のみを抽出して実行
        for (let i = 0; i <= targetOperationIndex && i < this.paintingOperations.length; i++) {
            const op = this.paintingOperations[i];
            
            if (op.type === 'draw') {
                ctx.fillStyle = '#000000';
                ctx.fillRect(op.position.x, op.position.y, 1, 1);
                this.paintedDots.push(op.position);
                currentDotIndex++;
                aButtonCount++;
            } else if (op.type === 'pen_up' || op.type === 'pen_down') {
                dpadCount++;
            } else if (op.type === 'move' && op.isDpadMove) {
                const distance = Math.abs(op.to.x - op.from.x) + Math.abs(op.to.y - op.from.y);
                dpadCount += distance;
            }
        }
        
        this.currentOperationIndex = targetOperationIndex;
        this.currentDotIndex = currentDotIndex;
        this.currentDpadCount = dpadCount;
        this.currentAButtonCount = aButtonCount;
        
        // カーソル位置を更新
        // カーソル位置を更新
        if (targetOperationIndex < this.paintingOperations.length) {
            const op = this.paintingOperations[targetOperationIndex];
            const cursor = document.getElementById('paintingCursor');
            const rect = paintingCanvas.getBoundingClientRect();
            const scaleX = rect.width / 320;
            const scaleY = rect.height / 120;
            
            // 操作タイプに応じてカーソル位置を決定
            let cursorX = 0;
            let cursorY = 0;
            
            if (op.type === 'draw') {
                cursorX = op.position.x;
                cursorY = op.position.y;
            } else if (op.type === 'move') {
                cursorX = op.to.x;
                cursorY = op.to.y;
            } else if (this.paintedDots.length > 0) {
                const lastDot = this.paintedDots[this.paintedDots.length - 1];
                cursorX = lastDot.x;
                cursorY = lastDot.y;
            }
            
            cursor.style.left = `${cursorX * scaleX}px`;
            cursor.style.top = `${cursorY * scaleY}px`;
            cursor.classList.remove('hidden');
            
            this.currentPosition = { x: cursorX, y: cursorY };
        }
        
        // 統計情報を更新
        this.updatePaintingStats();
        
        // スライダーの位置も更新
        const progressPercent = (progress * 100).toFixed(1);
        document.getElementById('progressSlider').value = progressPercent;
        document.getElementById('progressSliderValue').textContent = `${progressPercent}%`;
    }

    downloadResult() {
        if (!this.currentBinaryData) return;

        this.addLog('画像をダウンロード中...', 'info');
        
        // 320x120のキャンバスを作成
        const canvas = document.createElement('canvas');
        canvas.width = 320;
        canvas.height = 120;
        const ctx = canvas.getContext('2d');
        
        // 背景を白で塗りつぶし
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, 320, 120);
        
        // ドットを描画
        ctx.fillStyle = '#000000';
        for (let i = 0; i < this.currentBinaryData.length; i++) {
            if (this.currentBinaryData[i]) {
                const x = i % 320;
                const y = Math.floor(i / 320);
                ctx.fillRect(x, y, 1, 1);
            }
        }
        
        // キャンバスをPNGとしてダウンロード
        const url = canvas.toDataURL('image/png');
        
        // ダウンロードリンクを作成してクリック
        const a = document.createElement('a');
        a.href = url;
        a.download = `splatoon3-drawing-${new Date().toISOString().slice(0, 10)}.png`;
        document.body.appendChild(a);
        a.click();
        
        // クリーンアップ
        document.body.removeChild(a);
        
        this.addLog('ダウンロードが完了しました', 'success');
    }

    clearAll() {
        this.currentFile = null;
        this.currentArtworkId = null;
        this.currentBinaryData = null;
        document.getElementById('fileInput').value = '';
        
        // 元画像エリアを隠す
        document.getElementById('originalImageArea').classList.add('hidden');
        document.getElementById('uploadArea').classList.remove('hidden');
        
        // 変換後エリアを隠す
        document.getElementById('convertedImageArea').classList.add('hidden');
        document.getElementById('convertedArea').classList.remove('hidden');
        
        // 調整パネルを隠す
        document.getElementById('adjustmentPanel').classList.add('hidden');
        
        // 切り取りモードをリセット
        if (this.cropMode) {
            this.toggleCropMode();
        }
        this.cropArea = null;
        this.cropSelected = false;
        
        // 調整値をリセット
        this.resetAdjustments();
        
        this.updateButtonStates();
        this.addLog('データをクリアしました', 'info');
    }

    resetAdjustments() {
        // 値をリセット
        this.threshold = 128;
        this.brightness = 0;
        this.contrast = 0;
        this.gamma = 1.0;
        this.exposure = 0.0;
        this.highlights = 0;
        this.shadows = 0;
        this.blackPoint = 0;
        this.whitePoint = 255;
        this.previewMode = false;
        
        // UIを更新
        document.getElementById('thresholdSlider').value = 128;
        document.getElementById('thresholdValue').textContent = 128;
        document.getElementById('thresholdSlider').style.background = 'linear-gradient(to right, #000 0%, #000 50%, #fff 50%, #fff 100%)';
        
        document.getElementById('brightnessSlider').value = 0;
        document.getElementById('brightnessValue').textContent = 0;
        
        document.getElementById('contrastSlider').value = 0;
        document.getElementById('contrastValue').textContent = 0;
        
        document.getElementById('gammaSlider').value = 1.0;
        document.getElementById('gammaValue').textContent = '1.0';
        
        document.getElementById('exposureSlider').value = 0.0;
        document.getElementById('exposureValue').textContent = '0.0';
        
        document.getElementById('highlightsSlider').value = 0;
        document.getElementById('highlightsValue').textContent = 0;
        
        document.getElementById('shadowsSlider').value = 0;
        document.getElementById('shadowsValue').textContent = 0;
        
        document.getElementById('blackPointSlider').value = 0;
        document.getElementById('blackPointValue').textContent = 0;
        
        document.getElementById('whitePointSlider').value = 255;
        document.getElementById('whitePointValue').textContent = 255;
        
        const previewModeToggle = document.getElementById('previewModeToggle');
        if (previewModeToggle) {
            previewModeToggle.checked = false;
        }
        
        this.addLog('調整値をリセットしました', 'info');
        
        // プレビューを更新
        if (this.currentFile && this.currentArtworkId) {
            this.updatePreview();
        }
    }

    showProgress() {
        document.getElementById('progressContainer').classList.remove('hidden');
    }

    hideProgress() {
        document.getElementById('progressContainer').classList.add('hidden');
        this.updateProgress(0, '準備中...');
    }

    updateProgress(percent, text) {
        document.getElementById('progressFill').style.width = `${percent}%`;
        document.getElementById('progressText').textContent = text;
    }

    addLog(message, level = 'info') {
        const logArea = document.getElementById('logArea');
        const logEntry = document.createElement('div');
        logEntry.className = `log-entry ${level}`;
        
        const timestamp = new Date().toLocaleString('ja-JP');
        logEntry.innerHTML = `
            <span class="text-gray-500">[${timestamp}]</span>
            <span class="ml-2">${message}</span>
        `;
        
        logArea.appendChild(logEntry);
        logArea.scrollTop = logArea.scrollHeight;
    }

    clearLog() {
        const logArea = document.getElementById('logArea');
        logArea.innerHTML = '';
        this.addLog('ログをクリアしました', 'info');
    }

    downloadLog() {
        const logEntries = document.querySelectorAll('.log-entry');
        let logContent = '';
        
        logEntries.forEach(entry => {
            const timestamp = entry.querySelector('.text-gray-500').textContent;
            const message = entry.querySelector('.ml-2').textContent;
            logContent += `${timestamp} ${message}\n`;
        });
        
        const blob = new Blob([logContent], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `splatoon3-ghost-drawer-log-${new Date().toISOString().slice(0, 10)}.txt`;
        a.click();
        URL.revokeObjectURL(url);
        
        this.addLog('ログをダウンロードしました', 'success');
    }

    formatFileSize(bytes) {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const sizes = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    toggleCropMode() {
        this.cropMode = !this.cropMode;
        const cropButton = document.getElementById('cropButton');
        const cropOverlay = document.getElementById('cropOverlay');
        const cropInfo = document.getElementById('cropInfo');
        const applyCropButton = document.getElementById('applyCropButton');
        const originalImage = document.getElementById('originalImage');
        
        if (this.cropMode) {
            cropButton.classList.add('crop-active', 'bg-splatoon-yellow', 'text-gray-900');
            cropButton.classList.remove('bg-gray-700', 'text-gray-300');
            cropInfo.classList.remove('hidden');
            
            // オーバーレイキャンバスを設定
            this.setupCropOverlay();
            cropOverlay.classList.remove('hidden');
            
            this.addLog('切り取りモードを有効にしました', 'info');
        } else {
            cropButton.classList.remove('crop-active', 'bg-splatoon-yellow', 'text-gray-900');
            cropButton.classList.add('bg-gray-700', 'text-gray-300');
            cropInfo.classList.add('hidden');
            cropOverlay.classList.add('hidden');
            applyCropButton.classList.add('hidden');
            
            // イベントリスナーを削除
            this.removeCropListeners();
            
            this.addLog('切り取りモードを無効にしました', 'info');
        }
    }
    
    applyCrop() {
        if (!this.cropArea || !this.cropSelected) return;
        
        this.addLog('切り取りを適用して変換を開始します...', 'info');
        
        // 切り取りモードを終了
        this.toggleCropMode();
        
        // 変換を実行
        this.convertImage();
    }

    setupCropOverlay() {
        const originalImage = document.getElementById('originalImage');
        const cropOverlay = document.getElementById('cropOverlay');
        const imageContainer = document.getElementById('imageContainer');
        
        // キャンバスサイズを画像に合わせる
        cropOverlay.width = originalImage.width;
        cropOverlay.height = originalImage.height;
        
        // マウスイベントを設定
        cropOverlay.addEventListener('mousedown', this.handleCropMouseDown.bind(this));
        cropOverlay.addEventListener('mousemove', this.handleCropMouseMove.bind(this));
        cropOverlay.addEventListener('mouseup', this.handleCropMouseUp.bind(this));
        cropOverlay.addEventListener('mouseleave', this.handleCropMouseUp.bind(this));
        
        // タッチイベントも設定（モバイル対応）
        cropOverlay.addEventListener('touchstart', this.handleCropTouchStart.bind(this));
        cropOverlay.addEventListener('touchmove', this.handleCropTouchMove.bind(this));
        cropOverlay.addEventListener('touchend', this.handleCropTouchEnd.bind(this));
    }

    removeCropListeners() {
        const cropOverlay = document.getElementById('cropOverlay');
        const newOverlay = cropOverlay.cloneNode(true);
        cropOverlay.parentNode.replaceChild(newOverlay, cropOverlay);
    }

    handleCropMouseDown(e) {
        const rect = e.target.getBoundingClientRect();
        const x = e.clientX - rect.left;
        const y = e.clientY - rect.top;
        
        // 既に選択範囲がある場合
        if (this.cropSelected && this.cropArea) {
            // ハンドルをクリックしたかチェック
            const handle = this.getHandleAt(x, y);
            if (handle) {
                this.resizing = handle;
                this.dragStart = { x, y };
                return;
            }
            
            // 選択範囲内をクリックしたかチェック
            if (x >= this.cropArea.x && x <= this.cropArea.x + this.cropArea.width &&
                y >= this.cropArea.y && y <= this.cropArea.y + this.cropArea.height) {
                this.moving = true;
                this.moveStart = {
                    x: x - this.cropArea.x,
                    y: y - this.cropArea.y
                };
                return;
            }
        }
        
        // 新しい選択を開始
        this.isDragging = true;
        this.cropSelected = false;
        this.dragStart = { x, y };
    }
    
    getHandleAt(x, y) {
        if (!this.cropArea) return null;
        
        const handles = [
            { name: 'nw', x: this.cropArea.x, y: this.cropArea.y },
            { name: 'ne', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y },
            { name: 'sw', x: this.cropArea.x, y: this.cropArea.y + this.cropArea.height },
            { name: 'se', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y + this.cropArea.height },
            { name: 'n', x: this.cropArea.x + this.cropArea.width / 2, y: this.cropArea.y },
            { name: 's', x: this.cropArea.x + this.cropArea.width / 2, y: this.cropArea.y + this.cropArea.height },
            { name: 'w', x: this.cropArea.x, y: this.cropArea.y + this.cropArea.height / 2 },
            { name: 'e', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y + this.cropArea.height / 2 }
        ];
        
        for (const handle of handles) {
            if (Math.abs(x - handle.x) < 8 && Math.abs(y - handle.y) < 8) {
                return handle.name;
            }
        }
        
        return null;
    }

    handleCropMouseMove(e) {
        const rect = e.target.getBoundingClientRect();
        const currentX = e.clientX - rect.left;
        const currentY = e.clientY - rect.top;
        const canvas = e.target;
        
        // リサイズ中
        if (this.resizing) {
            this.handleResize(currentX, currentY, canvas);
        }
        // 移動中
        else if (this.moving) {
            this.handleMove(currentX, currentY, canvas);
        }
        // 新規選択中
        else if (this.isDragging) {
            // 320:120の比率（8:3）を維持
            const aspectRatio = 320 / 120;
            let width = Math.abs(currentX - this.dragStart.x);
            let height = Math.abs(currentY - this.dragStart.y);
            
            // 幅を基準にして高さを計算
            if (width / height > aspectRatio) {
                height = width / aspectRatio;
            } else {
                width = height * aspectRatio;
            }
            
            // 開始点の調整（ドラッグ方向に応じて）
            let x = this.dragStart.x;
            let y = this.dragStart.y;
            
            if (currentX < this.dragStart.x) {
                x = this.dragStart.x - width;
            }
            if (currentY < this.dragStart.y) {
                y = this.dragStart.y - height;
            }
            
            // キャンバス内に収まるように調整
            if (x < 0) x = 0;
            if (y < 0) y = 0;
            if (x + width > canvas.width) {
                x = canvas.width - width;
            }
            if (y + height > canvas.height) {
                y = canvas.height - height;
            }
            
            this.cropArea = { x, y, width, height };
            this.drawCropOverlay();
        }
    }
    
    handleResize(currentX, currentY, canvas) {
        const aspectRatio = 320 / 120;
        let { x, y, width, height } = this.cropArea;
        
        // ハンドルごとのリサイズ処理
        switch (this.resizing) {
            case 'se': // 右下
                width = currentX - x;
                height = width / aspectRatio;
                break;
            case 'sw': // 左下
                width = x + width - currentX;
                height = width / aspectRatio;
                x = currentX;
                break;
            case 'ne': // 右上
                width = currentX - x;
                height = width / aspectRatio;
                y = y + (this.cropArea.height - height);
                break;
            case 'nw': // 左上
                width = x + width - currentX;
                height = width / aspectRatio;
                x = currentX;
                y = y + (this.cropArea.height - height);
                break;
            case 'e': // 右
                width = currentX - x;
                height = width / aspectRatio;
                y = y + (this.cropArea.height - height) / 2;
                break;
            case 'w': // 左
                width = x + width - currentX;
                height = width / aspectRatio;
                x = currentX;
                y = y + (this.cropArea.height - height) / 2;
                break;
            case 'n': // 上
                height = y + height - currentY;
                width = height * aspectRatio;
                y = currentY;
                x = x + (this.cropArea.width - width) / 2;
                break;
            case 's': // 下
                height = currentY - y;
                width = height * aspectRatio;
                x = x + (this.cropArea.width - width) / 2;
                break;
        }
        
        // 最小サイズ制限
        if (width < 80) {
            width = 80;
            height = width / aspectRatio;
        }
        
        // キャンバス内に収まるように調整
        if (x < 0) x = 0;
        if (y < 0) y = 0;
        if (x + width > canvas.width) {
            width = canvas.width - x;
            height = width / aspectRatio;
        }
        if (y + height > canvas.height) {
            height = canvas.height - y;
            width = height * aspectRatio;
        }
        
        this.cropArea = { x, y, width, height };
        this.drawCropOverlay();
    }
    
    handleMove(currentX, currentY, canvas) {
        let x = currentX - this.moveStart.x;
        let y = currentY - this.moveStart.y;
        
        // キャンバス内に収まるように調整
        if (x < 0) x = 0;
        if (y < 0) y = 0;
        if (x + this.cropArea.width > canvas.width) {
            x = canvas.width - this.cropArea.width;
        }
        if (y + this.cropArea.height > canvas.height) {
            y = canvas.height - this.cropArea.height;
        }
        
        this.cropArea.x = x;
        this.cropArea.y = y;
        this.drawCropOverlay();
    }

    handleCropMouseUp(e) {
        if (this.isDragging && this.cropArea && this.cropArea.width > 10 && this.cropArea.height > 10) {
            this.addLog(`切り取り範囲: ${Math.round(this.cropArea.width)}x${Math.round(this.cropArea.height)} (8:3比率)`, 'info');
            this.cropSelected = true;
            this.drawCropOverlay();
            // 適用ボタンを表示
            document.getElementById('applyCropButton').classList.remove('hidden');
        }
        this.isDragging = false;
        this.resizing = null;
        this.moving = false;
    }

    handleCropTouchStart(e) {
        e.preventDefault();
        const touch = e.touches[0];
        const rect = e.target.getBoundingClientRect();
        this.isDragging = true;
        this.dragStart = {
            x: touch.clientX - rect.left,
            y: touch.clientY - rect.top
        };
    }

    handleCropTouchMove(e) {
        e.preventDefault();
        if (!this.isDragging) return;
        
        const touch = e.touches[0];
        const rect = e.target.getBoundingClientRect();
        const currentX = touch.clientX - rect.left;
        const currentY = touch.clientY - rect.top;
        
        // 320:120の比率（8:3）を維持
        const aspectRatio = 320 / 120;
        let width = Math.abs(currentX - this.dragStart.x);
        let height = Math.abs(currentY - this.dragStart.y);
        
        // 幅を基準にして高さを計算
        if (width / height > aspectRatio) {
            height = width / aspectRatio;
        } else {
            width = height * aspectRatio;
        }
        
        // 開始点の調整
        let x = this.dragStart.x;
        let y = this.dragStart.y;
        
        if (currentX < this.dragStart.x) {
            x = this.dragStart.x - width;
        }
        if (currentY < this.dragStart.y) {
            y = this.dragStart.y - height;
        }
        
        // キャンバス内に収まるように調整
        const canvas = e.target;
        if (x < 0) x = 0;
        if (y < 0) y = 0;
        if (x + width > canvas.width) {
            x = canvas.width - width;
        }
        if (y + height > canvas.height) {
            y = canvas.height - height;
        }
        
        this.cropArea = { x, y, width, height };
        this.drawCropOverlay();
    }

    handleCropTouchEnd(e) {
        e.preventDefault();
        this.handleCropMouseUp(e);
    }

    drawCropOverlay() {
        const cropOverlay = document.getElementById('cropOverlay');
        const ctx = cropOverlay.getContext('2d');
        
        // キャンバスをクリア
        ctx.clearRect(0, 0, cropOverlay.width, cropOverlay.height);
        
        if (!this.cropArea) return;
        
        // 半透明の黒で全体を覆う
        ctx.fillStyle = 'rgba(0, 0, 0, 0.5)';
        ctx.fillRect(0, 0, cropOverlay.width, cropOverlay.height);
        
        // 選択範囲をクリア（透明に）
        ctx.clearRect(this.cropArea.x, this.cropArea.y, this.cropArea.width, this.cropArea.height);
        
        // 選択範囲の枠を描画
        ctx.strokeStyle = '#F5D800';
        ctx.lineWidth = 2;
        ctx.strokeRect(this.cropArea.x, this.cropArea.y, this.cropArea.width, this.cropArea.height);
        
        // サイズ情報を表示
        ctx.fillStyle = '#F5D800';
        ctx.font = 'bold 14px system-ui';
        const sizeText = `320 × 120`;
        const textWidth = ctx.measureText(sizeText).width;
        ctx.fillText(
            sizeText,
            this.cropArea.x + (this.cropArea.width - textWidth) / 2,
            this.cropArea.y - 5
        );
        
        // 選択完了時はハンドルを表示
        if (this.cropSelected) {
            this.drawCropHandles(ctx);
        }
    }
    
    drawCropHandles(ctx) {
        const handles = [
            { name: 'nw', x: this.cropArea.x, y: this.cropArea.y },
            { name: 'ne', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y },
            { name: 'sw', x: this.cropArea.x, y: this.cropArea.y + this.cropArea.height },
            { name: 'se', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y + this.cropArea.height },
            { name: 'n', x: this.cropArea.x + this.cropArea.width / 2, y: this.cropArea.y },
            { name: 's', x: this.cropArea.x + this.cropArea.width / 2, y: this.cropArea.y + this.cropArea.height },
            { name: 'w', x: this.cropArea.x, y: this.cropArea.y + this.cropArea.height / 2 },
            { name: 'e', x: this.cropArea.x + this.cropArea.width, y: this.cropArea.y + this.cropArea.height / 2 }
        ];
        
        // ハンドルを描画
        handles.forEach(handle => {
            ctx.fillStyle = '#F5D800';
            ctx.fillRect(handle.x - 4, handle.y - 4, 8, 8);
            ctx.strokeStyle = '#1f2937';
            ctx.lineWidth = 1;
            ctx.strokeRect(handle.x - 4, handle.y - 4, 8, 8);
        });
    }

    async updatePreview() {
        if (!this.currentFile) return;

        try {
            this.addLog(`画像調整を適用中... (露出:${this.exposure.toFixed(1)}, コントラスト:${this.contrast}, 閾値:${this.threshold})`, 'info');
            
            // ブラウザ側で画像処理
            const adjustments = {
                brightness: this.brightness,
                contrast: this.contrast,
                gamma: this.gamma,
                exposure: this.exposure,
                highlights: this.highlights,
                shadows: this.shadows,
                blackPoint: this.blackPoint,
                whitePoint: this.whitePoint,
                previewMode: this.previewMode
            };
            
            // 切り取り範囲がある場合は、画像の表示サイズ情報を追加
            let cropAreaWithImageInfo = null;
            if (this.cropArea) {
                const originalImage = document.getElementById('originalImage');
                cropAreaWithImageInfo = {
                    ...this.cropArea,
                    originalImage: {
                        width: originalImage.width,
                        height: originalImage.height
                    }
                };
            }
            
            const processedData = await this.imageProcessor.processImage(
                this.currentFile, 
                this.threshold, 
                adjustments,
                cropAreaWithImageInfo
            );
            this.currentBinaryData = processedData.binaryData;
            
            // ドットデータに変換
            const dots = this.imageProcessor.convertToDotData(
                processedData.binaryData,
                processedData.width,
                processedData.height
            );
            
            if (dots.length === 0) {
                this.addLog('ドットが検出されませんでした。閾値を調整してください。', 'warning');
                this.displayProcessedCanvas(processedData.canvas);
                return;
            }
            
            this.addLog(`プレビュー更新完了: ${dots.length}個の描画ドット`, 'info');
            
            // 変換後の画像を表示
            this.displayProcessedCanvas(processedData.canvas);
            
            // サーバーに新しいデータを送信
            const response = await fetch('/api/artworks', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    name: this.currentFile.name.replace(/\.[^/.]+$/, '') || 'Untitled',
                    width: processedData.width,
                    height: processedData.height,
                    dots: dots
                })
            });

            if (response.ok) {
                const result = await response.json();
                this.currentArtworkId = result.id;
            }
            
        } catch (error) {
            this.addLog(`プレビュー更新エラー: ${error.message}`, 'error');
        }
    }

    // クリーンアップ
    destroy() {
        if (this.connectionCheckInterval) {
            clearInterval(this.connectionCheckInterval);
        }
        if (this.abortController) {
            this.abortController.abort();
        }
    }
}

// アプリケーション初期化
document.addEventListener('DOMContentLoaded', () => {
    window.ghostDrawerApp = new GhostDrawerApp();
});

// ページ終了時のクリーンアップ
window.addEventListener('beforeunload', () => {
    if (window.ghostDrawerApp) {
        window.ghostDrawerApp.destroy();
    }
});

// エラーハンドリング
window.addEventListener('error', (event) => {
    console.error('Global error:', event.error);
    if (window.ghostDrawerApp) {
        window.ghostDrawerApp.addLog(`エラー: ${event.error.message}`, 'error');
    }
});

window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
    if (window.ghostDrawerApp) {
        window.ghostDrawerApp.addLog(`Promise エラー: ${event.reason}`, 'error');
    }
});

// ============================================
// 速度キャリブレーション機能
// ============================================

class CalibrationManager {
    constructor() {
        this.modal = document.getElementById('calibrationModal');
        this.openButton = document.getElementById('openCalibrationButton');
        this.closeButton = document.getElementById('closeCalibrationButton');
        this.cancelButton = document.getElementById('cancelCalibrationButton');
        this.runButton = document.getElementById('runCalibrationButton');
        this.stopButton = document.getElementById('stopCalibrationButton');
        this.applyAndStartButton = document.getElementById('applyAndStartPaintingButton');

        // 速度スライダー（統合版）
        this.speedInput = document.getElementById('speedInput');
        this.speedValue = document.getElementById('speedValue');
        this.skipInitCheckbox = document.getElementById('skipInitializationCheckbox');

        this.isRunning = false;

        this.initEventListeners();
        this.updateValues(); // 初期値を設定
    }

    initEventListeners() {
        // モーダル開閉
        this.openButton?.addEventListener('click', () => this.openModal());
        this.closeButton?.addEventListener('click', () => this.closeModal());
        this.cancelButton?.addEventListener('click', () => this.closeModal());

        // 速度スライダー変更
        this.speedInput?.addEventListener('input', () => this.updateValues());

        // テスト実行と停止
        this.runButton?.addEventListener('click', () => this.runCalibration());
        this.stopButton?.addEventListener('click', () => this.stopCalibration());

        // 設定して描画開始ボタン
        this.applyAndStartButton?.addEventListener('click', () => this.applyAndStartPainting());

        // モーダル外クリックで閉じる
        this.modal?.addEventListener('click', (e) => {
            if (e.target === this.modal) {
                this.closeModal();
            }
        });
    }

    openModal(mode = 'device') {
        this.mode = mode;
        this.modal?.classList.remove('hidden');
        this.modal?.classList.add('flex');
        this.updateButtonStates();

        const specs = document.getElementById('calibrationSpecs');
        const skipInit = document.getElementById('calibrationSkipInit');
        const expected = document.getElementById('calibrationExpectedResult');
        const runBtn = document.getElementById('runCalibrationButton');
        
        if (mode === 'simulation') {
            specs?.classList.add('hidden');
            skipInit?.classList.add('hidden');
            expected?.classList.add('hidden');
            runBtn?.classList.add('hidden');
            if (this.applyAndStartButton) {
                this.applyAndStartButton.innerHTML = `
                    <svg class="inline-block mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                    </svg>
                    設定を適用
                `;
            }
        } else {
            specs?.classList.remove('hidden');
            skipInit?.classList.remove('hidden');
            expected?.classList.remove('hidden');
            runBtn?.classList.remove('hidden');
            if (this.applyAndStartButton) {
                this.applyAndStartButton.innerHTML = `
                    <svg class="inline-block mr-2 h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    設定して描画開始
                `;
            }
        }
    }

    closeModal() {
        // モーダルを閉じる際、テスト実行中なら停止処理を行う
        if (this.isRunning) {
            this.stopCalibration();
        }
        this.modal?.classList.add('hidden');
        this.modal?.classList.remove('flex');
    }

    updateValues() {
        // 速度スライダーの値を取得（50〜1000、デフォルト100）
        const speedValue = parseInt(this.speedInput?.value || 100);

        // speedValueは描画速度を表す（大きい = 速い）
        // speedMultiplierはタイミング倍率を表す（大きい = 遅い）
        // 反比例関係: speedMultiplier = 10000 / (speedValue * 100)
        const speedMultiplier = 10000 / (speedValue * 100);

        // 基準値（1.0xの場合、250ms/px）
        // ユーザー報告に基づき、より確実な「安全な速度」に戻す
        const basePressMs = 125;
        const baseReleaseMs = 75;
        const baseWaitMs = 50;

        // 速度倍率から各タイミング値を計算
        const pressMs = Math.round(basePressMs * speedMultiplier);
        const releaseMs = Math.round(baseReleaseMs * speedMultiplier);
        const waitMs = Math.round(baseWaitMs * speedMultiplier);

        // 速度表示を更新（スライダー値を100で割った値を表示）
        const displaySpeed = speedValue / 100.0;
        if (this.speedValue) {
            this.speedValue.textContent = `${displaySpeed.toFixed(1)}x`;
        }

        // 描画中ならリアルタイムで反映
        if (window.ghostDrawerApp && window.ghostDrawerApp.isPainting) {
            // デバウンス処理（頻繁なAPI呼び出しを防ぐ）
            if (this.updateTimeout) {
                clearTimeout(this.updateTimeout);
            }
            this.updateTimeout = setTimeout(async () => {
                try {
                    await fetch('/api/painting/timing', {
                        method: 'POST',
                        headers: { 'Content-Type': 'application/json' },
                        body: JSON.stringify({
                            press_ms: pressMs,
                            release_ms: releaseMs,
                            wait_ms: waitMs
                        })
                    });
                    console.log(`Updated timing: ${pressMs}/${releaseMs}/${waitMs} ms`);
                } catch (error) {
                    console.error('Failed to update timing:', error);
                }
            }, 200);
        }

        // 描画準備モーダルの値も更新
        if (window.ghostDrawerApp && window.ghostDrawerApp.updatePaintPrepareModalValues) {
            window.ghostDrawerApp.updatePaintPrepareModalValues();
        }
        
        // 推定時間を再計算して更新
        if (window.ghostDrawerApp && window.ghostDrawerApp.updateEstimatedTime) {
            window.ghostDrawerApp.updateEstimatedTime();
        }

        // 戦略比較テーブルも更新
        if (window.ghostDrawerApp && window.ghostDrawerApp.renderStrategyStats) {
            window.ghostDrawerApp.renderStrategyStats();
        }

        // シミュレーション画面のスライダーも更新
        const simSlider = document.getElementById('simulationSpeedSlider');
        const simValue = document.getElementById('simulationSpeedValue');
        if (simSlider) {
            simSlider.value = speedValue;
        }
        if (simValue) {
            simValue.textContent = `${displaySpeed.toFixed(1)}x`;
        }
    }

    // 計算されたタイミング値を取得するヘルパーメソッド
    getTimingValues() {
        const speedValue = parseInt(this.speedInput?.value || 100);

        // speedValueは描画速度を表す（大きい = 速い）
        // speedMultiplierはタイミング倍率を表す（大きい = 遅い）
        // 反比例関係: speedMultiplier = 10000 / (speedValue * 100)
        const speedMultiplier = 10000 / (speedValue * 100);

        // 基準値（1.0xの場合、250ms/px）
        const basePressMs = 125;
        const baseReleaseMs = 75;
        const baseWaitMs = 50;

        return {
            pressMs: Math.round(basePressMs * speedMultiplier),
            releaseMs: Math.round(baseReleaseMs * speedMultiplier),
            waitMs: Math.round(baseWaitMs * speedMultiplier)
        };
    }

    updateButtonStates() {
        if (this.isRunning) {
            // テスト実行中: 停止ボタンを表示、実行ボタンを非表示
            this.runButton?.classList.add('hidden');
            this.stopButton?.classList.remove('hidden');
        } else {
            // アイドル状態: 実行ボタンを表示、停止ボタンを非表示
            this.runButton?.classList.remove('hidden');
            this.stopButton?.classList.add('hidden');
        }
    }

    async runCalibration() {
        const { pressMs, releaseMs, waitMs } = this.getTimingValues();
        const skipInit = this.skipInitCheckbox?.checked || false;

        console.log('Starting calibration with params:', { pressMs, releaseMs, waitMs, skipInit });

        try {
            const response = await fetch('/api/calibration/start', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    press_ms: pressMs,
                    release_ms: releaseMs,
                    wait_ms: waitMs,
                    skip_initialization: skipInit
                })
            });

            const result = await response.json();

            if (result.success) {
                this.isRunning = true;
                this.updateButtonStates();

                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog(`キャリブレーションテスト開始: ${pressMs}+${releaseMs}+${waitMs}ms/pixel`, 'info');
                }

                // モーダルは閉じない
            } else {
                console.error('Calibration failed:', result);
                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog('キャリブレーションテストの開始に失敗しました', 'error');
                }
            }
        } catch (error) {
            console.error('Calibration error:', error);
            if (window.ghostDrawerApp) {
                window.ghostDrawerApp.addLog(`キャリブレーションエラー: ${error.message}`, 'error');
            }
        }
    }

    async stopCalibration() {
        console.log('Stopping calibration...');

        try {
            const response = await fetch('/api/painting/stop', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            const result = await response.json();

            if (result.success) {
                this.isRunning = false;
                this.updateButtonStates();

                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog('キャリブレーションテストを停止しました', 'info');
                }
            } else {
                console.error('Stop calibration failed:', result);
                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog('キャリブレーションテストの停止に失敗しました', 'warning');
                }
            }
        } catch (error) {
            console.error('Stop calibration error:', error);
            if (window.ghostDrawerApp) {
                window.ghostDrawerApp.addLog(`停止エラー: ${error.message}`, 'error');
            }
        }
    }

    /**
     * キャリブレーション完了通知を処理
     * WebSocketから呼び出される
     */
    handleCalibrationComplete(data) {
        console.log('Calibration complete notification received:', data);

        // 実行中フラグをリセット
        this.isRunning = false;
        this.updateButtonStates();

        // ステータスに応じたログメッセージ（既にdebug.jsで追加されているため不要）
        // if (window.ghostDrawerApp) {
        //     const logLevel = data.status === 'success' ? 'info' : data.status === 'error' ? 'error' : 'warning';
        //     window.ghostDrawerApp.addLog(data.message, logLevel);
        // }
    }

    async runPaintMoveTest() {
        const { pressMs, releaseMs, waitMs } = this.getTimingValues();

        console.log('Starting paint move test with params:', { pressMs, releaseMs, waitMs });

        try {
            const response = await fetch('/api/calibration/test/paint-move', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    press_ms: pressMs,
                    release_ms: releaseMs,
                    wait_ms: waitMs
                })
            });

            const result = await response.json();

            if (result.success) {
                this.isRunning = true;
                this.updateButtonStates();

                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog(`描画移動テスト開始: ${pressMs}+${releaseMs}+${waitMs}ms/pixel`, 'info');
                }
            } else {
                console.error('Paint move test failed:', result);
                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog('描画移動テストの開始に失敗しました', 'error');
                }
            }
        } catch (error) {
            console.error('Paint move test error:', error);
            if (window.ghostDrawerApp) {
                window.ghostDrawerApp.addLog(`描画移動テストエラー: ${error.message}`, 'error');
            }
        }
    }

    async runGapMoveTest() {
        const { pressMs, releaseMs, waitMs } = this.getTimingValues();

        console.log('Starting gap move test with params:', { pressMs, releaseMs, waitMs });

        try {
            const response = await fetch('/api/calibration/test/gap-move', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    press_ms: pressMs,
                    release_ms: releaseMs,
                    wait_ms: waitMs
                })
            });

            const result = await response.json();

            if (result.success) {
                this.isRunning = true;
                this.updateButtonStates();

                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog(`空白移動テスト開始: ${pressMs}+${releaseMs}+${waitMs}ms/pixel`, 'info');
                }
            } else {
                console.error('Gap move test failed:', result);
                if (window.ghostDrawerApp) {
                    window.ghostDrawerApp.addLog('空白移動テストの開始に失敗しました', 'error');
                }
            }
        } catch (error) {
            console.error('Gap move test error:', error);
            if (window.ghostDrawerApp) {
                window.ghostDrawerApp.addLog(`空白移動テストエラー: ${error.message}`, 'error');
            }
        }
    }

    applyAndStartPainting() {
        // キャリブレーションモーダルを閉じる
        this.closeModal();

        if (this.mode === 'simulation') {
            if (window.ghostDrawerApp) {
                window.ghostDrawerApp.addLog('シミュレーション速度設定を更新しました', 'info');
            }
            return;
        }

        // GhostDrawerAppが存在し、executePaintingメソッドが利用可能な場合
        if (window.ghostDrawerApp && typeof window.ghostDrawerApp.executePainting === 'function') {
            // 描画準備モーダルも閉じる
            if (window.ghostDrawerApp.closePaintPrepareModal) {
                window.ghostDrawerApp.closePaintPrepareModal();
            }

            // ハードウェアの接続状態を確認
            const isConnected = window.ghostDrawerApp.isServerConnected && window.ghostDrawerApp.isHardwareConnected;

            // 接続されていれば実機、そうでなければシミュレーションで描画を開始
            window.ghostDrawerApp.executePainting(isConnected);
        } else {
            console.error('GhostDrawerApp or executePainting method is not available');
        }
    }
}

// 初期化
document.addEventListener('DOMContentLoaded', () => {
    window.calibrationManager = new CalibrationManager();
}); 