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

use core::{Store, ContentType, MemoryStore};
use clipboard::{create_backend, Backend};
use ui::{DialogManager, ItemCardRenderer, CardAction};
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
use clipboard::ArboardBackend;

#[cfg(target_arch = "wasm32")]
use clipboard::WebBackend;

#[cfg(feature = "persistence")]
use core::SqliteStore;

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
    #[cfg(not(feature = "persistence"))]
    store: MemoryStore,
    /// Storage for clipboard items (persistent)
    #[cfg(feature = "persistence")]
    store: SqliteStore,
    /// Clipboard backend (desktop: ArboardBackend, WASM: WebBackend)
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: ArboardBackend,
    #[cfg(target_arch = "wasm32")]
    clipboard: WebBackend,
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
    /// Texture cache for images (texture_id -> TextureHandle)
    texture_cache: HashMap<String, egui::TextureHandle>,
    /// Pagination: current page number (0-based)
    current_page: usize,
    /// Pagination: items per page
    page_size: usize,
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
    #[cfg(not(feature = "persistence"))]
    pub fn new() -> Self {
        Self {
            store: MemoryStore::new(),
            clipboard: create_backend(),
            search_query: String::default(),
            filter_mode: FilterMode::default(),
            selected_index: None,
            auto_capture: true,
            dialogs: DialogManager::new(),
            pending_actions: Vec::new(),
            texture_cache: HashMap::new(),
            current_page: 0,
            page_size: 20,
        }
    }

    /// Create a new [`ClipboardHistory`] with persistent storage.
    ///
    /// # Examples
    ///
    /// ```
    /// use egui_demo_lib::demo::clipboard_history::ClipboardHistory;
    ///
    /// let app = ClipboardHistory::new();
    /// ```
    #[cfg(feature = "persistence")]
    pub fn new() -> Self {
        let store = SqliteStore::new().unwrap_or_else(|e| {
            eprintln!("Failed to initialize persistent storage, using memory: {}", e);
            // Fallback to in-memory if SQLite fails
            panic!("Failed to initialize SqliteStore: {}", e);
        });

        Self {
            store,
            clipboard: create_backend(),
            search_query: String::default(),
            filter_mode: FilterMode::default(),
            selected_index: None,
            auto_capture: true,
            dialogs: DialogManager::new(),
            pending_actions: Vec::new(),
            texture_cache: HashMap::new(),
            current_page: 0,
            page_size: 20,
        }
    }

    /// Add a clipboard item from text content.
    fn add_clipboard_item(&mut self, content: String) {
        let item = core::ClipboardItem::from_text(content);
        let _ = self.store.add(item);
        // Reset to page 0 to show new items
        self.current_page = 0;
    }

    /// Add a clipboard item from image data.
    fn add_image_item(&mut self, width: u32, height: u32, bytes: Vec<u8>) {
        println!("[DEBUG] Adding image: {}x{}, {} bytes", width, height, bytes.len());
        let item = core::ClipboardItem::from_image(width, height, bytes);
        println!("[DEBUG] Item image_data.is_some(): {}", item.image_data.is_some());
        let _ = self.store.add(item);
        // Reset to page 0 to show new items
        self.current_page = 0;
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
        if let Some((width, height, bytes)) = self.clipboard.get_image() {
            if self.clipboard.is_new_image(width, height) {
                self.clipboard.mark_content(format!("IMG_{}x{}", width, height));
                self.add_image_item(width, height, bytes);
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
        if let Some((width, height, bytes)) = self.clipboard.get_image() {
            if self.clipboard.is_new_image(width, height) {
                self.clipboard.mark_content(format!("IMG_{}x{}", width, height));
                self.add_image_item(width, height, bytes);
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

    /// Get the total count of filtered items (for pagination).
    fn filtered_count(&self) -> usize {
        self.filtered_items_internal().len()
    }

    /// Get filtered and searched items for the current page.
    fn filtered_items(&self) -> Vec<(usize, core::ClipboardItem)> {
        let filtered = self.filtered_items_internal();

        // Apply pagination
        let offset = self.current_page * self.page_size;
        filtered
            .into_iter()
            .skip(offset)
            .take(self.page_size)
            .collect()
    }

    /// Internal helper to get all filtered items (without pagination).
    fn filtered_items_internal(&self) -> Vec<(usize, core::ClipboardItem)> {
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

    /// Calculate the total number of pages for filtered results.
    fn total_pages(&self) -> usize {
        let count = self.filtered_count();
        if count == 0 {
            0
        } else {
            (count - 1) / self.page_size + 1
        }
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

        // Pagination controls (only show if there are items)
        if self.store.len() > 0 {
            self.render_pagination_controls(ui);
        }

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
        let old_filter = self.filter_mode;
        let old_query = self.search_query.clone();

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

        // Reset to page 0 when filter or search changes
        if old_filter != self.filter_mode || old_query != self.search_query {
            self.current_page = 0;
        }
    }

    fn render_pagination_controls(&mut self, ui: &mut egui::Ui) {
        let total_pages = self.total_pages();
        let filtered_count = self.filtered_count();

        // Don't show pagination if everything fits on one page
        if total_pages <= 1 {
            return;
        }

        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;

            // Page info
            let page_info = format!("Page {} / {} ({} items)", self.current_page + 1, total_pages, filtered_count);
            ui.label(egui::RichText::new(page_info).small());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Next button
                let has_next = self.current_page + 1 < total_pages;
                if ui.add_enabled(has_next, egui::Button::new("▶ Next").small()).clicked() {
                    if has_next {
                        self.current_page += 1;
                    }
                }

                // Previous button
                let has_prev = self.current_page > 0;
                if ui.add_enabled(has_prev, egui::Button::new("◀ Prev").small()).clicked() {
                    if has_prev {
                        self.current_page = self.current_page.saturating_sub(1);
                    }
                }

                // Page size selector
                ui.separator();
                ui.label(egui::RichText::new("Per page:").small());
                let old_page_size = self.page_size;
                egui::ComboBox::from_id_salt("page_size")
                    .selected_text(format!("{}", self.page_size))
                    .width(60.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.page_size, 10, "10");
                        ui.selectable_value(&mut self.page_size, 20, "20");
                        ui.selectable_value(&mut self.page_size, 50, "50");
                        ui.selectable_value(&mut self.page_size, 100, "100");
                    });
                // Adjust current page if page size changed and we're now out of bounds
                if old_page_size != self.page_size {
                    let max_page = self.total_pages().saturating_sub(1);
                    if self.current_page > max_page {
                        self.current_page = max_page;
                    }
                }
            });
        });

        ui.add_space(4.0);
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
                                // PERF: Create a load function for lazy image loading
                                // Currently a placeholder due to borrow checker limitations
                                let load_fn = move |_: &str| -> Option<Vec<u8>> {
                                    // TODO: Implement proper lazy loading
                                    // The texture_cache already handles caching after first load
                                    None
                                };

                                card_renderer.render(
                                    ui,
                                    &item,
                                    idx,
                                    &mut self.pending_actions,
                                    &mut self.texture_cache,
                                    Some(&load_fn),
                                );
                            });
                    }
                }

                // Execute pending actions
                self.execute_actions();
            });
    }

    fn render_status_bar(&mut self, ui: &mut egui::Ui) {
        let page_count = self.filtered_items().len();
        let total_filtered = self.filtered_count();
        let total_pages = self.total_pages();

        ui.horizontal(|ui| {
            let status = if total_pages > 1 {
                format!("Page {}/{} · {} items ({} filtered)", self.current_page + 1, total_pages, page_count, total_filtered)
            } else {
                format!("Showing {} items", total_filtered)
            };
            ui.label(egui::RichText::new(status).small().weak());

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🗑️ Clear All").clicked() {
                    let _ = self.store.clear();
                    self.current_page = 0;
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
