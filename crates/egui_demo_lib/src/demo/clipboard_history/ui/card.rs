//! Item card rendering component.
//!
//! This module provides the [`ItemCardRenderer`] which is responsible for
//! rendering individual clipboard item cards in the UI.

use crate::demo::clipboard_history::core::item::ClipboardItem;
use crate::demo::clipboard_history::core::item::ContentType;

/// Action that can be performed on an item card.
#[derive(Clone, Debug)]
pub enum CardAction {
    /// Copy the item's content to clipboard
    Copy(String),
    /// Edit the item's content
    Edit(usize),
    /// Delete the item
    Delete(usize),
}

/// Renderer for clipboard item cards.
///
/// This component handles the rendering of individual clipboard items,
/// including their icon, title, content preview, and action buttons.
///
/// # Examples
///
/// ```no_run
/// use clipboard_history::ui::card::ItemCardRenderer;
/// use clipboard_history::core::item::ClipboardItem;
///
/// let renderer = ItemCardRenderer::new();
/// // In your UI code:
/// // renderer.render(ui, &item, index, &mut actions);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct ItemCardRenderer;

impl ItemCardRenderer {
    /// Create a new [`ItemCardRenderer`].
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::card::ItemCardRenderer;
    ///
    /// let renderer = ItemCardRenderer::new();
    /// ```
    pub fn new() -> Self {
        Self
    }

    /// Render a clipboard item card.
    ///
    /// This method renders the item's icon, title, content preview, and action buttons.
    /// Any actions triggered by button clicks are collected in the `actions` vector.
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui UI context
    /// * `item` - The clipboard item to render
    /// * `index` - The index of the item (used for edit/delete actions)
    /// * `actions` - A vector to collect any triggered actions
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use egui::Ui;
    /// # use clipboard_history::ui::card::{ItemCardRenderer, CardAction};
    /// # use clipboard_history::core::item::ClipboardItem;
    ///
    /// fn render_item(ui: &mut egui::Ui, item: &ClipboardItem, index: usize) -> Vec<CardAction> {
    ///     let mut actions = Vec::new();
    ///     let renderer = ItemCardRenderer::new();
    ///     renderer.render(ui, item, index, &mut actions);
    ///     actions
    /// }
    /// ```
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        index: usize,
        actions: &mut Vec<CardAction>,
    ) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                self.render_title_row(ui, item);
                self.render_content_preview(ui, item);
                ui.separator();
                self.render_action_buttons(ui, item, index, actions);
            });
    }

    /// Render just the title row of an item card.
    ///
    /// This can be useful for compact display modes or custom layouts.
    pub fn render_title_row(&self, ui: &mut egui::Ui, item: &ClipboardItem) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(format!("{} {}", item.icon, item.title)).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(&item.timestamp).small().weak());
            });
        });
    }

    /// Render just the content preview of an item card.
    pub fn render_content_preview(&self, ui: &mut egui::Ui, item: &ClipboardItem) {
        // If this is an image, show a visual indicator
        if item.content_type == ContentType::Image {
            // Show image icon and metadata
            ui.horizontal(|ui| {
                // Image icon (large)
                ui.label(egui::RichText::new("🖼️").size(24.0));

                // Image info
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new(&item.title).strong());

                    // Show file size if available
                    if let Some(ref data) = item.image_data {
                        let size_kb = data.len() / 1024;
                        ui.label(egui::RichText::new(format!("{} KB", size_kb)).small().weak());
                    }

                    ui.label(egui::RichText::new(&item.content).small().weak());
                });
            });
        } else {
            // For text items, show text preview
            let preview = item.preview(100);
            ui.label(egui::RichText::new(preview).small().weak());
        }
    }

    /// Render just the action buttons of an item card.
    pub fn render_action_buttons(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        index: usize,
        actions: &mut Vec<CardAction>,
    ) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            if ui.button(egui::RichText::new("📋 Copy").small()).clicked() {
                actions.push(CardAction::Copy(item.content.clone()));
            }

            if ui.button(egui::RichText::new("✏️ Edit").small()).clicked() {
                actions.push(CardAction::Edit(index));
            }

            if ui
                .button(egui::RichText::new("🗑️ Delete").small().color(egui::Color32::RED))
                .clicked()
            {
                actions.push(CardAction::Delete(index));
            }

            // Type tag
            let tag_text = self.get_type_tag(&item.content_type);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new(tag_text).small().weak());
            });
        });
    }

    /// Get the display tag for a content type.
    fn get_type_tag(&self, content_type: &ContentType) -> &'static str {
        match content_type {
            ContentType::Text => "📝 Text",
            ContentType::Url => "🔗 Link",
            ContentType::Image => "🖼️ Image",
            ContentType::Code { .. } => "💻 Code",
            ContentType::File { .. } => "📁 File",
            ContentType::Custom(_) => "📦 Other",
        }
    }
}

impl Default for ItemCardRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_card_renderer_creation() {
        let renderer = ItemCardRenderer::new();
        let default_renderer = ItemCardRenderer::default();
        // Just verify they can be created
    }

    #[test]
    fn test_get_type_tag() {
        let renderer = ItemCardRenderer::new();

        assert_eq!(renderer.get_type_tag(&ContentType::Text), "📝 Text");
        assert_eq!(renderer.get_type_tag(&ContentType::Url), "🔗 Link");
        assert_eq!(renderer.get_type_tag(&ContentType::Image), "🖼️ Image");
        assert_eq!(
            renderer.get_type_tag(&ContentType::Code {
                language: "rust".to_string()
            }),
            "💻 Code"
        );
        assert_eq!(
            renderer.get_type_tag(&ContentType::File {
                extension: "txt".to_string()
            }),
            "📁 File"
        );
        assert_eq!(
            renderer.get_type_tag(&ContentType::Custom("my-type".to_string())),
            "📦 Other"
        );
    }

    #[test]
    fn test_card_action_types() {
        let copy_action = CardAction::Copy("test content".to_string());
        let edit_action = CardAction::Edit(5);
        let delete_action = CardAction::Delete(10);

        // Verify actions can be cloned
        assert!(copy_action.clone().matches(&copy_action));
        assert!(edit_action.clone().matches(&edit_action));
        assert!(delete_action.clone().matches(&delete_action));
    }
}

// Helper for testing
impl CardAction {
    fn matches(&self, other: &CardAction) -> bool {
        match (self, other) {
            (CardAction::Copy(a), CardAction::Copy(b)) => a == b,
            (CardAction::Edit(a), CardAction::Edit(b)) => a == b,
            (CardAction::Delete(a), CardAction::Delete(b)) => a == b,
            _ => false,
        }
    }
}
