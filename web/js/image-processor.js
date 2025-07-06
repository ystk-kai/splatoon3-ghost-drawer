// 画像処理用のクラス
class ImageProcessor {
    constructor() {
        this.targetWidth = 320;
        this.targetHeight = 120;
    }

    /**
     * 画像ファイルを処理（リサイズと2値化）
     * @param {File} file - 画像ファイル
     * @param {number} threshold - 2値化の閾値（0-255）
     * @param {Object} adjustments - 画像調整パラメータ
     * @param {Object} cropArea - 切り取り範囲
     * @returns {Promise<{canvas: HTMLCanvasElement, imageData: ImageData, binaryData: Array}>}
     */
    async processImage(file, threshold = 128, adjustments = {}, cropArea = null) {
        return new Promise((resolve, reject) => {
            const reader = new FileReader();
            
            reader.onload = (e) => {
                const img = new Image();
                
                img.onload = () => {
                    try {
                        // 切り取り範囲を決定
                        let sourceX = 0, sourceY = 0;
                        let sourceWidth = img.width;
                        let sourceHeight = img.height;
                        
                        if (cropArea && cropArea.originalImage) {
                            // 画像の実際のサイズと表示サイズの比率を計算
                            const scaleX = img.width / cropArea.originalImage.width;
                            const scaleY = img.height / cropArea.originalImage.height;
                            
                            sourceX = cropArea.x * scaleX;
                            sourceY = cropArea.y * scaleY;
                            sourceWidth = cropArea.width * scaleX;
                            sourceHeight = cropArea.height * scaleY;
                        }
                        
                        // リサイズ用のキャンバスを作成
                        const resizeCanvas = document.createElement('canvas');
                        resizeCanvas.width = this.targetWidth;
                        resizeCanvas.height = this.targetHeight;
                        const resizeCtx = resizeCanvas.getContext('2d');
                        
                        // アスペクト比を保持してリサイズ
                        const scale = Math.min(
                            this.targetWidth / sourceWidth,
                            this.targetHeight / sourceHeight
                        );
                        const scaledWidth = sourceWidth * scale;
                        const scaledHeight = sourceHeight * scale;
                        const offsetX = (this.targetWidth - scaledWidth) / 2;
                        const offsetY = (this.targetHeight - scaledHeight) / 2;
                        
                        // 背景を白で塗りつぶし
                        resizeCtx.fillStyle = '#FFFFFF';
                        resizeCtx.fillRect(0, 0, this.targetWidth, this.targetHeight);
                        
                        // 画像を描画（切り取り範囲を考慮）
                        resizeCtx.drawImage(
                            img,
                            sourceX,
                            sourceY,
                            sourceWidth,
                            sourceHeight,
                            offsetX,
                            offsetY,
                            scaledWidth,
                            scaledHeight
                        );
                        
                        // ピクセルデータを取得
                        let imageData = resizeCtx.getImageData(0, 0, this.targetWidth, this.targetHeight);
                        
                        // 画像調整を適用
                        if (adjustments.brightness !== undefined || adjustments.contrast !== undefined || 
                            adjustments.gamma !== undefined || adjustments.exposure !== undefined ||
                            adjustments.highlights !== undefined || adjustments.shadows !== undefined ||
                            adjustments.blackPoint !== undefined || adjustments.whitePoint !== undefined) {
                            imageData = this.applyAdjustments(imageData, adjustments);
                        }
                        
                        // 2値化処理
                        const binaryData = this.binarize(imageData, threshold, adjustments);
                        
                        // 調整済み画像のプレビューキャンバスを作成
                        const adjustedCanvas = document.createElement('canvas');
                        adjustedCanvas.width = this.targetWidth;
                        adjustedCanvas.height = this.targetHeight;
                        const adjustedCtx = adjustedCanvas.getContext('2d');
                        adjustedCtx.putImageData(imageData, 0, 0);
                        
                        // 表示用キャンバスを作成
                        const displayCanvas = document.createElement('canvas');
                        displayCanvas.width = this.targetWidth;
                        displayCanvas.height = this.targetHeight;
                        const displayCtx = displayCanvas.getContext('2d');
                        
                        // プレビューモードの場合は調整済み画像を表示、そうでなければ2値化画像を表示
                        if (adjustments.previewMode) {
                            // 調整済み画像をそのまま表示
                            displayCtx.drawImage(adjustedCanvas, 0, 0);
                        } else {
                            // 2値化データを描画
                            for (let y = 0; y < this.targetHeight; y++) {
                                for (let x = 0; x < this.targetWidth; x++) {
                                    const index = y * this.targetWidth + x;
                                    displayCtx.fillStyle = binaryData[index] ? '#000000' : '#FFFFFF';
                                    displayCtx.fillRect(x, y, 1, 1);
                                }
                            }
                        }
                        
                        resolve({
                            canvas: displayCanvas,
                            adjustedCanvas: adjustedCanvas,
                            imageData: imageData,
                            binaryData: binaryData,
                            width: this.targetWidth,
                            height: this.targetHeight
                        });
                    } catch (error) {
                        reject(error);
                    }
                };
                
                img.onerror = () => {
                    reject(new Error('画像の読み込みに失敗しました'));
                };
                
                img.src = e.target.result;
            };
            
            reader.onerror = () => {
                reject(new Error('ファイルの読み込みに失敗しました'));
            };
            
            reader.readAsDataURL(file);
        });
    }

    /**
     * ImageDataを2値化
     * @param {ImageData} imageData - 画像データ
     * @param {number} threshold - 閾値
     * @param {Object} adjustments - 調整パラメータ（適応的2値化用）
     * @returns {Array<boolean>} - 2値化データ（trueが黒、falseが白）
     */
    binarize(imageData, threshold, adjustments = {}) {
        const data = imageData.data;
        const width = imageData.width;
        const height = imageData.height;
        const binaryData = new Array(width * height);
        
        // 適応的2値化の場合
        if (adjustments.adaptiveThreshold) {
            const blockSize = adjustments.adaptiveBlockSize || 11;
            const constant = adjustments.adaptiveConstant || 2;
            const halfBlock = Math.floor(blockSize / 2);
            
            // まずグレースケール画像を作成
            const grayData = new Array(width * height);
            for (let i = 0; i < data.length; i += 4) {
                const pixelIndex = i / 4;
                grayData[pixelIndex] = (data[i] + data[i + 1] + data[i + 2]) / 3;
            }
            
            // 各ピクセルに対して局所的な閾値を計算
            for (let y = 0; y < height; y++) {
                for (let x = 0; x < width; x++) {
                    const index = y * width + x;
                    
                    // ブロック内の平均値を計算
                    let sum = 0;
                    let count = 0;
                    
                    for (let dy = -halfBlock; dy <= halfBlock; dy++) {
                        for (let dx = -halfBlock; dx <= halfBlock; dx++) {
                            const ny = y + dy;
                            const nx = x + dx;
                            
                            if (ny >= 0 && ny < height && nx >= 0 && nx < width) {
                                sum += grayData[ny * width + nx];
                                count++;
                            }
                        }
                    }
                    
                    const localMean = sum / count;
                    const localThreshold = localMean + constant;
                    
                    // 適応的閾値で2値化
                    binaryData[index] = grayData[index] < localThreshold;
                }
            }
        } else {
            // 通常の2値化
            for (let i = 0; i < data.length; i += 4) {
                // RGBの平均値を計算（グレースケール化）
                const gray = (data[i] + data[i + 1] + data[i + 2]) / 3;
                // 閾値で2値化
                const pixelIndex = i / 4;
                binaryData[pixelIndex] = gray < threshold;
            }
        }
        
        return binaryData;
    }

    /**
     * 2値化データをSplatoon3用のドットデータに変換
     * @param {Array<boolean>} binaryData - 2値化データ
     * @param {number} width - 幅
     * @param {number} height - 高さ
     * @returns {Array} - ドットデータの配列
     */
    convertToDotData(binaryData, width, height) {
        const dots = [];
        
        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const index = y * width + x;
                if (binaryData[index]) {
                    dots.push({
                        x: x,
                        y: y,
                        color: '#000000'
                    });
                }
            }
        }
        
        return dots;
    }

    /**
     * 画像調整を適用
     * @param {ImageData} imageData - 画像データ
     * @param {Object} adjustments - 調整パラメータ
     * @returns {ImageData} - 調整後の画像データ
     */
    applyAdjustments(imageData, adjustments) {
        const data = imageData.data;
        const brightness = adjustments.brightness || 0;
        const contrast = adjustments.contrast || 0;
        const gamma = adjustments.gamma || 1.0;
        const exposure = adjustments.exposure || 0.0;
        const highlights = adjustments.highlights || 0;
        const shadows = adjustments.shadows || 0;
        const blackPoint = adjustments.blackPoint || 0;
        const whitePoint = adjustments.whitePoint || 255;
        
        console.log('Applying adjustments:', adjustments);
        
        // コントラストファクターを計算
        const contrastFactor = (259 * (contrast + 100)) / (100 * (259 - contrast));
        
        // 露出ファクターを計算
        const exposureFactor = Math.pow(2, exposure);
        
        // ガンマ補正用のルックアップテーブルを作成
        const gammaCorrection = new Array(256);
        for (let i = 0; i < 256; i++) {
            gammaCorrection[i] = Math.pow(i / 255, 1 / gamma) * 255;
        }
        
        // 各ピクセルに調整を適用
        for (let i = 0; i < data.length; i += 4) {
            let r = data[i];
            let g = data[i + 1];
            let b = data[i + 2];
            
            // 1. 露出補正
            r = r * exposureFactor;
            g = g * exposureFactor;
            b = b * exposureFactor;
            
            // 2. ブラックポイント・ホワイトポイント調整
            const range = whitePoint - blackPoint;
            if (range > 0) {
                r = ((r - blackPoint) * 255 / range);
                g = ((g - blackPoint) * 255 / range);
                b = ((b - blackPoint) * 255 / range);
            }
            
            // 3. ガンマ補正（ルックアップテーブルを使用）
            r = Math.max(0, Math.min(255, r));
            g = Math.max(0, Math.min(255, g));
            b = Math.max(0, Math.min(255, b));
            r = gammaCorrection[Math.round(r)];
            g = gammaCorrection[Math.round(g)];
            b = gammaCorrection[Math.round(b)];
            
            // 4. 明度調整
            r = r + brightness;
            g = g + brightness;
            b = b + brightness;
            
            // 5. コントラスト調整
            r = contrastFactor * (r - 128) + 128;
            g = contrastFactor * (g - 128) + 128;
            b = contrastFactor * (b - 128) + 128;
            
            // 6. ハイライト・シャドウ調整
            const luminance = (r * 0.299 + g * 0.587 + b * 0.114) / 255;
            
            if (highlights !== 0 && luminance > 0.5) {
                const highlightFactor = 1 + (highlights / 100) * ((luminance - 0.5) * 2);
                r = r * highlightFactor;
                g = g * highlightFactor;
                b = b * highlightFactor;
            }
            
            if (shadows !== 0 && luminance < 0.5) {
                const shadowFactor = 1 + (shadows / 100) * ((0.5 - luminance) * 2);
                r = r * shadowFactor;
                g = g * shadowFactor;
                b = b * shadowFactor;
            }
            
            // 最終的な値のクランプ
            data[i] = Math.max(0, Math.min(255, r));
            data[i + 1] = Math.max(0, Math.min(255, g));
            data[i + 2] = Math.max(0, Math.min(255, b));
            // アルファ値は変更しない
        }
        
        return imageData;
    }

    /**
     * プレビュー用のキャンバスを拡大表示
     * @param {HTMLCanvasElement} sourceCanvas - ソースキャンバス
     * @param {number} scale - 拡大率
     * @returns {HTMLCanvasElement} - 拡大されたキャンバス
     */
    createScaledPreview(sourceCanvas, scale = 2) {
        const previewCanvas = document.createElement('canvas');
        previewCanvas.width = sourceCanvas.width * scale;
        previewCanvas.height = sourceCanvas.height * scale;
        const ctx = previewCanvas.getContext('2d');
        
        // 最近傍補間で拡大（ドット感を保持）
        ctx.imageSmoothingEnabled = false;
        ctx.drawImage(
            sourceCanvas,
            0, 0, sourceCanvas.width, sourceCanvas.height,
            0, 0, previewCanvas.width, previewCanvas.height
        );
        
        return previewCanvas;
    }
}

// グローバルに公開
window.ImageProcessor = ImageProcessor;