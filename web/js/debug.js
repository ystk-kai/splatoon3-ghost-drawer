/**
 * デバッグとログ管理機能
 * バックエンドからのログを表示し、デバッグ情報を管理
 */

class DebugManager {
    constructor() {
        this.logs = [];
        this.maxLogs = 1000;
        this.logLevels = ['DEBUG', 'INFO', 'WARN', 'ERROR'];
        this.currentFilter = 'ALL';
        this.isAutoScroll = true;
        this.isConnected = false;
        
        this.initializeWebSocket();
        this.setupEventListeners();
    }

    /**
     * WebSocketでバックエンドのログを受信
     */
    initializeWebSocket() {
        try {
            // WebSocketでリアルタイムログを受信
            this.ws = new WebSocket(`ws://${window.location.host}/ws/logs`);
            
            this.ws.onopen = () => {
                this.isConnected = true;
                this.addLog('INFO', 'ログストリーミングが開始されました', 'WebSocket');
                this.updateConnectionStatus(true);
            };

            this.ws.onmessage = (event) => {
                try {
                    const logData = JSON.parse(event.data);

                    if (logData.type === 'progress') {
                        if (window.ghostDrawerApp && typeof window.ghostDrawerApp.updatePaintingProgress === 'function') {
                            window.ghostDrawerApp.updatePaintingProgress(logData);
                        }
                    } else if (logData.type === 'calibration_complete') {
                        // キャリブレーション完了通知を処理
                        if (window.calibrationManager) {
                            window.calibrationManager.handleCalibrationComplete(logData);
                        }
                        // ログとしても表示
                        this.addLogFromBackend({
                            type: 'log',
                            timestamp: logData.timestamp,
                            level: logData.status === 'success' ? 'INFO' : logData.status === 'error' ? 'ERROR' : 'WARN',
                            message: logData.message,
                            target: 'calibration'
                        });
                    } else {
                        this.addLogFromBackend(logData);
                    }
                } catch (e) {
                    console.error('ログデータの解析に失敗:', e);
                }
            };

            this.ws.onclose = () => {
                this.isConnected = false;
                this.addLog('WARN', 'ログストリーミングが切断されました', 'WebSocket');
                this.updateConnectionStatus(false);
                
                // 再接続を試行
                setTimeout(() => this.initializeWebSocket(), 5000);
            };

            this.ws.onerror = (error) => {
                console.error('WebSocketエラー:', error);
                this.addLog('ERROR', 'WebSocket接続エラーが発生しました', 'WebSocket');
            };
        } catch (e) {
            console.error('WebSocket初期化エラー:', e);
            this.addLog('ERROR', 'WebSocket初期化に失敗しました', 'Debug');
        }
    }

    /**
     * イベントリスナーの設定
     */
    setupEventListeners() {
        // ログレベルフィルター
        const levelFilter = document.getElementById('log-level-filter');
        if (levelFilter) {
            levelFilter.addEventListener('change', (e) => {
                this.currentFilter = e.target.value;
                this.refreshLogDisplay();
            });
        }

        // ログクリアボタン
        const clearBtn = document.getElementById('clear-log');
        if (clearBtn) {
            clearBtn.addEventListener('click', () => this.clearLogs());
        }

        // ログダウンロードボタン
        const downloadBtn = document.getElementById('download-log');
        if (downloadBtn) {
            downloadBtn.addEventListener('click', () => this.downloadLogs());
        }

        // 自動スクロール切り替え
        const autoScrollToggle = document.getElementById('auto-scroll-toggle');
        if (autoScrollToggle) {
            autoScrollToggle.addEventListener('change', (e) => {
                this.isAutoScroll = e.target.checked;
            });
        }

        // デバッグパネルの切り替え
        const debugToggle = document.getElementById('debug-panel-toggle');
        if (debugToggle) {
            debugToggle.addEventListener('click', () => this.toggleDebugPanel());
        }
    }

    /**
     * バックエンドからのログを追加
     */
    addLogFromBackend(logData) {
        const {
            timestamp,
            level,
            message,
            target,
            fields = {},
            span = null
        } = logData;

        this.addLog(level, message, target, {
            timestamp: new Date(timestamp),
            fields,
            span,
            source: 'backend'
        });
    }

    /**
     * ログエントリを追加
     */
    addLog(level, message, source = 'frontend', options = {}) {
        const logEntry = {
            id: Date.now() + Math.random(),
            timestamp: options.timestamp || new Date(),
            level: level.toUpperCase(),
            message,
            source,
            fields: options.fields || {},
            span: options.span || null,
            origin: options.source || 'frontend'
        };

        this.logs.push(logEntry);

        // 最大ログ数を超えたら古いものを削除
        if (this.logs.length > this.maxLogs) {
            this.logs.shift();
        }

        // フィルターに合致する場合のみ表示
        if (this.shouldShowLog(logEntry)) {
            this.displayLog(logEntry);
        }

        // 統計を更新
        this.updateLogStatistics();
    }

    /**
     * ログをフィルターすべきかチェック
     */
    shouldShowLog(logEntry) {
        if (this.currentFilter === 'ALL') {
            return true;
        }
        return logEntry.level === this.currentFilter;
    }

    /**
     * ログエントリを画面に表示
     */
    displayLog(logEntry) {
        const logContainer = document.getElementById('log-content');
        if (!logContainer) return;

        const logElement = this.createLogElement(logEntry);
        logContainer.appendChild(logElement);

        // 自動スクロール
        if (this.isAutoScroll) {
            logContainer.scrollTop = logContainer.scrollHeight;
        }
    }

    /**
     * ログエントリのHTML要素を作成
     */
    createLogElement(logEntry) {
        const logDiv = document.createElement('div');
        logDiv.className = `log-entry log-${logEntry.level.toLowerCase()}`;
        logDiv.dataset.logId = logEntry.id;

        const timeStr = logEntry.timestamp.toLocaleString('ja-JP', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit',
            fractionalSecondDigits: 3
        });

        let fieldsStr = '';
        if (Object.keys(logEntry.fields).length > 0) {
            fieldsStr = Object.entries(logEntry.fields)
                .map(([key, value]) => `${key}=${value}`)
                .join(' ');
        }

        logDiv.innerHTML = `
            <span class="log-time">${timeStr}</span>
            <span class="log-level ${logEntry.level.toLowerCase()}">${logEntry.level}</span>
            <span class="log-source">${logEntry.source}</span>
            <span class="log-message">${this.escapeHtml(logEntry.message)}</span>
            ${fieldsStr ? `<span class="log-fields">${this.escapeHtml(fieldsStr)}</span>` : ''}
            ${logEntry.span ? `<span class="log-span">[${logEntry.span.name}]</span>` : ''}
        `;

        return logDiv;
    }

    /**
     * HTMLエスケープ
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    /**
     * ログ表示を更新
     */
    refreshLogDisplay() {
        const logContainer = document.getElementById('log-content');
        if (!logContainer) return;

        // 既存のログをクリア
        logContainer.innerHTML = '';

        // フィルターされたログを表示
        this.logs
            .filter(log => this.shouldShowLog(log))
            .forEach(log => this.displayLog(log));
    }

    /**
     * ログをクリア
     */
    clearLogs() {
        this.logs = [];
        const logContainer = document.getElementById('log-content');
        if (logContainer) {
            logContainer.innerHTML = '';
        }
        this.updateLogStatistics();
        this.addLog('INFO', 'ログがクリアされました', 'Debug');
    }

    /**
     * ログをダウンロード
     */
    downloadLogs() {
        const logData = this.logs.map(log => ({
            timestamp: log.timestamp.toISOString(),
            level: log.level,
            source: log.source,
            message: log.message,
            fields: log.fields,
            span: log.span
        }));

        const blob = new Blob([JSON.stringify(logData, null, 2)], {
            type: 'application/json'
        });

        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `splatoon3-logs-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        this.addLog('INFO', 'ログファイルをダウンロードしました', 'Debug');
    }

    /**
     * ログ統計を更新
     */
    updateLogStatistics() {
        const stats = this.logLevels.reduce((acc, level) => {
            acc[level] = this.logs.filter(log => log.level === level).length;
            return acc;
        }, {});

        // 統計表示を更新
        const statsContainer = document.getElementById('log-statistics');
        if (statsContainer) {
            statsContainer.innerHTML = `
                <div class="stat-item">
                    <span class="stat-label">総数:</span>
                    <span class="stat-value">${this.logs.length}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">エラー:</span>
                    <span class="stat-value error">${stats.ERROR || 0}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">警告:</span>
                    <span class="stat-value warn">${stats.WARN || 0}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">情報:</span>
                    <span class="stat-value info">${stats.INFO || 0}</span>
                </div>
                <div class="stat-item">
                    <span class="stat-label">デバッグ:</span>
                    <span class="stat-value debug">${stats.DEBUG || 0}</span>
                </div>
            `;
        }
    }

    /**
     * 接続状態を更新
     */
    updateConnectionStatus(isConnected) {
        const statusElement = document.getElementById('log-connection-status');
        if (statusElement) {
            statusElement.className = `connection-status ${isConnected ? 'connected' : 'disconnected'}`;
            statusElement.textContent = isConnected ? '接続中' : '切断中';
        }
    }

    /**
     * デバッグパネルの表示切り替え
     */
    toggleDebugPanel() {
        const panel = document.getElementById('debug-panel');
        if (panel) {
            panel.style.display = panel.style.display === 'none' ? 'block' : 'none';
        }
    }

    /**
     * パフォーマンス情報を取得
     */
    getPerformanceInfo() {
        if (performance && performance.memory) {
            return {
                usedJSHeapSize: performance.memory.usedJSHeapSize,
                totalJSHeapSize: performance.memory.totalJSHeapSize,
                jsHeapSizeLimit: performance.memory.jsHeapSizeLimit
            };
        }
        return null;
    }

    /**
     * システム情報をログに出力
     */
    logSystemInfo() {
        const info = {
            userAgent: navigator.userAgent,
            language: navigator.language,
            platform: navigator.platform,
            cookieEnabled: navigator.cookieEnabled,
            onLine: navigator.onLine,
            screen: {
                width: screen.width,
                height: screen.height,
                colorDepth: screen.colorDepth
            },
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight
            }
        };

        const perfInfo = this.getPerformanceInfo();
        if (perfInfo) {
            info.memory = perfInfo;
        }

        this.addLog('INFO', 'システム情報', 'Debug', {
            fields: info
        });
    }

    /**
     * エラーを記録
     */
    logError(error, context = '') {
        const errorInfo = {
            name: error.name,
            message: error.message,
            stack: error.stack,
            context
        };

        this.addLog('ERROR', `${context ? context + ': ' : ''}${error.message}`, 'Error', {
            fields: errorInfo
        });
    }

    /**
     * デバッグ情報を開始時に表示
     */
    initialize() {
        this.logSystemInfo();
        this.addLog('INFO', 'デバッグマネージャーが初期化されました', 'Debug');
    }
}

// グローバルエラーハンドラー
window.addEventListener('error', (event) => {
    if (window.debugManager) {
        window.debugManager.logError(event.error, 'Global Error');
    }
});

window.addEventListener('unhandledrejection', (event) => {
    if (window.debugManager) {
        window.debugManager.logError(new Error(event.reason), 'Unhandled Promise Rejection');
    }
});

// デバッグマネージャーのインスタンスを作成
window.debugManager = new DebugManager();

// ページ読み込み後に初期化
document.addEventListener('DOMContentLoaded', () => {
    window.debugManager.initialize();
}); 