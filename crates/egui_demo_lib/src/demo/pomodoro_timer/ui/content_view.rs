//! Content display component.

use crate::demo::pomodoro_timer::content::Pager;
use egui::{self, Ui};

/// Content view renderer.
pub struct ContentView {
    font_size: f32,
    line_height: f32,
    show_page_number: bool,
}

impl ContentView {
    /// Create new content view.
    pub fn new() -> Self {
        Self {
            font_size: 16.0,
            line_height: 1.5,
            show_page_number: true,
        }
    }

    /// Render content view.
    pub fn render(&self, ui: &mut Ui, pager: &dyn Pager) {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(16.0, 8.0))
            .show(ui, |ui| {
                // Content display
                egui::ScrollArea::vertical()
                    .auto_shrink(false)
                    .max_height(200.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(pager.current_content().as_ref())
                                .size(self.font_size)
                        );
                    });

                ui.separator();

                // Page info
                ui.horizontal(|ui| {
                    if self.show_page_number {
                        ui.label(format!(
                            "第 {} 页 / 共 {} 页",
                            pager.current_page() + 1,
                            pager.total_pages()
                        ));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Navigation hints
                        if !pager.has_previous() {
                            ui.label(egui::RichText::new("◀").weak());
                        } else {
                            ui.label("◀");
                        }

                        ui.label(" ");

                        if !pager.has_next() {
                            ui.label(egui::RichText::new("▶").weak());
                        } else {
                            ui.label("▶");
                        }
                    });
                });
            });
    }

    /// Set whether to show page numbers.
    pub fn with_show_page_number(mut self, show: bool) -> Self {
        self.show_page_number = show;
        self
    }

    /// Set font size.
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
}

impl Default for ContentView {
    fn default() -> Self {
        Self::new()
    }
}
