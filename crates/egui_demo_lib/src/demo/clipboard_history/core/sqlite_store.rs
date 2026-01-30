//! SQLite-based persistent storage for clipboard items.
//!
//! This module provides a persistent storage backend using SQLite,
//! with images saved to the filesystem.

use super::store::{Error as StoreError, Result as StoreResult, Store};
use super::item::ClipboardItem;
use super::image_manager::ImageManager;
use std::path::PathBuf;

/// SQLite-based persistent clipboard storage.
#[derive(Debug)]
pub struct SqliteStore {
    conn: std::sync::Mutex<rusqlite::Connection>,
    image_manager: ImageManager,
    max_items: usize,
}

impl SqliteStore {
    #[cfg(feature = "persistence")]
    pub fn new() -> Result<Self, StoreError> {
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clipboard_history");

        std::fs::create_dir_all(&data_dir).map_err(|e| {
            StoreError::Io(format!("Failed to create data directory: {}", e))
        })?;

        let db_path = data_dir.join("clipboard_history.db");
        Self::with_db_path(db_path)
    }

    pub fn with_db_path<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, StoreError> {
        let db_path = db_path.as_ref();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StoreError::Io(format!("Failed to create database directory: {}", e))
            })?;
        }

        let conn = rusqlite::Connection::open(db_path).map_err(|e| {
            StoreError::Io(format!("Failed to open database: {}", e))
        })?;

        let mut store = Self {
            conn: std::sync::Mutex::new(conn),
            image_manager: ImageManager::new(),
            max_items: 100,
        };

        store.init_db()?;
        Ok(store)
    }

    pub fn with_settings<P: AsRef<std::path::Path>>(
        db_path: P,
        images_dir: P,
        max_items: usize,
    ) -> Result<Self, StoreError> {
        let db_path = db_path.as_ref();
        let images_dir = images_dir.as_ref();

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                StoreError::Io(format!("Failed to create database directory: {}", e))
            })?;
        }
        std::fs::create_dir_all(images_dir).map_err(|e| {
            StoreError::Io(format!("Failed to create images directory: {}", e))
        })?;

        let conn = rusqlite::Connection::open(db_path).map_err(|e| {
            StoreError::Io(format!("Failed to open database: {}", e))
        })?;

        let mut store = Self {
            conn: std::sync::Mutex::new(conn),
            image_manager: ImageManager::with_base_dir(images_dir.to_path_buf()),
            max_items,
        };

        store.init_db()?;
        Ok(store)
    }

    fn init_db(&mut self) -> Result<(), StoreError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS clipboard_items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                icon TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                content_type TEXT NOT NULL,
                image_path TEXT
            )",
            [],
        ).map_err(|e| StoreError::Io(format!("Failed to create table: {}", e)))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_items(timestamp DESC)",
            [],
        ).map_err(|e| StoreError::Io(format!("Failed to create index: {}", e)))?;

        Ok(())
    }

    fn save_image(&self, timestamp: &str, width: u32, height: u32, png_bytes: &[u8]) -> Option<String> {
        self.image_manager
            .save_image(timestamp, width, height, png_bytes)
            .ok()
    }

    fn delete_image(&self, relative_path: &str) {
        let _ = self.image_manager.delete_image(relative_path);
    }

    fn count(&self) -> usize {
        self.with_conn(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM clipboard_items",
                [],
                |row| row.get::<_, i64>(0).map(|count| count as usize)
            ).unwrap_or(0)
        })
    }

    fn with_conn<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&rusqlite::Connection) -> R,
    {
        let conn = self.conn.lock().unwrap();
        f(&conn)
    }
}

impl Store for SqliteStore {
    fn add(&mut self, item: ClipboardItem) -> StoreResult<()> {
        let image_path = if let Some(ref png_bytes) = item.image_data {
            let (width, height) = parse_image_dimensions(&item.content);
            self.save_image(&item.timestamp, width, height, png_bytes)
        } else {
            None
        };

        let content_type_str = format!("{:?}", item.content_type);

        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO clipboard_items (icon, title, content, timestamp, content_type, image_path)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [&item.icon, &item.title, &item.content, &item.timestamp, &content_type_str, &image_path.unwrap_or_default()],
            ).map_err(|e| StoreError::Io(format!("Failed to insert item: {}", e)))?;
            Ok(())
        })?;

        let count = self.count();
        if count > self.max_items {
            let excess = count - self.max_items;

            self.with_conn(|conn| {
                let ids: Vec<i64> = conn.prepare(
                    "SELECT id FROM clipboard_items ORDER BY id ASC LIMIT ?1"
                ).map_err(|e| StoreError::Io(format!("Failed to prepare query: {}", e)))?
                    .query([excess as i64])
                    .map_err(|e| StoreError::Io(format!("Failed to query items: {}", e)))?
                    .mapped(|row| row.get::<_, i64>(0))
                    .collect::<Result<_, _>>()
                    .map_err(|e| StoreError::Io(format!("Failed to map rows: {}", e)))?;

                for id in ids {
                    if let Ok(mut stmt) = conn.prepare("SELECT image_path FROM clipboard_items WHERE id = ?1") {
                        if let Ok(rows) = stmt.query([id]) {
                            if let Some(row) = rows.mapped(|r| r.get::<_, Option<String>>(0)).next() {
                                if let Ok(Some(path)) = row {
                                    if !path.is_empty() {
                                        self.delete_image(&path);
                                    }
                                }
                            }
                        }
                    }

                    conn.execute("DELETE FROM clipboard_items WHERE id = ?1", [id])
                        .map_err(|e| StoreError::Io(format!("Failed to delete item: {}", e)))?;
                }

                Ok(())
            })?;
        }

        Ok(())
    }

    fn remove(&mut self, index: usize) -> StoreResult<()> {
        let (id, image_path) = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, image_path FROM clipboard_items ORDER BY timestamp DESC, id DESC LIMIT 1 OFFSET ?1"
            ).map_err(|e| StoreError::Io(format!("Failed to prepare query: {}", e)))?;

            let rows = stmt.query([index as i64])
                .map_err(|e| StoreError::Io(format!("Failed to query item: {}", e)))?;

            let mut result = None;
            // Use mapped() to properly iterate
            for row_result in rows.mapped(|row| {
                Ok((row.get::<_, i64>(0), row.get::<_, Option<String>>(1)))
            }) {
                if let Ok((Ok(id_val), Ok(path_opt))) = row_result {
                    result = Some((id_val, path_opt));
                    break;
                }
            }

            result.ok_or(StoreError::NotFound(index))?
        };

        if let Some(path) = image_path {
            if !path.is_empty() {
                self.delete_image(&path);
            }
        }

        self.with_conn(|conn| {
            conn.execute("DELETE FROM clipboard_items WHERE id = ?1", [id])
                .map_err(|e| StoreError::Io(format!("Failed to delete item: {}", e)))?;
            Ok(())
        })
    }

    fn get_all(&self) -> Vec<ClipboardItem> {
        self.with_conn(|conn| {
            match conn.prepare(
                "SELECT icon, title, content, timestamp, content_type, image_path
                 FROM clipboard_items
                 ORDER BY timestamp DESC, id DESC"
            ) {
                Ok(mut stmt) => {
                    match stmt.query([]) {
                        Ok(rows) => {
                            let mut items = Vec::new();
                            // Use mapped() to convert Rows to an iterator
                            for row_result in rows.mapped(|r| {
                                Ok((
                                    r.get::<_, String>(0),
                                    r.get::<_, String>(1),
                                    r.get::<_, String>(2),
                                    r.get::<_, String>(3),
                                    r.get::<_, String>(4),
                                    r.get::<_, Option<String>>(5),
                                ))
                            }) {
                                if let Ok((Ok(icon), Ok(title), Ok(content), Ok(timestamp), Ok(content_type_str), Ok(image_path))) = row_result {
                                    let content_type = parse_content_type(&content_type_str);

                                    let image_data = match image_path {
                                        Some(path) if !path.is_empty() => {
                                            let full_path = self.image_manager.get_full_path(&path);
                                            std::fs::read(&full_path).ok()
                                        },
                                        _ => None,
                                    };

                                    items.push(ClipboardItem {
                                        icon, title, content, timestamp, content_type, image_data,
                                    });
                                }
                            }
                            items
                        }
                        Err(_) => Vec::new()
                    }
                }
                Err(_) => Vec::new()
            }
        })
    }

    fn get(&self, index: usize) -> Option<ClipboardItem> {
        self.with_conn(|conn| {
            match conn.prepare(
                "SELECT icon, title, content, timestamp, content_type, image_path
                 FROM clipboard_items
                 ORDER BY timestamp DESC, id DESC
                 LIMIT 1 OFFSET ?1"
            ) {
                Ok(mut stmt) => {
                    match stmt.query([index as i64]) {
                        Ok(rows) => {
                            for row_result in rows.mapped(|r| {
                                Ok((
                                    r.get::<_, String>(0),
                                    r.get::<_, String>(1),
                                    r.get::<_, String>(2),
                                    r.get::<_, String>(3),
                                    r.get::<_, String>(4),
                                    r.get::<_, Option<String>>(5),
                                ))
                            }) {
                                if let Ok((Ok(icon), Ok(title), Ok(content), Ok(timestamp), Ok(content_type_str), Ok(image_path))) = row_result {
                                    let content_type = parse_content_type(&content_type_str);

                                    let image_data = match image_path {
                                        Some(path) if !path.is_empty() => {
                                            let full_path = self.image_manager.get_full_path(&path);
                                            std::fs::read(&full_path).ok()
                                        },
                                        _ => None,
                                    };

                                    return Some(ClipboardItem {
                                        icon, title, content, timestamp, content_type, image_data,
                                    });
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
            None
        })
    }

    fn update(&mut self, index: usize, item: ClipboardItem) -> StoreResult<()> {
        let (id, old_image_path) = {
            let conn = self.conn.lock().unwrap();
            let mut stmt = conn.prepare(
                "SELECT id, image_path FROM clipboard_items ORDER BY timestamp DESC, id DESC LIMIT 1 OFFSET ?1"
            ).map_err(|e| StoreError::Io(format!("Failed to prepare query: {}", e)))?;

            let rows = stmt.query([index as i64])
                .map_err(|e| StoreError::Io(format!("Failed to query item: {}", e)))?;

            let mut result = None;
            // Use mapped() to properly iterate
            for row_result in rows.mapped(|row| {
                Ok((row.get::<_, i64>(0), row.get::<_, Option<String>>(1)))
            }) {
                if let Ok((Ok(id_val), Ok(path_opt))) = row_result {
                    result = Some((id_val, path_opt));
                    break;
                }
            }

            result.ok_or(StoreError::NotFound(index))?
        };

        let new_image_path = if let Some(ref png_bytes) = item.image_data {
            let (width, height) = parse_image_dimensions(&item.content);
            self.save_image(&item.timestamp, width, height, png_bytes)
        } else {
            None
        };

        if let Some(old_path) = old_image_path {
            if !old_path.is_empty() && new_image_path.as_ref() != Some(&old_path) {
                self.delete_image(&old_path);
            }
        }

        let content_type_str = format!("{:?}", item.content_type);

        self.with_conn(|conn| {
            conn.execute(
                "UPDATE clipboard_items
                 SET icon = ?1, title = ?2, content = ?3, timestamp = ?4,
                     content_type = ?5, image_path = ?6
                 WHERE id = ?7",
                (&item.icon, &item.title, &item.content, &item.timestamp, &content_type_str, &new_image_path.unwrap_or_default(), id),
            ).map_err(|e| StoreError::Io(format!("Failed to update item: {}", e)))?;
            Ok(())
        })
    }

    fn clear(&mut self) -> StoreResult<()> {
        let image_paths: Vec<String> = self.with_conn(|conn| {
            match conn.prepare("SELECT image_path FROM clipboard_items") {
                Ok(mut stmt) => {
                    match stmt.query([]) {
                        Ok(rows) => {
                            rows.mapped(|row| row.get::<_, Option<String>>(0))
                                .filter_map(|p| p.ok())
                                .filter_map(|p| p)
                                .collect()
                        }
                        Err(_) => Vec::new()
                    }
                }
                Err(_) => Vec::new()
            }
        });

        for path in image_paths {
            if !path.is_empty() {
                self.delete_image(&path);
            }
        }

        self.with_conn(|conn| {
            conn.execute("DELETE FROM clipboard_items", [])
                .map_err(|e| StoreError::Io(format!("Failed to clear table: {}", e)))?;
            Ok(())
        })
    }

    fn len(&self) -> usize {
        self.count()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn max_capacity(&self) -> Option<usize> {
        Some(self.max_items)
    }
}

/// Parse image dimensions from content string.
fn parse_image_dimensions(content: &str) -> (u32, u32) {
    if let Some(start) = content.find(' ') {
        if let Some(end) = content[start..].find('x') {
            let width_str = &content[start + 1..start + end];
            if let Ok(width) = width_str.parse::<u32>() {
                if let Some(height_end) = content[start + end + 1..].find(' ') {
                    let height_str = &content[start + end + 1..start + end + 1 + height_end];
                    if let Ok(height) = height_str.parse::<u32>() {
                        return (width, height);
                    }
                }
            }
        }
    }
    (1920, 1080)
}

/// Parse content type from string.
fn parse_content_type(s: &str) -> crate::demo::clipboard_history::core::item::ContentType {
    use crate::demo::clipboard_history::core::item::ContentType;

    match s {
        "Text" => ContentType::Text,
        "Url" => ContentType::Url,
        "Image" => ContentType::Image,
        "Code" => ContentType::Code { language: "unknown".to_string() },
        "File" => ContentType::File { extension: "unknown".to_string() },
        _ => ContentType::Text,
    }
}
