//! 共有値オブジェクト
//!
//! 複数の集約で使用される共通の値オブジェクトを定義

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// エンティティの基本トレイト
pub trait Entity {
    /// エンティティのID型
    type Id;

    /// エンティティのIDを取得
    fn id(&self) -> &Self::Id;
}

/// 2次元座標を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Coordinates {
    pub x: u16,
    pub y: u16,
}

impl Coordinates {
    /// 新しい座標を作成
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    /// 原点座標 (0, 0)
    pub fn origin() -> Self {
        Self::new(0, 0)
    }

    /// 座標が指定された範囲内にあるかチェック
    pub fn is_within_bounds(&self, width: u16, height: u16) -> bool {
        self.x < width && self.y < height
    }

    /// 他の座標との距離を計算
    pub fn distance_to(&self, other: &Coordinates) -> f64 {
        let dx = (self.x as f64) - (other.x as f64);
        let dy = (self.y as f64) - (other.y as f64);
        (dx * dx + dy * dy).sqrt()
    }

    /// マンハッタン距離を計算
    pub fn manhattan_distance_to(&self, other: &Coordinates) -> u32 {
        let dx = self.x.abs_diff(other.x);
        let dy = self.y.abs_diff(other.y);
        (dx as u32) + (dy as u32)
    }

    /// 座標を指定された方向に移動
    pub fn move_by(&self, dx: i16, dy: i16) -> Option<Coordinates> {
        let new_x = (self.x as i32) + (dx as i32);
        let new_y = (self.y as i32) + (dy as i32);

        if new_x >= 0 && new_y >= 0 && new_x <= u16::MAX as i32 && new_y <= u16::MAX as i32 {
            Some(Coordinates::new(new_x as u16, new_y as u16))
        } else {
            None
        }
    }

    /// 座標の配列から境界ボックスを計算
    pub fn bounding_box(coords: &[Coordinates]) -> Option<(Coordinates, Coordinates)> {
        if coords.is_empty() {
            return None;
        }

        let min_x = coords.iter().map(|c| c.x).min().unwrap();
        let min_y = coords.iter().map(|c| c.y).min().unwrap();
        let max_x = coords.iter().map(|c| c.x).max().unwrap();
        let max_y = coords.iter().map(|c| c.y).max().unwrap();

        Some((
            Coordinates::new(min_x, min_y),
            Coordinates::new(max_x, max_y),
        ))
    }
}

impl fmt::Display for Coordinates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl FromStr for Coordinates {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim_start_matches('(').trim_end_matches(')');
        let parts: Vec<&str> = s.split(',').collect();

        if parts.len() != 2 {
            return Err("Invalid format. Expected (x, y)".to_string());
        }

        let x = parts[0]
            .trim()
            .parse::<u16>()
            .map_err(|_| "Invalid x coordinate".to_string())?;
        let y = parts[1]
            .trim()
            .parse::<u16>()
            .map_err(|_| "Invalid y coordinate".to_string())?;

        Ok(Coordinates::new(x, y))
    }
}

/// 色の値を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// 新しい色を作成
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// RGB値から作成（アルファ値は255）
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// HSV値から作成
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        let r = ((r_prime + m) * 255.0) as u8;
        let g = ((g_prime + m) * 255.0) as u8;
        let b = ((b_prime + m) * 255.0) as u8;

        Self::from_rgb(r, g, b)
    }

    /// 16進数文字列から作成
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| "Invalid red component".to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| "Invalid green component".to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| "Invalid blue component".to_string())?;
                Ok(Self::from_rgb(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16)
                    .map_err(|_| "Invalid red component".to_string())?;
                let g = u8::from_str_radix(&hex[2..4], 16)
                    .map_err(|_| "Invalid green component".to_string())?;
                let b = u8::from_str_radix(&hex[4..6], 16)
                    .map_err(|_| "Invalid blue component".to_string())?;
                let a = u8::from_str_radix(&hex[6..8], 16)
                    .map_err(|_| "Invalid alpha component".to_string())?;
                Ok(Self::new(r, g, b, a))
            }
            _ => Err("Invalid hex color format".to_string()),
        }
    }

    /// 16進数文字列として出力
    pub fn to_hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }

    /// 黒色
    pub fn black() -> Self {
        Self::from_rgb(0, 0, 0)
    }

    /// 白色
    pub fn white() -> Self {
        Self::from_rgb(255, 255, 255)
    }

    /// 赤色
    pub fn red() -> Self {
        Self::from_rgb(255, 0, 0)
    }

    /// 緑色
    pub fn green() -> Self {
        Self::from_rgb(0, 255, 0)
    }

    /// 青色
    pub fn blue() -> Self {
        Self::from_rgb(0, 0, 255)
    }

    /// 透明
    pub fn transparent() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// グレースケール値に変換
    pub fn to_grayscale(&self) -> u8 {
        // 標準的なグレースケール変換式（ITU-R BT.709）
        (0.2126 * self.r as f64 + 0.7152 * self.g as f64 + 0.0722 * self.b as f64) as u8
    }

    /// 2値化（閾値による白黒変換）
    pub fn to_binary(&self, threshold: u8) -> bool {
        self.to_grayscale() >= threshold
    }

    /// 輝度を計算
    pub fn luminance(&self) -> f64 {
        let r = self.r as f64 / 255.0;
        let g = self.g as f64 / 255.0;
        let b = self.b as f64 / 255.0;

        let r_lin = if r <= 0.03928 {
            r / 12.92
        } else {
            ((r + 0.055) / 1.055).powf(2.4)
        };
        let g_lin = if g <= 0.03928 {
            g / 12.92
        } else {
            ((g + 0.055) / 1.055).powf(2.4)
        };
        let b_lin = if b <= 0.03928 {
            b / 12.92
        } else {
            ((b + 0.055) / 1.055).powf(2.4)
        };

        0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin
    }

    /// 色の混合
    pub fn blend(&self, other: &Color, ratio: f64) -> Color {
        let ratio = ratio.clamp(0.0, 1.0);
        let inv_ratio = 1.0 - ratio;

        Color::new(
            (self.r as f64 * inv_ratio + other.r as f64 * ratio) as u8,
            (self.g as f64 * inv_ratio + other.g as f64 * ratio) as u8,
            (self.b as f64 * inv_ratio + other.b as f64 * ratio) as u8,
            (self.a as f64 * inv_ratio + other.a as f64 * ratio) as u8,
        )
    }

    /// 色の反転
    pub fn invert(&self) -> Color {
        Color::new(255 - self.r, 255 - self.g, 255 - self.b, self.a)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.a == 255 {
            write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
        } else {
            write!(f, "rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
        }
    }
}

impl FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.starts_with('#') {
            return Self::from_hex(s);
        }

        if s.starts_with("rgb(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();

            if parts.len() != 3 {
                return Err("Invalid RGB format".to_string());
            }

            let r = parts[0]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid red component".to_string())?;
            let g = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid green component".to_string())?;
            let b = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid blue component".to_string())?;

            return Ok(Self::from_rgb(r, g, b));
        }

        if s.starts_with("rgba(") && s.ends_with(')') {
            let inner = &s[5..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();

            if parts.len() != 4 {
                return Err("Invalid RGBA format".to_string());
            }

            let r = parts[0]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid red component".to_string())?;
            let g = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid green component".to_string())?;
            let b = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid blue component".to_string())?;
            let a = parts[3]
                .trim()
                .parse::<u8>()
                .map_err(|_| "Invalid alpha component".to_string())?;

            return Ok(Self::new(r, g, b, a));
        }

        // 名前付き色の対応
        match s.to_lowercase().as_str() {
            "black" => Ok(Self::black()),
            "white" => Ok(Self::white()),
            "red" => Ok(Self::red()),
            "green" => Ok(Self::green()),
            "blue" => Ok(Self::blue()),
            "transparent" => Ok(Self::transparent()),
            _ => Err(format!("Unknown color name: {s}")),
        }
    }
}

/// タイムスタンプを表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp {
    pub epoch_millis: u64,
}

impl Timestamp {
    /// 現在時刻のタイムスタンプを作成
    pub fn now() -> Self {
        Self {
            epoch_millis: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        }
    }

    /// エポック時刻（ミリ秒）から作成
    pub fn from_millis(millis: u64) -> Self {
        Self {
            epoch_millis: millis,
        }
    }

    /// エポック時刻（秒）から作成
    pub fn from_secs(secs: u64) -> Self {
        Self {
            epoch_millis: secs * 1000,
        }
    }

    /// 秒として取得
    pub fn as_secs(&self) -> u64 {
        self.epoch_millis / 1000
    }

    /// 経過時間を計算（ミリ秒）
    pub fn elapsed_millis(&self) -> u64 {
        Self::now().epoch_millis.saturating_sub(self.epoch_millis)
    }

    /// 経過時間を計算（秒）
    pub fn elapsed_secs(&self) -> u64 {
        self.elapsed_millis() / 1000
    }

    /// 指定した時間後のタイムスタンプを作成
    pub fn add_millis(&self, millis: u64) -> Self {
        Self {
            epoch_millis: self.epoch_millis.saturating_add(millis),
        }
    }

    /// 指定した時間後のタイムスタンプを作成
    pub fn add_secs(&self, secs: u64) -> Self {
        self.add_millis(secs * 1000)
    }

    /// ISO 8601形式の文字列として出力
    pub fn to_iso8601(&self) -> String {
        let secs = self.epoch_millis / 1000;
        let millis = self.epoch_millis % 1000;

        let _datetime = std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs);

        // 簡易的なISO 8601形式（実際の実装では chrono クレートを使用することを推奨）
        format!("{secs}.{millis:03}Z")
    }

    /// 人間が読みやすい形式で出力
    pub fn to_human_readable(&self) -> String {
        let elapsed = self.elapsed_secs();

        if elapsed < 60 {
            format!("{elapsed}秒前")
        } else if elapsed < 3600 {
            format!("{}分前", elapsed / 60)
        } else if elapsed < 86400 {
            format!("{}時間前", elapsed / 3600)
        } else {
            format!("{}日前", elapsed / 86400)
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.epoch_millis)
    }
}

impl FromStr for Timestamp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let millis = s
            .parse::<u64>()
            .map_err(|_| "Invalid timestamp format".to_string())?;
        Ok(Self::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinates() {
        let coord = Coordinates::new(10, 20);
        assert_eq!(coord.x, 10);
        assert_eq!(coord.y, 20);
        assert!(coord.is_within_bounds(100, 100));
        assert!(!coord.is_within_bounds(5, 5));

        let other = Coordinates::new(13, 24);
        assert_eq!(coord.manhattan_distance_to(&other), 7);

        assert_eq!(coord.to_string(), "(10, 20)");
        assert_eq!("(10, 20)".parse::<Coordinates>().unwrap(), coord);
    }

    #[test]
    fn test_color() {
        let color = Color::from_rgb(128, 128, 128);
        assert_eq!(color.to_grayscale(), 128);
        assert!(color.to_binary(100));
        assert!(!color.to_binary(200));

        let hex_color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(hex_color, Color::red());
        assert_eq!(hex_color.to_hex(), "#FF0000");

        let blended = Color::black().blend(&Color::white(), 0.5);
        assert_eq!(blended.r, 127);
        assert_eq!(blended.g, 127);
        assert_eq!(blended.b, 127);
    }

    #[test]
    fn test_timestamp() {
        let ts1 = Timestamp::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let ts2 = Timestamp::now();
        assert!(ts2.epoch_millis > ts1.epoch_millis);

        let ts_from_secs = Timestamp::from_secs(1609459200); // 2021-01-01 00:00:00 UTC
        assert_eq!(ts_from_secs.as_secs(), 1609459200);

        let future = ts1.add_secs(3600);
        assert_eq!(future.epoch_millis, ts1.epoch_millis + 3600000);
    }
}
