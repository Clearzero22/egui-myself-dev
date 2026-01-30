//! Dialog components for clipboard history.
//!
//! This module provides dialog components for adding and editing clipboard items.

/// Manager for all clipboard history dialogs.
///
/// This component manages the state and rendering of various dialogs
/// in the clipboard history UI.
#[derive(Default, Debug)]
pub struct DialogManager {
    /// Show the "add content" dialog
    pub show_add: bool,
    /// Show the "edit content" dialog
    pub show_edit: bool,
    /// Content buffer for add dialog
    pub add_content: String,
    /// Content buffer for edit dialog
    pub edit_content: String,
    /// Index of item being edited
    pub edit_index: Option<usize>,
}

impl DialogManager {
    /// Create a new [`DialogManager`].
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let dialogs = DialogManager::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Show the add dialog with optional initial content.
    ///
    /// # Arguments
    ///
    /// * `initial_content` - Optional initial content for the dialog
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let mut dialogs = DialogManager::new();
    /// dialogs.show_add_dialog(Some("Initial text".to_string()));
    /// assert!(dialogs.show_add);
    /// ```
    pub fn show_add_dialog(&mut self, initial_content: Option<String>) {
        self.show_add = true;
        self.add_content = initial_content.unwrap_or_default();
    }

    /// Show the edit dialog for an item.
    ///
    /// # Arguments
    ///
    /// * `index` - Index of the item to edit
    /// * `content` - Current content of the item
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let mut dialogs = DialogManager::new();
    /// dialogs.show_edit_dialog(5, "Current content".to_string());
    /// assert!(dialogs.show_edit);
    /// assert_eq!(dialogs.edit_index, Some(5));
    /// ```
    pub fn show_edit_dialog(&mut self, index: usize, content: String) {
        self.show_edit = true;
        self.edit_index = Some(index);
        self.edit_content = content;
    }

    /// Hide the add dialog and clear its content.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let mut dialogs = DialogManager::new();
    /// dialogs.show_add_dialog(Some("test".to_string()));
    /// dialogs.close_add_dialog();
    /// assert!(!dialogs.show_add);
    /// assert!(dialogs.add_content.is_empty());
    /// ```
    pub fn close_add_dialog(&mut self) {
        self.show_add = false;
        self.add_content.clear();
    }

    /// Hide the edit dialog and clear its state.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let mut dialogs = DialogManager::new();
    /// dialogs.show_edit_dialog(5, "content".to_string());
    /// dialogs.close_edit_dialog();
    /// assert!(!dialogs.show_edit);
    /// assert_eq!(dialogs.edit_index, None);
    /// ```
    pub fn close_edit_dialog(&mut self) {
        self.show_edit = false;
        self.edit_index = None;
        self.edit_content.clear();
    }

    /// Check if any dialog is currently visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use clipboard_history::ui::dialog::DialogManager;
    ///
    /// let dialogs = DialogManager::new();
    /// assert!(!dialogs.has_visible_dialog());
    /// ```
    pub fn has_visible_dialog(&self) -> bool {
        self.show_add || self.show_edit
    }

    /// Render all dialogs and return any actions.
    ///
    /// This method should be called from the main UI rendering loop.
    /// It will show the appropriate dialog(s) based on current state.
    ///
    /// # Returns
    ///
    /// A vector of [`DialogAction`] representing any actions triggered by the dialogs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use egui::Context;
    /// # use clipboard_history::ui::dialog::{DialogManager, DialogAction};
    ///
    /// fn render_ui(ctx: &Context, dialogs: &mut DialogManager) {
    ///     let actions = dialogs.render(ctx);
    ///     for action in actions {
    ///         match action {
    ///             DialogAction::Add(content) => {
    ///                 println!("Add: {}", content);
    ///             }
    ///             DialogAction::Edit(index, content) => {
    ///                 println!("Edit {}: {}", index, content);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub fn render(&mut self, ctx: &egui::Context) -> Vec<DialogAction> {
        let mut actions = Vec::new();

        if self.show_add {
            self.render_add_dialog(ctx, &mut actions);
        }

        if self.show_edit {
            self.render_edit_dialog(ctx, &mut actions);
        }

        actions
    }

    /// Render the "add content" dialog.
    fn render_add_dialog(&mut self, ctx: &egui::Context, actions: &mut Vec<DialogAction>) {
        egui::Window::new("Add New Content")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Content:");
                ui.add_sized(
                    [400.0, 100.0],
                    egui::TextEdit::multiline(&mut self.add_content)
                        .hint_text("Enter content to save..."),
                );

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.close_add_dialog();
                    }
                    if ui.button("Add").clicked() {
                        if !self.add_content.trim().is_empty() {
                            actions.push(DialogAction::Add(self.add_content.clone()));
                            self.close_add_dialog();
                        }
                    }
                });
            });
    }

    /// Render the "edit content" dialog.
    fn render_edit_dialog(&mut self, ctx: &egui::Context, actions: &mut Vec<DialogAction>) {
        let edit_idx = self.edit_index;

        egui::Window::new("Edit Content")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Edit Content:");
                ui.add_sized(
                    [400.0, 100.0],
                    egui::TextEdit::multiline(&mut self.edit_content),
                );

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.close_edit_dialog();
                    }
                    if ui.button("Save").clicked() {
                        if !self.edit_content.trim().is_empty() {
                            if let Some(idx) = edit_idx {
                                actions.push(DialogAction::Edit(idx, self.edit_content.clone()));
                            }
                            self.close_edit_dialog();
                        } else {
                            self.close_edit_dialog();
                        }
                    }
                });
            });
    }
}

/// Action that can be triggered by a dialog.
///
/// These actions are returned by [`DialogManager::render`] and should be
/// handled by the main application.
///
/// # Examples
///
/// ```
/// use clipboard_history::ui::dialog::DialogAction;
///
/// match action {
///     DialogAction::Add(content) => {
///         // Handle adding new content
///     }
///     DialogAction::Edit(index, content) => {
///         // Handle editing content at index
///     }
/// }
/// ```
#[derive(Clone, Debug)]
pub enum DialogAction {
    /// Add new content (from "add content" dialog)
    Add(String),
    /// Edit content at index (from "edit content" dialog)
    Edit(usize, String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialog_manager_creation() {
        let dialogs = DialogManager::new();
        assert!(!dialogs.show_add);
        assert!(!dialogs.show_edit);
        assert!(dialogs.add_content.is_empty());
        assert!(dialogs.edit_content.is_empty());
        assert_eq!(dialogs.edit_index, None);
    }

    #[test]
    fn test_show_add_dialog() {
        let mut dialogs = DialogManager::new();
        dialogs.show_add_dialog(Some("test content".to_string()));

        assert!(dialogs.show_add);
        assert_eq!(dialogs.add_content, "test content");
    }

    #[test]
    fn test_show_edit_dialog() {
        let mut dialogs = DialogManager::new();
        dialogs.show_edit_dialog(10, "edit content".to_string());

        assert!(dialogs.show_edit);
        assert_eq!(dialogs.edit_index, Some(10));
        assert_eq!(dialogs.edit_content, "edit content");
    }

    #[test]
    fn test_close_add_dialog() {
        let mut dialogs = DialogManager::new();
        dialogs.show_add_dialog(Some("test".to_string()));
        dialogs.close_add_dialog();

        assert!(!dialogs.show_add);
        assert!(dialogs.add_content.is_empty());
    }

    #[test]
    fn test_close_edit_dialog() {
        let mut dialogs = DialogManager::new();
        dialogs.show_edit_dialog(5, "content".to_string());
        dialogs.close_edit_dialog();

        assert!(!dialogs.show_edit);
        assert_eq!(dialogs.edit_index, None);
        assert!(dialogs.edit_content.is_empty());
    }

    #[test]
    fn test_has_visible_dialog() {
        let mut dialogs = DialogManager::new();
        assert!(!dialogs.has_visible_dialog());

        dialogs.show_add = true;
        assert!(dialogs.has_visible_dialog());

        dialogs.show_add = false;
        assert!(!dialogs.has_visible_dialog());

        dialogs.show_edit = true;
        assert!(dialogs.has_visible_dialog());
    }

    #[test]
    fn test_dialog_action_types() {
        let add_action = DialogAction::Add("test content".to_string());
        let edit_action = DialogAction::Edit(5, "edited content".to_string());

        match add_action {
            DialogAction::Add(content) => assert_eq!(content, "test content"),
            _ => panic!("Expected Add action"),
        }

        match edit_action {
            DialogAction::Edit(index, content) => {
                assert_eq!(index, 5);
                assert_eq!(content, "edited content");
            }
            _ => panic!("Expected Edit action"),
        }
    }
}
