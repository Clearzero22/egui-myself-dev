//! Item card rendering component.
//!
//! This module provides the [`ItemCardRenderer`] which is responsible for
//! rendering individual clipboard item cards in the UI.

use crate::demo::clipboard_history::core::item::ClipboardItem;
use crate::demo::clipboard_history::core::item::ContentType;
use egui::Widget; // For .ui() method on Image
use std::collections::HashMap;

/// Unique ID for storing image textures per item.
///
/// Each clipboard item with image data gets a unique texture ID
/// based on its timestamp, allowing the texture to be cached and reused.
fn image_texture_id(item: &ClipboardItem) -> String {
    format!("clipboard_img_{}", item.timestamp)
}

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
    /// * `texture_cache` - A cache for image textures to avoid reloading
    /// * `load_image_fn` - Optional callback to load image data on-demand
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use egui::Ui;
    /// # use clipboard_history::ui::card::{ItemCardRenderer, CardAction};
    /// # use clipboard_history::core::item::ClipboardItem;
    /// # use std::collections::HashMap;
    ///
    /// fn render_item(
    ///     ui: &mut egui::Ui,
    ///     item: &ClipboardItem,
    ///     index: usize,
    ///     texture_cache: &mut HashMap<String, egui::TextureHandle>,
    ///     load_image_fn: impl Fn(&str) -> Option<Vec<u8>>,
    /// ) -> Vec<CardAction> {
    ///     let mut actions = Vec::new();
    ///     let renderer = ItemCardRenderer::new();
    ///     renderer.render(ui, item, index, &mut actions, texture_cache, Some(&load_image_fn));
    ///     actions
    /// }
    /// ```
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        index: usize,
        actions: &mut Vec<CardAction>,
        texture_cache: &mut HashMap<String, egui::TextureHandle>,
        load_image_fn: Option<&impl Fn(&str) -> Option<Vec<u8>>>,
    ) {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                self.render_title_row(ui, item);
                self.render_content_preview(ui, item, texture_cache, load_image_fn);
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
    pub fn render_content_preview(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        texture_cache: &mut HashMap<String, egui::TextureHandle>,
        load_image_fn: Option<&impl Fn(&str) -> Option<Vec<u8>>>,
    ) {
        // If this is an image, show a visual indicator
        if item.content_type == ContentType::Image {
            // Try to load and display the actual image
            if let Some(ref png_bytes) = item.image_data {
                self.render_image_thumbnail(ui, item, png_bytes, texture_cache);
            } else if let Some(load_fn) = load_image_fn {
                // PERF: Lazy load image from persistent storage
                if let Some(png_bytes) = load_fn(&item.timestamp) {
                    self.render_image_thumbnail(ui, item, &png_bytes, texture_cache);
                } else {
                    self.render_image_metadata(ui, item);
                }
            } else {
                // Fallback: show image icon and metadata only
                self.render_image_metadata(ui, item);
            }
        } else {
            // For text items, show text preview
            let preview = item.preview(100);
            ui.label(egui::RichText::new(preview).small().weak());
        }
    }

    /// Render an actual image thumbnail from PNG bytes.
    fn render_image_thumbnail(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        png_bytes: &[u8],
        texture_cache: &mut HashMap<String, egui::TextureHandle>,
    ) {
        let texture_id = image_texture_id(item);

        // Check cache first - only decode and load texture if not cached
        let texture = texture_cache.entry(texture_id.clone()).or_insert_with(|| {
            // Decode the PNG using egui_extras
            match egui_extras::image::load_image_bytes(png_bytes) {
                Ok(color_image) => {
                    ui.ctx().load_texture(
                        texture_id,
                        color_image,
                        egui::TextureOptions::LINEAR,
                    )
                }
                Err(_) => {
                    // Create a fallback error texture
                    let size = [1, 1];
                    let pixels = vec![egui::Color32::RED];
                    let color_image = egui::ColorImage::new(size, pixels);
                    ui.ctx().load_texture(
                        format!("{}_error", texture_id),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    )
                }
            }
        });

        // Get image dimensions from texture
        let size = texture.size_vec2();

        // Display the image thumbnail
        ui.horizontal(|ui| {
            // Show thumbnail with max height of 80px, maintaining aspect ratio
            egui::Image::new(&*texture)
                .max_height(80.0)
                .maintain_aspect_ratio(true)
                .shrink_to_fit()
                .ui(ui);

            // Image info next to thumbnail
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&item.title).strong());

                let size_kb = png_bytes.len() / 1024;
                ui.label(
                    egui::RichText::new(format!("{} × {} · {} KB", size.x as usize, size.y as usize, size_kb))
                        .small()
                        .weak(),
                );
            });
        });
    }

    /// Fallback: render image metadata when image data is not available.
    fn render_image_metadata(&self, ui: &mut egui::Ui, item: &ClipboardItem) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("🖼️").size(24.0));
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&item.title).strong());
                ui.label(egui::RichText::new(&item.content).small().weak());
            });
        });
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
