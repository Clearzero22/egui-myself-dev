//! Web clipboard backend implementation for WASM.
//!
//! This module provides a clipboard backend using the Web Clipboard API
//! for browser environments (WASM target).

use super::backend::{Backend, ClipboardContent};
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Clipboard};

/// Web-based clipboard backend for WASM.
///
/// This backend uses the Web Clipboard API which is asynchronous by nature.
/// Due to WASM's single-threaded nature and the sync trait requirements,
/// this implementation uses cached values for clipboard operations.
///
/// # Features
///
/// - Text clipboard support (via async polling in UI loop)
/// - No image support (Web Clipboard API image support is complex)
/// - Deduplication tracking for auto-capture
///
/// # Limitations
///
/// - The Web Clipboard API requires:
///   - User activation (user gesture) for write operations
///   - Document focus
///   - Permission granted by the browser
/// - `get_text()` returns cached content (updated via async operations)
/// - `set_text()` may fail without user activation
///
/// # Examples
///
/// ```
/// use clipboard_history::clipboard::web::WebBackend;
/// use clipboard_history::clipboard::backend::Backend;
///
/// let backend = WebBackend::new();
/// let content = backend.get_content();
/// ```
#[derive(Clone)]
pub struct WebBackend {
    /// Cached text content for sync access
    cached_text: Arc<Mutex<Option<String>>>,
    /// Last seen content for deduplication
    last_content: Arc<Mutex<String>>,
}

impl WebBackend {
    /// Create a new [`WebBackend`].
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// ```
    pub fn new() -> Self {
        Self {
            cached_text: Arc::default(),
            last_content: Arc::default(),
        }
    }

    /// Get text content from clipboard (returns cached value).
    ///
    /// Returns `None` if:
    /// - The cache is empty
    /// - The cached content is empty
    ///
    /// # Note
    ///
    /// In WASM, clipboard access is async. This method returns the cached value.
    /// Use `refresh_cache()` to update the cache from the browser's clipboard.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// if let Some(text) = backend.get_text() {
    ///     println!("Clipboard contains: {}", text);
    /// }
    /// ```
    pub fn get_text(&self) -> Option<String> {
        self.cached_text
            .lock()
            .unwrap()
            .as_ref()
            .filter(|s| !s.is_empty())
            .cloned()
    }

    /// Refresh the cached clipboard content from the browser.
    ///
    /// This should be called periodically (e.g., in the UI loop) to update
    /// the cached clipboard content.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// backend.refresh_cache();
    /// if let Some(text) = backend.get_text() {
    ///     println!("Updated clipboard: {}", text);
    /// }
    /// ```
    pub fn refresh_cache(&self) {
        let win = match window() {
            Some(w) => w,
            None => return,
        };

        let clipboard = win.navigator().clipboard();

        // read_text() returns Promise directly
        let promise = clipboard.read_text();

        let cached_text = self.cached_text.clone();
        let future = async move {
            match JsFuture::from(promise).await {
                Ok(result) => {
                    if let Some(text) = result.as_string() {
                        *cached_text.lock().unwrap() = Some(text);
                    }
                }
                Err(_) => {
                    *cached_text.lock().unwrap() = None;
                }
            }
        };

        // Spawn the future to run in the background
        wasm_bindgen_futures::spawn_local(future);
    }

    /// Set text content to clipboard.
    ///
    /// Returns `true` if the operation was initiated successfully, `false` otherwise.
    ///
    /// This operation requires:
    /// - User activation (gesture)
    /// - Document focus
    /// - Browser permission
    ///
    /// # Note
    ///
    /// In WASM, clipboard write is async. This method returns `true` if the
    /// async operation was successfully initiated. The actual write may fail
    /// if there's no user activation.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// if backend.set_text_direct("Hello, world!") {
    ///     println!("Clipboard write initiated");
    /// }
    /// ```
    pub fn set_text_direct(&self, text: &str) -> bool {
        let win = match window() {
            Some(w) => w,
            None => return false,
        };

        let clipboard = win.navigator().clipboard();

        // write_text takes &str and returns Promise directly
        let promise = clipboard.write_text(text);

        let cached_text = self.cached_text.clone();
        let text_clone = text.to_string();
        let future = async move {
            match JsFuture::from(promise).await {
                Ok(_) => {
                    // Update cache on successful write
                    *cached_text.lock().unwrap() = Some(text_clone);
                }
                Err(_) => {
                    // Write failed
                }
            }
        };

        wasm_bindgen_futures::spawn_local(future);
        true
    }

    /// Check if the given text content is new (different from last seen).
    ///
    /// This is useful for auto-capture to avoid adding duplicate items.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// if backend.is_new_content("new text") {
    ///     println!("This is new content!");
    /// }
    /// ```
    pub fn is_new_content(&self, content: &str) -> bool {
        let last = self.last_content.lock().unwrap();
        content != *last && !content.is_empty()
    }

    /// Mark content as seen (for deduplication).
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::web::WebBackend;
    ///
    /// let backend = WebBackend::new();
    /// backend.mark_content("seen this text".to_string());
    /// assert!(!backend.is_new_content("seen this text"));
    /// ```
    pub fn mark_content(&self, content: String) {
        *self.last_content.lock().unwrap() = content;
    }
}

impl Default for WebBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for WebBackend {
    fn get_content(&self) -> ClipboardContent {
        // Web backend only supports text (via cache)
        self.get_text()
            .map(ClipboardContent::Text)
            .unwrap_or(ClipboardContent::Unknown)
    }

    fn set_text(&self, text: &str) -> bool {
        self.set_text_direct(text)
    }

    fn supports_images(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "web"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_backend_creation() {
        let backend = WebBackend::new();
        assert_eq!(backend.name(), "web");
        assert!(!backend.supports_images());
    }

    #[test]
    fn test_deduplication() {
        let backend = WebBackend::new();

        // First check should be new
        assert!(backend.is_new_content("test"));

        // Mark as seen
        backend.mark_content("test".to_string());

        // Second check should not be new
        assert!(!backend.is_new_content("test"));

        // Different content should be new
        assert!(backend.is_new_content("different"));
    }
}
