use rusqlite::Connection;
use anyhow::Result;
use std::path::Path;

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
}

#[cfg(test)]
impl Database {
    pub fn table_exists(&self, table_name: &str) -> bool {
        let mut stmt = self.conn.prepare(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1"
        ).unwrap();
        let count: i64 = stmt.query_row([table_name], |row| row.get(0)).unwrap();
        count > 0
    }
}
