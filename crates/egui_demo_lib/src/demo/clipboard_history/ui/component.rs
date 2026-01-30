//! Component system for building composable UI.
//!
//! This module provides a component-based architecture for building reusable,
//! composable UI components in egui. Inspired by React/Vue component patterns.
//!
//! # Component Pattern
//!
//! Each component has:
//! - **Props**: Input properties passed from parent
//! - **State**: Internal mutable state
//! - **Events**: Output events sent to parent
//!
//! # Extension Points
//!
//! - Implement [`Component`] trait for custom components
//! - Implement [`Theme`] for custom styling
//! - Use [`ComponentRegistry`] for component discovery

use std::any::Any;
use std::collections::HashMap;

// -----------------------------------------------------------------------------
// Component Trait
// -----------------------------------------------------------------------------

/// Base trait for all UI components.
///
/// This trait defines the interface for building reusable, composable UI components.
///
/// # Type Parameters
///
/// * `Props` - Input properties passed to the component
/// * `State` - Internal state managed by the component
/// * `Event` - Output events emitted by the component
pub trait Component: 'static {
    /// Input properties passed to the component
    type Props: Props + Clone + 'static;
    /// Internal state managed by the component
    type State: State + 'static;
    /// Output events emitted by the component
    type Event: Event + Clone + 'static;

    /// Render the component.
    ///
    /// This method is called each frame to render the component.
    /// It should return a list of events that occurred during rendering.
    fn render(&self, ui: &mut egui::Ui, props: &Self::Props, state: &mut Self::State) -> Vec<Self::Event>;

    /// Optional: Initialize state from props.
    ///
    /// Called when a component is first created with props.
    fn init_state(&self, _props: &Self::Props) -> Self::State {
        Self::State::default()
    }
}

// -----------------------------------------------------------------------------
// Props Trait
// -----------------------------------------------------------------------------

/// Marker trait for component props.
///
/// Props are the input properties passed to a component.
/// They should be cheap to clone.
pub trait Props: Clone + PartialEq + 'static {}

// Blanket implementation for all types that meet the requirements
impl<T: Clone + PartialEq + 'static> Props for T {}

// -----------------------------------------------------------------------------
// State Trait
// -----------------------------------------------------------------------------

/// Marker trait for component state.
///
/// State is the internal mutable state managed by a component.
pub trait State: Clone + Default + 'static {}

// Blanket implementation for all types that meet the requirements
impl<T: Clone + Default + 'static> State for T {}

// -----------------------------------------------------------------------------
// Event Trait
// -----------------------------------------------------------------------------

/// Marker trait for component events.
///
/// Events are output values emitted by components to notify parent components.
pub trait Event: Clone + 'static {}

// Blanket implementation for all types that meet the requirements
impl<T: Clone + 'static> Event for T {}

// -----------------------------------------------------------------------------
// Component Builder
// -----------------------------------------------------------------------------

/// Builder for creating and managing components.
///
/// This struct handles component lifecycle including:
/// - State initialization
/// - Component rendering
/// - Event collection
pub struct ComponentBuilder<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> ComponentBuilder<C> {
    /// Create a new component builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Render the component with the given props.
    ///
    /// This method initializes state if needed, renders the component,
    /// and returns any emitted events.
    pub fn render(component: &C, ui: &mut egui::Ui, props: &C::Props) -> Vec<C::Event> {
        // Create default state for now
        // In a full implementation, this would manage state across frames
        let mut state = component.init_state(props);
        component.render(ui, props, &mut state)
    }
}

impl<C: Component> Default for ComponentBuilder<C> {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Theme System
// -----------------------------------------------------------------------------

/// Theme configuration for components.
///
/// This trait allows components to be styled consistently.
///
/// # Extension Points
///
/// Implement [`Theme`] for custom styling schemes:
/// - `LightTheme`, `DarkTheme` - Color schemes
/// - `CompactTheme`, `SpaciousTheme` - Layout variations
/// - Custom themes per component type
pub trait Theme: 'static {
    /// Get the primary color for the theme.
    fn primary_color(&self) -> egui::Color32;

    /// Get the background color.
    fn background_color(&self) -> egui::Color32;

    /// Get the text color.
    fn text_color(&self) -> egui::Color32;

    /// Get the border radius for rounded corners.
    fn border_radius(&self) -> f32;

    /// Get the spacing between elements.
    fn spacing(&self) -> egui::Vec2;

    /// Get the font size for text.
    fn font_size(&self) -> f32;
}

/// Default light theme.
#[derive(Clone, Debug)]
pub struct LightTheme;

impl Theme for LightTheme {
    fn primary_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(0, 120, 215)
    }

    fn background_color(&self) -> egui::Color32 {
        egui::Color32::WHITE
    }

    fn text_color(&self) -> egui::Color32 {
        egui::Color32::BLACK
    }

    fn border_radius(&self) -> f32 {
        4.0
    }

    fn spacing(&self) -> egui::Vec2 {
        egui::Vec2::new(8.0, 4.0)
    }

    fn font_size(&self) -> f32 {
        14.0
    }
}

/// Default dark theme.
#[derive(Clone, Debug)]
pub struct DarkTheme;

impl Theme for DarkTheme {
    fn primary_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(70, 130, 180)
    }

    fn background_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(32, 32, 32)
    }

    fn text_color(&self) -> egui::Color32 {
        egui::Color32::from_gray(220)
    }

    fn border_radius(&self) -> f32 {
        4.0
    }

    fn spacing(&self) -> egui::Vec2 {
        egui::Vec2::new(8.0, 4.0)
    }

    fn font_size(&self) -> f32 {
        14.0
    }
}

// -----------------------------------------------------------------------------
// Component Registry
// -----------------------------------------------------------------------------

/// Registry for component discovery and instantiation.
///
/// This allows components to be registered and retrieved by name,
/// enabling dynamic composition and plugin systems.
pub struct ComponentRegistry {
    components: HashMap<String, Box<dyn Any>>,
}

impl ComponentRegistry {
    /// Create a new component registry.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    /// Register a component.
    pub fn register<C: Component + 'static>(&mut self, name: String, component: C) {
        self.components.insert(name, Box::new(component));
    }

    /// Get a component by name.
    pub fn get<C: Component + 'static>(&self, name: &str) -> Option<&C> {
        self.components
            .get(name)
            .and_then(|c| c.downcast_ref::<C>())
    }

    /// Check if a component is registered.
    pub fn has(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }

    /// List all registered component names.
    pub fn list(&self) -> Vec<String> {
        self.components.keys().cloned().collect()
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Utility Types
// -----------------------------------------------------------------------------

/// Empty props for components that don't need input.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct EmptyProps;

/// Empty state for stateless components.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct EmptyState;

/// Empty event for components that don't emit events.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub struct EmptyEvent;

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    struct TestComponent;

    #[derive(Clone, PartialEq, Default)]
    struct TestProps {
        value: String,
    }

    #[derive(Clone, Default)]
    struct TestState {
        counter: u32,
    }

    #[derive(Clone, Debug)]
    enum TestEvent {
        Clicked,
    }

    impl Component for TestComponent {
        type Props = TestProps;
        type State = TestState;
        type Event = TestEvent;

        fn render(&self, _ui: &mut egui::Ui, _props: &Self::Props, _state: &mut Self::State) -> Vec<Self::Event> {
            Vec::new()
        }

        fn init_state(&self, _props: &Self::Props) -> Self::State {
            TestState { counter: 0 }
        }
    }

    #[test]
    fn test_component_builder() {
        let component = TestComponent;
        let _builder = ComponentBuilder::<TestComponent>::new();

        // Would need actual UI context for full test
        assert_eq!(component.init_state(&TestProps::default()).counter, 0);
    }

    #[test]
    fn test_theme() {
        let light = LightTheme;
        let dark = DarkTheme;

        // Different colors
        assert_ne!(light.primary_color(), dark.primary_color());
        assert_ne!(light.background_color(), dark.background_color());
    }

    #[test]
    fn test_registry() {
        let mut registry = ComponentRegistry::new();

        registry.register("test".to_string(), TestComponent);

        assert!(registry.has("test"));
        assert!(!registry.has("nonexistent"));

        let names = registry.list();
        assert_eq!(names, vec!["test".to_string()]);
    }
}
