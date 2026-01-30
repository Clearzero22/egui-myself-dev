//! User interface components for clipboard history.
//!
//! This module provides UI components for rendering the clipboard history interface.

pub mod card;
pub mod dialog;

// Re-export commonly used types
pub use card::{ItemCardRenderer, CardAction};
pub use dialog::{DialogManager, DialogAction};
