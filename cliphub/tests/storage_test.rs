use cliphub::storage::Database;

#[test]
fn test_database_creation() {
    let db = Database::in_memory().unwrap();

    // Verify tables exist
    assert!(db.table_exists("clipboard_items"));
    assert!(db.table_exists("sync_status"));
}

#[test]
fn test_indexes_created() {
    let db = Database::in_memory().unwrap();

    // Verify indexes exist
    assert!(db.table_exists("idx_clipboard_items_created_at"));
    assert!(db.table_exists("idx_clipboard_items_source_app"));
    assert!(db.table_exists("idx_clipboard_items_is_deleted"));
    assert!(db.table_exists("idx_sync_status_item_id"));
}
