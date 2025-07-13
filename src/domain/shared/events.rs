//! 共有ドメインイベント
//!
//! システム全体で使用されるドメインイベントを定義

use crate::domain::shared::value_objects::Timestamp;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// イベントID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(Uuid);

impl EventId {
    /// 新しいイベントIDを生成
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    /// UUIDから作成
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// 文字列から作成
    pub fn parse(s: &str) -> Result<Self, String> {
        let uuid = Uuid::parse_str(s).map_err(|e| format!("Invalid UUID format: {e}"))?;
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

impl FromStr for EventId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for EventId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<EventId> for Uuid {
    fn from(id: EventId) -> Self {
        id.0
    }
}

/// ドメインイベントのベーストレイト
pub trait DomainEvent: Send + Sync + fmt::Debug {
    /// イベントタイプを取得
    fn event_type(&self) -> &'static str;

    /// イベントIDを取得
    fn event_id(&self) -> &EventId;

    /// 発生時刻を取得
    fn occurred_at(&self) -> Timestamp;

    /// イベントバージョンを取得
    fn version(&self) -> u32;

    /// 集約IDを取得
    fn aggregate_id(&self) -> String;

    /// イベントデータをJSONとしてシリアライズ
    fn as_json(&self) -> Result<String, serde_json::Error>;

    /// イベントメタデータを取得
    fn metadata(&self) -> &EventMetadata;
}

/// イベントメタデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Option<String>,
    pub causation_id: Option<EventId>,
    pub user_id: Option<String>,
    pub source: String,
    pub custom_properties: HashMap<String, String>,
}

impl EventMetadata {
    /// 新しいメタデータを作成
    pub fn new(source: String) -> Self {
        Self {
            correlation_id: None,
            causation_id: None,
            user_id: None,
            source,
            custom_properties: HashMap::new(),
        }
    }

    /// 相関IDを設定
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// 原因イベントIDを設定
    pub fn with_causation_id(mut self, causation_id: EventId) -> Self {
        self.causation_id = Some(causation_id);
        self
    }

    /// ユーザーIDを設定
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// カスタムプロパティを追加
    pub fn add_property(mut self, key: String, value: String) -> Self {
        self.custom_properties.insert(key, value);
        self
    }

    /// カスタムプロパティを取得
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.custom_properties.get(key)
    }
}

impl Default for EventMetadata {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}

/// イベントエンベロープ
///
/// イベントストアでの保存や転送に使用される包装構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub event_id: EventId,
    pub event_type: String,
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub version: u32,
    pub occurred_at: Timestamp,
    pub data: serde_json::Value,
    pub metadata: EventMetadata,
}

impl EventEnvelope {
    /// 新しいエンベロープを作成
    pub fn new<E: DomainEvent + Serialize>(
        event: &E,
        aggregate_type: String,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            event_id: event.event_id().clone(),
            event_type: event.event_type().to_string(),
            aggregate_id: event.aggregate_id(),
            aggregate_type,
            version: event.version(),
            occurred_at: event.occurred_at(),
            data: serde_json::to_value(event)?,
            metadata: event.metadata().clone(),
        })
    }

    /// イベントデータをデシリアライズ
    pub fn deserialize_data<T>(&self) -> Result<T, serde_json::Error>
    where
        T: for<'de> Deserialize<'de>,
    {
        serde_json::from_value(self.data.clone())
    }

    /// イベントの年齢を取得（ミリ秒）
    pub fn age_millis(&self) -> u64 {
        self.occurred_at.elapsed_millis()
    }

    /// イベントが古いかチェック
    pub fn is_older_than(&self, duration_millis: u64) -> bool {
        self.age_millis() > duration_millis
    }
}

/// イベントストリーム
///
/// 特定の集約に関連するイベントのシーケンス
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStream {
    pub aggregate_id: String,
    pub aggregate_type: String,
    pub events: Vec<EventEnvelope>,
    pub version: u32,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl EventStream {
    /// 新しいイベントストリームを作成
    pub fn new(aggregate_id: String, aggregate_type: String) -> Self {
        let now = Timestamp::now();
        Self {
            aggregate_id,
            aggregate_type,
            events: Vec::new(),
            version: 0,
            created_at: now,
            updated_at: now,
        }
    }

    /// イベントを追加
    pub fn append_event(&mut self, envelope: EventEnvelope) -> Result<(), EventStreamError> {
        // バージョンチェック
        if envelope.version != self.version + 1 {
            return Err(EventStreamError::VersionMismatch {
                expected: self.version + 1,
                actual: envelope.version,
            });
        }

        // 集約IDチェック
        if envelope.aggregate_id != self.aggregate_id {
            return Err(EventStreamError::AggregateIdMismatch {
                expected: self.aggregate_id.clone(),
                actual: envelope.aggregate_id,
            });
        }

        self.events.push(envelope);
        self.version += 1;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// 指定されたバージョン以降のイベントを取得
    pub fn events_since_version(&self, version: u32) -> Vec<&EventEnvelope> {
        self.events.iter().filter(|e| e.version > version).collect()
    }

    /// 最新のイベントを取得
    pub fn latest_event(&self) -> Option<&EventEnvelope> {
        self.events.last()
    }

    /// イベント数を取得
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// ストリームが空かチェック
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// 指定されたイベントタイプのイベントを取得
    pub fn events_of_type(&self, event_type: &str) -> Vec<&EventEnvelope> {
        self.events
            .iter()
            .filter(|e| e.event_type == event_type)
            .collect()
    }

    /// 指定された期間のイベントを取得
    pub fn events_in_range(&self, start: Timestamp, end: Timestamp) -> Vec<&EventEnvelope> {
        self.events
            .iter()
            .filter(|e| {
                e.occurred_at.epoch_millis >= start.epoch_millis
                    && e.occurred_at.epoch_millis <= end.epoch_millis
            })
            .collect()
    }

    /// ストリームの統計情報を取得
    pub fn statistics(&self) -> EventStreamStatistics {
        let event_types: std::collections::HashSet<String> =
            self.events.iter().map(|e| e.event_type.clone()).collect();

        let first_event_time = self.events.first().map(|e| e.occurred_at);
        let last_event_time = self.events.last().map(|e| e.occurred_at);

        EventStreamStatistics {
            total_events: self.event_count(),
            unique_event_types: event_types.len(),
            current_version: self.version,
            first_event_at: first_event_time,
            last_event_at: last_event_time,
            stream_age_millis: self.created_at.elapsed_millis(),
        }
    }
}

/// イベントストリームの統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStreamStatistics {
    pub total_events: usize,
    pub unique_event_types: usize,
    pub current_version: u32,
    pub first_event_at: Option<Timestamp>,
    pub last_event_at: Option<Timestamp>,
    pub stream_age_millis: u64,
}

/// イベントストリームエラー
#[derive(Debug, Clone, thiserror::Error)]
pub enum EventStreamError {
    #[error("Version mismatch: expected {expected}, got {actual}")]
    VersionMismatch { expected: u32, actual: u32 },
    #[error("Aggregate ID mismatch: expected {expected}, got {actual}")]
    AggregateIdMismatch { expected: String, actual: String },
    #[error("Event serialization error: {message}")]
    SerializationError { message: String },
    #[error("Invalid event data: {message}")]
    InvalidEventData { message: String },
}

/// イベントハンドラートレイト
#[async_trait::async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// イベントを処理
    async fn handle(&self, event: &E) -> Result<(), Self::Error>;

    /// ハンドラーが処理できるイベントタイプ
    fn can_handle(&self, event_type: &str) -> bool;

    /// ハンドラーの名前
    fn name(&self) -> &'static str;
}

/// イベントハンドラーのエラー型
type HandlerError = Box<dyn std::error::Error + Send + Sync>;

/// イベントハンドラーのボックス型
type BoxedEventHandler = Box<dyn EventHandler<dyn DomainEvent, Error = HandlerError>>;

/// イベントディスパッチャー
pub struct EventDispatcher {
    handlers: HashMap<String, Vec<BoxedEventHandler>>,
}

impl EventDispatcher {
    /// 新しいディスパッチャーを作成
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// イベントハンドラーを登録
    pub fn register_handler<E: DomainEvent + 'static>(
        &mut self,
        event_type: String,
        _handler: Box<dyn EventHandler<E, Error = Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        // Note: 実際の実装では型安全性を保つためにより複雑な設計が必要
        // ここでは簡略化した実装を示す
        self.handlers.entry(event_type).or_default();
        // .push(handler);
    }

    /// イベントをディスパッチ
    pub async fn dispatch<E: DomainEvent>(
        &self,
        event: &E,
    ) -> Result<(), Vec<Box<dyn std::error::Error + Send + Sync>>> {
        let event_type = event.event_type();
        let errors = Vec::new();

        if let Some(handlers) = self.handlers.get(event_type) {
            for _handler in handlers {
                // Note: 実際の実装では型キャストが必要
                // if let Err(e) = handler.handle(event).await {
                //     errors.push(e);
                // }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 登録されているハンドラー数を取得
    pub fn handler_count(&self) -> usize {
        self.handlers.values().map(|v| v.len()).sum()
    }

    /// 特定のイベントタイプのハンドラー数を取得
    pub fn handler_count_for_type(&self, event_type: &str) -> usize {
        self.handlers.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for EventDispatcher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EventDispatcher")
            .field("handler_count", &self.handler_count())
            .finish()
    }
}

/// イベントストアトレイト
#[async_trait::async_trait]
pub trait EventStore: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// イベントを保存
    async fn save_events(
        &self,
        aggregate_id: &str,
        expected_version: u32,
        events: Vec<EventEnvelope>,
    ) -> Result<(), Self::Error>;

    /// イベントストリームを取得
    async fn get_events(
        &self,
        aggregate_id: &str,
        from_version: Option<u32>,
    ) -> Result<EventStream, Self::Error>;

    /// すべてのイベントを取得（ページネーション付き）
    async fn get_all_events(
        &self,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<EventEnvelope>, Self::Error>;

    /// 特定のイベントタイプのイベントを取得
    async fn get_events_by_type(
        &self,
        event_type: &str,
        limit: Option<usize>,
    ) -> Result<Vec<EventEnvelope>, Self::Error>;

    /// イベントストアの初期化
    async fn initialize(&self) -> Result<(), Self::Error>;

    /// イベントストアのクリーンアップ
    async fn cleanup(&self) -> Result<(), Self::Error>;

    /// ヘルスチェック
    async fn health_check(&self) -> Result<EventStoreHealth, Self::Error>;
}

/// イベントストアのヘルス状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStoreHealth {
    pub is_healthy: bool,
    pub total_events: usize,
    pub total_streams: usize,
    pub storage_usage_bytes: u64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
}

impl EventStoreHealth {
    pub fn healthy() -> Self {
        Self {
            is_healthy: true,
            total_events: 0,
            total_streams: 0,
            storage_usage_bytes: 0,
            average_response_time_ms: 0.0,
            error_rate: 0.0,
        }
    }

    pub fn unhealthy() -> Self {
        Self {
            is_healthy: false,
            total_events: 0,
            total_streams: 0,
            storage_usage_bytes: 0,
            average_response_time_ms: 0.0,
            error_rate: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_id() {
        let id1 = EventId::generate();
        let id2 = EventId::generate();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id_from_uuid = EventId::from_uuid(uuid);
        assert_eq!(id_from_uuid.as_uuid(), uuid);

        let id_str = id1.as_str();
        let id_from_str = EventId::from_str(&id_str).unwrap();
        assert_eq!(id1, id_from_str);
    }

    #[test]
    fn test_event_metadata() {
        let metadata = EventMetadata::new("test-source".to_string())
            .with_correlation_id("corr-123".to_string())
            .with_user_id("user-456".to_string())
            .add_property("key1".to_string(), "value1".to_string());

        assert_eq!(metadata.source, "test-source");
        assert_eq!(metadata.correlation_id, Some("corr-123".to_string()));
        assert_eq!(metadata.user_id, Some("user-456".to_string()));
        assert_eq!(metadata.get_property("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_event_stream() {
        let stream = EventStream::new("agg-123".to_string(), "TestAggregate".to_string());
        assert!(stream.is_empty());
        assert_eq!(stream.version, 0);

        // Note: 実際のテストではEventEnvelopeを作成してテストする
        let stats = stream.statistics();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.current_version, 0);
    }

    #[test]
    fn test_event_dispatcher() {
        let dispatcher = EventDispatcher::new();
        assert_eq!(dispatcher.handler_count(), 0);
        assert_eq!(dispatcher.handler_count_for_type("TestEvent"), 0);
    }

    #[test]
    fn test_event_envelope_age() {
        let envelope = EventEnvelope {
            event_id: EventId::generate(),
            event_type: "TestEvent".to_string(),
            aggregate_id: "agg-123".to_string(),
            aggregate_type: "TestAggregate".to_string(),
            version: 1,
            occurred_at: Timestamp::now(),
            data: serde_json::Value::Null,
            metadata: EventMetadata::default(),
        };

        // 新しく作成されたイベントは古くない
        assert!(!envelope.is_older_than(1000));

        // 年齢は0に近い値になる
        assert!(envelope.age_millis() < 100);
    }
}
