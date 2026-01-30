//! Storage abstraction for clipboard items.
//!
//! This module provides a trait-based storage abstraction that allows for
//! different storage backends (in-memory, database, file-based, etc.).
//!
//! # Extension Points
//!
//! - Implement [`Store`] trait for database persistence (SQLite, sled)
//! - Implement [`Store`] trait for file storage (JSON, bincode)
//! - Implement [`Store`] trait for cloud synchronization
//!
//! # Examples
//!
//! ```
//! use clipboard_history::core::store::{Store, MemoryStore};
//! use clipboard_history::core::item::ClipboardItem;
//!
//! let mut store = MemoryStore::new();
//! let item = ClipboardItem::from_text("Hello, world!".to_string());
//!
//! store.add(item).unwrap();
//! assert_eq!(store.len(), 1);
//! ```

use crate::demo::clipboard_history::core::item::ClipboardItem;

// -----------------------------------------------------------------------------
// Error types
// -----------------------------------------------------------------------------

/// Error type for store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Item not found at the given index
    NotFound(usize),
    /// Storage capacity would be exceeded
    CapacityExceeded,
    /// I/O error with message
    Io(String),
    /// Other error with message
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(idx) => write!(f, "Item not found: {}", idx),
            Self::CapacityExceeded => write!(f, "Storage capacity exceeded"),
            Self::Io(msg) => write!(f, "I/O error: {}", msg),
            Self::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

/// Result type for store operations.
pub type Result<T> = std::result::Result<T, Error>;

// -----------------------------------------------------------------------------
// Store trait
// -----------------------------------------------------------------------------

/// Storage abstraction for clipboard items.
///
/// This trait defines the interface for storing and managing clipboard history.
/// Implementations can use different storage backends (memory, database, file, etc.).
///
/// # Extension Points
///
/// - **Database**: Implement with SQLite or sled for persistence
/// - **File**: Implement with JSON or bincode for file-based storage
/// - **Cloud**: Implement with cloud APIs for synchronization
/// - **Custom**: Implement for application-specific needs
///
/// # Examples
///
/// ```
/// use clipboard_history::core::store::{Store, MemoryStore};
/// use clipboard_history::core::item::ClipboardItem;
///
/// let mut store = MemoryStore::new();
/// let item = ClipboardItem::from_text("Test".to_string());
///
/// assert!(store.add(item).is_ok());
/// assert_eq!(store.len(), 1);
/// assert!(!store.is_empty());
/// ```
pub trait Store: Send + Sync {
    /// Add a new item to the store.
    ///
    /// The item should be inserted at the beginning (most recent).
    /// If capacity is exceeded, the oldest item should be removed.
    ///
    /// # Errors
    ///
    /// Returns [`Error::CapacityExceeded`] if the store cannot accept more items.
    fn add(&mut self, item: ClipboardItem) -> Result<()>;

    /// Remove item at the given index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the index is out of bounds.
    fn remove(&mut self, index: usize) -> Result<()>;

    /// Get all items in the store.
    ///
    /// Returns a vector of items in order (most recent first).
    fn get_all(&self) -> Vec<ClipboardItem>;

    /// Get item at the given index.
    ///
    /// Returns `None` if the index is out of bounds.
    fn get(&self, index: usize) -> Option<ClipboardItem>;

    /// Update item at the given index.
    ///
    /// # Errors
    ///
    /// Returns [`Error::NotFound`] if the index is out of bounds.
    fn update(&mut self, index: usize, item: ClipboardItem) -> Result<()>;

    /// Clear all items from the store.
    ///
    /// # Errors
    ///
    /// May return I/O errors for persistent storage backends.
    fn clear(&mut self) -> Result<()>;

    /// Get a page of items with pagination.
    ///
    /// This is more efficient than `get_all()` for large datasets.
    ///
    /// # Arguments
    ///
    /// * `offset` - Number of items to skip (0-based)
    /// * `limit` - Maximum number of items to return
    ///
    /// # Returns
    ///
    /// A vector of items in order (most recent first), up to `limit` items.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clipboard_history::core::store::Store;
    /// # use clipboard_history::core::item::ClipboardItem;
    ///
    /// fn show_first_page<S: Store>(store: &S) {
    ///     let page = store.get_page(0, 20);  // First 20 items
    ///     for item in page {
    ///         println!("{}", item.title);
    ///     }
    /// }
    /// ```
    fn get_page(&self, offset: usize, limit: usize) -> Vec<ClipboardItem> {
        // Default implementation: get all and slice
        // Persistent stores should override this for efficiency
        self.get_all()
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect()
    }

    /// Get the number of items in the store.
    fn len(&self) -> usize;

    /// Check if the store is empty.
    fn is_empty(&self) -> bool;

    /// Get the maximum capacity of the store.
    ///
    /// Returns `None` if there is no limit.
    fn max_capacity(&self) -> Option<usize> {
        None
    }
}

// -----------------------------------------------------------------------------
// MemoryStore implementation
// -----------------------------------------------------------------------------

/// In-memory clipboard item storage.
///
/// This is the default storage implementation that keeps items in a Vec.
/// Items are stored in order (most recent first) and limited by a maximum capacity.
///
/// # Examples
///
/// ```
/// use clipboard_history::core::store::{Store, MemoryStore};
/// use clipboard_history::core::item::ClipboardItem;
///
/// let mut store = MemoryStore::new();
/// assert_eq!(store.max_capacity(), Some(50));
///
/// let store_large = MemoryStore::with_capacity(1000);
/// assert_eq!(store_large.max_capacity(), Some(1000));
/// ```
#[derive(Debug, Clone)]
pub struct MemoryStore {
    items: Vec<ClipboardItem>,
    max_items: usize,
}

impl MemoryStore {
    /// Create a new [`MemoryStore`] with default capacity (50 items).
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::store::MemoryStore;
    ///
    /// let store = MemoryStore::new();
    /// assert_eq!(store.max_capacity(), Some(50));
    /// ```
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_items: 50,
        }
    }

    /// Create a [`MemoryStore`] with a custom maximum capacity.
    ///
    /// # Arguments
    ///
    /// * `max_items` - Maximum number of items to store
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::store::MemoryStore;
    ///
    /// let store = MemoryStore::with_capacity(100);
    /// assert_eq!(store.max_capacity(), Some(100));
    /// ```
    pub fn with_capacity(max_items: usize) -> Self {
        Self {
            items: Vec::with_capacity(max_items),
            max_items,
        }
    }

    /// Get the current maximum capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::store::MemoryStore;
    ///
    /// let store = MemoryStore::with_capacity(25);
    /// assert_eq!(store.capacity(), 25);
    /// ```
    pub fn capacity(&self) -> usize {
        self.max_items
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for MemoryStore {
    fn add(&mut self, item: ClipboardItem) -> Result<()> {
        // Insert at the beginning (most recent)
        self.items.insert(0, item);

        // Truncate if over capacity
        if self.items.len() > self.max_items {
            self.items.truncate(self.max_items);
        }

        Ok(())
    }

    fn remove(&mut self, index: usize) -> Result<()> {
        if index >= self.items.len() {
            return Err(Error::NotFound(index));
        }
        self.items.remove(index);
        Ok(())
    }

    fn get_all(&self) -> Vec<ClipboardItem> {
        self.items.clone()
    }

    fn get(&self, index: usize) -> Option<ClipboardItem> {
        self.items.get(index).cloned()
    }

    fn update(&mut self, index: usize, item: ClipboardItem) -> Result<()> {
        if index >= self.items.len() {
            return Err(Error::NotFound(index));
        }
        self.items[index] = item;
        Ok(())
    }

    fn clear(&mut self) -> Result<()> {
        self.items.clear();
        Ok(())
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    fn max_capacity(&self) -> Option<usize> {
        Some(self.max_items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_store_add() {
        let mut store = MemoryStore::new();
        let item = ClipboardItem::from_text("Test".to_string());

        assert!(store.add(item.clone()).is_ok());
        assert_eq!(store.len(), 1);
        assert!(!store.is_empty());
    }

    #[test]
    fn test_memory_store_capacity_limit() {
        let mut store = MemoryStore::with_capacity(3);

        for i in 0..5 {
            let _ = store.add(ClipboardItem::from_text(format!("Item {}", i)));
        }

        // Should only keep 3 most recent items
        assert_eq!(store.len(), 3);
        assert_eq!(store.get(0).unwrap().content, "Item 4");
    }

    #[test]
    fn test_memory_store_remove() {
        let mut store = MemoryStore::new();

        store.add(ClipboardItem::from_text("Item 1".to_string())).unwrap();
        store.add(ClipboardItem::from_text("Item 2".to_string())).unwrap();

        assert!(store.remove(0).is_ok());
        assert_eq!(store.len(), 1);
        assert_eq!(store.get(0).unwrap().content, "Item 1");
    }

    #[test]
    fn test_memory_store_remove_not_found() {
        let mut store = MemoryStore::new();
        assert_eq!(store.remove(0), Err(Error::NotFound(0)));
    }

    #[test]
    fn test_memory_store_update() {
        let mut store = MemoryStore::new();
        store.add(ClipboardItem::from_text("Original".to_string())).unwrap();

        let new_item = ClipboardItem::from_text("Updated".to_string());
        assert!(store.update(0, new_item).is_ok());
        assert_eq!(store.get(0).unwrap().content, "Updated");
    }

    #[test]
    fn test_memory_store_clear() {
        let mut store = MemoryStore::new();
        store.add(ClipboardItem::from_text("Test".to_string())).unwrap();

        assert!(store.clear().is_ok());
        assert_eq!(store.len(), 0);
        assert!(store.is_empty());
    }

    #[test]
    fn test_memory_store_get_all() {
        let mut store = MemoryStore::new();

        store.add(ClipboardItem::from_text("Item 1".to_string())).unwrap();
        store.add(ClipboardItem::from_text("Item 2".to_string())).unwrap();

        let all = store.get_all();
        assert_eq!(all.len(), 2);
        // Most recent first
        assert_eq!(all[0].content, "Item 2");
        assert_eq!(all[1].content, "Item 1");
    }
}
