use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentType {
    Text,
    Code { language: String },
    Image { path: String, width: u32, height: u32 },
    Url { url: String, title: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: Option<i64>,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub content: String,
    pub source_app: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_synced: bool,
    pub sync_targets: Vec<String>,
    pub tags: Vec<String>,
}

impl Default for ClipboardItem {
    fn default() -> Self {
        Self {
            id: None,
            content_type: ContentType::Text,
            title: None,
            content: String::new(),
            source_app: None,
            created_at: Utc::now(),
            is_synced: false,
            sync_targets: Vec::new(),
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncTarget {
    Notion,
    Server,
    Obsidian,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub item_id: i64,
    pub target: SyncTarget,
    pub status: SyncState,
    pub error_message: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncState {
    Pending,
    Synced,
    Failed,
}
