//! ドメインイベント
//!
//! アートワーク集約で発生するドメインイベントを定義

use crate::domain::artwork::entities::{ArtworkId, ArtworkMetadata, Canvas};
use crate::domain::shared::events::{DomainEvent, EventId, EventMetadata};
use crate::domain::shared::value_objects::{Coordinates, Timestamp};
use serde::{Deserialize, Serialize};

/// アートワーク関連のドメインイベント
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArtworkEvent {
    /// アートワークが作成された
    ArtworkCreated {
        event_id: EventId,
        artwork_id: ArtworkId,
        metadata: ArtworkMetadata,
        original_format: String,
        canvas_width: u16,
        canvas_height: u16,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークのメタデータが更新された
    ArtworkMetadataUpdated {
        event_id: EventId,
        artwork_id: ArtworkId,
        old_metadata: ArtworkMetadata,
        new_metadata: ArtworkMetadata,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークのキャンバスが更新された
    ArtworkCanvasUpdated {
        event_id: EventId,
        artwork_id: ArtworkId,
        canvas_width: u16,
        canvas_height: u16,
        total_dots: usize,
        drawable_dots: usize,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークが削除された
    ArtworkDeleted {
        event_id: EventId,
        artwork_id: ArtworkId,
        artwork_name: String,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークの描画が開始された
    PaintingStarted {
        event_id: EventId,
        artwork_id: ArtworkId,
        total_dots_to_paint: usize,
        estimated_duration_seconds: u64,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// ドットが描画された
    DotPainted {
        event_id: EventId,
        artwork_id: ArtworkId,
        coordinates: Coordinates,
        color: crate::domain::shared::value_objects::Color,
        sequence_number: u32,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// 描画が一時停止された
    PaintingPaused {
        event_id: EventId,
        artwork_id: ArtworkId,
        painted_dots: usize,
        remaining_dots: usize,
        completion_ratio: f64,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// 描画が再開された
    PaintingResumed {
        event_id: EventId,
        artwork_id: ArtworkId,
        remaining_dots: usize,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークの描画が完了した
    PaintingCompleted {
        event_id: EventId,
        artwork_id: ArtworkId,
        total_dots_painted: usize,
        painting_duration_seconds: u64,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// 描画がキャンセルされた
    PaintingCancelled {
        event_id: EventId,
        artwork_id: ArtworkId,
        painted_dots: usize,
        completion_ratio: f64,
        reason: String,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// 描画エラーが発生した
    PaintingErrorOccurred {
        event_id: EventId,
        artwork_id: ArtworkId,
        coordinates: Option<Coordinates>,
        error_message: String,
        retry_count: u32,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
    /// アートワークがリセットされた
    ArtworkReset {
        event_id: EventId,
        artwork_id: ArtworkId,
        previous_completion_ratio: f64,
        occurred_at: Timestamp,
        version: u32,
        event_metadata: EventMetadata,
    },
}

impl ArtworkEvent {
    /// アートワーク作成イベントを作成
    pub fn artwork_created(
        artwork_id: ArtworkId,
        metadata: ArtworkMetadata,
        original_format: String,
        canvas: &Canvas,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::ArtworkCreated {
            event_id: EventId::generate(),
            artwork_id,
            metadata,
            original_format,
            canvas_width: canvas.width,
            canvas_height: canvas.height,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// メタデータ更新イベントを作成
    pub fn metadata_updated(
        artwork_id: ArtworkId,
        old_metadata: ArtworkMetadata,
        new_metadata: ArtworkMetadata,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::ArtworkMetadataUpdated {
            event_id: EventId::generate(),
            artwork_id,
            old_metadata,
            new_metadata,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// キャンバス更新イベントを作成
    pub fn canvas_updated(
        artwork_id: ArtworkId,
        canvas: &Canvas,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        let drawable_dots = canvas.drawable_dots().len();
        Self::ArtworkCanvasUpdated {
            event_id: EventId::generate(),
            artwork_id,
            canvas_width: canvas.width,
            canvas_height: canvas.height,
            total_dots: canvas.dots.len(),
            drawable_dots,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// アートワーク削除イベントを作成
    pub fn artwork_deleted(
        artwork_id: ArtworkId,
        artwork_name: String,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::ArtworkDeleted {
            event_id: EventId::generate(),
            artwork_id,
            artwork_name,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画開始イベントを作成
    pub fn painting_started(
        artwork_id: ArtworkId,
        total_dots_to_paint: usize,
        estimated_duration_seconds: u64,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingStarted {
            event_id: EventId::generate(),
            artwork_id,
            total_dots_to_paint,
            estimated_duration_seconds,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// ドット描画イベントを作成
    pub fn dot_painted(
        artwork_id: ArtworkId,
        coordinates: Coordinates,
        color: crate::domain::shared::value_objects::Color,
        sequence_number: u32,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::DotPainted {
            event_id: EventId::generate(),
            artwork_id,
            coordinates,
            color,
            sequence_number,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画一時停止イベントを作成
    pub fn painting_paused(
        artwork_id: ArtworkId,
        painted_dots: usize,
        remaining_dots: usize,
        completion_ratio: f64,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingPaused {
            event_id: EventId::generate(),
            artwork_id,
            painted_dots,
            remaining_dots,
            completion_ratio,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画再開イベントを作成
    pub fn painting_resumed(
        artwork_id: ArtworkId,
        remaining_dots: usize,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingResumed {
            event_id: EventId::generate(),
            artwork_id,
            remaining_dots,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画完了イベントを作成
    pub fn painting_completed(
        artwork_id: ArtworkId,
        total_dots_painted: usize,
        painting_duration_seconds: u64,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingCompleted {
            event_id: EventId::generate(),
            artwork_id,
            total_dots_painted,
            painting_duration_seconds,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画キャンセルイベントを作成
    pub fn painting_cancelled(
        artwork_id: ArtworkId,
        painted_dots: usize,
        completion_ratio: f64,
        reason: String,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingCancelled {
            event_id: EventId::generate(),
            artwork_id,
            painted_dots,
            completion_ratio,
            reason,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// 描画エラーイベントを作成
    pub fn painting_error_occurred(
        artwork_id: ArtworkId,
        coordinates: Option<Coordinates>,
        error_message: String,
        retry_count: u32,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::PaintingErrorOccurred {
            event_id: EventId::generate(),
            artwork_id,
            coordinates,
            error_message,
            retry_count,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// アートワークリセットイベントを作成
    pub fn artwork_reset(
        artwork_id: ArtworkId,
        previous_completion_ratio: f64,
        version: u32,
        event_metadata: EventMetadata,
    ) -> Self {
        Self::ArtworkReset {
            event_id: EventId::generate(),
            artwork_id,
            previous_completion_ratio,
            occurred_at: Timestamp::now(),
            version,
            event_metadata,
        }
    }

    /// イベントの重要度を取得
    pub fn severity(&self) -> EventSeverity {
        match self {
            Self::ArtworkCreated { .. } => EventSeverity::Info,
            Self::ArtworkMetadataUpdated { .. } => EventSeverity::Info,
            Self::ArtworkCanvasUpdated { .. } => EventSeverity::Info,
            Self::ArtworkDeleted { .. } => EventSeverity::Warning,
            Self::PaintingStarted { .. } => EventSeverity::Info,
            Self::DotPainted { .. } => EventSeverity::Debug,
            Self::PaintingPaused { .. } => EventSeverity::Info,
            Self::PaintingResumed { .. } => EventSeverity::Info,
            Self::PaintingCompleted { .. } => EventSeverity::Info,
            Self::PaintingCancelled { .. } => EventSeverity::Warning,
            Self::PaintingErrorOccurred { .. } => EventSeverity::Error,
            Self::ArtworkReset { .. } => EventSeverity::Warning,
        }
    }

    /// イベントがユーザーに表示すべきかチェック
    pub fn should_notify_user(&self) -> bool {
        match self.severity() {
            EventSeverity::Error | EventSeverity::Warning => true,
            EventSeverity::Info => matches!(
                self,
                Self::PaintingStarted { .. }
                    | Self::PaintingCompleted { .. }
                    | Self::ArtworkCreated { .. }
            ),
            EventSeverity::Debug => false,
        }
    }

    /// イベントのカテゴリを取得
    pub fn category(&self) -> EventCategory {
        match self {
            Self::ArtworkCreated { .. }
            | Self::ArtworkMetadataUpdated { .. }
            | Self::ArtworkCanvasUpdated { .. }
            | Self::ArtworkDeleted { .. }
            | Self::ArtworkReset { .. } => EventCategory::Artwork,

            Self::PaintingStarted { .. }
            | Self::DotPainted { .. }
            | Self::PaintingPaused { .. }
            | Self::PaintingResumed { .. }
            | Self::PaintingCompleted { .. }
            | Self::PaintingCancelled { .. }
            | Self::PaintingErrorOccurred { .. } => EventCategory::Painting,
        }
    }

    /// イベントに関連する座標を取得
    pub fn coordinates(&self) -> Option<Coordinates> {
        match self {
            Self::DotPainted { coordinates, .. } => Some(*coordinates),
            Self::PaintingErrorOccurred { coordinates, .. } => *coordinates,
            _ => None,
        }
    }

    /// イベントのサマリーメッセージを取得
    pub fn summary(&self) -> String {
        match self {
            Self::ArtworkCreated { metadata, .. } => {
                format!("アートワーク「{}」が作成されました", metadata.name)
            }
            Self::ArtworkMetadataUpdated { new_metadata, .. } => {
                format!(
                    "アートワーク「{}」のメタデータが更新されました",
                    new_metadata.name
                )
            }
            Self::ArtworkCanvasUpdated { drawable_dots, .. } => {
                format!(
                    "キャンバスが更新されました（描画可能ドット: {drawable_dots}個）"
                )
            }
            Self::ArtworkDeleted { artwork_name, .. } => {
                format!("アートワーク「{artwork_name}」が削除されました")
            }
            Self::PaintingStarted {
                total_dots_to_paint,
                ..
            } => {
                format!("描画を開始しました（{total_dots_to_paint}個のドット）")
            }
            Self::DotPainted {
                coordinates,
                sequence_number,
                ..
            } => {
                format!(
                    "ドット #{sequence_number} を座標 {coordinates} に描画しました"
                )
            }
            Self::PaintingPaused {
                completion_ratio, ..
            } => {
                format!(
                    "描画を一時停止しました（進捗: {:.1}%）",
                    completion_ratio * 100.0
                )
            }
            Self::PaintingResumed { remaining_dots, .. } => {
                format!("描画を再開しました（残り: {remaining_dots}個）")
            }
            Self::PaintingCompleted {
                total_dots_painted,
                painting_duration_seconds,
                ..
            } => {
                format!(
                    "描画が完了しました（{total_dots_painted}個のドット、{painting_duration_seconds}秒）"
                )
            }
            Self::PaintingCancelled {
                completion_ratio,
                reason,
                ..
            } => {
                format!(
                    "描画がキャンセルされました（進捗: {:.1}%、理由: {}）",
                    completion_ratio * 100.0,
                    reason
                )
            }
            Self::PaintingErrorOccurred {
                error_message,
                retry_count,
                ..
            } => {
                format!(
                    "描画エラーが発生しました（リトライ: {retry_count}回、エラー: {error_message}）"
                )
            }
            Self::ArtworkReset {
                previous_completion_ratio,
                ..
            } => {
                format!(
                    "アートワークがリセットされました（以前の進捗: {:.1}%）",
                    previous_completion_ratio * 100.0
                )
            }
        }
    }
}

impl DomainEvent for ArtworkEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::ArtworkCreated { .. } => "ArtworkCreated",
            Self::ArtworkMetadataUpdated { .. } => "ArtworkMetadataUpdated",
            Self::ArtworkCanvasUpdated { .. } => "ArtworkCanvasUpdated",
            Self::ArtworkDeleted { .. } => "ArtworkDeleted",
            Self::PaintingStarted { .. } => "PaintingStarted",
            Self::DotPainted { .. } => "DotPainted",
            Self::PaintingPaused { .. } => "PaintingPaused",
            Self::PaintingResumed { .. } => "PaintingResumed",
            Self::PaintingCompleted { .. } => "PaintingCompleted",
            Self::PaintingCancelled { .. } => "PaintingCancelled",
            Self::PaintingErrorOccurred { .. } => "PaintingErrorOccurred",
            Self::ArtworkReset { .. } => "ArtworkReset",
        }
    }

    fn event_id(&self) -> &EventId {
        match self {
            Self::ArtworkCreated { event_id, .. }
            | Self::ArtworkMetadataUpdated { event_id, .. }
            | Self::ArtworkCanvasUpdated { event_id, .. }
            | Self::ArtworkDeleted { event_id, .. }
            | Self::PaintingStarted { event_id, .. }
            | Self::DotPainted { event_id, .. }
            | Self::PaintingPaused { event_id, .. }
            | Self::PaintingResumed { event_id, .. }
            | Self::PaintingCompleted { event_id, .. }
            | Self::PaintingCancelled { event_id, .. }
            | Self::PaintingErrorOccurred { event_id, .. }
            | Self::ArtworkReset { event_id, .. } => event_id,
        }
    }

    fn occurred_at(&self) -> Timestamp {
        match self {
            Self::ArtworkCreated { occurred_at, .. }
            | Self::ArtworkMetadataUpdated { occurred_at, .. }
            | Self::ArtworkCanvasUpdated { occurred_at, .. }
            | Self::ArtworkDeleted { occurred_at, .. }
            | Self::PaintingStarted { occurred_at, .. }
            | Self::DotPainted { occurred_at, .. }
            | Self::PaintingPaused { occurred_at, .. }
            | Self::PaintingResumed { occurred_at, .. }
            | Self::PaintingCompleted { occurred_at, .. }
            | Self::PaintingCancelled { occurred_at, .. }
            | Self::PaintingErrorOccurred { occurred_at, .. }
            | Self::ArtworkReset { occurred_at, .. } => *occurred_at,
        }
    }

    fn version(&self) -> u32 {
        match self {
            Self::ArtworkCreated { version, .. }
            | Self::ArtworkMetadataUpdated { version, .. }
            | Self::ArtworkCanvasUpdated { version, .. }
            | Self::ArtworkDeleted { version, .. }
            | Self::PaintingStarted { version, .. }
            | Self::DotPainted { version, .. }
            | Self::PaintingPaused { version, .. }
            | Self::PaintingResumed { version, .. }
            | Self::PaintingCompleted { version, .. }
            | Self::PaintingCancelled { version, .. }
            | Self::PaintingErrorOccurred { version, .. }
            | Self::ArtworkReset { version, .. } => *version,
        }
    }

    fn aggregate_id(&self) -> String {
        match self {
            Self::ArtworkCreated { artwork_id, .. }
            | Self::ArtworkMetadataUpdated { artwork_id, .. }
            | Self::ArtworkCanvasUpdated { artwork_id, .. }
            | Self::ArtworkDeleted { artwork_id, .. }
            | Self::PaintingStarted { artwork_id, .. }
            | Self::DotPainted { artwork_id, .. }
            | Self::PaintingPaused { artwork_id, .. }
            | Self::PaintingResumed { artwork_id, .. }
            | Self::PaintingCompleted { artwork_id, .. }
            | Self::PaintingCancelled { artwork_id, .. }
            | Self::PaintingErrorOccurred { artwork_id, .. }
            | Self::ArtworkReset { artwork_id, .. } => artwork_id.as_str(),
        }
    }

    fn as_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    fn metadata(&self) -> &EventMetadata {
        match self {
            Self::ArtworkCreated { event_metadata, .. }
            | Self::ArtworkMetadataUpdated { event_metadata, .. }
            | Self::ArtworkCanvasUpdated { event_metadata, .. }
            | Self::ArtworkDeleted { event_metadata, .. }
            | Self::PaintingStarted { event_metadata, .. }
            | Self::DotPainted { event_metadata, .. }
            | Self::PaintingPaused { event_metadata, .. }
            | Self::PaintingResumed { event_metadata, .. }
            | Self::PaintingCompleted { event_metadata, .. }
            | Self::PaintingCancelled { event_metadata, .. }
            | Self::PaintingErrorOccurred { event_metadata, .. }
            | Self::ArtworkReset { event_metadata, .. } => event_metadata,
        }
    }
}

/// イベントの重要度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventSeverity {
    Debug,
    Info,
    Warning,
    Error,
}

impl EventSeverity {
    /// 重要度が指定レベル以上かチェック
    pub fn is_at_least(&self, level: EventSeverity) -> bool {
        *self >= level
    }

    /// 重要度を文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Error => "ERROR",
        }
    }
}

/// イベントのカテゴリ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    Artwork,
    Painting,
}

impl EventCategory {
    /// カテゴリを文字列として取得
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Artwork => "ARTWORK",
            Self::Painting => "PAINTING",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artwork::entities::Canvas;

    #[test]
    fn test_artwork_event_creation() {
        let artwork_id = ArtworkId::generate();
        let metadata = ArtworkMetadata::new("Test Artwork".to_string());
        let canvas = Canvas::new(10, 10);
        let event_metadata = EventMetadata::new("test".to_string());

        let event = ArtworkEvent::artwork_created(
            artwork_id.clone(),
            metadata.clone(),
            "png".to_string(),
            &canvas,
            1,
            event_metadata,
        );

        assert_eq!(event.event_type(), "ArtworkCreated");
        assert_eq!(event.aggregate_id(), artwork_id.as_str());
        assert_eq!(event.version(), 1);
        assert_eq!(event.category(), EventCategory::Artwork);
        assert_eq!(event.severity(), EventSeverity::Info);
        assert!(event.should_notify_user());
    }

    #[test]
    fn test_dot_painted_event() {
        let artwork_id = ArtworkId::generate();
        let coordinates = Coordinates::new(5, 5);
        let color = crate::domain::shared::value_objects::Color::red();
        let event_metadata = EventMetadata::new("test".to_string());

        let event =
            ArtworkEvent::dot_painted(artwork_id.clone(), coordinates, color, 1, 2, event_metadata);

        assert_eq!(event.event_type(), "DotPainted");
        assert_eq!(event.coordinates(), Some(coordinates));
        assert_eq!(event.category(), EventCategory::Painting);
        assert_eq!(event.severity(), EventSeverity::Debug);
        assert!(!event.should_notify_user());
    }

    #[test]
    fn test_painting_error_event() {
        let artwork_id = ArtworkId::generate();
        let coordinates = Some(Coordinates::new(10, 10));
        let event_metadata = EventMetadata::new("test".to_string());

        let event = ArtworkEvent::painting_error_occurred(
            artwork_id.clone(),
            coordinates,
            "Connection lost".to_string(),
            3,
            5,
            event_metadata,
        );

        assert_eq!(event.event_type(), "PaintingErrorOccurred");
        assert_eq!(event.coordinates(), coordinates);
        assert_eq!(event.severity(), EventSeverity::Error);
        assert!(event.should_notify_user());

        let summary = event.summary();
        assert!(summary.contains("描画エラーが発生しました"));
        assert!(summary.contains("リトライ: 3回"));
    }

    #[test]
    fn test_event_severity_comparison() {
        assert!(EventSeverity::Error > EventSeverity::Warning);
        assert!(EventSeverity::Warning > EventSeverity::Info);
        assert!(EventSeverity::Info > EventSeverity::Debug);

        assert!(EventSeverity::Error.is_at_least(EventSeverity::Warning));
        assert!(!EventSeverity::Debug.is_at_least(EventSeverity::Info));
    }

    #[test]
    fn test_event_serialization() {
        let artwork_id = ArtworkId::generate();
        let event_metadata = EventMetadata::new("test".to_string());

        let event = ArtworkEvent::painting_completed(artwork_id, 100, 300, 10, event_metadata);

        let json = event.as_json().unwrap();
        assert!(!json.is_empty());

        // JSONから復元できることを確認
        let deserialized: ArtworkEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.event_type(), "PaintingCompleted");
    }
}
