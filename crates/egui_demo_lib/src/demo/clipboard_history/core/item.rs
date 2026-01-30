//! Clipboard item data model and content type detection.
//!
//! This module provides:
//! - [`ContentType`]: Enum representing different clipboard content types
//! - [`ClipboardItem`]: Struct representing a single clipboard history item
//!
//! # Extension Points
//!
//! - Custom content types via [`ContentType::Custom`]
//! - Language detection for code via [`ContentType::Code`]
//! - File type detection via [`ContentType::File`]

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// -----------------------------------------------------------------------------
// ContentType
// -----------------------------------------------------------------------------

/// Represents the type of clipboard content.
///
/// This enum classifies clipboard content for display and filtering purposes.
///
/// # Extension Points
///
/// - [`ContentType::Custom`]: For application-specific content types
/// - [`ContentType::Code`]: Can be extended with language detection
/// - [`ContentType::File`]: Can be extended with MIME type detection
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum ContentType {
    /// Plain text content
    Text,
    /// URL/Link content
    Url,
    /// Image content (screenshot, copied image)
    Image,
    /// Code snippet with optional language identifier
    Code { language: String },
    /// File path with extension
    File { extension: String },
    /// Custom content type for extensibility
    Custom(String),
}

impl ContentType {
    /// Detect content type from raw string content.
    ///
    /// This method uses heuristics to classify content:
    /// - URLs starting with `http://` or `https://`
    /// - File paths (Unix: `/`, `~/`; Windows: `C:\`)
    /// - Code patterns (function definitions, imports, etc.)
    /// - Default: Text
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ContentType;
    ///
    /// assert_eq!(ContentType::detect("https://example.com"), ContentType::Url);
    /// assert_eq!(ContentType::detect("/home/user/file.txt"), ContentType::File { extension: "txt".to_string() });
    /// assert_eq!(ContentType::detect("fn main() {}"), ContentType::Code { language: "unknown".to_string() });
    /// ```
    pub fn detect(content: &str) -> Self {
        let trimmed = content.trim();

        // Empty or whitespace-only content
        if trimmed.is_empty() {
            return Self::Text;
        }

        // URL detection
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return Self::Url;
        }

        // File path detection
        if Self::looks_like_file_path(trimmed) {
            if let Some(ext) = Path::new(trimmed).extension() {
                return Self::File {
                    extension: ext.to_string_lossy().to_string(),
                };
            }
        }

        // Code detection (heuristic)
        if Self::looks_like_code(trimmed) {
            return Self::Code {
                language: "unknown".to_string(),
            };
        }

        Self::Text
    }

    /// Get the display icon (emoji) for this content type.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ContentType;
    ///
    /// assert_eq!(ContentType::Text.icon(), "📄");
    /// assert_eq!(ContentType::Url.icon(), "🔗");
    /// ```
    pub fn icon(&self) -> &str {
        match self {
            Self::Text => "📄",
            Self::Url => "🔗",
            Self::Image => "🖼️",
            Self::Code { .. } => "💻",
            Self::File { .. } => "📁",
            Self::Custom(_) => "📦",
        }
    }

    /// Get the display label for this content type.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ContentType;
    ///
    /// assert_eq!(ContentType::Text.label(), "Text");
    /// assert_eq!(ContentType::Url.label(), "Link");
    /// ```
    pub fn label(&self) -> &str {
        match self {
            Self::Text => "Text",
            Self::Url => "Link",
            Self::Image => "Image",
            Self::Code { .. } => "Code",
            Self::File { .. } => "File",
            Self::Custom(_) => "Other",
        }
    }

    /// Check if this content type represents an image.
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image)
    }

    /// Check if this content type is text-based (Text, Url, or Code).
    pub fn is_text(&self) -> bool {
        matches!(self, Self::Text | Self::Url | Self::Code { .. })
    }

    // --- Private helpers ---

    fn looks_like_file_path(content: &str) -> bool {
        // Unix paths
        if content.starts_with('/') || content.starts_with("~/") {
            return true;
        }
        // Windows paths (e.g., "C:\")
        if content.len() > 2 && content.as_bytes()[1] == b':' {
            if let Some(c) = content.chars().nth(2) {
                return c == '\\' || c == '/';
            }
        }
        false
    }

    fn looks_like_code(content: &str) -> bool {
        let lines: Vec<&str> = content.lines().collect();
        if lines.len() < 2 {
            return false;
        }

        let first_line = lines[0].trim();
        let lower = first_line.to_lowercase();

        // Common code patterns
        let code_keywords = [
            "fn ", "def ", "function ", "class ", "import ", "use ",
            "pub ", "let ", "const ", "var ", "async ", "await ",
            "return ", "if ", "for ", "while ", "match ",
        ];

        for keyword in code_keywords {
            if lower.starts_with(keyword) {
                return true;
            }
        }

        // Code-specific characters
        if content.contains(" => ") || content.contains(" {") || content.contains("::") {
            return true;
        }

        false
    }
}

impl Default for ContentType {
    fn default() -> Self {
        Self::Text
    }
}

// -----------------------------------------------------------------------------
// ClipboardItem
// -----------------------------------------------------------------------------

/// A single clipboard history item.
///
/// Contains the content, metadata, and classification for a single clipboard entry.
///
/// # Examples
///
/// ```
/// use clipboard_history::core::item::ClipboardItem;
///
/// let item = ClipboardItem::from_text("Hello, world!".to_string());
/// assert_eq!(item.content_type.icon(), "📄");
/// ```
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ClipboardItem {
    /// Display icon (emoji representing content type)
    pub icon: String,
    /// Display title (truncated preview of content)
    pub title: String,
    /// Full content (text description or image metadata)
    pub content: String,
    /// Timestamp string (HH:MM format)
    pub timestamp: String,
    /// Content type classification
    pub content_type: ContentType,
    /// Image data (PNG bytes) if this is an image item
    pub image_data: Option<Vec<u8>>,
}

impl ClipboardItem {
    /// Create a new [`ClipboardItem`] from text content.
    ///
    /// This method automatically detects the content type, generates an icon,
    /// creates a title preview, and adds a timestamp.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ClipboardItem;
    ///
    /// let item = ClipboardItem::from_text("https://example.com".to_string());
    /// assert_eq!(item.icon, "🔗");
    /// assert_eq!(item.content_type, ContentType::Url);
    /// ```
    pub fn from_text(content: String) -> Self {
        let content_type = ContentType::detect(&content);
        let icon = content_type.icon().to_string();
        let title = Self::generate_title(&content);
        let timestamp = Self::current_timestamp();

        Self {
            icon,
            title,
            content,
            timestamp,
            content_type,
            image_data: None,
        }
    }

    /// Create a new [`ClipboardItem`] from image data.
    ///
    /// Stores the actual PNG image bytes for preview.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ClipboardItem;
    ///
    /// let item = ClipboardItem::from_image(1920, 1080, png_bytes);
    /// assert_eq!(item.icon, "🖼️");
    /// assert!(item.title.contains("1920x1080"));
    /// assert!(item.image_data.is_some());
    /// ```
    pub fn from_image(width: u32, height: u32, bytes: Vec<u8>) -> Self {
        Self {
            icon: "🖼️".to_string(),
            title: format!("Screenshot ({}x{})", width, height),
            content: format!("[Image: {}x{} pixels, {} KB]", width, height, bytes.len() / 1024),
            timestamp: Self::current_timestamp(),
            content_type: ContentType::Image,
            image_data: Some(bytes),
        }
    }

    /// Create a [`ClipboardItem`] with a custom content type.
    ///
    /// Use this when you want to override automatic content type detection.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::{ClipboardItem, ContentType};
    ///
    /// let item = ClipboardItem::with_type(
    ///     "custom content".to_string(),
    ///     ContentType::Custom("my-type".to_string())
    /// );
    /// assert_eq!(item.icon, "📦");
    /// ```
    pub fn with_type(content: String, content_type: ContentType) -> Self {
        let icon = content_type.icon().to_string();
        let title = Self::generate_title(&content);

        Self {
            icon,
            title,
            content,
            timestamp: Self::current_timestamp(),
            content_type,
            image_data: None,
        }
    }

    /// Get a truncated preview of the content.
    ///
    /// Returns the content up to `max_len` characters, with "..." appended
    /// if the content was truncated.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ClipboardItem;
    ///
    /// let item = ClipboardItem::from_text("This is a very long string that needs truncation".to_string());
    /// let preview = item.preview(20);
    /// assert!(preview.ends_with("..."));
    /// assert!(preview.len() <= 23); // 20 + "..."
    /// ```
    pub fn preview(&self, max_len: usize) -> String {
        if self.content.len() > max_len {
            format!("{}...", &self.content[..max_len.saturating_sub(3)])
        } else {
            self.content.clone()
        }
    }

    /// Check if this item matches a search query.
    ///
    /// The search is case-insensitive and checks both title and content.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::ClipboardItem;
    ///
    /// let item = ClipboardItem::from_text("Hello World".to_string());
    /// assert!(item.matches_query("hello"));
    /// assert!(item.matches_query("world"));
    /// assert!(!item.matches_query("goodbye"));
    /// ```
    pub fn matches_query(&self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let query_lower = query.to_lowercase();
        self.title.to_lowercase().contains(&query_lower)
            || self.content.to_lowercase().contains(&query_lower)
    }

    /// Check if this item matches a specific content type filter.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::item::{ClipboardItem, ContentType};
    ///
    /// let item = ClipboardItem::from_text("https://example.com".to_string());
    /// assert!(item.matches_type(&ContentType::Url));
    /// assert!(!item.matches_type(&ContentType::Image));
    /// ```
    pub fn matches_type(&self, content_type: &ContentType) -> bool {
        match (&self.content_type, content_type) {
            (ContentType::Text, ContentType::Text) => true,
            (ContentType::Url, ContentType::Url) => true,
            (ContentType::Image, ContentType::Image) => true,
            (ContentType::Code { .. }, ContentType::Code { .. }) => true,
            (ContentType::File { .. }, ContentType::File { .. }) => true,
            (ContentType::Custom(a), ContentType::Custom(b)) => a == b,
            _ => false,
        }
    }

    // --- Private helpers ---

    fn generate_title(content: &str) -> String {
        let preview = content.lines().next().unwrap_or(content);
        if preview.len() > 40 {
            format!("{}...", &preview[..37])
        } else {
            preview.to_string()
        }
    }

    fn current_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!(
            "{:02}:{:02}",
            (now % 86400 / 3600) as u32,
            (now % 3600 / 60) as u32
        )
    }
}

impl Default for ClipboardItem {
    fn default() -> Self {
        Self::from_text("Empty".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_url_detection() {
        assert_eq!(
            ContentType::detect("https://example.com"),
            ContentType::Url
        );
        assert_eq!(
            ContentType::detect("http://test.org"),
            ContentType::Url
        );
    }

    #[test]
    fn test_content_type_file_detection() {
        assert_eq!(
            ContentType::detect("/home/user/file.txt"),
            ContentType::File {
                extension: "txt".to_string()
            }
        );
        assert_eq!(
            ContentType::detect("~/Documents/file.rs"),
            ContentType::File {
                extension: "rs".to_string()
            }
        );
    }

    #[test]
    fn test_content_type_code_detection() {
        assert!(matches!(
            ContentType::detect("fn main() {\nprintln!(\"Hello\");\n}"),
            ContentType::Code { .. }
        ));
        assert!(matches!(
            ContentType::detect("import os\nprint(os.getcwd())"),
            ContentType::Code { .. }
        ));
    }

    #[test]
    fn test_clipboard_item_from_text() {
        let item = ClipboardItem::from_text("Hello, World!".to_string());
        assert_eq!(item.content, "Hello, World!");
        assert_eq!(item.icon, "📄");
        assert!(item.timestamp.contains(':'));
    }

    #[test]
    fn test_clipboard_item_from_image() {
        let bytes = vec![0u8; 1024]; // Fake PNG data
        let item = ClipboardItem::from_image(1920, 1080, bytes);
        assert_eq!(item.icon, "🖼️");
        assert!(item.title.contains("1920x1080"));
        assert_eq!(item.content_type, ContentType::Image);
        assert!(item.image_data.is_some());
    }

    #[test]
    fn test_matches_query() {
        let item = ClipboardItem::from_text("Hello World".to_string());
        assert!(item.matches_query("hello"));
        assert!(item.matches_query("world"));
        assert!(!item.matches_query("goodbye"));
        assert!(item.matches_query("")); // Empty query matches all
    }

    #[test]
    fn test_preview() {
        let long_content = "This is a very long string that needs to be truncated";
        let item = ClipboardItem::from_text(long_content.to_string());
        let preview = item.preview(20);
        assert!(preview.len() <= 23);
        assert!(preview.ends_with("..."));
    }
}
