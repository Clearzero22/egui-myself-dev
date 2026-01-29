use rusqlite::{Connection, Result as SqliteResult};
use anyhow::Result;
use std::path::Path;

pub struct Database {
    pub conn: Connection,
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

        Ok(())
    }
}
