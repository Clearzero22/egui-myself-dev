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

// Re-export commonly used types
pub use item::{ClipboardItem, ContentType};
pub use store::{Store, MemoryStore, Error as StoreError, Result as StoreResult};
pub use filter::{Filter, FilterBox, TypeFilter, TextSearchFilter, CompositeFilter, LogicOperator};
