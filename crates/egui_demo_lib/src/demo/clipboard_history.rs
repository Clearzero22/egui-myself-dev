use egui::Ui;

/// Clipboard History Demo
///
/// Demonstrates how to build a clipboard history manager interface.
/// This demo shows a list of clipboard items with search and filtering capabilities.
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct ClipboardHistory {
    items: Vec<ClipboardItem>,
    search_query: String,
    filter_type: FilterType,
    selected_index: Option<usize>,
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
                    title: "Code snippet".to_string(),
                    content: "fn main() { println!(\"Hello, world!\"); }".to_string(),
                    timestamp: "10:30".to_string(),
                    content_type: FilterType::Text,
                },
                ClipboardItem {
                    icon: "🔗",
                    title: "https://github.com".to_string(),
                    content: "https://github.com/emilk/egui".to_string(),
                    timestamp: "10:25".to_string(),
                    content_type: FilterType::Url,
                },
                ClipboardItem {
                    icon: "💻",
                    title: "RUST - Configuration".to_string(),
                    content: "[dependencies]\negui = \"0.33\"".to_string(),
                    timestamp: "10:20".to_string(),
                    content_type: FilterType::Text,
                },
                ClipboardItem {
                    icon: "📄",
                    title: "Meeting notes".to_string(),
                    content: "Discussed the new UI design for the clipboard manager".to_string(),
                    timestamp: "09:45".to_string(),
                    content_type: FilterType::Text,
                },
                ClipboardItem {
                    icon: "🔗",
                    title: "Documentation".to_string(),
                    content: "https://docs.rs/egui/".to_string(),
                    timestamp: "09:30".to_string(),
                    content_type: FilterType::Url,
                },
            ],
            search_query: String::new(),
            filter_type: FilterType::All,
            selected_index: None,
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
            .default_width(600.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

impl crate::View for ClipboardHistory {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Clipboard History");
        ui.label("A demo showing how to build a clipboard history manager interface");
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

                    egui::Frame::none()
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
                if ui.button("Add Random").clicked() {
                    self.add_random_item();
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

    fn add_random_item(&mut self) {
        let examples = [
            ("📄", "Text snippet", "Some example text content"),
            ("🔗", "https://example.com", "https://example.com/page"),
            ("💻", "RUST - Function", "pub fn example() { true }"),
            ("📄", "Note to self", "Remember to check the PR"),
        ];

        let item = examples[self.items.len() % examples.len()];
        self.items.insert(0, ClipboardItem {
            icon: item.0,
            title: item.1.to_string(),
            content: item.2.to_string(),
            timestamp: format!("{:02}:{:02}",
                (self.items.len() * 7) % 24,
                (self.items.len() * 13) % 60),
            content_type: if item.0 == "🔗" { FilterType::Url } else { FilterType::Text },
        });

        // Keep only 20 items
        if self.items.len() > 20 {
            self.items.truncate(20);
        }
    }
}
