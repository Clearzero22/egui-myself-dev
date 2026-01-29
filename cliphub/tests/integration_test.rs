use cliphub::storage::Database;
use cliphub::types::{ClipboardItem, ContentType};

#[test]
fn test_full_workflow() {
    // 1. Create database
    let db = Database::in_memory().unwrap();

    // 2. Insert item
    let item = ClipboardItem {
        content_type: ContentType::Text,
        title: Some("Integration Test".to_string()),
        content: "Test content for integration test".to_string(),
        source_app: Some("TestApp".to_string()),
        ..Default::default()
    };

    let id = db.insert_item(&item).unwrap();
    assert!(id > 0);

    // 3. Query items
    let items = db.get_recent_items(10).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].title, Some("Integration Test".to_string()));
    assert_eq!(items[0].content, "Test content for integration test");
}
