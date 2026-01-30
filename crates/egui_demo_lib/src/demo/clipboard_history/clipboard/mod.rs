//! Clipboard integration module.
//!
//! This module provides clipboard functionality through pluggable backends.
//! The default backend uses `arboard` for cross-platform support.

pub mod backend;
pub mod arboard;

// Re-export commonly used types
pub use backend::{Backend, ClipboardContent, Error as ClipboardError, Result as ClipboardResult};
pub use arboard::ArboardBackend;
