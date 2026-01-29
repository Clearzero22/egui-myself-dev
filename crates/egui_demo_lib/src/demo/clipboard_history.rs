use egui::Ui;
use std::sync::{Arc, Mutex};

/// Clipboard History Demo
///
/// Demonstrates how to build a clipboard history manager interface.
/// This demo shows a list of clipboard items with search and filtering capabilities.
/// Now with real clipboard integration using arboard!
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    last_content: Arc<Mutex<String>>,
    search_query: String,
    filter_type: FilterType,
    selected_index: Option<usize>,
    auto_capture: bool,
}

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
enum FilterType {
    All,
    Text,
    Url,
}

#[derive(Clone, PartialEq)]
struct ClipboardItem {
    icon: &'static str,
    title: String,
    content: String,
    timestamp: String,
    content_type: FilterType,
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self {
            items: vec![
                ClipboardItem {
                    icon: "📄",
                    title: "Welcome!".to_string(),
                    content: "Copy some text to see it appear here!".to_string(),
                    timestamp: "Now".to_string(),
                    content_type: FilterType::Text,
                },
            ],
            last_content: Arc::new(Mutex::new(String::new())),
            search_query: String::new(),
            filter_type: FilterType::All,
            selected_index: None,
            auto_capture: true,
        }
    }
}

impl crate::Demo for ClipboardHistory {
    fn name(&self) -> &'static str {
        "📋 Clipboard History"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(700.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

impl crate::View for ClipboardHistory {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Clipboard History");
        ui.label("Real-time clipboard monitoring - copy anything to see it here!");
        ui.separator();

        // Settings
        ui.horizontal(|ui| {
            ui.label("Auto-capture:");
            ui.checkbox(&mut self.auto_capture, "Enabled");
        });

        ui.separator();

        // Try to read clipboard when auto_capture is enabled
        if self.auto_capture {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(content) = clipboard.get_text() {
                    let is_new = {
                        let last = self.last_content.lock().unwrap();
                        content != *last && !content.is_empty()
                    };

                    if is_new {
                        // Add new item to the list
                        self.add_clipboard_item(content.clone());
                        *self.last_content.lock().unwrap() = content;
                    }
                }
            }
        }

        ui.separator();

        // Search and filter bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.text_edit_singleline(&mut self.search_query);

            ui.separator();

            ui.label("Filter:");
            ui.radio_value(&mut self.filter_type, FilterType::All, "All");
            ui.radio_value(&mut self.filter_type, FilterType::Text, "Text");
            ui.radio_value(&mut self.filter_type, FilterType::Url, "URLs");
        });

        ui.separator();

        // Item list
        let filtered_count = self.items
            .iter()
            .filter(|item| {
                // Apply type filter
                if self.filter_type != FilterType::All && item.content_type != self.filter_type {
                    return false;
                }
                // Apply search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    item.title.to_lowercase().contains(&query)
                        || item.content.to_lowercase().contains(&query)
                } else {
                    true
                }
            })
            .count();

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .max_height(400.0)
            .show(ui, |ui| {
                for (idx, item) in self.items.iter().enumerate() {
                    // Apply filters again for display
                    if self.filter_type != FilterType::All && item.content_type != self.filter_type {
                        continue;
                    }
                    if !self.search_query.is_empty() {
                        let query = self.search_query.to_lowercase();
                        if !item.title.to_lowercase().contains(&query)
                            && !item.content.to_lowercase().contains(&query) {
                            continue;
                        }
                    }

                    let is_selected = self.selected_index == Some(idx);
                    let fill = if is_selected {
                        ui.visuals().faint_bg_color
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    egui::Frame::NONE
                        .fill(fill)
                        .show(ui, |ui| {
                            self.show_item_card(ui, item);
                        });
                }
            });

        ui.separator();

        // Status bar
        ui.horizontal(|ui| {
            ui.label(format!("Showing {} items", filtered_count));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear All").clicked() {
                    self.items.clear();
                }
                if ui.button("Copy Last").clicked() {
                    if let Some(last) = self.items.first() {
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(&last.content);
                        }
                    }
                }
            });
        });
    }
}

impl ClipboardHistory {
    fn show_item_card(&self, ui: &mut egui::Ui, item: &ClipboardItem) {
        ui.horizontal(|ui| {
            ui.label(format!("{} {}", item.icon, item.title));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&item.timestamp).small().weak());
            });
        });

        let preview = if item.content.len() > 80 {
            format!("{}...", &item.content[..77])
        } else {
            item.content.clone()
        };
        ui.label(egui::RichText::new(preview).small().weak());
    }

    fn add_clipboard_item(&mut self, content: String) {
        // Detect content type
        let (icon, content_type) = self.detect_content_type(&content);

        // Generate title from content
        let title = self.generate_title(&content);

        // Generate timestamp
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let timestamp = format!("{:02}:{:02}",
            (now % 86400 / 3600) as u32,
            (now % 3600 / 60) as u32
        );

        let item = ClipboardItem {
            icon,
            title,
            content,
            timestamp,
            content_type,
        };

        self.items.insert(0, item);

        // Keep only 50 items
        if self.items.len() > 50 {
            self.items.truncate(50);
        }
    }

    fn detect_content_type(&self, content: &str) -> (&'static str, FilterType) {
        let content_lower = content.to_lowercase();

        // Check for URL
        if content_lower.starts_with("http://") || content_lower.starts_with("https://") {
            return ("🔗", FilterType::Url);
        }

        // Check for code patterns
        if content_lower.contains("fn ") || content_lower.contains("def ")
            || content_lower.contains("function ") || content_lower.contains("=> ") {
            return ("💻", FilterType::Text);
        }

        // Default to text
        ("📄", FilterType::Text)
    }

    fn generate_title(&self, content: &str) -> String {
        let preview = content.lines().next().unwrap_or(content);
        let truncated = if preview.len() > 40 {
            format!("{}...", &preview[..37])
        } else {
            preview.to_string()
        };

        truncated
    }
}
