//! アートワーク集約のエンティティ
//!
//! 画像データの管理、変換、検証に関するエンティティを定義

use crate::domain::shared::value_objects::{Color, Coordinates, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

/// アートワークID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtworkId(Uuid);

impl ArtworkId {
    /// 新しいIDを生成
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// UUIDから作成
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 文字列から作成
    pub fn parse(s: &str) -> Result<Self, String> {
        let uuid = Uuid::parse_str(s).map_err(|e| format!("Invalid UUID format: {}", e))?;
        Ok(Self(uuid))
    }

    /// UUIDとして取得
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }

    /// 文字列として取得
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl FromStr for ArtworkId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for ArtworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ArtworkId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<ArtworkId> for Uuid {
    fn from(id: ArtworkId) -> Self {
        id.0
    }
}

/// アートワークのメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub original_filename: Option<String>,
    pub file_size: u64,
    pub checksum: String,
}

impl ArtworkMetadata {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            tags: Vec::new(),
            author: None,
            original_filename: None,
            file_size: 0,
            checksum: String::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(&tag.to_string())
    }
}

/// アートワークエンティティ
///
/// 画像データとメタデータを管理する集約ルート
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artwork {
    pub id: ArtworkId,
    pub metadata: ArtworkMetadata,
    pub original_format: String,
    pub canvas: Canvas,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
    pub version: u32,
}

impl Artwork {
    /// 新しいアートワークを作成
    #[instrument(skip(canvas), fields(name = %metadata.name, format = %original_format))]
    pub fn new(metadata: ArtworkMetadata, original_format: String, canvas: Canvas) -> Self {
        debug!("新しいアートワークを作成中");
        let now = Timestamp::now();
        let artwork = Self {
            id: ArtworkId::generate(),
            metadata,
            original_format,
            canvas,
            created_at: now,
            updated_at: now,
            version: 1,
        };

        info!(
            artwork_id = %artwork.id,
            name = %artwork.metadata.name,
            format = %artwork.original_format,
            canvas_size = format!("{}x{}", artwork.canvas.width, artwork.canvas.height),
            total_dots = %artwork.total_dots(),
            drawable_dots = %artwork.drawable_dots(),
            "アートワークが作成されました"
        );

        artwork
    }

    /// 指定されたIDでアートワークを作成
    pub fn with_id(
        id: ArtworkId,
        metadata: ArtworkMetadata,
        original_format: String,
        canvas: Canvas,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id,
            metadata,
            original_format,
            canvas,
            created_at: now,
            updated_at: now,
            version: 1,
        }
    }

    /// アートワークを更新
    #[instrument(skip(self, canvas), fields(artwork_id = %self.id))]
    pub fn update_canvas(&mut self, canvas: Canvas) {
        debug!("キャンバスを更新中");
        let old_dots = self.total_dots();
        let old_drawable = self.drawable_dots();

        self.canvas = canvas;
        self.updated_at = Timestamp::now();
        self.version += 1;

        info!(
            artwork_id = %self.id,
            old_total_dots = %old_dots,
            new_total_dots = %self.total_dots(),
            old_drawable_dots = %old_drawable,
            new_drawable_dots = %self.drawable_dots(),
            new_version = %self.version,
            "キャンバスが更新されました"
        );
    }

    /// メタデータを更新
    #[instrument(skip(self), fields(artwork_id = %self.id, new_name = %metadata.name))]
    pub fn update_metadata(&mut self, metadata: ArtworkMetadata) {
        debug!("メタデータを更新中");
        let old_name = self.metadata.name.clone();

        self.metadata = metadata;
        self.updated_at = Timestamp::now();
        self.version += 1;

        info!(
            artwork_id = %self.id,
            old_name = %old_name,
            new_name = %self.metadata.name,
            new_version = %self.version,
            "メタデータが更新されました"
        );
    }

    /// アートワークの総ドット数を取得
    pub fn total_dots(&self) -> usize {
        self.canvas.dots.len()
    }

    /// アートワークの描画可能ドット数を取得
    pub fn drawable_dots(&self) -> usize {
        self.canvas
            .dots
            .values()
            .filter(|dot| dot.is_drawable())
            .count()
    }

    /// アートワークの完成度を計算（0.0-1.0）
    pub fn completion_ratio(&self) -> f64 {
        let total = self.total_dots();
        if total == 0 {
            return 1.0;
        }
        let painted = self
            .canvas
            .dots
            .values()
            .filter(|dot| dot.is_painted)
            .count();
        painted as f64 / total as f64
    }

    /// アートワークの推定描画時間を計算（秒）
    pub fn estimated_painting_time(&self, dots_per_second: f64) -> u64 {
        let drawable = self.drawable_dots();
        if dots_per_second <= 0.0 {
            return 0;
        }
        (drawable as f64 / dots_per_second).ceil() as u64
    }

    /// アートワークの複雑度を計算
    pub fn complexity_score(&self) -> f64 {
        let total_dots = self.total_dots() as f64;
        let drawable_dots = self.drawable_dots() as f64;
        let canvas_size = (self.canvas.width as f64) * (self.canvas.height as f64);

        if canvas_size == 0.0 {
            return 0.0;
        }

        let density = drawable_dots / canvas_size;
        let coverage = drawable_dots / total_dots.max(1.0);

        (density + coverage) / 2.0
    }

    /// アートワークの統計情報を取得
    pub fn statistics(&self) -> ArtworkStatistics {
        let total_dots = self.total_dots();
        let drawable_dots = self.drawable_dots();
        let painted_dots = self
            .canvas
            .dots
            .values()
            .filter(|dot| dot.is_painted)
            .count();

        let colors: std::collections::HashSet<Color> =
            self.canvas.dots.values().map(|dot| dot.color).collect();

        ArtworkStatistics {
            total_dots,
            drawable_dots,
            painted_dots,
            unique_colors: colors.len(),
            completion_ratio: self.completion_ratio(),
            complexity_score: self.complexity_score(),
            canvas_size: (self.canvas.width, self.canvas.height),
        }
    }

    /// アートワークをリセット（全ドットの描画状態をクリア）
    pub fn reset_painting_state(&mut self) {
        for dot in self.canvas.dots.values_mut() {
            dot.reset_paint_status();
        }
        self.updated_at = Timestamp::now();
        self.version += 1;
    }

    /// アートワークの検証
    #[instrument(skip(self), fields(artwork_id = %self.id, name = %self.metadata.name))]
    pub fn validate(&self) -> Result<(), ArtworkValidationError> {
        debug!("アートワークの検証を開始");

        // 名前の検証
        debug!("名前の検証: '{}'", self.metadata.name);
        if self.metadata.name.trim().is_empty() {
            error!("アートワーク名が空です");
            return Err(ArtworkValidationError::EmptyName);
        }

        // キャンバスサイズの検証
        debug!(
            "キャンバスサイズの検証: {}x{}",
            self.canvas.width, self.canvas.height
        );
        if self.canvas.width == 0 || self.canvas.height == 0 {
            error!(
                "無効なキャンバスサイズ: {}x{}",
                self.canvas.width, self.canvas.height
            );
            return Err(ArtworkValidationError::InvalidCanvasSize);
        }

        // 最大サイズの制限
        if self.canvas.width > 1000 || self.canvas.height > 1000 {
            error!(
                "キャンバスサイズが大きすぎます: {}x{} (最大: 1000x1000)",
                self.canvas.width, self.canvas.height
            );
            return Err(ArtworkValidationError::CanvasTooLarge);
        }

        // ドットの座標検証
        debug!("ドット座標の検証: {}個のドット", self.canvas.dots.len());
        for coord in self.canvas.dots.keys() {
            if !coord.is_within_bounds(self.canvas.width, self.canvas.height) {
                error!("ドットが範囲外の座標にあります: {}", coord);
                return Err(ArtworkValidationError::DotOutOfBounds(*coord));
            }
        }

        info!(
            artwork_id = %self.id,
            name = %self.metadata.name,
            canvas_size = format!("{}x{}", self.canvas.width, self.canvas.height),
            total_dots = %self.total_dots(),
            "アートワークの検証が完了しました"
        );

        Ok(())
    }
}

/// アートワークの統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkStatistics {
    pub total_dots: usize,
    pub drawable_dots: usize,
    pub painted_dots: usize,
    pub unique_colors: usize,
    pub completion_ratio: f64,
    pub complexity_score: f64,
    pub canvas_size: (u16, u16),
}

/// アートワークの検証エラー
#[derive(Debug, Clone, thiserror::Error)]
pub enum ArtworkValidationError {
    #[error("Artwork name cannot be empty")]
    EmptyName,
    #[error("Invalid canvas size")]
    InvalidCanvasSize,
    #[error("Canvas size too large")]
    CanvasTooLarge,
    #[error("Dot at coordinates {0} is out of bounds")]
    DotOutOfBounds(Coordinates),
}

/// キャンバスエンティティ
///
/// 320x120の描画領域を表現
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub dots: HashMap<Coordinates, Dot>,
    pub background_color: Color,
}

impl Canvas {
    /// 新しいキャンバスを作成
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            dots: HashMap::new(),
            background_color: Color::white(),
        }
    }

    /// Splatoon3標準サイズのキャンバスを作成
    pub fn splatoon3_standard() -> Self {
        Self::new(320, 120)
    }

    /// 背景色を指定してキャンバスを作成
    pub fn with_background(width: u16, height: u16, background_color: Color) -> Self {
        Self {
            width,
            height,
            dots: HashMap::new(),
            background_color,
        }
    }

    /// 指定座標にドットを設定
    pub fn set_dot(&mut self, coordinates: Coordinates, dot: Dot) -> Result<(), CanvasError> {
        if !coordinates.is_within_bounds(self.width, self.height) {
            return Err(CanvasError::OutOfBounds(coordinates));
        }
        self.dots.insert(coordinates, dot);
        Ok(())
    }

    /// 指定座標のドットを取得
    pub fn get_dot(&self, coordinates: &Coordinates) -> Option<&Dot> {
        self.dots.get(coordinates)
    }

    /// 指定座標のドットを可変参照で取得
    pub fn get_dot_mut(&mut self, coordinates: &Coordinates) -> Option<&mut Dot> {
        self.dots.get_mut(coordinates)
    }

    /// 指定座標のドットを削除
    pub fn remove_dot(&mut self, coordinates: &Coordinates) -> Option<Dot> {
        self.dots.remove(coordinates)
    }

    /// キャンバスをクリア
    pub fn clear(&mut self) {
        self.dots.clear();
    }

    /// 描画可能なドットのリストを取得
    pub fn drawable_dots(&self) -> Vec<(&Coordinates, &Dot)> {
        self.dots
            .iter()
            .filter(|(_, dot)| dot.is_drawable())
            .collect()
    }

    /// 描画済みドットのリストを取得
    pub fn painted_dots(&self) -> Vec<(&Coordinates, &Dot)> {
        self.dots.iter().filter(|(_, dot)| dot.is_painted).collect()
    }

    /// 未描画ドットのリストを取得
    pub fn unpainted_dots(&self) -> Vec<(&Coordinates, &Dot)> {
        self.dots
            .iter()
            .filter(|(_, dot)| dot.is_drawable() && !dot.is_painted)
            .collect()
    }

    /// キャンバスの境界をチェック
    pub fn is_valid_coordinate(&self, coordinates: &Coordinates) -> bool {
        coordinates.is_within_bounds(self.width, self.height)
    }

    /// キャンバスのサイズを変更
    pub fn resize(&mut self, new_width: u16, new_height: u16) -> Result<(), CanvasError> {
        if new_width == 0 || new_height == 0 {
            return Err(CanvasError::InvalidSize);
        }

        // 新しいサイズに収まらないドットを削除
        self.dots
            .retain(|coord, _| coord.is_within_bounds(new_width, new_height));

        self.width = new_width;
        self.height = new_height;
        Ok(())
    }

    /// 指定された領域のドットを取得
    pub fn get_region(
        &self,
        top_left: Coordinates,
        bottom_right: Coordinates,
    ) -> Vec<(&Coordinates, &Dot)> {
        self.dots
            .iter()
            .filter(|(coord, _)| {
                coord.x >= top_left.x
                    && coord.x <= bottom_right.x
                    && coord.y >= top_left.y
                    && coord.y <= bottom_right.y
            })
            .collect()
    }

    /// キャンバスの密度を計算
    pub fn density(&self) -> f64 {
        let total_pixels = (self.width as f64) * (self.height as f64);
        if total_pixels == 0.0 {
            return 0.0;
        }
        self.dots.len() as f64 / total_pixels
    }

    /// キャンバスを別のキャンバスとマージ
    pub fn merge(&mut self, other: &Canvas, offset: Coordinates) -> Result<(), CanvasError> {
        for (coord, dot) in &other.dots {
            let new_coord = coord
                .move_by(offset.x as i16, offset.y as i16)
                .ok_or(CanvasError::OutOfBounds(*coord))?;

            if !self.is_valid_coordinate(&new_coord) {
                continue; // 範囲外のドットはスキップ
            }

            self.dots.insert(new_coord, dot.clone());
        }
        Ok(())
    }
}

/// キャンバスエラー
#[derive(Debug, Clone, thiserror::Error)]
pub enum CanvasError {
    #[error("Coordinates {0} are out of bounds")]
    OutOfBounds(Coordinates),
    #[error("Invalid canvas size")]
    InvalidSize,
}

/// ドットエンティティ
///
/// キャンバス上の個別ドットを表現
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dot {
    pub color: Color,
    pub opacity: u8,
    pub is_painted: bool,
    pub created_at: Timestamp,
    pub painted_at: Option<Timestamp>,
    pub layer: u8,
}

impl Dot {
    /// 新しいドットを作成
    pub fn new(color: Color, opacity: u8) -> Self {
        Self {
            color,
            opacity,
            is_painted: false,
            created_at: Timestamp::now(),
            painted_at: None,
            layer: 0,
        }
    }

    /// レイヤーを指定してドットを作成
    pub fn with_layer(color: Color, opacity: u8, layer: u8) -> Self {
        Self {
            color,
            opacity,
            is_painted: false,
            created_at: Timestamp::now(),
            painted_at: None,
            layer,
        }
    }

    /// 黒いドットを作成
    pub fn black() -> Self {
        Self::new(Color::black(), 255)
    }

    /// 白いドットを作成
    pub fn white() -> Self {
        Self::new(Color::white(), 255)
    }

    /// 透明なドットを作成
    pub fn transparent() -> Self {
        Self::new(Color::white(), 0)
    }

    /// ドットが描画可能かチェック
    pub fn is_drawable(&self) -> bool {
        self.opacity > 0 && !self.is_painted
    }

    /// ドットが可視かチェック
    pub fn is_visible(&self) -> bool {
        self.opacity > 0
    }

    /// ドットを描画済みにマーク
    pub fn mark_as_painted(&mut self) {
        self.is_painted = true;
        self.painted_at = Some(Timestamp::now());
    }

    /// ドットの描画をリセット
    pub fn reset_paint_status(&mut self) {
        self.is_painted = false;
        self.painted_at = None;
    }

    /// 2値化されたドットかチェック
    pub fn is_binary(&self, threshold: u8) -> bool {
        self.color.to_binary(threshold)
    }

    /// ドットの年齢を取得（作成からの経過時間、ミリ秒）
    pub fn age_millis(&self) -> u64 {
        self.created_at.elapsed_millis()
    }

    /// ドットが描画されてからの経過時間を取得（ミリ秒）
    pub fn painted_age_millis(&self) -> Option<u64> {
        self.painted_at.map(|t| t.elapsed_millis())
    }

    /// ドットの色を変更
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// ドットの透明度を変更
    pub fn set_opacity(&mut self, opacity: u8) {
        self.opacity = opacity;
    }

    /// ドットを他のドットとブレンド
    pub fn blend_with(&self, other: &Dot, ratio: f64) -> Dot {
        let blended_color = self.color.blend(&other.color, ratio);
        let blended_opacity =
            (self.opacity as f64 * (1.0 - ratio) + other.opacity as f64 * ratio) as u8;

        Dot::new(blended_color, blended_opacity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artwork_id() {
        let id1 = ArtworkId::generate();
        let id2 = ArtworkId::generate();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id_from_uuid = ArtworkId::from_uuid(uuid);
        assert_eq!(id_from_uuid.as_uuid(), uuid);

        let id_str = id1.as_str();
        let id_from_str = ArtworkId::from_str(&id_str).unwrap();
        assert_eq!(id1, id_from_str);
    }

    #[test]
    fn test_artwork_creation() {
        let metadata = ArtworkMetadata::new("Test Artwork".to_string())
            .with_description("A test artwork".to_string())
            .with_tags(vec!["test".to_string(), "sample".to_string()]);

        let canvas = Canvas::splatoon3_standard();
        let artwork = Artwork::new(metadata, "png".to_string(), canvas);

        assert_eq!(artwork.metadata.name, "Test Artwork");
        assert_eq!(artwork.original_format, "png");
        assert_eq!(artwork.canvas.width, 320);
        assert_eq!(artwork.canvas.height, 120);
        assert_eq!(artwork.version, 1);
    }

    #[test]
    fn test_canvas_operations() {
        let mut canvas = Canvas::new(10, 10);
        let coord = Coordinates::new(5, 5);
        let dot = Dot::black();

        assert!(canvas.set_dot(coord, dot).is_ok());
        assert!(canvas.get_dot(&coord).is_some());

        let invalid_coord = Coordinates::new(15, 15);
        let invalid_dot = Dot::white();
        assert!(canvas.set_dot(invalid_coord, invalid_dot).is_err());

        assert_eq!(canvas.dots.len(), 1);
        assert_eq!(canvas.drawable_dots().len(), 1);
        assert_eq!(canvas.painted_dots().len(), 0);
    }

    #[test]
    fn test_dot_properties() {
        let mut dot = Dot::black();
        assert!(dot.is_drawable());
        assert!(dot.is_visible());
        assert!(!dot.is_painted);

        dot.mark_as_painted();
        assert!(!dot.is_drawable());
        assert!(dot.is_painted);
        assert!(dot.painted_at.is_some());

        dot.reset_paint_status();
        assert!(dot.is_drawable());
        assert!(!dot.is_painted);
        assert!(dot.painted_at.is_none());
    }

    #[test]
    fn test_artwork_statistics() {
        let metadata = ArtworkMetadata::new("Test".to_string());
        let mut canvas = Canvas::new(5, 5);

        // いくつかのドットを追加
        canvas
            .set_dot(Coordinates::new(0, 0), Dot::black())
            .unwrap();
        canvas
            .set_dot(Coordinates::new(1, 1), Dot::white())
            .unwrap();
        canvas
            .set_dot(Coordinates::new(2, 2), Dot::new(Color::red(), 255))
            .unwrap();

        let artwork = Artwork::new(metadata, "png".to_string(), canvas);
        let stats = artwork.statistics();

        assert_eq!(stats.total_dots, 3);
        assert_eq!(stats.drawable_dots, 3);
        assert_eq!(stats.painted_dots, 0);
        assert_eq!(stats.unique_colors, 3);
        assert_eq!(stats.completion_ratio, 0.0);
    }

    #[test]
    fn test_canvas_merge() {
        let mut canvas1 = Canvas::new(10, 10);
        let mut canvas2 = Canvas::new(5, 5);

        canvas1
            .set_dot(Coordinates::new(0, 0), Dot::black())
            .unwrap();
        canvas2
            .set_dot(Coordinates::new(0, 0), Dot::new(Color::red(), 255))
            .unwrap();

        let offset = Coordinates::new(2, 2);
        canvas1.merge(&canvas2, offset).unwrap();

        assert_eq!(canvas1.dots.len(), 2);
        assert!(canvas1.get_dot(&Coordinates::new(2, 2)).is_some());
    }
}
