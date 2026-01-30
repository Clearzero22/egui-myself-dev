//! Clipboard integration module.
//!
//! This module provides clipboard functionality through pluggable backends.
//! The default backend uses `arboard` for desktop platforms and `web-sys` for WASM.

pub mod backend;

#[cfg(not(target_arch = "wasm32"))]
pub mod arboard;
#[cfg(not(target_arch = "wasm32"))]
pub use arboard::ArboardBackend;

#[cfg(target_arch = "wasm32")]
pub mod web;
#[cfg(target_arch = "wasm32")]
pub use web::WebBackend;

// Re-export commonly used types
pub use backend::{Backend, ClipboardContent, Error as ClipboardError, Result as ClipboardResult};

/// Create the appropriate clipboard backend for the current platform.
///
/// On desktop platforms (not WASM), this creates an [`ArboardBackend`].
/// On WASM, this creates a [`WebBackend`].
///
/// # Examples
///
/// ```
/// use clipboard_history::clipboard::create_backend;
/// use clipboard_history::clipboard::backend::Backend;
///
/// let backend = create_backend();
/// let content = backend.get_content();
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn create_backend() -> ArboardBackend {
    ArboardBackend::new()
}

/// Create the appropriate clipboard backend for the current platform.
///
/// On desktop platforms (not WASM), this creates an [`ArboardBackend`].
/// On WASM, this creates a [`WebBackend`].
///
/// # Examples
///
/// ```
/// use clipboard_history::clipboard::create_backend;
/// use clipboard_history::clipboard::backend::Backend;
///
/// let backend = create_backend();
/// let content = backend.get_content();
/// ```
#[cfg(target_arch = "wasm32")]
pub fn create_backend() -> WebBackend {
    WebBackend::new()
}
