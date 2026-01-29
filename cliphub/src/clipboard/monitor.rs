use arboard::Clipboard;
use std::time::Duration;
use tokio::time::interval;
use crate::types::ClipboardItem;
use crate::clipboard::detector::detect_content_type;
use crate::storage::Database;
use anyhow::Result;
use chrono::Utc;

pub struct ClipboardMonitor {
    last_content: String,
    interval: Duration,
}

impl ClipboardMonitor {
    pub fn new(interval: Duration) -> Self {
        Self {
            last_content: String::new(),
            interval,
        }
    }

    pub async fn run(&mut self, db: &Database) -> Result<()> {
        let mut timer = interval(self.interval);
        let mut clipboard = Clipboard::new()?;

        loop {
            timer.tick().await;

            if let Ok(content) = clipboard.get_text() {
                if content != self.last_content && !content.is_empty() {
                    let detection = detect_content_type(&content);

                    let item = ClipboardItem {
                        id: None,
                        content_type: detection.content_type.clone(),
                        title: generate_title(&content, &detection),
                        content: content.clone(),
                        source_app: detect_source_app(),
                        created_at: Utc::now(),
                        is_synced: false,
                        sync_targets: Vec::new(),
                        tags: Vec::new(),
                    };

                    if let Ok(id) = db.insert_item(&item) {
                        println!("Captured clipboard item: {}", id);
                    }

                    self.last_content = content;
                }
            }
        }
    }
}

fn generate_title(content: &str, detection: &crate::clipboard::detector::ContentTypeDetection) -> Option<String> {
    let preview = content.lines().next().unwrap_or("");
    let truncated = if preview.len() > 50 {
        format!("{}...", &preview[..50])
    } else {
        preview.to_string()
    };

    if let Some(lang) = &detection.language {
        Some(format!("{} - {}", lang.to_uppercase(), truncated))
    } else {
        Some(truncated)
    }
}

fn detect_source_app() -> Option<String> {
    // Platform-specific implementation
    #[cfg(target_os = "linux")]
    {
        return None; // TODO: implement
    }

    #[cfg(target_os = "macos")]
    {
        return None; // TODO: use NSWorkspace
    }

    #[cfg(target_os = "windows")]
    {
        return None; // TODO: use GetForegroundWindow
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        None
    }
}
