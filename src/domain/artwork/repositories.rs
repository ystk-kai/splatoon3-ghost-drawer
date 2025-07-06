//! アートワーク集約のリポジトリトレイト
//! 
//! アートワークの永続化に関するトレイトを定義

use crate::domain::artwork::entities::{Artwork, ArtworkId, ArtworkMetadata, ArtworkStatistics};
use crate::domain::shared::value_objects::Timestamp;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// アートワークリポジトリのエラー
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum RepositoryError {
    #[error("Artwork not found: {id}")]
    NotFound { id: ArtworkId },
    #[error("Artwork already exists: {id}")]
    AlreadyExists { id: ArtworkId },
    #[error("Database connection error: {message}")]
    ConnectionError { message: String },
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    #[error("Validation error: {message}")]
    ValidationError { message: String },
    #[error("Permission denied for artwork: {id}")]
    PermissionDenied { id: ArtworkId },
    #[error("Storage quota exceeded")]
    QuotaExceeded,
    #[error("Concurrent modification detected")]
    ConcurrentModification,
    #[error("Invalid query parameters: {message}")]
    InvalidQuery { message: String },
    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl RepositoryError {
    /// エラーが一時的なものかチェック
    pub fn is_transient(&self) -> bool {
        matches!(self, 
            Self::ConnectionError { .. } | 
            Self::ConcurrentModification |
            Self::Internal { .. }
        )
    }

    /// エラーがクライアント側の問題かチェック
    pub fn is_client_error(&self) -> bool {
        matches!(self,
            Self::NotFound { .. } |
            Self::AlreadyExists { .. } |
            Self::ValidationError { .. } |
            Self::PermissionDenied { .. } |
            Self::InvalidQuery { .. }
        )
    }
}

/// アートワーク検索クエリ
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtworkQuery {
    /// ID による検索
    pub ids: Option<Vec<ArtworkId>>,
    /// 名前による部分一致検索
    pub name_contains: Option<String>,
    /// タグによる検索
    pub tags: Option<Vec<String>>,
    /// 作者による検索
    pub author: Option<String>,
    /// 作成日時範囲
    pub created_after: Option<Timestamp>,
    pub created_before: Option<Timestamp>,
    /// 更新日時範囲
    pub updated_after: Option<Timestamp>,
    pub updated_before: Option<Timestamp>,
    /// ファイル形式による検索
    pub format: Option<String>,
    /// 完成度による検索
    pub min_completion: Option<f64>,
    pub max_completion: Option<f64>,
    /// 複雑度による検索
    pub min_complexity: Option<f64>,
    pub max_complexity: Option<f64>,
    /// ソート設定
    pub sort_by: Option<SortField>,
    pub sort_order: Option<SortOrder>,
    /// ページネーション
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl ArtworkQuery {
    /// 新しいクエリを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// IDで検索するクエリ
    pub fn by_id(id: ArtworkId) -> Self {
        Self {
            ids: Some(vec![id]),
            ..Default::default()
        }
    }

    /// 複数IDで検索するクエリ
    pub fn by_ids(ids: Vec<ArtworkId>) -> Self {
        Self {
            ids: Some(ids),
            ..Default::default()
        }
    }

    /// 名前で検索するクエリ
    pub fn by_name_contains(name: String) -> Self {
        Self {
            name_contains: Some(name),
            ..Default::default()
        }
    }

    /// タグで検索するクエリ
    pub fn by_tags(tags: Vec<String>) -> Self {
        Self {
            tags: Some(tags),
            ..Default::default()
        }
    }

    /// 作者で検索するクエリ
    pub fn by_author(author: String) -> Self {
        Self {
            author: Some(author),
            ..Default::default()
        }
    }

    /// 最近作成されたアートワークを検索するクエリ
    pub fn recent(limit: usize) -> Self {
        Self {
            sort_by: Some(SortField::CreatedAt),
            sort_order: Some(SortOrder::Descending),
            limit: Some(limit),
            ..Default::default()
        }
    }

    /// ページネーションを設定
    pub fn with_pagination(mut self, limit: usize, offset: usize) -> Self {
        self.limit = Some(limit);
        self.offset = Some(offset);
        self
    }

    /// ソートを設定
    pub fn with_sort(mut self, field: SortField, order: SortOrder) -> Self {
        self.sort_by = Some(field);
        self.sort_order = Some(order);
        self
    }

    /// 日付範囲を設定
    pub fn with_date_range(mut self, after: Option<Timestamp>, before: Option<Timestamp>) -> Self {
        self.created_after = after;
        self.created_before = before;
        self
    }

    /// クエリの検証
    pub fn validate(&self) -> Result<(), RepositoryError> {
        if let Some(limit) = self.limit {
            if limit == 0 || limit > 1000 {
                return Err(RepositoryError::InvalidQuery {
                    message: "Limit must be between 1 and 1000".to_string(),
                });
            }
        }

        if let (Some(after), Some(before)) = (&self.created_after, &self.created_before) {
            if after.epoch_millis >= before.epoch_millis {
                return Err(RepositoryError::InvalidQuery {
                    message: "created_after must be before created_before".to_string(),
                });
            }
        }

        if let (Some(min), Some(max)) = (self.min_completion, self.max_completion) {
            if min < 0.0 || max > 1.0 || min > max {
                return Err(RepositoryError::InvalidQuery {
                    message: "Invalid completion range".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// ソートフィールド
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortField {
    Name,
    CreatedAt,
    UpdatedAt,
    CompletionRatio,
    ComplexityScore,
    TotalDots,
    FileSize,
}

/// ソート順序
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// 検索結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub artworks: Vec<Artwork>,
    pub total_count: usize,
    pub has_more: bool,
    pub query_time_ms: u64,
}

impl SearchResult {
    pub fn new(artworks: Vec<Artwork>, total_count: usize, has_more: bool, query_time_ms: u64) -> Self {
        Self {
            artworks,
            total_count,
            has_more,
            query_time_ms,
        }
    }

    pub fn empty() -> Self {
        Self {
            artworks: Vec::new(),
            total_count: 0,
            has_more: false,
            query_time_ms: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.artworks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.artworks.len()
    }
}

/// バッチ操作結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub successful: Vec<ArtworkId>,
    pub failed: HashMap<ArtworkId, RepositoryError>,
    pub total_processed: usize,
}

impl BatchResult {
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: HashMap::new(),
            total_processed: 0,
        }
    }

    pub fn add_success(&mut self, id: ArtworkId) {
        self.successful.push(id);
        self.total_processed += 1;
    }

    pub fn add_failure(&mut self, id: ArtworkId, error: RepositoryError) {
        self.failed.insert(id, error);
        self.total_processed += 1;
    }

    pub fn success_count(&self) -> usize {
        self.successful.len()
    }

    pub fn failure_count(&self) -> usize {
        self.failed.len()
    }

    pub fn is_all_successful(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn has_failures(&self) -> bool {
        !self.failed.is_empty()
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self::new()
    }
}

/// アートワークリポジトリトレイト
#[async_trait]
pub trait ArtworkRepository: Send + Sync {
    /// アートワークを保存
    async fn save(&self, artwork: &Artwork) -> Result<(), RepositoryError>;

    /// アートワークを取得
    async fn find_by_id(&self, id: &ArtworkId) -> Result<Option<Artwork>, RepositoryError>;

    /// アートワークを削除
    async fn delete(&self, id: &ArtworkId) -> Result<(), RepositoryError>;

    /// アートワークを検索
    async fn search(&self, query: &ArtworkQuery) -> Result<SearchResult, RepositoryError>;

    /// すべてのアートワークを取得
    async fn find_all(&self) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::new();
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// 複数のアートワークを取得
    async fn find_by_ids(&self, ids: &[ArtworkId]) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::by_ids(ids.to_vec());
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// アートワークが存在するかチェック
    async fn exists(&self, id: &ArtworkId) -> Result<bool, RepositoryError> {
        match self.find_by_id(id).await? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    /// アートワークの総数を取得
    async fn count(&self) -> Result<usize, RepositoryError> {
        let query = ArtworkQuery::new();
        let result = self.search(&query).await?;
        Ok(result.total_count)
    }

    /// アートワークの統計情報を取得
    async fn get_statistics(&self, id: &ArtworkId) -> Result<Option<ArtworkStatistics>, RepositoryError> {
        if let Some(artwork) = self.find_by_id(id).await? {
            Ok(Some(artwork.statistics()))
        } else {
            Ok(None)
        }
    }

    /// 複数のアートワークを一括保存
    async fn save_batch(&self, artworks: &[Artwork]) -> Result<BatchResult, RepositoryError> {
        let mut result = BatchResult::new();
        
        for artwork in artworks {
            match self.save(artwork).await {
                Ok(()) => result.add_success(artwork.id.clone()),
                Err(error) => result.add_failure(artwork.id.clone(), error),
            }
        }
        
        Ok(result)
    }

    /// 複数のアートワークを一括削除
    async fn delete_batch(&self, ids: &[ArtworkId]) -> Result<BatchResult, RepositoryError> {
        let mut result = BatchResult::new();
        
        for id in ids {
            match self.delete(id).await {
                Ok(()) => result.add_success(id.clone()),
                Err(error) => result.add_failure(id.clone(), error),
            }
        }
        
        Ok(result)
    }

    /// メタデータのみを更新
    async fn update_metadata(&self, id: &ArtworkId, metadata: &ArtworkMetadata) -> Result<(), RepositoryError> {
        if let Some(mut artwork) = self.find_by_id(id).await? {
            artwork.update_metadata(metadata.clone());
            self.save(&artwork).await
        } else {
            Err(RepositoryError::NotFound { id: id.clone() })
        }
    }

    /// 最近更新されたアートワークを取得
    async fn find_recently_updated(&self, limit: usize) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::new()
            .with_sort(SortField::UpdatedAt, SortOrder::Descending)
            .with_pagination(limit, 0);
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// 指定されたタグを持つアートワークを取得
    async fn find_by_tags(&self, tags: &[String]) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::by_tags(tags.to_vec());
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// 指定された作者のアートワークを取得
    async fn find_by_author(&self, author: &str) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::by_author(author.to_string());
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// 指定された期間のアートワークを取得
    async fn find_by_date_range(&self, start: Timestamp, end: Timestamp) -> Result<Vec<Artwork>, RepositoryError> {
        let query = ArtworkQuery::new().with_date_range(Some(start), Some(end));
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// 完成度による検索
    async fn find_by_completion_range(&self, min: f64, max: f64) -> Result<Vec<Artwork>, RepositoryError> {
        let mut query = ArtworkQuery::new();
        query.min_completion = Some(min);
        query.max_completion = Some(max);
        let result = self.search(&query).await?;
        Ok(result.artworks)
    }

    /// ヘルスチェック
    async fn health_check(&self) -> Result<RepositoryHealth, RepositoryError>;

    /// リポジトリの初期化
    async fn initialize(&self) -> Result<(), RepositoryError>;

    /// リポジトリのクリーンアップ
    async fn cleanup(&self) -> Result<(), RepositoryError>;
}

/// リポジトリのヘルス状態
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryHealth {
    pub is_healthy: bool,
    pub connection_status: ConnectionStatus,
    pub total_artworks: usize,
    pub storage_usage_bytes: u64,
    pub last_backup: Option<Timestamp>,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
}

/// 接続状態
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Degraded,
    Unknown,
}

impl RepositoryHealth {
    pub fn healthy() -> Self {
        Self {
            is_healthy: true,
            connection_status: ConnectionStatus::Connected,
            total_artworks: 0,
            storage_usage_bytes: 0,
            last_backup: None,
            error_rate: 0.0,
            average_response_time_ms: 0.0,
        }
    }

    pub fn unhealthy(_reason: &str) -> Self {
        Self {
            is_healthy: false,
            connection_status: ConnectionStatus::Disconnected,
            total_artworks: 0,
            storage_usage_bytes: 0,
            last_backup: None,
            error_rate: 1.0,
            average_response_time_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::artwork::entities::{ArtworkMetadata, Canvas};

    #[test]
    fn test_artwork_query_validation() {
        let mut query = ArtworkQuery::new();
        assert!(query.validate().is_ok());

        query.limit = Some(0);
        assert!(query.validate().is_err());

        query.limit = Some(500);
        assert!(query.validate().is_ok());

        query.min_completion = Some(0.5);
        query.max_completion = Some(0.3);
        assert!(query.validate().is_err());
    }

    #[test]
    fn test_batch_result() {
        let mut result = BatchResult::new();
        let id1 = ArtworkId::generate();
        let id2 = ArtworkId::generate();

        result.add_success(id1.clone());
        result.add_failure(id2.clone(), RepositoryError::NotFound { id: id2.clone() });

        assert_eq!(result.success_count(), 1);
        assert_eq!(result.failure_count(), 1);
        assert!(!result.is_all_successful());
        assert!(result.has_failures());
    }

    #[test]
    fn test_search_result() {
        let result = SearchResult::empty();
        assert!(result.is_empty());
        assert_eq!(result.len(), 0);

        let metadata = ArtworkMetadata::new("Test".to_string());
        let canvas = Canvas::new(10, 10);
        let artwork = Artwork::new(metadata, "png".to_string(), canvas);
        
        let result = SearchResult::new(vec![artwork], 1, false, 100);
        assert!(!result.is_empty());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_repository_error_classification() {
        let not_found = RepositoryError::NotFound { id: ArtworkId::generate() };
        assert!(not_found.is_client_error());
        assert!(!not_found.is_transient());

        let connection_error = RepositoryError::ConnectionError { 
            message: "Connection lost".to_string() 
        };
        assert!(!connection_error.is_client_error());
        assert!(connection_error.is_transient());
    }
} 