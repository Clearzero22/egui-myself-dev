//! Clipboard backend abstraction.
//!
//! This module provides a trait-based abstraction for different clipboard
//! implementations, allowing for platform-specific optimizations and
//! alternative clipboard managers.
//!
//! # Extension Points
//!
//! - Implement [`Backend`] for Wayland-specific features
//! - Implement [`Backend`] for X11-specific features
//! - Implement [`Backend`] for macOS-specific features
//! - Implement [`Backend`] for Windows-specific features
//! - Implement [`Backend`] for custom clipboard managers

use std::sync::mpsc;

/// Clipboard content representation.
///
/// This enum represents the different types of content that can be
/// stored in the clipboard.
#[derive(Clone, Debug)]
pub enum ClipboardContent {
    /// Text content (plain text, URLs, etc.)
    Text(String),
    /// Image content with dimensions
    Image { width: u32, height: u32 },
    /// File list (for file copy operations)
    Files(Vec<String>),
    /// Unknown or unsupported content type
    Unknown,
}

impl ClipboardContent {
    /// Check if this content is text.
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text(_))
    }

    /// Check if this content is an image.
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image { .. })
    }

    /// Get text content if present.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Get image dimensions if present.
    pub fn as_image(&self) -> Option<(u32, u32)> {
        match self {
            Self::Image { width, height } => Some((*width, *height)),
            _ => None,
        }
    }
}

/// Clipboard backend trait.
///
/// This trait defines the interface for clipboard implementations.
/// Different backends can provide platform-specific optimizations or
/// integrate with custom clipboard managers.
///
/// # Extension Points
///
/// - **Wayland**: Implement for Wayland-specific features like clipboard history
/// - **X11**: Implement for X11-specific features and selections
/// - **macOS**: Implement for macOS-specific features like universal clipboard
/// - **Windows**: Implement for Windows-specific features like clipboard formats
/// - **Custom**: Implement for integration with clipboard managers like clipit, parcellite
///
/// # Examples
///
/// ```
/// use clipboard_history::clipboard::backend::{Backend, ClipboardContent};
///
/// struct MyBackend;
///
/// impl Backend for MyBackend {
///     fn get_content(&self) -> ClipboardContent {
///         // Implementation...
///         ClipboardContent::Unknown
///     }
///
///     fn set_text(&self, text: &str) -> bool {
///         // Implementation...
///         true
///     }
///
///     fn name(&self) -> &str {
///         "my-backend"
///     }
/// }
/// ```
pub trait Backend: Send + Sync {
    /// Get current clipboard content.
    ///
    /// This method should return the current clipboard content, trying
    /// different content types in order of preference (text first, then image, etc.).
    fn get_content(&self) -> ClipboardContent;

    /// Set text content to clipboard.
    ///
    /// Returns `true` if successful, `false` otherwise.
    fn set_text(&self, text: &str) -> bool;

    /// Subscribe to clipboard changes (optional, event-driven).
    ///
    /// This method should return a receiver that gets notified when
    /// the clipboard content changes. If the backend doesn't support
    /// event-driven updates, return `None`.
    ///
    /// # Default Implementation
    ///
    /// Returns `None` (not supported).
    fn subscribe(&self) -> Option<mpsc::Receiver<ClipboardContent>> {
        None
    }

    /// Check if this backend supports image clipboard operations.
    ///
    /// # Default Implementation
    ///
    /// Returns `false` (not supported).
    fn supports_images(&self) -> bool {
        false
    }

    /// Backend name for debugging and logging.
    ///
    /// # Default Implementation
    ///
    /// Returns `"unknown"`.
    fn name(&self) -> &str {
        "unknown"
    }
}

/// Result type for clipboard operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for clipboard operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Clipboard is not accessible (permission denied, etc.)
    NotAccessible,
    /// The requested content type is not available
    ContentNotAvailable,
    /// Operation failed with a message
    Failed(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotAccessible => write!(f, "Clipboard not accessible"),
            Self::ContentNotAvailable => write!(f, "Content not available"),
            Self::Failed(msg) => write!(f, "Operation failed: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_content_text() {
        let content = ClipboardContent::Text("Hello".to_string());
        assert!(content.is_text());
        assert!(!content.is_image());
        assert_eq!(content.as_text(), Some("Hello"));
    }

    #[test]
    fn test_clipboard_content_image() {
        let content = ClipboardContent::Image {
            width: 1920,
            height: 1080,
        };
        assert!(!content.is_text());
        assert!(content.is_image());
        assert_eq!(content.as_image(), Some((1920, 1080)));
    }

    #[test]
    fn test_clipboard_content_unknown() {
        let content = ClipboardContent::Unknown;
        assert!(!content.is_text());
        assert!(!content.is_image());
        assert_eq!(content.as_text(), None);
        assert_eq!(content.as_image(), None);
    }
}
