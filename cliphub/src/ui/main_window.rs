use crate::types::ClipboardItem;
use crate::storage::Database;

pub struct MainWindow {
    items: Vec<ClipboardItem>,
    search_query: String,
    selected_index: Option<usize>,
    db: Database,
}

impl MainWindow {
    pub fn new(db: Database) -> Self {
        let items = db.get_recent_items(50).unwrap_or_default();

        Self {
            items,
            search_query: String::new(),
            selected_index: None,
            db,
        }
    }

    pub fn refresh_items(&mut self) {
        self.items = self.db.get_recent_items(50).unwrap_or_default();
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("📋 ClipHub");
                ui.separator();
                if ui.button("🔄 Refresh").clicked() {
                    self.refresh_items();
                }
            });
        });

        egui::TopBottomPanel::top("search").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.search_query);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.show_item_list(ui);
        });
    }

    fn show_item_list(&mut self, ui: &mut egui::Ui) {
        let filtered: Vec<_> = self.items
            .iter()
            .filter(|item| {
                self.search_query.is_empty() ||
                item.content.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                item.title.as_ref().map(|t| t.to_lowercase().contains(&self.search_query.to_lowercase())).unwrap_or(false)
            })
            .enumerate()
            .collect();

        for (idx, item) in filtered {
            let is_selected = self.selected_index == Some(idx);

            egui::Frame::none()
                .fill(if is_selected { ui.visuals().faint_bg_color } else { egui::Color32::TRANSPARENT })
                .show(ui, |ui| {
                    self.show_item_card(ui, item);
                });
        }
    }

    fn show_item_card(&self, ui: &mut egui::Ui, item: &ClipboardItem) {
        ui.vertical(|ui| {
            // Title row
            ui.horizontal(|ui| {
                let icon = match &item.content_type {
                    crate::types::ContentType::Text => "📄",
                    crate::types::ContentType::Code { .. } => "💻",
                    crate::types::ContentType::Image { .. } => "🖼️",
                    crate::types::ContentType::Url { .. } => "🔗",
                };
                ui.label(format!("{} {}", icon, item.title.as_ref().unwrap_or(&"Untitled".to_string())));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("{}", item.created_at.format("%H:%M")));
                });
            });

            // Content preview
            let preview = if item.content.len() > 100 {
                format!("{}...", &item.content[..100])
            } else {
                item.content.clone()
            };
            ui.label(egui::RichText::new(preview).small().weak());

            // Tags
            if !item.tags.is_empty() {
                ui.horizontal(|ui| {
                    for tag in &item.tags {
                        ui.label(egui::RichText::new(format!("🏷️ {}", tag)).small());
                    }
                });
            }
        });
        ui.separator();
    }
}
