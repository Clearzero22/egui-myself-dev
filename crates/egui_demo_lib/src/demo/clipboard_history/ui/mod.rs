//! User interface components for clipboard history.
//!
//! This module provides a component-based UI system with reusable,
//! composable components following a Props/State/Event pattern.
//!
//! # Architecture
//!
//! ```text
//! ui/
//! ├── component.rs     # Component system base traits
//! ├── components.rs    # Reusable UI component library
//! ├── card.rs          # Item card renderer (legacy)
//! ├── dialog.rs        # Dialog manager (legacy)
//! └── mod.rs
//! ```
//!
//! # Extension Points
//!
//! - Implement [`component::Component`] for custom components
//! - Use [`components`] for pre-built UI widgets
//! - Extend [`card::ItemCardRenderer`] for custom card styles

pub mod component;
pub mod components;
pub mod card;
pub mod dialog;

// Re-export component system (types used by external code)
pub use component::{Component, Props, State, Event, Theme, LightTheme, DarkTheme};

// Re-export UI component library (when actually used)
// pub use components::{
//     ButtonComponent, ButtonProps, ButtonEvent, ButtonStyle, ButtonVariant,
//     InputComponent, InputProps, InputEvent, InputType,
//     CardComponent, CardProps, CardTheme,
//     IconComponent, IconProps, Icon,
//     BadgeComponent, BadgeProps, BadgeVariant,
//     DividerComponent, DividerProps, DividerOrientation,
// };

// Re-export icon types
// pub use components::icon::NamedIcon;

// Re-export legacy components
pub use card::{ItemCardRenderer, CardAction};
pub use dialog::{DialogManager, DialogAction};
