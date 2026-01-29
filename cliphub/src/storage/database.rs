use rusqlite::Connection;
use anyhow::Result;
use std::path::Path;
use crate::types::{ClipboardItem, ContentType};
use chrono::Utc;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        // Enable foreign key enforcement
        self.conn.execute("PRAGMA foreign_keys = ON;", [])?;

        // Create tables
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type    TEXT NOT NULL,
                title           TEXT,
                content         TEXT NOT NULL,
                metadata        TEXT,
                source_app      TEXT,
                created_at      TEXT NOT NULL,
                is_synced       BOOLEAN DEFAULT 0,
                sync_targets    TEXT,
                tags            TEXT,
                is_deleted      BOOLEAN DEFAULT 0
            )",
            [],
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_status (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id         INTEGER REFERENCES clipboard_items(id),
                target          TEXT NOT NULL,
                status          TEXT NOT NULL,
                error_message   TEXT,
                last_sync_at    TEXT
            )",
            [],
        )?;

        // Create indexes for performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_created_at
             ON clipboard_items(created_at)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_source_app
             ON clipboard_items(source_app)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_clipboard_items_is_deleted
             ON clipboard_items(is_deleted)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sync_status_item_id
             ON sync_status(item_id)",
            [],
        )?;

        Ok(())
    }

    pub fn insert_item(&self, item: &ClipboardItem) -> Result<i64> {
        let content_type = match &item.content_type {
            ContentType::Text => "text".to_string(),
            ContentType::Code { language } => format!("code:{}", language),
            ContentType::Image { path, .. } => format!("image:{}", path),
            ContentType::Url { url, .. } => format!("url:{}", url),
        };

        let sync_targets = serde_json::to_string(&item.sync_targets)?;
        let tags = serde_json::to_string(&item.tags)?;

        self.conn.execute(
            "INSERT INTO clipboard_items (content_type, title, content, source_app, created_at, sync_targets, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &content_type,
                item.title.as_deref(),
                &item.content,
                item.source_app.as_deref(),
                &item.created_at.to_rfc3339(),
                &sync_targets,
                &tags,
            ),
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent_items(&self, limit: usize) -> Result<Vec<ClipboardItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, content_type, title, content, source_app, created_at, is_synced, sync_targets, tags
             FROM clipboard_items
             WHERE is_deleted = 0
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let items = stmt.query_map([limit as i64], |row| {
            let content_type_str: String = row.get(1)?;
            let content_type = Self::parse_content_type(&content_type_str);

            let created_at_str: String = row.get(5)?;
            let created_at = chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?
                .with_timezone(&Utc);

            let is_synced: bool = row.get(6)?;
            let sync_targets_str: String = row.get(7)?;
            let sync_targets = serde_json::from_str(&sync_targets_str).unwrap_or_default();

            let tags_str: String = row.get(8)?;
            let tags = serde_json::from_str(&tags_str).unwrap_or_default();

            Ok(ClipboardItem {
                id: Some(row.get(0)?),
                content_type,
                title: row.get(2)?,
                content: row.get(3)?,
                source_app: row.get(4)?,
                created_at,
                is_synced,
                sync_targets,
                tags,
            })
        })?;

        items.collect::<Result<Vec<ClipboardItem>, rusqlite::Error>>()
            .map_err(|e| e.into())
    }

    fn parse_content_type(s: &str) -> ContentType {
        if let Some(rest) = s.strip_prefix("code:") {
            ContentType::Code { language: rest.to_string() }
        } else if let Some(rest) = s.strip_prefix("image:") {
            ContentType::Image { path: rest.to_string(), width: 0, height: 0 }
        } else if let Some(rest) = s.strip_prefix("url:") {
            ContentType::Url { url: rest.to_string(), title: None }
        } else {
            ContentType::Text
        }
    }
}

impl Database {
    pub fn table_exists(&self, name: &str) -> bool {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM sqlite_master WHERE type IN ('table', 'index') AND name=?1"
        ).unwrap();
        let count: i64 = stmt.query_row([name], |row| row.get(0)).unwrap();
        count > 0
    }
}
