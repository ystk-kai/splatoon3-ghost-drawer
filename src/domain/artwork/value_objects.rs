//! アートワーク集約の値オブジェクト
//! 
//! 画像形式、解像度、変換パラメータなどの値オブジェクトを定義

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// 画像フォーマットを表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Bmp,
    Webp,
    Svg,
    Ico,
    Tiff,
    Raw,
}

impl ImageFormat {
    /// 拡張子から画像フォーマットを推定
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(Self::Png),
            "jpg" | "jpeg" => Some(Self::Jpeg),
            "gif" => Some(Self::Gif),
            "bmp" => Some(Self::Bmp),
            "webp" => Some(Self::Webp),
            "svg" => Some(Self::Svg),
            "ico" => Some(Self::Ico),
            "tiff" | "tif" => Some(Self::Tiff),
            "raw" => Some(Self::Raw),
            _ => None,
        }
    }

    /// ファイル名から画像フォーマットを推定
    pub fn from_filename(filename: &str) -> Option<Self> {
        filename
            .rsplit('.')
            .next()
            .and_then(Self::from_extension)
    }

    /// MIMEタイプから画像フォーマットを推定
    pub fn from_mime_type(mime: &str) -> Option<Self> {
        match mime {
            "image/png" => Some(Self::Png),
            "image/jpeg" => Some(Self::Jpeg),
            "image/gif" => Some(Self::Gif),
            "image/bmp" => Some(Self::Bmp),
            "image/webp" => Some(Self::Webp),
            "image/svg+xml" => Some(Self::Svg),
            "image/x-icon" | "image/vnd.microsoft.icon" => Some(Self::Ico),
            "image/tiff" => Some(Self::Tiff),
            _ => None,
        }
    }

    /// 拡張子として取得
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::Gif => "gif",
            Self::Bmp => "bmp",
            Self::Webp => "webp",
            Self::Svg => "svg",
            Self::Ico => "ico",
            Self::Tiff => "tiff",
            Self::Raw => "raw",
        }
    }

    /// MIMEタイプとして取得
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
            Self::Gif => "image/gif",
            Self::Bmp => "image/bmp",
            Self::Webp => "image/webp",
            Self::Svg => "image/svg+xml",
            Self::Ico => "image/x-icon",
            Self::Tiff => "image/tiff",
            Self::Raw => "application/octet-stream",
        }
    }

    /// ロスレス圧縮かチェック
    pub fn is_lossless(&self) -> bool {
        matches!(self, Self::Png | Self::Gif | Self::Bmp | Self::Tiff | Self::Raw)
    }

    /// アニメーションサポートかチェック
    pub fn supports_animation(&self) -> bool {
        matches!(self, Self::Gif | Self::Webp)
    }

    /// 透明度サポートかチェック
    pub fn supports_transparency(&self) -> bool {
        matches!(self, Self::Png | Self::Gif | Self::Webp | Self::Svg | Self::Ico)
    }

    /// メタデータサポートかチェック
    pub fn supports_metadata(&self) -> bool {
        matches!(self, Self::Png | Self::Jpeg | Self::Tiff)
    }

    /// 推奨される品質レベルを取得（1-100、該当しない場合はNone）
    pub fn recommended_quality(&self) -> Option<u8> {
        match self {
            Self::Jpeg => Some(85),
            Self::Webp => Some(80),
            _ => None,
        }
    }

    /// すべてのサポートされているフォーマットを取得
    pub fn all_formats() -> Vec<Self> {
        vec![
            Self::Png,
            Self::Jpeg,
            Self::Gif,
            Self::Bmp,
            Self::Webp,
            Self::Svg,
            Self::Ico,
            Self::Tiff,
            Self::Raw,
        ]
    }

    /// Web表示に適しているかチェック
    pub fn is_web_compatible(&self) -> bool {
        matches!(self, Self::Png | Self::Jpeg | Self::Gif | Self::Webp | Self::Svg)
    }
}

impl fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.extension().to_uppercase())
    }
}

impl FromStr for ImageFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_extension(s)
            .ok_or_else(|| format!("Unsupported image format: {}", s))
    }
}

/// 解像度を表す値オブジェクト
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

impl Resolution {
    /// 新しい解像度を作成
    pub fn new(width: u32, height: u32) -> Result<Self, ResolutionError> {
        if width == 0 || height == 0 {
            return Err(ResolutionError::ZeroDimension);
        }
        if width > 65535 || height > 65535 {
            return Err(ResolutionError::TooLarge);
        }
        Ok(Self { width, height })
    }

    /// 正方形の解像度を作成
    pub fn square(size: u32) -> Result<Self, ResolutionError> {
        Self::new(size, size)
    }

    /// Splatoon3標準解像度
    pub fn splatoon3_standard() -> Self {
        Self { width: 320, height: 120 }
    }

    /// 一般的な解像度プリセット
    pub fn preset(preset: ResolutionPreset) -> Self {
        match preset {
            ResolutionPreset::Splatoon3 => Self::splatoon3_standard(),
            ResolutionPreset::Qvga => Self { width: 320, height: 240 },
            ResolutionPreset::Vga => Self { width: 640, height: 480 },
            ResolutionPreset::Svga => Self { width: 800, height: 600 },
            ResolutionPreset::Xga => Self { width: 1024, height: 768 },
            ResolutionPreset::Hd => Self { width: 1280, height: 720 },
            ResolutionPreset::FullHd => Self { width: 1920, height: 1080 },
            ResolutionPreset::UltraHd => Self { width: 3840, height: 2160 },
        }
    }

    /// 総ピクセル数を計算
    pub fn total_pixels(&self) -> u64 {
        (self.width as u64) * (self.height as u64)
    }

    /// アスペクト比を計算
    pub fn aspect_ratio(&self) -> f64 {
        self.width as f64 / self.height as f64
    }

    /// 最大公約数を計算してアスペクト比を簡約
    pub fn simplified_aspect_ratio(&self) -> (u32, u32) {
        let gcd = Self::gcd(self.width, self.height);
        (self.width / gcd, self.height / gcd)
    }

    /// 最大公約数を計算
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 {
            a
        } else {
            Self::gcd(b, a % b)
        }
    }

    /// 指定された最大サイズに収まるようにスケール
    pub fn scale_to_fit(&self, max_width: u32, max_height: u32) -> Self {
        let width_ratio = max_width as f64 / self.width as f64;
        let height_ratio = max_height as f64 / self.height as f64;
        let scale = width_ratio.min(height_ratio);

        let new_width = (self.width as f64 * scale).round() as u32;
        let new_height = (self.height as f64 * scale).round() as u32;

        Self {
            width: new_width.max(1),
            height: new_height.max(1),
        }
    }

    /// 指定された倍率でスケール
    pub fn scale(&self, factor: f64) -> Result<Self, ResolutionError> {
        if factor <= 0.0 {
            return Err(ResolutionError::InvalidScale);
        }

        let new_width = (self.width as f64 * factor).round() as u32;
        let new_height = (self.height as f64 * factor).round() as u32;

        Self::new(new_width, new_height)
    }

    /// 横向きかチェック
    pub fn is_landscape(&self) -> bool {
        self.width > self.height
    }

    /// 縦向きかチェック
    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }

    /// 正方形かチェック
    pub fn is_square(&self) -> bool {
        self.width == self.height
    }

    /// 解像度を90度回転
    pub fn rotate_90(&self) -> Self {
        Self {
            width: self.height,
            height: self.width,
        }
    }

    /// 解像度が別の解像度に収まるかチェック
    pub fn fits_in(&self, other: &Resolution) -> bool {
        self.width <= other.width && self.height <= other.height
    }

    /// 解像度の面積比を計算
    pub fn area_ratio(&self, other: &Resolution) -> f64 {
        self.total_pixels() as f64 / other.total_pixels() as f64
    }

    /// 解像度をパディングして指定サイズに合わせる
    pub fn pad_to(&self, target: &Resolution) -> PaddingInfo {
        let x_padding = if target.width > self.width {
            (target.width - self.width) / 2
        } else {
            0
        };
        let y_padding = if target.height > self.height {
            (target.height - self.height) / 2
        } else {
            0
        };

        PaddingInfo {
            left: x_padding,
            right: target.width.saturating_sub(self.width + x_padding),
            top: y_padding,
            bottom: target.height.saturating_sub(self.height + y_padding),
        }
    }

    /// 解像度を文字列として表現
    pub fn to_string_short(&self) -> String {
        format!("{}x{}", self.width, self.height)
    }

    /// 解像度の分類を取得
    pub fn classification(&self) -> ResolutionClass {
        let pixels = self.total_pixels();
        
        if pixels <= 320 * 240 {
            ResolutionClass::Low
        } else if pixels <= 1280 * 720 {
            ResolutionClass::Medium
        } else if pixels <= 1920 * 1080 {
            ResolutionClass::High
        } else {
            ResolutionClass::VeryHigh
        }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} x {} ({} pixels)", 
            self.width, 
            self.height, 
            self.total_pixels()
        )
    }
}

impl FromStr for Resolution {
    type Err = ResolutionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('x').collect();
        if parts.len() != 2 {
            return Err(ResolutionError::InvalidFormat);
        }

        let width = parts[0].trim().parse::<u32>()
            .map_err(|_| ResolutionError::InvalidFormat)?;
        let height = parts[1].trim().parse::<u32>()
            .map_err(|_| ResolutionError::InvalidFormat)?;

        Self::new(width, height)
    }
}

/// 解像度エラー
#[derive(Debug, Clone, thiserror::Error)]
pub enum ResolutionError {
    #[error("Width or height cannot be zero")]
    ZeroDimension,
    #[error("Resolution too large")]
    TooLarge,
    #[error("Invalid scale factor")]
    InvalidScale,
    #[error("Invalid resolution format")]
    InvalidFormat,
}

/// 解像度プリセット
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionPreset {
    Splatoon3,
    Qvga,
    Vga,
    Svga,
    Xga,
    Hd,
    FullHd,
    UltraHd,
}

/// 解像度分類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionClass {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// パディング情報
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PaddingInfo {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}

impl PaddingInfo {
    /// パディングが必要かチェック
    pub fn is_needed(&self) -> bool {
        self.left > 0 || self.right > 0 || self.top > 0 || self.bottom > 0
    }

    /// 総パディング量を計算
    pub fn total_padding(&self) -> u32 {
        self.left + self.right + self.top + self.bottom
    }
}

/// 画像変換パラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionParameters {
    pub target_format: ImageFormat,
    pub target_resolution: Resolution,
    pub quality: Option<u8>,
    pub preserve_aspect_ratio: bool,
    pub background_color: Option<crate::domain::shared::value_objects::Color>,
    pub dithering: bool,
    pub color_reduction: Option<ColorReduction>,
    // 画像調整パラメータ
    pub adjustments: ImageAdjustments,
}

/// 画像調整パラメータ
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageAdjustments {
    /// 露出補正 (-2.0 ~ +2.0, 0.0 = 変更なし)
    pub exposure: f32,
    /// コントラスト (-100 ~ +100, 0 = 変更なし)
    pub contrast: i8,
    /// ブラックポイント (0 ~ 255, デフォルト: 0)
    pub black_point: u8,
    /// ホワイトポイント (0 ~ 255, デフォルト: 255)
    pub white_point: u8,
    /// ガンマ補正 (0.1 ~ 10.0, 1.0 = 変更なし)
    pub gamma: f32,
    /// ハイライト調整 (-100 ~ +100, 0 = 変更なし)
    pub highlights: i8,
    /// シャドウ調整 (-100 ~ +100, 0 = 変更なし)
    pub shadows: i8,
    /// 明度調整 (-100 ~ +100, 0 = 変更なし)
    pub brightness: i8,
    /// 2値化の閾値 (0 ~ 255, デフォルト: 128)
    pub threshold: u8,
    /// 適応的2値化を使用するか
    pub adaptive_threshold: bool,
    /// 適応的2値化のブロックサイズ (3以上の奇数)
    pub adaptive_block_size: u16,
    /// 適応的2値化の定数 (-100 ~ +100)
    pub adaptive_constant: i8,
}

impl Default for ImageAdjustments {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            contrast: 0,
            black_point: 0,
            white_point: 255,
            gamma: 1.0,
            highlights: 0,
            shadows: 0,
            brightness: 0,
            threshold: 128,
            adaptive_threshold: false,
            adaptive_block_size: 11,
            adaptive_constant: 2,
        }
    }
}

impl ImageAdjustments {
    /// Splatoon3向けの推奨設定
    pub fn splatoon3_recommended() -> Self {
        Self {
            exposure: 0.2,
            contrast: 20,
            black_point: 10,
            white_point: 245,
            gamma: 1.2,
            highlights: -10,
            shadows: 10,
            brightness: 0,
            threshold: 120,
            adaptive_threshold: true,
            adaptive_block_size: 15,
            adaptive_constant: 5,
        }
    }

    /// 高コントラスト設定
    pub fn high_contrast() -> Self {
        Self {
            exposure: 0.0,
            contrast: 50,
            black_point: 20,
            white_point: 235,
            gamma: 0.8,
            highlights: -20,
            shadows: 20,
            brightness: 0,
            threshold: 110,
            adaptive_threshold: false,
            adaptive_block_size: 11,
            adaptive_constant: 2,
        }
    }

    /// ソフトな設定（細かい線を保持）
    pub fn soft_detail() -> Self {
        Self {
            exposure: -0.2,
            contrast: -10,
            black_point: 5,
            white_point: 250,
            gamma: 1.5,
            highlights: 10,
            shadows: -10,
            brightness: 5,
            threshold: 135,
            adaptive_threshold: true,
            adaptive_block_size: 21,
            adaptive_constant: 8,
        }
    }

    /// 露出が有効範囲内かチェック
    pub fn validate_exposure(&self) -> bool {
        self.exposure >= -2.0 && self.exposure <= 2.0
    }

    /// ガンマが有効範囲内かチェック
    pub fn validate_gamma(&self) -> bool {
        self.gamma >= 0.1 && self.gamma <= 10.0
    }

    /// ブラックポイントとホワイトポイントの妥当性チェック
    pub fn validate_points(&self) -> bool {
        self.black_point < self.white_point
    }

    /// 適応的2値化のパラメータチェック
    pub fn validate_adaptive(&self) -> bool {
        self.adaptive_block_size >= 3 && self.adaptive_block_size % 2 == 1
    }

    /// すべてのパラメータを検証
    pub fn validate(&self) -> Result<(), String> {
        if !self.validate_exposure() {
            return Err("露出は-2.0から+2.0の範囲で指定してください".to_string());
        }
        if !self.validate_gamma() {
            return Err("ガンマは0.1から10.0の範囲で指定してください".to_string());
        }
        if !self.validate_points() {
            return Err("ブラックポイントはホワイトポイントより小さくする必要があります".to_string());
        }
        if self.adaptive_threshold && !self.validate_adaptive() {
            return Err("適応的2値化のブロックサイズは3以上の奇数である必要があります".to_string());
        }
        Ok(())
    }
}

impl ConversionParameters {
    /// 新しい変換パラメータを作成
    pub fn new(target_format: ImageFormat, target_resolution: Resolution) -> Self {
        Self {
            target_format,
            target_resolution,
            quality: target_format.recommended_quality(),
            preserve_aspect_ratio: true,
            background_color: None,
            dithering: false,
            color_reduction: None,
            adjustments: ImageAdjustments::default(),
        }
    }

    /// Splatoon3用の変換パラメータ
    pub fn for_splatoon3() -> Self {
        Self {
            target_format: ImageFormat::Png,
            target_resolution: Resolution::splatoon3_standard(),
            quality: None,
            preserve_aspect_ratio: true,
            background_color: Some(crate::domain::shared::value_objects::Color::white()),
            dithering: true,
            color_reduction: Some(ColorReduction::Palette(16)),
            adjustments: ImageAdjustments::splatoon3_recommended(),
        }
    }

    /// 品質を設定
    pub fn with_quality(mut self, quality: u8) -> Self {
        self.quality = Some(quality.min(100));
        self
    }

    /// アスペクト比保持設定
    pub fn with_aspect_ratio_preservation(mut self, preserve: bool) -> Self {
        self.preserve_aspect_ratio = preserve;
        self
    }

    /// 背景色を設定
    pub fn with_background_color(mut self, color: crate::domain::shared::value_objects::Color) -> Self {
        self.background_color = Some(color);
        self
    }

    /// ディザリング設定
    pub fn with_dithering(mut self, enabled: bool) -> Self {
        self.dithering = enabled;
        self
    }

    /// 色削減設定
    pub fn with_color_reduction(mut self, reduction: ColorReduction) -> Self {
        self.color_reduction = Some(reduction);
        self
    }

    /// 画像調整を設定
    pub fn with_adjustments(mut self, adjustments: ImageAdjustments) -> Self {
        self.adjustments = adjustments;
        self
    }

    /// 露出を調整
    pub fn with_exposure(mut self, exposure: f32) -> Self {
        self.adjustments.exposure = exposure.clamp(-2.0, 2.0);
        self
    }

    /// コントラストを調整
    pub fn with_contrast(mut self, contrast: i8) -> Self {
        self.adjustments.contrast = contrast.clamp(-100, 100);
        self
    }

    /// ブラックポイントとホワイトポイントを設定
    pub fn with_points(mut self, black_point: u8, white_point: u8) -> Self {
        self.adjustments.black_point = black_point;
        self.adjustments.white_point = white_point;
        self
    }

    /// ガンマ補正を設定
    pub fn with_gamma(mut self, gamma: f32) -> Self {
        self.adjustments.gamma = gamma.clamp(0.1, 10.0);
        self
    }

    /// 2値化の閾値を設定
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.adjustments.threshold = threshold;
        self
    }

    /// 適応的2値化を設定
    pub fn with_adaptive_threshold(mut self, enable: bool, block_size: u16, constant: i8) -> Self {
        self.adjustments.adaptive_threshold = enable;
        if enable {
            self.adjustments.adaptive_block_size = if block_size % 2 == 0 { block_size + 1 } else { block_size };
            self.adjustments.adaptive_constant = constant.clamp(-100, 100);
        }
        self
    }

    /// パラメータの検証
    pub fn validate(&self) -> Result<(), ConversionError> {
        if let Some(quality) = self.quality {
            if quality == 0 || quality > 100 {
                return Err(ConversionError::InvalidQuality);
            }
        }

        if self.target_resolution.total_pixels() == 0 {
            return Err(ConversionError::InvalidResolution);
        }

        if let Some(ColorReduction::Palette(colors)) = self.color_reduction {
            if colors < 2 {
                return Err(ConversionError::InvalidColorCount);
            }
        }

        // 画像調整パラメータの検証
        if let Err(e) = self.adjustments.validate() {
            return Err(ConversionError::InvalidAdjustments(e));
        }

        Ok(())
    }
}

/// 色削減方法
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorReduction {
    /// パレット色数制限
    Palette(u8),
    /// グレースケール変換
    Grayscale,
    /// 2値化（白黒）
    Binary(u8), // 閾値
}

/// 変換エラー
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConversionError {
    #[error("Invalid quality value")]
    InvalidQuality,
    #[error("Invalid resolution")]
    InvalidResolution,
    #[error("Invalid color count")]
    InvalidColorCount,
    #[error("Invalid adjustments: {0}")]
    InvalidAdjustments(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::from_extension("png"), Some(ImageFormat::Png));
        assert_eq!(ImageFormat::from_extension("jpg"), Some(ImageFormat::Jpeg));
        assert_eq!(ImageFormat::from_extension("unknown"), None);
        
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.mime_type(), "image/jpeg");
        
        assert!(ImageFormat::Png.is_lossless());
        assert!(!ImageFormat::Jpeg.is_lossless());
        
        assert!(ImageFormat::Png.supports_transparency());
        assert!(!ImageFormat::Jpeg.supports_transparency());
    }

    #[test]
    fn test_resolution() {
        let res = Resolution::new(1920, 1080).unwrap();
        assert_eq!(res.total_pixels(), 2073600);
        assert_eq!(res.aspect_ratio(), 1920.0 / 1080.0);
        assert!(res.is_landscape());
        assert!(!res.is_portrait());
        
        let scaled = res.scale(0.5).unwrap();
        assert_eq!(scaled.width, 960);
        assert_eq!(scaled.height, 540);
        
        let fitted = res.scale_to_fit(800, 600);
        assert!(fitted.width <= 800);
        assert!(fitted.height <= 600);
    }

    #[test]
    fn test_resolution_parsing() {
        let res = "1920x1080".parse::<Resolution>().unwrap();
        assert_eq!(res.width, 1920);
        assert_eq!(res.height, 1080);
        
        assert!("invalid".parse::<Resolution>().is_err());
        assert!("1920".parse::<Resolution>().is_err());
    }

    #[test]
    fn test_conversion_parameters() {
        let params = ConversionParameters::for_splatoon3();
        assert_eq!(params.target_format, ImageFormat::Png);
        assert_eq!(params.target_resolution, Resolution::splatoon3_standard());
        assert!(params.dithering);
        assert!(params.color_reduction.is_some());
        
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_resolution_classification() {
        let low_res = Resolution::new(320, 240).unwrap();
        assert_eq!(low_res.classification(), ResolutionClass::Low);
        
        let hd_res = Resolution::new(1280, 720).unwrap();
        assert_eq!(hd_res.classification(), ResolutionClass::Medium);
        
        let full_hd = Resolution::new(1920, 1080).unwrap();
        assert_eq!(full_hd.classification(), ResolutionClass::High);
    }

    #[test]
    fn test_padding_info() {
        let source = Resolution::new(100, 100).unwrap();
        let target = Resolution::new(120, 140).unwrap();
        let padding = source.pad_to(&target);
        
        assert_eq!(padding.left, 10);
        assert_eq!(padding.right, 10);
        assert_eq!(padding.top, 20);
        assert_eq!(padding.bottom, 20);
        assert!(padding.is_needed());
    }
} 