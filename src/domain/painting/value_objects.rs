use crate::domain::controller::{Button, DPad};
use crate::domain::shared::value_objects::Coordinates;
use serde::{Deserialize, Serialize};

/// Splatoon3の描画モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawingMode {
    /// ピクセルペン（ドット単位）
    PixelPen,
    /// 通常ペン
    NormalPen,
    /// 太いペン
    ThickPen,
    /// 消しゴム
    Eraser,
}

impl DrawingMode {
    /// 描画モードを選択するためのボタン
    pub fn select_button(&self) -> Button {
        match self {
            DrawingMode::PixelPen => Button::L,
            DrawingMode::NormalPen => Button::R,
            DrawingMode::ThickPen => Button::ZL,
            DrawingMode::Eraser => Button::ZR,
        }
    }
}

/// 描画キャンバスの設定
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingCanvasConfig {
    /// キャンバスの幅（ピクセル）
    pub width: u16,
    /// キャンバスの高さ（ピクセル）
    pub height: u16,
    /// カーソルの移動速度（ミリ秒/ピクセル）
    pub cursor_speed_ms: u32,
    /// ドット描画の待機時間（ミリ秒）
    pub dot_draw_delay_ms: u32,
    /// 行の折り返し時の追加待機時間（ミリ秒）
    pub line_wrap_delay_ms: u32,
    /// 描画モード
    pub drawing_mode: DrawingMode,
}

impl Default for DrawingCanvasConfig {
    fn default() -> Self {
        Self {
            width: 320,
            height: 120,
            cursor_speed_ms: 50,      // 1ピクセル移動に50ms
            dot_draw_delay_ms: 100,   // ドット描画に100ms
            line_wrap_delay_ms: 200,  // 行折り返しに追加200ms
            drawing_mode: DrawingMode::PixelPen,
        }
    }
}

/// カーソルの移動方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CursorDirection {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl CursorDirection {
    /// 方向をDPadの値に変換
    pub fn to_dpad(&self) -> DPad {
        match self {
            CursorDirection::Up => DPad::UP,
            CursorDirection::Down => DPad::DOWN,
            CursorDirection::Left => DPad::LEFT,
            CursorDirection::Right => DPad::RIGHT,
            CursorDirection::UpLeft => DPad::UP_LEFT,
            CursorDirection::UpRight => DPad::UP_RIGHT,
            CursorDirection::DownLeft => DPad::DOWN_LEFT,
            CursorDirection::DownRight => DPad::DOWN_RIGHT,
        }
    }

    /// 2つの座標間の方向を計算
    pub fn from_coordinates(from: &Coordinates, to: &Coordinates) -> Option<Self> {
        let dx = to.x as i32 - from.x as i32;
        let dy = to.y as i32 - from.y as i32;

        match (dx.signum(), dy.signum()) {
            (0, -1) => Some(CursorDirection::Up),
            (0, 1) => Some(CursorDirection::Down),
            (-1, 0) => Some(CursorDirection::Left),
            (1, 0) => Some(CursorDirection::Right),
            (-1, -1) => Some(CursorDirection::UpLeft),
            (1, -1) => Some(CursorDirection::UpRight),
            (-1, 1) => Some(CursorDirection::DownLeft),
            (1, 1) => Some(CursorDirection::DownRight),
            _ => None,
        }
    }
}

/// 描画パス（効率的な描画順序）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrawingPath {
    /// 描画する座標のリスト（順序付き）
    pub coordinates: Vec<Coordinates>,
    /// 総移動距離
    pub total_distance: u32,
    /// 推定所要時間（ミリ秒）
    pub estimated_time_ms: u32,
}

impl DrawingPath {
    pub fn new(coordinates: Vec<Coordinates>) -> Self {
        let total_distance = Self::calculate_total_distance(&coordinates);
        Self {
            coordinates,
            total_distance,
            estimated_time_ms: 0,
        }
    }

    fn calculate_total_distance(coordinates: &[Coordinates]) -> u32 {
        if coordinates.len() < 2 {
            return 0;
        }

        coordinates.windows(2)
            .map(|pair| pair[0].manhattan_distance_to(&pair[1]))
            .sum()
    }

    /// 設定に基づいて推定時間を計算
    pub fn calculate_estimated_time(&mut self, config: &DrawingCanvasConfig) {
        if self.coordinates.is_empty() {
            self.estimated_time_ms = 0;
            return;
        }

        let mut time_ms = 0u32;
        let mut prev_coord: Option<&Coordinates> = None;

        for coord in &self.coordinates {
            if let Some(prev) = prev_coord {
                // 移動時間を計算
                let distance = prev.manhattan_distance_to(coord);
                time_ms += distance * config.cursor_speed_ms;

                // 行が変わった場合は追加の待機時間
                if prev.y != coord.y {
                    time_ms += config.line_wrap_delay_ms;
                }
            }

            // ドット描画時間
            time_ms += config.dot_draw_delay_ms;
            prev_coord = Some(coord);
        }

        self.estimated_time_ms = time_ms;
    }
}

/// 描画戦略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrawingStrategy {
    /// 左から右、上から下へのラスタースキャン
    RasterScan,
    /// ジグザグパターン（行ごとに方向を反転）
    ZigZag,
    /// 最近傍探索（移動距離最小化）
    NearestNeighbor,
    /// スパイラル（渦巻き）パターン
    Spiral,
}