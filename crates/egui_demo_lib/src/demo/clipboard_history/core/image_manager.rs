//! Image file management for persistent clipboard storage.
//!
//! This module handles saving clipboard images to the filesystem
//! with organized directory structure by date.

use std::path::{Path, PathBuf};

/// Error type for image file operations.
#[derive(Debug, Clone)]
pub enum ImageError {
    /// Failed to create directory
    DirectoryCreateFailed(String),
    /// Failed to write image file
    WriteFailed(String),
    /// Invalid image data
    InvalidImageData(String),
}

impl std::fmt::Display for ImageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DirectoryCreateFailed(msg) => write!(f, "Failed to create directory: {}", msg),
            Self::WriteFailed(msg) => write!(f, "Failed to write image: {}", msg),
            Self::InvalidImageData(msg) => write!(f, "Invalid image data: {}", msg),
        }
    }
}

impl std::error::Error for ImageError {}

/// Result type for image operations.
pub type Result<T> = std::result::Result<T, ImageError>;

/// Manager for saving clipboard images to disk.
///
/// Images are organized by date in the following structure:
///
/// ```text
/// ~/.local/share/clipboard_history/
/// └── images/
///     └── 2025-01-30/
///         ├── 143022_1584x852.png
///         └── 143145_1920x1080.png
/// ```
///
/// # Examples
///
/// ```no_run
/// use clipboard_history::core::image_manager::ImageManager;
///
/// let manager = ImageManager::new();
/// let image_data = vec![0x89, 0x50, 0x4E, 0x47]; // PNG magic bytes
///
/// match manager.save_image("143022", 1584, 852, &image_data) {
///     Ok(path) => println!("Saved to: {:?}", path),
///     Err(e) => eprintln!("Failed to save: {}", e),
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ImageManager {
    /// Base directory for image storage
    base_dir: PathBuf,
}

impl ImageManager {
    /// Create a new [`ImageManager`] with default directory.
    ///
    /// Uses `~/.local/share/clipboard_history/images/` on Linux,
    /// `~/Library/Application Support/clipboard_history/images/` on macOS,
    /// and `%APPDATA%\clipboard_history\images\` on Windows.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::core::image_manager::ImageManager;
    ///
    /// let manager = ImageManager::new();
    /// ```
    #[cfg(feature = "persistence")]
    pub fn new() -> Self {
        let base_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clipboard_history")
            .join("images");

        Self { base_dir }
    }

    /// Create a new [`ImageManager`] with a custom base directory.
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for image storage
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use clipboard_history::core::image_manager::ImageManager;
    ///
    /// let manager = ImageManager::with_base_dir(PathBuf::from("/tmp/clipboard_images"));
    /// ```
    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    /// Save an image to disk.
    ///
    /// Images are saved in subdirectories organized by date (YYYY-MM-DD format).
    /// The filename format is `{timestamp}_{width}x{height}.png`.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Timestamp string (HH:MM format, without colons becomes HHMM)
    /// * `width` - Image width in pixels
    /// * `height` - Image height in pixels
    /// * `png_bytes` - PNG image data
    ///
    /// # Returns
    ///
    /// Returns the relative path from `base_dir` to the saved image.
    ///
    /// # Errors
    ///
    /// Returns [`ImageError::DirectoryCreateFailed`] if the date directory cannot be created.
    /// Returns [`ImageError::WriteFailed`] if the image cannot be written.
    /// Returns [`ImageError::InvalidImageData`] if the data is empty.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clipboard_history::core::image_manager::ImageManager;
    /// # let manager = ImageManager::with_base_dir("/tmp/images".into());
    /// let png_bytes = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG header
    /// manager.save_image("143022", 1920, 1080, &png_bytes).unwrap();
    /// ```
    pub fn save_image(
        &self,
        timestamp: &str,
        width: u32,
        height: u32,
        png_bytes: &[u8],
    ) -> Result<String> {
        if png_bytes.is_empty() {
            return Err(ImageError::InvalidImageData("Empty image data".to_string()));
        }

        // Get current date for directory name
        let date_str = chrono_date_string();
        let date_dir = self.base_dir.join(&date_str);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&date_dir).map_err(|e| {
            ImageError::DirectoryCreateFailed(format!("{}: {}", date_dir.display(), e))
        })?;

        // Create filename: HHMM_WxH.png
        // Remove colons from timestamp (e.g., "14:30" -> "1430")
        let clean_timestamp = timestamp.replace(':', "");
        let filename = format!("{}_{}x{}.png", clean_timestamp, width, height);

        let file_path = date_dir.join(&filename);

        // Write image data
        std::fs::write(&file_path, png_bytes)
            .map_err(|e| ImageError::WriteFailed(format!("{}: {}", file_path.display(), e)))?;

        // Return relative path from base_dir
        Ok(format!("{}/{}", date_str, filename))
    }

    /// Get the full path for an image from its relative path.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Relative path returned by [`save_image`](Self::save_image)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use clipboard_history::core::image_manager::ImageManager;
    /// # let manager = ImageManager::with_base_dir("/tmp/images".into());
    /// let full_path = manager.get_full_path("2025-01-30/143022_1920x1080.png");
    /// assert_eq!(full_path, "/tmp/images/2025-01-30/143022_1920x1080.png");
    /// ```
    pub fn get_full_path(&self, relative_path: &str) -> PathBuf {
        self.base_dir.join(relative_path)
    }

    /// Delete an image file.
    ///
    /// # Arguments
    ///
    /// * `relative_path` - Relative path returned by [`save_image`](Self::save_image)
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be deleted.
    pub fn delete_image(&self, relative_path: &str) -> Result<()> {
        let full_path = self.get_full_path(relative_path);
        std::fs::remove_file(&full_path).map_err(|e| {
            ImageError::WriteFailed(format!("Failed to delete {}: {}", full_path.display(), e))
        })?;
        Ok(())
    }

    /// Get the base directory where images are stored.
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

impl Default for ImageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current date as YYYY-MM-DD string.
fn chrono_date_string() -> String {
    // Use a simple date format without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};

    if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
        let secs = duration.as_secs();
        let days_since_epoch = secs / 86400;

        // Unix epoch: January 1, 1970
        // Calculate year, month, day
        let mut days = days_since_epoch;
        let mut year = 1970;

        // Approximate leap year calculation
        while days >= 366 {
            let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            let days_in_year = if is_leap { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        // Simple month calculation (not perfectly accurate but sufficient for directory names)
        let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let mut month = 0;
        let mut day = days + 1; // 1-indexed day

        for (i, &days_in_month) in month_days.iter().enumerate() {
            let dim = if i == 1 && is_leap { 29 } else { days_in_month };
            if day <= dim {
                month = i + 1;
                break;
            }
            day -= dim;
        }

        format!("{:04}-{:02}-{:02}", year, month, day)
    } else {
        "1970-01-01".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_manager_creation() {
        let manager = ImageManager::with_base_dir(PathBuf::from("/tmp/test"));
        assert_eq!(manager.base_dir(), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_get_full_path() {
        let manager = ImageManager::with_base_dir(PathBuf::from("/tmp/images"));
        let path = manager.get_full_path("2025-01-30/143022_1920x1080.png");
        assert_eq!(path, PathBuf::from("/tmp/images/2025-01-30/143022_1920x1080.png"));
    }

    #[test]
    fn test_chrono_date_string_format() {
        let date = chrono_date_string();
        assert!(date.len() == 10); // YYYY-MM-DD
        assert!(date.contains('-'));
    }

    #[test]
    fn test_save_image_requires_data() {
        let manager = ImageManager::with_base_dir(PathBuf::from("/tmp/test"));
        let result = manager.save_image("143022", 1920, 1080, &[]);
        assert!(matches!(result, Err(ImageError::InvalidImageData(_))));
    }
}
