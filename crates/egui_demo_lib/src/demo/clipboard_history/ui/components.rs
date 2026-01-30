//! Reusable UI component library.
//!
//! This module provides a collection of pre-built, composable UI components
//! that can be used throughout the application.
//!
//! # Available Components
//!
//! - [`ButtonComponent`] - Configurable button with styles
//! - [`InputComponent`] - Text input with validation
//! - [`CardComponent`] - Container card with frame
//! - [`IconComponent`] - Icon/text label
//! - [`BadgeComponent`] - Small status badge
//! - [`DividerComponent`] - Visual separator
//!
//! # Extension Points
//!
//! - Custom button styles via [`ButtonStyle`]
//! - Custom input validation via [`Validator`]
//! - Custom card themes via [`CardTheme`]
//! - Custom icons via [`Icon`]

pub use button::{ButtonComponent, ButtonProps, ButtonEvent, ButtonStyle, ButtonVariant};
pub use input::{InputComponent, InputProps, InputEvent, InputType, Validator};
pub use card::{CardComponent, CardProps, CardTheme};
pub use icon::{IconComponent, IconProps, Icon};
pub use badge::{BadgeComponent, BadgeProps, BadgeVariant};
pub use divider::{DividerComponent, DividerProps, DividerOrientation};

// -----------------------------------------------------------------------------
// Button Component
// -----------------------------------------------------------------------------

mod button {
    use super::super::component::{Component, Props, State, Event};

    /// Button component with configurable styles and variants.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use egui::Ui;
    /// use clipboard_history::ui::components::{ButtonComponent, ButtonProps, ButtonVariant, ButtonStyle};
    ///
    /// fn render_button(ui: &mut Ui) {
    ///     let button = ButtonComponent::new();
    ///     let props = ButtonProps {
    ///         label: "Click Me".to_string(),
    ///         variant: ButtonVariant::Primary,
    ///         enabled: true,
    ///         ..Default::default()
    ///     };
    ///     // button.render(ui, &props, &mut Default::default());
    /// }
    /// ```
    #[derive(Clone, Copy, Debug)]
    pub struct ButtonComponent;

    impl ButtonComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for ButtonComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for button component.
    #[derive(Clone, PartialEq, Default)]
    pub struct ButtonProps {
        /// Text label on the button
        pub label: String,
        /// Button variant (style)
        pub variant: ButtonVariant,
        /// Whether the button is enabled
        pub enabled: bool,
        /// Optional icon (emoji)
        pub icon: Option<String>,
        /// Custom width (None = auto)
        pub width: Option<f32>,
        /// Small size button
        pub small: bool,
    }

    /// Events emitted by button.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum ButtonEvent {
        /// Button was clicked
        Clicked,
        /// Button was hovered
        Hovered,
    }

    /// Button style variants.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum ButtonVariant {
        /// Primary action button (blue)
        #[default]
        Primary,
        /// Secondary button (gray)
        Secondary,
        /// Danger button (red)
        Danger,
        /// Success button (green)
        Success,
        /// Ghost button (transparent with hover)
        Ghost,
    }

    impl Component for ButtonComponent {
        type Props = ButtonProps;
        type State = ();
        type Event = ButtonEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let mut events = Vec::new();

            let button = match props.variant {
                ButtonVariant::Primary => egui::Button::new(&props.label).fill(egui::Color32::from_rgb(0, 120, 215)),
                ButtonVariant::Secondary => egui::Button::new(&props.label).fill(egui::Color32::from_gray(150)),
                ButtonVariant::Danger => egui::Button::new(&props.label).fill(egui::Color32::from_rgb(220, 50, 50)),
                ButtonVariant::Success => egui::Button::new(&props.label).fill(egui::Color32::from_rgb(50, 180, 50)),
                ButtonVariant::Ghost => egui::Button::new(&props.label),
            };

            let button = if props.small {
                button.small()
            } else {
                button
            };

            let label_with_icon = if let Some(ref icon) = props.icon {
                format!("{} {}", icon, props.label)
            } else {
                props.label.clone()
            };

            let button = egui::Button::new(&label_with_icon);

            let response = ui.add_enabled(props.enabled, button);

            if response.clicked() {
                events.push(ButtonEvent::Clicked);
            }
            if response.hovered() {
                events.push(ButtonEvent::Hovered);
            }

            events
        }
    }

    /// Button style configuration.
    #[derive(Clone, Debug)]
    pub struct ButtonStyle {
        pub background_color: egui::Color32,
        pub hover_color: egui::Color32,
        pub text_color: egui::Color32,
        pub border_radius: f32,
        pub padding: egui::Vec2,
    }

    impl ButtonStyle {
        pub fn primary() -> Self {
            Self {
                background_color: egui::Color32::from_rgb(0, 120, 215),
                hover_color: egui::Color32::from_rgb(0, 100, 180),
                text_color: egui::Color32::WHITE,
                border_radius: 4.0,
                padding: egui::Vec2::new(16.0, 8.0),
            }
        }

        pub fn danger() -> Self {
            Self {
                background_color: egui::Color32::from_rgb(220, 50, 50),
                hover_color: egui::Color32::from_rgb(200, 40, 40),
                text_color: egui::Color32::WHITE,
                border_radius: 4.0,
                padding: egui::Vec2::new(16.0, 8.0),
            }
        }

        pub fn ghost() -> Self {
            Self {
                background_color: egui::Color32::TRANSPARENT,
                hover_color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20),
                text_color: egui::Color32::GRAY,
                border_radius: 4.0,
                padding: egui::Vec2::new(12.0, 6.0),
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Input Component
// -----------------------------------------------------------------------------

mod input {
    use super::super::component::{Component, Props, State, Event};

    /// Text input component with validation.
    #[derive(Clone, Copy, Debug)]
    pub struct InputComponent;

    impl InputComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for InputComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for input component.
    #[derive(Clone, PartialEq, Default)]
    pub struct InputProps {
        /// Current text value
        pub value: String,
        /// Placeholder text
        pub placeholder: Option<String>,
        /// Input type
        pub input_type: InputType,
        /// Whether input is enabled
        pub enabled: bool,
        /// Optional validator
        pub validator: Option<Validator>,
        /// Width (None = auto)
        pub width: Option<f32>,
        /// Previous value (for change detection)
        #[doc(hidden)]
        pub _previous_value: String,
    }

    /// Events emitted by input.
    #[derive(Clone, Debug, PartialEq)]
    pub enum InputEvent {
        /// Text changed
        Changed(String),
        /// Enter key pressed
        EnterPressed(String),
        /// Focus lost
        FocusLost(String),
        /// Validation failed
        ValidationError(String),
    }

    /// Input type variants.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum InputType {
        /// Single-line text
        #[default]
        Text,
        /// Multi-line text
        TextArea,
        /// Password input
        Password,
        /// Email input
        Email,
        /// Number input
        Number,
    }

    /// Validator for input values.
    pub type Validator = fn(&str) -> Result<(), String>;

    impl Component for InputComponent {
        type Props = InputProps;
        type State = ();
        type Event = InputEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let mut events = Vec::new();
            let mut temp_value = props.value.clone();
            let previous_value = props._previous_value.clone();

            let response = match props.input_type {
                InputType::Text => {
                    let mut builder = egui::TextEdit::singleline(&mut temp_value);
                    if let Some(ref placeholder) = props.placeholder {
                        builder = builder.hint_text(placeholder);
                    }
                    ui.add_enabled(props.enabled, builder)
                }
                InputType::TextArea => {
                    let mut builder = egui::TextEdit::multiline(&mut temp_value);
                    if let Some(ref placeholder) = props.placeholder {
                        builder = builder.hint_text(placeholder);
                    }
                    if let Some(width) = props.width {
                        builder = builder.desired_width(width);
                    }
                    ui.add_enabled(props.enabled, builder)
                }
                InputType::Password => {
                    let mut builder = egui::TextEdit::singleline(&mut temp_value)
                        .password(true);
                    if let Some(ref placeholder) = props.placeholder {
                        builder = builder.hint_text(placeholder);
                    }
                    ui.add_enabled(props.enabled, builder)
                }
                _ => {
                    ui.add_enabled(props.enabled, egui::TextEdit::singleline(&mut temp_value))
                }
            };

            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                events.push(InputEvent::EnterPressed(temp_value.clone()));
            }

            if response.lost_focus() {
                events.push(InputEvent::FocusLost(temp_value.clone()));
            }

            // Detect changes by comparing with previous value
            // Note: In egui, actual value changes need to be handled by the parent
            // This component emits events but doesn't modify props directly
            if temp_value != previous_value && temp_value != props.value {
                if let Some(validator) = props.validator {
                    if let Err(err) = validator(&temp_value) {
                        events.push(InputEvent::ValidationError(err));
                        return events;
                    }
                }
                events.push(InputEvent::Changed(temp_value));
            }

            events
        }
    }
}

// -----------------------------------------------------------------------------
// Card Component
// -----------------------------------------------------------------------------

mod card {
    use super::super::component::{Component, Props, State, Event};

    /// Container card component with frame and styling.
    #[derive(Clone, Copy, Debug)]
    pub struct CardComponent;

    impl CardComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for CardComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for card component.
    #[derive(Clone, PartialEq, Default)]
    pub struct CardProps {
        /// Card theme/styling
        pub theme: CardTheme,
        /// Corner radius
        pub corner_radius: Option<f32>,
        /// Inner margin
        pub margin: Option<egui::Margin>,
        /// Background fill (None = use theme)
        pub fill: Option<egui::Color32>,
        /// Stroke (None = use theme)
        pub stroke: Option<egui::Stroke>,
        /// Shadow effect
        pub shadow: bool,
    }

    /// No events emitted by card (container only).
    #[derive(Clone, Debug)]
    pub enum CardEvent {}

    /// Card theme presets.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum CardTheme {
        /// Default white card with border
        #[default]
        Default,
        /// Elevated card with shadow
        Elevated,
        /// Flat card without border
        Flat,
        /// Outlined card with prominent border
        Outlined,
        /// Interactive card (hover effect)
        Interactive,
    }

    impl Component for CardComponent {
        type Props = CardProps;
        type State = ();
        type Event = CardEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let fill = props.fill.unwrap_or_else(|| match props.theme {
                CardTheme::Default => egui::Color32::WHITE,
                CardTheme::Elevated => egui::Color32::WHITE,
                CardTheme::Flat => egui::Color32::TRANSPARENT,
                CardTheme::Outlined => egui::Color32::WHITE,
                CardTheme::Interactive => egui::Color32::WHITE,
            });

            let stroke = props.stroke.unwrap_or_else(|| match props.theme {
                CardTheme::Default => egui::Stroke::new(1.0, egui::Color32::from_gray(200)),
                CardTheme::Elevated => egui::Stroke::NONE,
                CardTheme::Flat => egui::Stroke::NONE,
                CardTheme::Outlined => egui::Stroke::new(2.0, egui::Color32::from_gray(150)),
                CardTheme::Interactive => egui::Stroke::new(1.0, egui::Color32::from_rgb(0, 120, 215)),
            });

            let corner_radius = props.corner_radius.unwrap_or_else(|| match props.theme {
                CardTheme::Default | CardTheme::Flat | CardTheme::Outlined => 4.0,
                CardTheme::Elevated => 8.0,
                CardTheme::Interactive => 6.0,
            });

            let margin = props.margin.unwrap_or_else(|| match props.theme {
                CardTheme::Default => egui::Margin::symmetric(12, 8),
                CardTheme::Elevated => egui::Margin::symmetric(16, 12),
                CardTheme::Flat => egui::Margin::ZERO,
                CardTheme::Outlined => egui::Margin::symmetric(8, 6),
                CardTheme::Interactive => egui::Margin::symmetric(12, 8),
            });

            let mut frame = egui::Frame::NONE
                .fill(fill)
                .stroke(stroke)
                .corner_radius(corner_radius)
                .inner_margin(margin);

            if props.shadow {
                frame = frame.shadow(egui::epaint::Shadow {
                    offset: [0, 2],
                    blur: 4,
                    spread: 0,
                    color: egui::Color32::from_rgba_unmultiplied(0, 0, 0, 20),
                });
            }

            frame.show(ui, |ui| {
                // Content will be rendered here by children
            });

            Vec::new()
        }
    }
}

// -----------------------------------------------------------------------------
// Icon Component
// -----------------------------------------------------------------------------

pub mod icon {
    use super::super::component::{Component, Props, State, Event};

    /// Icon component (using emoji or unicode).
    #[derive(Clone, Copy, Debug)]
    pub struct IconComponent;

    impl IconComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for IconComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for icon component.
    #[derive(Clone, PartialEq, Default)]
    pub struct IconProps {
        /// Icon to display
        pub icon: Icon,
        /// Icon size in points
        pub size: f32,
        /// Optional color tint
        pub color: Option<egui::Color32>,
    }

    /// No events emitted by icon (display only).
    #[derive(Clone, Debug)]
    pub enum IconEvent {}

    /// Icon definitions.
    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum Icon {
        /// Custom emoji string
        Emoji(String),
        /// Unicode symbol
        Symbol(String),
        /// Named icon (preset)
        Named(NamedIcon),
    }

    /// Predefined named icons.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum NamedIcon {
        Add,
        Edit,
        Delete,
        Search,
        Filter,
        Settings,
        Copy,
        Paste,
        Cut,
        Undo,
        Redo,
        Save,
        Open,
        Close,
        Folder,
        File,
        Image,
        Text,
        Link,
        Code,
        Checkmark,
        Cross,
        Warning,
        Info,
        Error,
    }

    impl NamedIcon {
        fn as_str(&self) -> &str {
            match self {
                NamedIcon::Add => "➕",
                NamedIcon::Edit => "✏️",
                NamedIcon::Delete => "🗑️",
                NamedIcon::Search => "🔍",
                NamedIcon::Filter => "🔽",
                NamedIcon::Settings => "⚙️",
                NamedIcon::Copy => "📋",
                NamedIcon::Paste => "⏏️",
                NamedIcon::Cut => "✂️",
                NamedIcon::Undo => "↩️",
                NamedIcon::Redo => "↪️",
                NamedIcon::Save => "💾",
                NamedIcon::Open => "📂",
                NamedIcon::Close => "✕",
                NamedIcon::Folder => "📁",
                NamedIcon::File => "📄",
                NamedIcon::Image => "🖼️",
                NamedIcon::Text => "📝",
                NamedIcon::Link => "🔗",
                NamedIcon::Code => "💻",
                NamedIcon::Checkmark => "✓",
                NamedIcon::Cross => "✕",
                NamedIcon::Warning => "⚠️",
                NamedIcon::Info => "ℹ️",
                NamedIcon::Error => "❌",
            }
        }
    }

    impl Component for IconComponent {
        type Props = IconProps;
        type State = ();
        type Event = IconEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let text = match &props.icon {
                Icon::Emoji(s) => s.clone(),
                Icon::Symbol(s) => s.clone(),
                Icon::Named(named) => named.as_str().to_string(),
            };

            let mut label = egui::RichText::new(text).size(props.size);
            if let Some(color) = props.color {
                label = label.color(color);
            }

            ui.label(label);

            Vec::new()
        }
    }

    impl Default for Icon {
        fn default() -> Self {
            Self::Emoji("❓".to_string())
        }
    }
}

// -----------------------------------------------------------------------------
// Badge Component
// -----------------------------------------------------------------------------

mod badge {
    use super::super::component::{Component, Props, State, Event};

    /// Small status badge component.
    #[derive(Clone, Copy, Debug)]
    pub struct BadgeComponent;

    impl BadgeComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for BadgeComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for badge component.
    #[derive(Clone, PartialEq, Default)]
    pub struct BadgeProps {
        /// Text to display
        pub text: String,
        /// Badge variant (determines color)
        pub variant: BadgeVariant,
        /// Small size
        pub small: bool,
    }

    /// No events emitted by badge (display only).
    #[derive(Clone, Debug)]
    pub enum BadgeEvent {}

    /// Badge color variants.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum BadgeVariant {
        /// Gray badge
        #[default]
        Default,
        /// Blue info badge
        Info,
        /// Green success badge
        Success,
        /// Yellow warning badge
        Warning,
        /// Red error/danger badge
        Danger,
    }

    impl Component for BadgeComponent {
        type Props = BadgeProps;
        type State = ();
        type Event = BadgeEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let (bg_color, text_color) = match props.variant {
                BadgeVariant::Default => (egui::Color32::from_gray(200), egui::Color32::BLACK),
                BadgeVariant::Info => (egui::Color32::from_rgb(0, 120, 215), egui::Color32::WHITE),
                BadgeVariant::Success => (egui::Color32::from_rgb(50, 180, 50), egui::Color32::WHITE),
                BadgeVariant::Warning => (egui::Color32::from_rgb(220, 150, 0), egui::Color32::BLACK),
                BadgeVariant::Danger => (egui::Color32::from_rgb(220, 50, 50), egui::Color32::WHITE),
            };

            let font_size = if props.small { 10.0 } else { 12.0 };
            let padding = if props.small { 4 } else { 6 };

            egui::Frame::NONE
                .fill(bg_color)
                .corner_radius(999.0) // Fully rounded
                .inner_margin(egui::Margin::symmetric(padding, 2))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new(&props.text).size(font_size).color(text_color));
                });

            Vec::new()
        }
    }
}

// -----------------------------------------------------------------------------
// Divider Component
// -----------------------------------------------------------------------------

mod divider {
    use super::super::component::{Component, Props, State, Event};

    /// Visual separator/divider component.
    #[derive(Clone, Copy, Debug)]
    pub struct DividerComponent;

    impl DividerComponent {
        pub fn new() -> Self {
            Self
        }
    }

    impl Default for DividerComponent {
        fn default() -> Self {
            Self::new()
        }
    }

    /// Properties for divider component.
    #[derive(Clone, PartialEq, Default)]
    pub struct DividerProps {
        /// Orientation of the divider
        pub orientation: DividerOrientation,
        /// Custom thickness (None = use default)
        pub thickness: Option<f32>,
        /// Custom color (None = use theme default)
        pub color: Option<egui::Color32>,
        /// Spacing around divider
        pub spacing: Option<f32>,
    }

    /// No events emitted by divider (display only).
    #[derive(Clone, Debug)]
    pub enum DividerEvent {}

    /// Divider orientation.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum DividerOrientation {
        /// Horizontal divider (most common)
        #[default]
        Horizontal,
        /// Vertical divider
        Vertical,
    }

    impl Component for DividerComponent {
        type Props = DividerProps;
        type State = ();
        type Event = DividerEvent;

        fn render(&self, ui: &mut egui::Ui, props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            let thickness = props.thickness.unwrap_or(1.0);
            let color = props.color.unwrap_or(egui::Color32::from_gray(200));
            let spacing = props.spacing.unwrap_or(8.0);

            match props.orientation {
                DividerOrientation::Horizontal => {
                    ui.add_space(spacing);
                    let rect = egui::Rect::from_min_max(
                        ui.cursor().min,
                        egui::pos2(ui.cursor().max.x, ui.cursor().min.y + thickness),
                    );
                    ui.painter().rect_filled(rect, 0.0, color);
                    ui.add_space(spacing);
                }
                DividerOrientation::Vertical => {
                    let rect = egui::Rect::from_min_max(
                        ui.cursor().min,
                        egui::pos2(ui.cursor().min.x + thickness, ui.cursor().max.y),
                    );
                    ui.painter().rect_filled(rect, 0.0, color);
                }
            }

            Vec::new()
        }
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_props_default() {
        let props = ButtonProps::default();
        assert_eq!(props.variant, ButtonVariant::Primary);
        assert!(props.enabled);
        assert!(props.icon.is_none());
    }

    #[test]
    fn test_input_types() {
        assert_eq!(InputType::Text, InputType::Text);
        assert_eq!(InputType::TextArea, InputType::TextArea);
    }

    #[test]
    fn test_icon_variants() {
        let emoji = Icon::Emoji("😀".to_string());
        let named = Icon::Named(NamedIcon::Add);
        assert_ne!(emoji, named);
    }

    #[test]
    fn test_badge_variants() {
        let variants = [
            BadgeVariant::Default,
            BadgeVariant::Info,
            BadgeVariant::Success,
            BadgeVariant::Warning,
            BadgeVariant::Danger,
        ];
        // Just verify they're all distinct
        for i in 0..variants.len() {
            for j in (i+1)..variants.len() {
                assert_ne!(variants[i], variants[j]);
            }
        }
    }
}
