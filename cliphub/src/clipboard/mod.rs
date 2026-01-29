pub mod detector;
pub mod monitor;

pub use detector::{detect_content_type, ContentTypeDetection};
pub use monitor::ClipboardMonitor;
