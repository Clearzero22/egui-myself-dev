//! arboard clipboard backend implementation.
//!
//! This module provides the default clipboard backend using the `arboard` crate,
//! which offers cross-platform clipboard support.

use std::sync::{Arc, Mutex};
use super::backend::{Backend, ClipboardContent};

/// arboard-based clipboard backend.
///
/// This is the default clipboard backend that uses the `arboard` crate
/// for cross-platform clipboard access.
///
/// # Features
///
/// - Text clipboard support
/// - Image clipboard support (with `image-data` feature)
/// - Deduplication tracking for auto-capture
///
/// # Examples
///
/// ```
/// use clipboard_history::clipboard::arboard::ArboardBackend;
/// use clipboard_history::clipboard::backend::Backend;
///
/// let backend = ArboardBackend::new();
/// let content = backend.get_content();
///
/// if let Err(e) = backend.set_text("Hello, world!") {
///     eprintln!("Failed to set clipboard: {}", e);
/// }
/// ```
#[derive(Clone)]
pub struct ArboardBackend {
    last_content: Arc<Mutex<String>>,
}

impl ArboardBackend {
    /// Create a new [`ArboardBackend`].
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// ```
    pub fn new() -> Self {
        Self {
            last_content: Arc::default(),
        }
    }

    /// Get text content from clipboard.
    ///
    /// Returns `None` if the clipboard doesn't contain text or is inaccessible.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// if let Some(text) = backend.get_text() {
    ///     println!("Clipboard contains: {}", text);
    /// }
    /// ```
    pub fn get_text(&self) -> Option<String> {
        arboard::Clipboard::new()
            .ok()?
            .get_text()
            .ok()
            .filter(|s| !s.is_empty())
    }

    /// Set text content to clipboard.
    ///
    /// Returns `true` if successful, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// if backend.set_text_direct("Hello, world!") {
    ///     println!("Text copied to clipboard");
    /// }
    /// ```
    pub fn set_text_direct(&self, text: &str) -> bool {
        arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(text))
            .is_ok()
    }

    /// Get image data from clipboard.
    ///
    /// Returns `(width, height, bytes)` if the clipboard contains an image.
    /// Returns `None` if the clipboard doesn't contain an image or is inaccessible.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// if let Some((width, height, bytes)) = backend.get_image() {
    ///     println!("Clipboard contains image: {}x{} ({} bytes)", width, height, bytes.len());
    /// }
    /// ```
    pub fn get_image(&self) -> Option<(u32, u32, Vec<u8>)> {
        let image = arboard::Clipboard::new()
            .ok()?
            .get_image()
            .ok()?;
        Some((image.width as u32, image.height as u32, image.bytes.to_vec()))
    }

    /// Check if the given text content is new (different from last seen).
    ///
    /// This is useful for auto-capture to avoid adding duplicate items.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
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
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// backend.mark_content("seen this text".to_string());
    /// assert!(!backend.is_new_content("seen this text"));
    /// ```
    pub fn mark_content(&self, content: String) {
        *self.last_content.lock().unwrap() = content;
    }

    /// Check if the given image is new (different from last seen).
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::clipboard::arboard::ArboardBackend;
    ///
    /// let backend = ArboardBackend::new();
    /// if backend.is_new_image(1920, 1080) {
    ///     println!("This is a new image!");
    /// }
    /// ```
    pub fn is_new_image(&self, width: u32, height: u32) -> bool {
        let key = format!("IMG_{}x{}", width, height);
        self.is_new_content(&key)
    }
}

impl Default for ArboardBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl Backend for ArboardBackend {
    fn get_content(&self) -> ClipboardContent {
        // Try text first
        if let Some(text) = self.get_text() {
            return ClipboardContent::Text(text);
        }

        // Try image
        if let Some((width, height, _bytes)) = self.get_image() {
            return ClipboardContent::Image { width, height };
        }

        ClipboardContent::Unknown
    }

    fn set_text(&self, text: &str) -> bool {
        self.set_text_direct(text)
    }

    fn supports_images(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "arboard"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arboard_backend_creation() {
        let backend = ArboardBackend::new();
        assert_eq!(backend.name(), "arboard");
        assert!(backend.supports_images());
    }

    #[test]
    fn test_deduplication() {
        let backend = ArboardBackend::new();

        // First check should be new
        assert!(backend.is_new_content("test"));

        // Mark as seen
        backend.mark_content("test".to_string());

        // Second check should not be new
        assert!(!backend.is_new_content("test"));

        // Different content should be new
        assert!(backend.is_new_content("different"));

        // Image deduplication
        assert!(backend.is_new_image(1920, 1080));
        backend.mark_content(format!("IMG_1920x1080"));
        assert!(!backend.is_new_image(1920, 1080));
    }

    #[test]
    fn test_clipboard_content_from_text() {
        let content = ClipboardContent::Text("Hello, world!".to_string());
        assert!(content.is_text());
        assert_eq!(content.as_text(), Some("Hello, world!"));
    }

    #[test]
    fn test_clipboard_content_from_image() {
        let content = ClipboardContent::Image {
            width: 1920,
            height: 1080,
        };
        assert!(content.is_image());
        assert_eq!(content.as_image(), Some((1920, 1080)));
    }
}
