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
            this.closePaintPrepareModal();
            this.executePainting(this.pendingPaintUseDevice);
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
                if (!this.isHardwareConnected && this.paintingPath && this.paintingPath.length > 0) {
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







    preparePaintingData() {
        // 黒いドットのみを抽出（白はスキップ）
        this.paintedDots = [];
        const dots = [];
        
        for (let y = 0; y < 120; y++) {
            for (let x = 0; x < 320; x++) {
                const index = y * 320 + x;
                if (this.currentBinaryData[index]) {
                    dots.push({ x, y, index });
                }
            }
        }
        
        // 最適な描画パスを計算（左上から右下へジグザグ）
        this.paintingPath = [];
        for (let y = 0; y < 120; y++) {
            const rowDots = dots.filter(d => d.y === y);
            if (y % 2 === 0) {
                // 偶数行は左から右へ
                this.paintingPath.push(...rowDots.sort((a, b) => a.x - b.x));
            } else {
                // 奇数行は右から左へ
                this.paintingPath.push(...rowDots.sort((a, b) => b.x - a.x));
            }
        }
        
        // コントローラー操作を含む実際の操作シーケンスを生成
        this.paintingOperations = this.generateOperations();
        
        // 操作回数をカウント
        this.dpadCount = 0;
        this.aButtonCount = 0;
        for (const op of this.paintingOperations) {
            if (op.type === 'pen_up' || op.type === 'pen_down') {
                this.dpadCount++; // ペンの上げ下げ（ZL+十字キー）
            } else if (op.type === 'move' && op.isDpadMove) {
                // 移動距離に基づいて十字キー操作数を計算
                const distance = Math.abs(op.to.x - op.from.x) + Math.abs(op.to.y - op.from.y);
                this.dpadCount += distance; // 1ピクセルにつき1回の十字キー操作
            } else if (op.type === 'draw') {
                this.aButtonCount++;
            }
        }
        
        // 統計情報を更新
        const totalPixels = 320 * 120;
        const blackDots = this.paintingPath.length;
        const whiteDots = totalPixels - blackDots;
        
        // 実際の時間を計算（移動時間を含む）
        const estimatedSeconds = this.calculateRealPaintingTime();
        
        document.getElementById('totalDots').textContent = blackDots.toLocaleString();
        document.getElementById('skippedDots').textContent = whiteDots.toLocaleString();
        document.getElementById('estimatedTime').textContent = this.formatTime(estimatedSeconds);
        document.getElementById('dpadOperations').textContent = `0/${this.dpadCount.toLocaleString()}回`;
        document.getElementById('aButtonPresses').textContent = `0/${this.aButtonCount.toLocaleString()}回`;
        
        this.addLog(`描画ドット数: ${blackDots}個（白部分${whiteDots}ピクセルも移動）`, 'info');
        this.addLog(`推定描画時間: ${this.formatTime(estimatedSeconds)}（全移動・操作時間含む）`, 'info');
        this.addLog(`操作回数 - 十字キー: ${this.dpadCount}回、Aボタン: ${this.aButtonCount}回`, 'info');
    }
    
    generateOperations() {
        const operations = [];
        let currentPos = { x: 0, y: 0 };
        let penIsDown = false;
        
        // 各行の黒いピクセルの範囲を事前に計算
        const rowRanges = [];
        for (let y = 0; y < 120; y++) {
            let firstBlack = -1;
            let lastBlack = -1;
            
            for (let x = 0; x < 320; x++) {
                const index = y * 320 + x;
                if (this.currentBinaryData[index]) {
                    if (firstBlack === -1) firstBlack = x;
                    lastBlack = x;
                }
            }
            
            rowRanges.push({ firstBlack, lastBlack });
        }
        
        // ジグザグパスで移動
        for (let y = 0; y < 120; y++) {
            const range = rowRanges[y];
            
            // この行に黒いピクセルがない場合はスキップ
            if (range.firstBlack === -1) continue;
            
            const isEvenRow = y % 2 === 0;
            const startX = isEvenRow ? range.firstBlack : range.lastBlack;
            const endX = isEvenRow ? range.lastBlack : range.firstBlack;
            const step = isEvenRow ? 1 : -1;
            
            // 行の最初の黒ピクセルへ移動
            if (startX !== currentPos.x || y !== currentPos.y) {
                // ペンが下がっている場合は上げる
                if (penIsDown) {
                    operations.push({ type: 'pen_up' });
                    penIsDown = false;
                }
                
                // 移動操作（十字キーでの移動）
                operations.push({ 
                    type: 'move', 
                    from: { ...currentPos }, 
                    to: { x: startX, y },
                    duration: this.calculateMoveDuration(currentPos, { x: startX, y }),
                    isDpadMove: true  // 十字キー移動フラグ
                });
                
                currentPos = { x: startX, y };
            }
            
            // 行内をスキャン
            for (let x = startX; isEvenRow ? (x <= endX) : (x >= endX); x += step) {
                const index = y * 320 + x;
                const isBlack = this.currentBinaryData[index];
                
                // 現在位置からの移動が必要かチェック
                if (x !== currentPos.x) {
                    // ペンが下がっている場合は上げる
                    if (penIsDown) {
                        operations.push({ type: 'pen_up' });
                        penIsDown = false;
                    }
                    
                    // 横移動（十字キー）
                    operations.push({ 
                        type: 'move', 
                        from: { ...currentPos }, 
                        to: { x, y },
                        duration: this.calculateMoveDuration(currentPos, { x, y }),
                        isDpadMove: true
                    });
                    
                    currentPos = { x, y };
                }
                
                // 黒いピクセルの場合
                if (isBlack) {
                    // ペンが上がっている場合は下げる
                    if (!penIsDown) {
                        operations.push({ type: 'pen_down' });
                        penIsDown = true;
                    }
                    
                    // ドットを描画
                    operations.push({ 
                        type: 'draw', 
                        position: { x, y }
                    });
                }
            }
        }
        
        // 最後にペンが下がっている場合は上げる
        if (penIsDown) {
            operations.push({ type: 'pen_up' });
        }
        
        return operations;
    }
    
    calculateMoveDuration(from, to) {
        // 実機と同じ速度計算: 1ピクセルあたり base_duration × 2 (移動+ニュートラル)
        // 実機の base_duration = max(50 / speed, 30) ms
        const baseDuration = Math.max(50 / this.paintingSpeed, 30); // ms
        const dx = Math.abs(to.x - from.x);
        const dy = Math.abs(to.y - from.y);
        const totalPixels = dx + dy; // マンハッタン距離（実機は直線移動ではなくX→Y順に移動）
        // 各ピクセル移動: base_duration(移動) + base_duration(ニュートラル)
        return (totalPixels * baseDuration * 2) / 1000; // 秒
    }
    
    calculateRealPaintingTime() {
        let totalTime = 0;
        
        // 速度調整係数（標準速度2.0を基準に）
        const speedFactor = 2.0 / this.paintingSpeed;
        
        for (const op of this.paintingOperations) {
            switch (op.type) {
                case 'pen_up':
                case 'pen_down':
                    // ボタン操作時間も速度に応じて調整
                    totalTime += 0.1 * speedFactor;
                    break;
                case 'move':
                    // 移動時間はすでにcalculateMoveDurationで調整済み
                    totalTime += op.duration;
                    break;
                case 'draw':
                    // ドット描画時間
                    totalTime += 1 / this.paintingSpeed;
                    break;
            }
        }
        
        return totalTime;
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

        // 事前計算された推定時間を保存（安定した初期推定のため）
        this.preCalculatedEstimate = this.calculateRealPaintingTime();
        
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
    }

    updatePaintingProgress(data) {
        if (!this.isPainting) return;

        const { current, total, x, y, dpad_operations, a_button_presses, is_paint } = data;

        // is_paintがtrueの場合のみキャンバスにドットを描画
        if (is_paint !== false) {
            const paintingCanvas = document.getElementById('paintingCanvas');
            const ctx = paintingCanvas.getContext('2d');
            ctx.fillStyle = '#000000';
            ctx.fillRect(x, y, 1, 1);
        }

        // カーソルを移動（常に更新）
        const cursor = document.getElementById('paintingCursor');
        const rect = document.getElementById('paintingCanvas').getBoundingClientRect();
        const scaleX = rect.width / 320;
        const scaleY = rect.height / 120;

        cursor.style.left = `${x * scaleX}px`;
        cursor.style.top = `${y * scaleY}px`;

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

            if (percentage < 10) {
                // 最初の10%: 事前計算された推定時間を使用（安定した初期推定）
                const progressRate = current / total;
                estimatedRemaining = this.preCalculatedEstimate * (1 - progressRate) - elapsed;
                estimatedRemaining = Math.max(0, estimatedRemaining); // 負の値を防ぐ
            } else if (percentage < 30) {
                // 10-30%: 事前計算とリアルタイム平均をブレンド
                const blendFactor = (percentage - 10) / 20; // 0.0 -> 1.0

                // 事前計算による推定
                const progressRate = current / total;
                const preCalcRemaining = Math.max(0, this.preCalculatedEstimate * (1 - progressRate) - elapsed);

                // リアルタイム平均による推定
                const averageTimePerDot = elapsed / current;
                const realTimeRemaining = remainingDots * averageTimePerDot;

                // ブレンド（徐々にリアルタイム推定に移行）
                estimatedRemaining = preCalcRemaining * (1 - blendFactor) + realTimeRemaining * blendFactor;
            } else {
                // 30%以降: リアルタイム平均を使用（十分なサンプルデータ）
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

        // モーダルを表示
        const modal = document.getElementById('paintPrepareModal');
        modal?.classList.remove('hidden');
        modal?.classList.add('flex');
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

        if (isDevicePainting && !isConnected) {
            alert('実機が接続されていません。');
            return;
        }

        this.isPainting = true;
        this.updateButtonStates();

        // 描画データを準備
        this.preparePaintingData();

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

                const response = await fetch(`/api/artworks/${this.currentArtworkId}/paint`, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({
                        press_ms: timing.pressMs,
                        release_ms: timing.releaseMs,
                        wait_ms: timing.waitMs,
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
        this.preparePaintingData();
        
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
        
        switch (operation.type) {
            case 'pen_up':
                this.penState = 'up';
                cursor.classList.add('pen-up');
                cursor.classList.remove('pen-down');
                this.currentDpadCount++;
                // ペンを上げる操作の時間（速度調整付き）
                const penUpTime = (100 * (2.0 / this.paintingSpeed)) / this.simulationMultiplier;
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
                // ペンを下げる操作の時間（速度調整付き）
                const penDownTime = (100 * (2.0 / this.paintingSpeed)) / this.simulationMultiplier;
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
                
                // 描画操作の時間
                setTimeout(() => {
                    this.currentOperationIndex++;
                    this.executeNextOperation();
                }, (1000 / this.paintingSpeed) / this.simulationMultiplier);
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
        const progress = this.currentDotIndex / this.paintingPath.length;
        const operationProgress = this.currentOperationIndex / this.paintingOperations.length;
        
        // 残り時間の計算
        let remainingTime = 0;
        const speedFactor = 2.0 / this.paintingSpeed; // 速度調整係数
        
        for (let i = this.currentOperationIndex; i < this.paintingOperations.length; i++) {
            const op = this.paintingOperations[i];
            switch (op.type) {
                case 'pen_up':
                case 'pen_down':
                    remainingTime += 0.1 * speedFactor;
                    break;
                case 'move':
                    remainingTime += op.duration; // すでに速度調整済み
                    break;
                case 'draw':
                    remainingTime += 1 / this.paintingSpeed;
                    break;
            }
        }
        remainingTime = remainingTime / this.simulationMultiplier;
        
        document.getElementById('paintedDots').textContent = this.currentDotIndex.toLocaleString();
        document.getElementById('elapsedTime').textContent = this.formatTime(elapsed);
        document.getElementById('estimatedTime').textContent = this.formatTime(remainingTime);
        
        // ボタン操作の進捗を更新
        document.getElementById('dpadOperations').textContent = `${this.currentDpadCount.toLocaleString()}/${this.dpadCount.toLocaleString()}回`;
        document.getElementById('aButtonPresses').textContent = `${this.currentAButtonCount.toLocaleString()}/${this.aButtonCount.toLocaleString()}回`;
        
        // プログレスバーも更新
        const progressPercent = Math.round(progress * 100);
        document.getElementById('progressFill').style.width = `${progressPercent}%`;
        document.getElementById('progressText').textContent = `描画中... ${progressPercent}%`;
        
        // 進捗スライダーも更新（シミュレーション時のみ）
        if (!this.isHardwareConnected) {
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

        // 描画進捗エリアと進捗バーを少し表示してから隠す
        setTimeout(() => {
            document.getElementById('paintingProgress').classList.add('hidden');
            const progressContainer = document.getElementById('progressContainer');
            if (progressContainer) {
                progressContainer.classList.add('hidden');
            }
        }, 3000);
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
        
        // 目標のドット数を計算
        const targetDotIndex = Math.floor(this.paintingPath.length * progress);
        
        // キャンバスを再描画
        const paintingCanvas = document.getElementById('paintingCanvas');
        const ctx = paintingCanvas.getContext('2d');
        
        // 背景を白で塗りつぶし
        ctx.fillStyle = '#FFFFFF';
        ctx.fillRect(0, 0, 320, 120);
        
        // 目標位置までのドットを描画
        ctx.fillStyle = '#000000';
        for (let i = 0; i < targetDotIndex && i < this.paintingPath.length; i++) {
            const dot = this.paintingPath[i];
            ctx.fillRect(dot.x, dot.y, 1, 1);
        }
        
        // 現在の状態を更新
        this.currentDotIndex = targetDotIndex;
        
        // 操作インデックスと操作数を計算
        let opIndex = 0;
        let dotCount = 0;
        let dpadCount = 0;
        let aButtonCount = 0;
        
        for (let i = 0; i < this.paintingOperations.length; i++) {
            const op = this.paintingOperations[i];
            
            if (op.type === 'draw') {
                if (dotCount >= targetDotIndex) {
                    opIndex = i;
                    break;
                }
                dotCount++;
                aButtonCount++;
            } else if (op.type === 'pen_up' || op.type === 'pen_down') {
                dpadCount++;
            } else if (op.type === 'move' && op.isDpadMove) {
                const distance = Math.abs(op.to.x - op.from.x) + Math.abs(op.to.y - op.from.y);
                dpadCount += distance;
            }
        }
        
        this.currentOperationIndex = opIndex;
        this.currentDpadCount = dpadCount;
        this.currentAButtonCount = aButtonCount;
        
        // カーソル位置を更新
        if (targetDotIndex < this.paintingPath.length) {
            const currentDot = this.paintingPath[targetDotIndex];
            const cursor = document.getElementById('paintingCursor');
            const rect = paintingCanvas.getBoundingClientRect();
            const scaleX = rect.width / 320;
            const scaleY = rect.height / 120;
            
            cursor.style.left = `${currentDot.x * scaleX}px`;
            cursor.style.top = `${currentDot.y * scaleY}px`;
            cursor.classList.remove('hidden');
            
            this.currentPosition = { x: currentDot.x, y: currentDot.y };
        }
        
        // 統計情報を更新
        this.updatePaintingStats();
        
        // スライダーの位置も更新
        const progressPercent = (progress * 100).toFixed(1);
        document.getElementById('progressSlider').value = progressPercent;
        document.getElementById('progressSliderValue').textContent = `${progressPercent}%`;
    }

    downloadResult() {
        if (!this.currentFile) return;

        this.addLog('結果をダウンロード中...', 'info');
        // ダウンロード機能の実装
        // 実際の実装では、変換結果をダウンロード
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

    openModal() {
        this.modal?.classList.remove('hidden');
        this.modal?.classList.add('flex');
        this.updateButtonStates();
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
        // 速度スライダーの値を取得（50〜200、デフォルト100）
        const speedValue = parseInt(this.speedInput?.value || 100);

        // speedValueは描画速度を表す（大きい = 速い）
        // speedMultiplierはタイミング倍率を表す（大きい = 遅い）
        // 反比例関係: speedMultiplier = 10000 / (speedValue * 100)
        const speedMultiplier = 10000 / (speedValue * 100);

        // 基準値（1.0xの場合、200ms/px）
        const basePressMs = 100;
        const baseReleaseMs = 60;
        const baseWaitMs = 40;

        // 速度倍率から各タイミング値を計算
        const pressMs = Math.round(basePressMs * speedMultiplier);
        const releaseMs = Math.round(baseReleaseMs * speedMultiplier);
        const waitMs = Math.round(baseWaitMs * speedMultiplier);

        // 速度表示を更新（スライダー値を100で割った値を表示）
        const displaySpeed = speedValue / 100.0;
        if (this.speedValue) {
            this.speedValue.textContent = `${displaySpeed.toFixed(1)}x`;
        }
    }

    // 計算されたタイミング値を取得するヘルパーメソッド
    getTimingValues() {
        const speedValue = parseInt(this.speedInput?.value || 100);

        // speedValueは描画速度を表す（大きい = 速い）
        // speedMultiplierはタイミング倍率を表す（大きい = 遅い）
        // 反比例関係: speedMultiplier = 10000 / (speedValue * 100)
        const speedMultiplier = 10000 / (speedValue * 100);

        // 基準値（1.0xの場合、200ms/px）
        const basePressMs = 100;
        const baseReleaseMs = 60;
        const baseWaitMs = 40;

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