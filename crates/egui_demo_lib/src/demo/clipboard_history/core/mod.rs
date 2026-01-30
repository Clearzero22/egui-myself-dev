//! Core data models and abstractions.
//!
//! This module contains the fundamental data structures and traits for the
//! clipboard history system:
//!
//! - [`item`]: Clipboard item data model and content type detection
//! - [`store`]: Storage abstraction for clipboard items
//! - [`filter`]: Filtering and search abstractions

pub mod item;
pub mod store;
pub mod filter;

// Async storage wrapper (always available)
pub mod async_store;

// Persistent storage modules (requires "persistence" feature)
#[cfg(feature = "persistence")]
pub mod image_manager;
#[cfg(feature = "persistence")]
pub mod sqlite_store;

// Re-export commonly used types
pub use item::{ClipboardItem, ContentType};
pub use store::{Store, MemoryStore, Error as StoreError, Result as StoreResult};
pub use filter::{Filter, FilterBox, TypeFilter, TextSearchFilter, CompositeFilter, LogicOperator};
pub use async_store::AsyncStore;

// Re-export persistent storage types when feature is enabled
#[cfg(feature = "persistence")]
pub use image_manager::{ImageManager, ImageError};
#[cfg(feature = "persistence")]
pub use sqlite_store::SqliteStore;
