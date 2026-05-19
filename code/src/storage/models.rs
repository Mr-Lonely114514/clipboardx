use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// 剪切板记录数据模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipRecord {
    pub id: i64,
    pub content_hash: String,
    pub content_type: String, // "text" | "image" | "file"
    pub content_text: Option<String>,
    pub content_blob: Option<Vec<u8>>,
    pub source_app: Option<String>,
    pub source: String, // "local_clipboard" | "cloud_sync"
    pub is_favorite: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 用于插入新记录的简化结构
#[derive(Debug, Clone)]
pub struct NewClipRecord {
    pub content_hash: String,
    pub content_type: String,
    pub content_text: Option<String>,
    pub content_blob: Option<Vec<u8>>,
    pub source_app: Option<String>,
    pub source: String,
}

/// 搜索查询参数
#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub keyword: Option<String>,
    pub content_type_filter: Option<String>,
    pub favorite_only: bool,
    pub limit: i64,
    pub offset: i64,
}
