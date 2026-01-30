use std::sync::{Arc, Mutex};

/// Clipboard History Demo
///
/// Demonstrates how to build a clipboard history manager interface.
/// This demo shows a list of clipboard items with search and filtering capabilities.
/// Now with real clipboard integration using arboard!
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    last_content: Arc<Mutex<String>>,
    search_query: String,
    filter_type: FilterType,
    selected_index: Option<usize>,
    auto_capture: bool,
    new_item_content: String,
    show_add_dialog: bool,
    editing_index: Option<usize>,
    edit_content: String,
}

#[derive(PartialEq, Eq, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum FilterType {
    #[default]
    All,
    Text,
    Url,
    Image,
}

#[derive(Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct ClipboardItem {
    icon: String,
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
                    icon: "📄".to_string(),
                    title: "Welcome!".to_string(),
                    content: "Copy some text to see it appear here!".to_string(),
                    timestamp: "Now".to_string(),
                    content_type: FilterType::Text,
                },
            ],
            last_content: Arc::default(),
            search_query: String::default(),
            filter_type: FilterType::default(),
            selected_index: None,
            auto_capture: true,
            new_item_content: String::default(),
            show_add_dialog: false,
            editing_index: None,
            edit_content: String::default(),
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
        ui.heading("📋 Clipboard History");
        ui.label("Real-time clipboard monitoring - copy anything to see it here!");
        ui.separator();

        // Top action bar
        ui.horizontal(|ui| {
            ui.label("Auto-capture:");
            ui.checkbox(&mut self.auto_capture, "Enabled");

            ui.separator();

            if ui.button("➕ Add Content").clicked() {
                self.show_add_dialog = true;
                // Try to paste from clipboard
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(content) = clipboard.get_text() {
                        self.new_item_content = content;
                    }
                }
            }

            if ui.button("📋 Paste Current").clicked() {
                self.paste_from_clipboard();
            }
        });

        // Add content dialog
        if self.show_add_dialog {
            egui::Window::new("Add New Content")
                .collapsible(false)
                .resizable(false)
                .show(ui.ctx(), |ui| {
                    ui.label("Content:");
                    ui.add_sized(
                        [400.0, 100.0],
                        egui::TextEdit::multiline(&mut self.new_item_content)
                            .hint_text("Enter content to save...")
                    );

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_add_dialog = false;
                            self.new_item_content.clear();
                        }
                        if ui.button("Add").clicked() {
                            if !self.new_item_content.trim().is_empty() {
                                self.add_clipboard_item(self.new_item_content.clone());
                                self.new_item_content.clear();
                                self.show_add_dialog = false;
                            }
                        }
                    });
                });
        }

        // Edit dialog
        if let Some(edit_idx) = self.editing_index {
            if edit_idx < self.items.len() {
                egui::Window::new("Edit Content")
                    .collapsible(false)
                    .resizable(false)
                    .show(ui.ctx(), |ui| {
                        ui.label("Edit Content:");
                        ui.add_sized(
                            [400.0, 100.0],
                            egui::TextEdit::multiline(&mut self.edit_content)
                        );

                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {
                                self.editing_index = None;
                                self.edit_content.clear();
                            }
                            if ui.button("Save").clicked() {
                                if !self.edit_content.trim().is_empty() {
                                    self.items[edit_idx].content = self.edit_content.clone();
                                    self.items[edit_idx].title = self.generate_title(&self.edit_content);
                                }
                                self.editing_index = None;
                                self.edit_content.clear();
                            }
                        });
                    });
            } else {
                self.editing_index = None;
            }
        }

        ui.separator();

        // Auto-capture clipboard
        if self.auto_capture {
            self.auto_capture_clipboard();
        }

        ui.separator();

        // Search and filter bar
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search content..."));

            ui.separator();

            ui.label("Filter:");
            ui.radio_value(&mut self.filter_type, FilterType::All, "All");
            ui.radio_value(&mut self.filter_type, FilterType::Text, "Text");
            ui.radio_value(&mut self.filter_type, FilterType::Url, "URLs");
            ui.radio_value(&mut self.filter_type, FilterType::Image, "Images");
        });

        ui.separator();

        // Item list
        let filtered_count = self.items
            .iter()
            .filter(|item| {
                if self.filter_type != FilterType::All && item.content_type != self.filter_type {
                    return false;
                }
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
                let mut items_to_remove: Vec<usize> = Vec::new();
                let mut items_to_copy: Vec<String> = Vec::new();
                let mut items_to_edit: Vec<usize> = Vec::new();

                for (idx, item) in self.items.iter().enumerate() {
                    // Apply filters
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
                            show_item_card_static(ui, item, idx, &mut items_to_remove, &mut items_to_copy, &mut items_to_edit);
                        });
                }

                // Handle actions
                for content in items_to_copy {
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(&content);
                    }
                }
                for idx in items_to_edit {
                    if idx < self.items.len() {
                        self.editing_index = Some(idx);
                        self.edit_content = self.items[idx].content.clone();
                    }
                }
                // Remove from back to front to avoid index issues
                items_to_remove.sort();
                items_to_remove.dedup();
                for idx in items_to_remove.into_iter().rev() {
                    if idx < self.items.len() {
                        self.items.remove(idx);
                    }
                }
            });

        ui.separator();

        // Status bar
        ui.horizontal(|ui| {
            ui.label(format!("Showing {} items", filtered_count));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear All").clicked() {
                    self.items.clear();
                }
                if ui.button("📋 Copy Latest").clicked() {
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

    fn add_image_item(&mut self, width: u32, height: u32) {
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
            icon: "🖼️".to_string(),
            title: format!("Screenshot ({}x{})", width, height),
            content: format!("[Image: {}x{} pixels]", width, height),
            timestamp,
            content_type: FilterType::Image,
        };

        self.items.insert(0, item);

        // Keep only 50 items
        if self.items.len() > 50 {
            self.items.truncate(50);
        }
    }

    fn paste_from_clipboard(&mut self) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            // First try to get text
            if let Ok(content) = clipboard.get_text() {
                if !content.is_empty() {
                    self.add_clipboard_item(content);
                    return;
                }
            }

            // If no text, try to get image
            if let Ok(image) = clipboard.get_image() {
                self.add_image_item(
                    image.width.try_into().unwrap(),
                    image.height.try_into().unwrap()
                );
            }
        }
    }

    fn auto_capture_clipboard(&mut self) {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            // Try text first
            if let Ok(content) = clipboard.get_text() {
                let is_new = {
                    let last = self.last_content.lock().unwrap();
                    content != *last && !content.is_empty()
                };

                if is_new {
                    self.add_clipboard_item(content.clone());
                    *self.last_content.lock().unwrap() = content;
                    return;
                }
            }

            // Try image if no text
            if let Ok(image) = clipboard.get_image() {
                let image_key = format!("IMG_{}x{}", image.width, image.height);
                let is_new = {
                    let last = self.last_content.lock().unwrap();
                    image_key != *last
                };

                if is_new {
                    self.add_image_item(
                        image.width.try_into().unwrap(),
                        image.height.try_into().unwrap()
                    );
                    *self.last_content.lock().unwrap() = image_key;
                }
            }
        }
    }

    fn detect_content_type(&self, content: &str) -> (String, FilterType) {
        let content_lower = content.to_lowercase();

        // Check for image file extensions
        if content_lower.ends_with(".png") || content_lower.ends_with(".jpg")
            || content_lower.ends_with(".jpeg") || content_lower.ends_with(".gif")
            || content_lower.ends_with(".webp") || content_lower.ends_with(".svg")
            || content_lower.contains(".png") || content_lower.contains(".jpg")
            || content_lower.contains(".jpeg") || content_lower.contains(".gif") {
            return ("🖼️".to_string(), FilterType::Image);
        }

        // Check for URL
        if content_lower.starts_with("http://") || content_lower.starts_with("https://") {
            return ("🔗".to_string(), FilterType::Url);
        }

        // Check for code patterns
        if content_lower.contains("fn ") || content_lower.contains("def ")
            || content_lower.contains("function ") || content_lower.contains("=> ")
            || content_lower.contains("class ") || content_lower.contains("import ") {
            return ("💻".to_string(), FilterType::Text);
        }

        // Check for email
        if content_lower.contains('@') && content_lower.contains('.') {
            return ("📧".to_string(), FilterType::Text);
        }

        // Default to text
        ("📄".to_string(), FilterType::Text)
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

/// Static function: Display clipboard item card
/// Does not require &mut self, collects actions via callbacks
fn show_item_card_static(
    ui: &mut egui::Ui,
    item: &ClipboardItem,
    idx: usize,
    items_to_remove: &mut Vec<usize>,
    items_to_copy: &mut Vec<String>,
    items_to_edit: &mut Vec<usize>,
) {
    egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(8, 4))
        .show(ui, |ui| {
            // Title row
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{} {}", item.icon, item.title)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&item.timestamp).small().weak());
                });
            });

            // Content preview
            let preview = if item.content.len() > 100 {
                format!("{}...", &item.content[..97])
            } else {
                item.content.clone()
            };
            ui.label(egui::RichText::new(preview).small().weak());

            // Action buttons
            ui.separator();
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                if ui.button(egui::RichText::new("📋 Copy").small()).clicked() {
                    items_to_copy.push(item.content.clone());
                }

                if ui.button(egui::RichText::new("✏️ Edit").small()).clicked() {
                    items_to_edit.push(idx);
                }

                if ui.button(egui::RichText::new("🗑️ Delete").small().color(egui::Color32::RED)).clicked() {
                    items_to_remove.push(idx);
                }

                // Tag
                let tag_text = match item.content_type {
                    FilterType::Text => "📝 Text",
                    FilterType::Url => "🔗 Link",
                    FilterType::Image => "🖼️ Image",
                    FilterType::All => "📄 Other",
                };
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(tag_text).small().weak());
                });
            });
        });
}
