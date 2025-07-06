use crate::domain::artwork::value_objects::{ImageAdjustments, ColorReduction};
use crate::domain::shared::value_objects::Color;

/// 画像処理サービス
pub struct ImageProcessingService;

impl ImageProcessingService {
    /// ピクセルに画像調整を適用
    pub fn apply_adjustments(pixel: &Color, adjustments: &ImageAdjustments) -> Color {
        let mut r = pixel.r as f32;
        let mut g = pixel.g as f32;
        let mut b = pixel.b as f32;

        // 1. 露出補正
        if adjustments.exposure != 0.0 {
            let factor = (2.0_f32).powf(adjustments.exposure);
            r = (r * factor).clamp(0.0, 255.0);
            g = (g * factor).clamp(0.0, 255.0);
            b = (b * factor).clamp(0.0, 255.0);
        }

        // 2. ブラックポイント・ホワイトポイント調整
        let black = adjustments.black_point as f32;
        let white = adjustments.white_point as f32;
        let range = white - black;
        
        if range > 0.0 {
            r = ((r - black) * 255.0 / range).clamp(0.0, 255.0);
            g = ((g - black) * 255.0 / range).clamp(0.0, 255.0);
            b = ((b - black) * 255.0 / range).clamp(0.0, 255.0);
        }

        // 3. ガンマ補正
        if adjustments.gamma != 1.0 {
            let inv_gamma = 1.0 / adjustments.gamma;
            r = ((r / 255.0).powf(inv_gamma) * 255.0).clamp(0.0, 255.0);
            g = ((g / 255.0).powf(inv_gamma) * 255.0).clamp(0.0, 255.0);
            b = ((b / 255.0).powf(inv_gamma) * 255.0).clamp(0.0, 255.0);
        }

        // 4. 明度調整
        if adjustments.brightness != 0 {
            let brightness = adjustments.brightness as f32;
            r = (r + brightness).clamp(0.0, 255.0);
            g = (g + brightness).clamp(0.0, 255.0);
            b = (b + brightness).clamp(0.0, 255.0);
        }

        // 5. コントラスト調整
        if adjustments.contrast != 0 {
            let contrast = (100.0 + adjustments.contrast as f32) / 100.0;
            r = ((r - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
            g = ((g - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
            b = ((b - 128.0) * contrast + 128.0).clamp(0.0, 255.0);
        }

        // 6. ハイライト・シャドウ調整
        let luminance = pixel.luminance() as f32;
        
        if adjustments.highlights != 0 && luminance > 0.5 {
            let highlight_factor = 1.0 + (adjustments.highlights as f32 / 100.0) * ((luminance - 0.5) * 2.0);
            r = (r * highlight_factor).clamp(0.0, 255.0);
            g = (g * highlight_factor).clamp(0.0, 255.0);
            b = (b * highlight_factor).clamp(0.0, 255.0);
        }
        
        if adjustments.shadows != 0 && luminance < 0.5 {
            let shadow_factor = 1.0 + (adjustments.shadows as f32 / 100.0) * ((0.5 - luminance) * 2.0);
            r = (r * shadow_factor).clamp(0.0, 255.0);
            g = (g * shadow_factor).clamp(0.0, 255.0);
            b = (b * shadow_factor).clamp(0.0, 255.0);
        }

        Color::new(r as u8, g as u8, b as u8, pixel.a)
    }

    /// 2値化処理
    pub fn apply_threshold(pixel: &Color, adjustments: &ImageAdjustments) -> Color {
        let grayscale = pixel.to_grayscale();
        
        if grayscale >= adjustments.threshold {
            Color::white()
        } else {
            Color::black()
        }
    }

    /// 適応的2値化（簡易実装）
    pub fn apply_adaptive_threshold(
        pixel: &Color,
        local_average: u8,
        adjustments: &ImageAdjustments,
    ) -> Color {
        let grayscale = pixel.to_grayscale();
        let threshold = (local_average as i16 + adjustments.adaptive_constant as i16).clamp(0, 255) as u8;
        
        if grayscale >= threshold {
            Color::white()
        } else {
            Color::black()
        }
    }

    /// フロイド・スタインバーグ・ディザリング用のエラー計算
    pub fn calculate_dither_error(original: f32, quantized: f32) -> f32 {
        original - quantized
    }

    /// ディザリングエラーの分配
    pub fn distribute_error(error: f32, factor: f32) -> f32 {
        error * factor
    }

    /// 色削減の適用
    pub fn apply_color_reduction(pixel: &Color, reduction: &ColorReduction) -> Color {
        match reduction {
            ColorReduction::Grayscale => {
                let gray = pixel.to_grayscale();
                Color::new(gray, gray, gray, pixel.a)
            }
            ColorReduction::Binary(threshold) => {
                if pixel.to_grayscale() >= *threshold {
                    Color::white()
                } else {
                    Color::black()
                }
            }
            ColorReduction::Palette(levels) => {
                let levels = *levels as f32;
                let step = 255.0 / (levels - 1.0);
                
                let r = ((pixel.r as f32 / step).round() * step).clamp(0.0, 255.0) as u8;
                let g = ((pixel.g as f32 / step).round() * step).clamp(0.0, 255.0) as u8;
                let b = ((pixel.b as f32 / step).round() * step).clamp(0.0, 255.0) as u8;
                
                Color::new(r, g, b, pixel.a)
            }
        }
    }

    /// ヒストグラム均等化（コントラスト改善）
    pub fn calculate_histogram_equalization_lut(histogram: &[u32; 256]) -> [u8; 256] {
        let total_pixels: u32 = histogram.iter().sum();
        let mut cdf = [0u32; 256];
        let mut lut = [0u8; 256];
        
        // 累積分布関数（CDF）の計算
        cdf[0] = histogram[0];
        for i in 1..256 {
            cdf[i] = cdf[i - 1] + histogram[i];
        }
        
        // ルックアップテーブルの作成
        let cdf_min = cdf.iter().find(|&&x| x > 0).copied().unwrap_or(0);
        let scale = 255.0 / (total_pixels - cdf_min) as f32;
        
        for i in 0..256 {
            if cdf[i] > cdf_min {
                lut[i] = ((cdf[i] - cdf_min) as f32 * scale).round() as u8;
            }
        }
        
        lut
    }

    /// エッジ検出（Sobelフィルタ）
    pub fn sobel_edge_magnitude(pixels: &[[u8; 3]; 3]) -> u8 {
        let gx = [
            [-1, 0, 1],
            [-2, 0, 2],
            [-1, 0, 1],
        ];
        
        let gy = [
            [-1, -2, -1],
            [0, 0, 0],
            [1, 2, 1],
        ];
        
        let mut sum_x = 0i32;
        let mut sum_y = 0i32;
        
        for i in 0..3 {
            for j in 0..3 {
                let pixel_value = pixels[i][j] as i32;
                sum_x += pixel_value * gx[i][j];
                sum_y += pixel_value * gy[i][j];
            }
        }
        
        let magnitude = ((sum_x * sum_x + sum_y * sum_y) as f32).sqrt();
        magnitude.clamp(0.0, 255.0) as u8
    }

    /// ノイズ除去（メディアンフィルタ）
    pub fn median_filter(pixels: &[u8]) -> u8 {
        let mut sorted = pixels.to_vec();
        sorted.sort_unstable();
        sorted[sorted.len() / 2]
    }
}