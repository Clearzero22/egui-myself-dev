//! Clipboard History Demo
//!
//! This demo showcases a clipboard history manager with:
//!
//! - Real-time clipboard monitoring
//! - Manual content addition
//! - Search and filtering
//! - Edit and delete operations
//!
//! # Architecture
//!
//! The clipboard history system is organized into modules:
//!
//! - [`core`]: Data models, storage abstraction, and filtering
//! - [`clipboard`]: Clipboard backend abstraction and implementations
//! - [`ui`]: UI components for rendering dialogs and item cards
//!
//! # Extension Points
//!
//! - Implement [`core::store::Store`] for different storage backends
//! - Implement [`clipboard::backend::Backend`] for platform-specific clipboards
//! - Implement [`core::filter::Filter`] for custom filtering strategies

pub mod core;
pub mod clipboard;
pub mod ui;

use core::{Store, MemoryStore, ContentType};
use clipboard::{ArboardBackend, Backend};
use ui::{DialogManager, ItemCardRenderer, CardAction};

/// Filter mode for the UI.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    All,
    Text,
    Url,
    Image,
}

impl FilterMode {
    /// Get the corresponding [`ContentType`] for this filter mode.
    fn to_content_types(self) -> Vec<ContentType> {
        match self {
            Self::All => vec![
                ContentType::Text,
                ContentType::Url,
                ContentType::Image,
                ContentType::Code { language: "any".to_string() },
                ContentType::File { extension: "any".to_string() },
            ],
            Self::Text => vec![ContentType::Text],
            Self::Url => vec![ContentType::Url],
            Self::Image => vec![ContentType::Image],
        }
    }
}

/// Clipboard History Demo Application.
///
/// This is the main application struct that manages the clipboard history,
/// including storage, clipboard integration, and UI state.
///
/// # Examples
///
/// ```no_run
/// use egui_demo_lib::demo::clipboard_history::ClipboardHistory;
///
/// let app = ClipboardHistory::default();
/// // In your egui app:
/// // app.show(&ctx, &mut open);
/// ```
pub struct ClipboardHistory {
    /// Storage for clipboard items
    store: MemoryStore,
    /// Clipboard backend
    clipboard: ArboardBackend,
    /// Search query string
    search_query: String,
    /// Current filter mode
    filter_mode: FilterMode,
    /// Selected item index
    selected_index: Option<usize>,
    /// Auto-capture enabled
    auto_capture: bool,
    /// Dialog manager
    dialogs: DialogManager,
    /// Pending actions from item cards
    pending_actions: Vec<CardAction>,
}

impl ClipboardHistory {
    /// Create a new [`ClipboardHistory`] with default settings.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_demo_lib::demo::clipboard_history::ClipboardHistory;
    ///
    /// let app = ClipboardHistory::new();
    /// ```
    pub fn new() -> Self {
        Self {
            store: MemoryStore::new(),
            clipboard: ArboardBackend::new(),
            search_query: String::default(),
            filter_mode: FilterMode::default(),
            selected_index: None,
            auto_capture: true,
            dialogs: DialogManager::new(),
            pending_actions: Vec::new(),
        }
    }

    /// Add a clipboard item from text content.
    fn add_clipboard_item(&mut self, content: String) {
        let item = core::ClipboardItem::from_text(content);
        let _ = self.store.add(item);
    }

    /// Add a clipboard item from image data.
    fn add_image_item(&mut self, width: u32, height: u32) {
        let item = core::ClipboardItem::from_image(width, height);
        let _ = self.store.add(item);
    }

    /// Paste current clipboard content.
    fn paste_from_clipboard(&mut self) {
        // Try text first
        if let Some(text) = self.clipboard.get_text() {
            if self.clipboard.is_new_content(&text) {
                self.clipboard.mark_content(text.clone());
                self.add_clipboard_item(text);
                return;
            }
        }

        // Try image
        if let Some((width, height)) = self.clipboard.get_image() {
            if self.clipboard.is_new_image(width, height) {
                self.clipboard.mark_content(format!("IMG_{}x{}", width, height));
                self.add_image_item(width, height);
            }
        }
    }

    /// Auto-capture from clipboard (if enabled).
    fn auto_capture_clipboard(&mut self) {
        if !self.auto_capture {
            return;
        }

        // Try text first
        if let Some(text) = self.clipboard.get_text() {
            if self.clipboard.is_new_content(&text) {
                self.clipboard.mark_content(text.clone());
                self.add_clipboard_item(text);
                return;
            }
        }

        // Try image
        if let Some((width, height)) = self.clipboard.get_image() {
            if self.clipboard.is_new_image(width, height) {
                self.clipboard.mark_content(format!("IMG_{}x{}", width, height));
                self.add_image_item(width, height);
            }
        }
    }

    /// Execute pending card actions.
    fn execute_actions(&mut self) {
        for action in self.pending_actions.drain(..) {
            match action {
                CardAction::Copy(content) => {
                    self.clipboard.set_text_direct(&content);
                }
                CardAction::Edit(index) => {
                    if let Some(item) = self.store.get(index) {
                        self.dialogs.show_edit_dialog(index, item.content);
                    }
                }
                CardAction::Delete(index) => {
                    let _ = self.store.remove(index);
                }
            }
        }
    }

    /// Get filtered and searched items.
    fn filtered_items(&self) -> Vec<(usize, core::ClipboardItem)> {
        let all_items = self.store.get_all();
        let allowed_types = self.filter_mode.to_content_types();

        all_items
            .into_iter()
            .enumerate()
            .filter(|(_, item)| {
                // Type filter
                let type_match = allowed_types.iter().any(|t| item.matches_type(t));
                if !type_match {
                    return false;
                }

                // Search filter
                item.matches_query(&self.search_query)
            })
            .collect()
    }
}

impl Default for ClipboardHistory {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Demo trait implementation
// -----------------------------------------------------------------------------

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

// -----------------------------------------------------------------------------
// View trait implementation
// ----------------------------------------------------------------------------

impl crate::View for ClipboardHistory {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Header
        ui.heading("📋 Clipboard History");
        ui.label("Real-time clipboard monitoring - copy anything to see it here!");
        ui.separator();

        // Action bar
        self.render_action_bar(ui);

        // Auto-capture
        self.auto_capture_clipboard();

        // Dialogs
        let actions = self.dialogs.render(ui.ctx());
        for action in actions {
            match action {
                ui::DialogAction::Add(content) => {
                    self.add_clipboard_item(content);
                }
                ui::DialogAction::Edit(index, content) => {
                    let item = core::ClipboardItem::from_text(content);
                    let _ = self.store.update(index, item);
                }
            }
        }

        ui.separator();

        // Search and filter bar
        self.render_search_bar(ui);

        // Item list
        self.render_item_list(ui);

        ui.separator();

        // Status bar
        self.render_status_bar(ui);
    }
}

// Private UI rendering methods
impl ClipboardHistory {
    fn render_action_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Auto-capture:");
            ui.checkbox(&mut self.auto_capture, "Enabled");

            ui.separator();

            if ui.button("➕ Add Content").clicked() {
                if let Some(content) = self.clipboard.get_text() {
                    self.dialogs.show_add_dialog(Some(content));
                } else {
                    self.dialogs.show_add_dialog(None);
                }
            }

            if ui.button("📋 Paste Current").clicked() {
                self.paste_from_clipboard();
            }
        });
    }

    fn render_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Search content..."),
            );

            ui.separator();

            ui.label("Filter:");
            ui.radio_value(&mut self.filter_mode, FilterMode::All, "All");
            ui.radio_value(&mut self.filter_mode, FilterMode::Text, "Text");
            ui.radio_value(&mut self.filter_mode, FilterMode::Url, "URLs");
            ui.radio_value(&mut self.filter_mode, FilterMode::Image, "Images");
        });
    }

    fn render_item_list(&mut self, ui: &mut egui::Ui) {
        let filtered = self.filtered_items();
        let card_renderer = ItemCardRenderer::new();

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .max_height(400.0)
            .show(ui, |ui| {
                if filtered.is_empty() {
                    ui.centered_and_justified(|ui| {
                        ui.label(egui::RichText::new("No items to display").weak());
                    });
                } else {
                    for (idx, item) in filtered {
                        let is_selected = self.selected_index == Some(idx);
                        let fill = if is_selected {
                            ui.visuals().faint_bg_color
                        } else {
                            egui::Color32::TRANSPARENT
                        };

                        egui::Frame::NONE
                            .fill(fill)
                            .show(ui, |ui| {
                                card_renderer.render(ui, &item, idx, &mut self.pending_actions);
                            });
                    }
                }

                // Execute pending actions
                self.execute_actions();
            });
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let filtered_count = self.filtered_items().len();

        ui.horizontal(|ui| {
            ui.label(format!("Showing {} items", filtered_count));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear All").clicked() {
                    let _ = self.store.clear();
                }
                if ui.button("📋 Copy Latest").clicked() {
                    if let Some(item) = self.store.get(0) {
                        self.clipboard.set_text_direct(&item.content);
                    }
                }
            });
        });
    }
}
