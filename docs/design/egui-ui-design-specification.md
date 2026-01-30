# EGUI OFFICIAL UI DESIGN SPECIFICATION

> Version: 1.0.0
> Date: 2025-01-30
> Based on: egui official codebase analysis

## Table of Contents

1. [Typography System](#1-typography-system)
2. [Spacing & Layout](#2-spacing--layout)
3. [Color System](#3-color-system)
4. [Widget Patterns](#4-widget-patterns)
5. [Frame Design](#5-frame-design)
6. [Window Patterns](#6-window-patterns)
7. [Panel Layouts](#7-panel-layouts)
8. [Interactive States](#8-interactive-states)
9. [Common UI Patterns](#9-common-ui-patterns)
10. [Naming Conventions](#10-naming-conventions)
11. [Animation & Motion](#11-animation--motion)
12. [Accessibility](#12-accessibility)
13. [Design Principles](#13-design-principles)
14. [Component Checklist](#14-component-checklist)

---

## 1. Typography System

### Text Style Hierarchy

| Style | Size | Font Family | Usage |
|-------|------|-------------|-------|
| `TextStyle::Small` | 9.0pt | Proportional | Secondary/tertiary information |
| `TextStyle::Body` | 13.0pt | Proportional | Default text |
| `TextStyle::Button` | 13.0pt | Proportional | Interactive elements |
| `TextStyle::Heading` | 18.0pt | Proportional | Section headers |
| `TextStyle::Monospace` | 13.0pt | Monospace | Code/data |

### Default Font Configuration

```rust
pub fn default_text_styles() -> BTreeMap<TextStyle, FontId> {
    use FontFamily::{Monospace, Proportional};
    [
        (TextStyle::Small, FontId::new(9.0, Proportional)),
        (TextStyle::Body, FontId::new(13.0, Proportional)),
        (TextStyle::Button, FontId::new(13.0, Proportional)),
        (TextStyle::Heading, FontId::new(18.0, Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, Monospace)),
    ].into()
}
```

### Typography Usage Patterns

```rust
// Labels
ui.label("Normal text");
ui.label(RichText::new("Styled").size(20.0).color(egui::Color32::RED));

// Headings
ui.heading("Main Header");
ui.label(RichText::new("Subheader").text_style(egui::TextStyle::Heading));

// Code/monospace
ui.code("monospace text");
ui.label(RichText::new("inline code").code().monospace());

// Weak/subtle text
ui.label(RichText::new("Secondary").weak());

// Strong/emphasized text
ui.label(RichText::new("Important").strong());
```

### Typography Guidelines

- **Body text** (13pt) is the default for most content
- **Headings** (18pt) for section titles only
- **Small** (9pt) for metadata, timestamps, secondary info
- **Monospace** (13pt) for code, paths, data values
- Use `weak()` for non-critical information
- Use `strong()` for emphasis only (sparingly)

---

## 2. Spacing & Layout

### Default Spacing Constants

| Property | Value | Description |
|----------|-------|-------------|
| `item_spacing` | `vec2(8.0, 3.0)` | Space between widgets (horizontal, vertical) |
| `window_margin` | `Margin::same(6)` | Window frame margins |
| `menu_margin` | `Margin::same(6)` | Menu margins |
| `button_padding` | `vec2(4.0, 1.0)` | Button internal padding |
| `indent` | `18.0` | Indent for nested regions |
| `interact_size` | `vec2(40.0, 18.0)` | Min clickable widget size |
| `slider_width` | `100.0` | Default slider width |
| `text_edit_width` | `280.0` | TextEdit default width |
| `icon_width` | `14.0` | Checkbox/radio outer icon size |
| `icon_width_inner` | `8.0` | Checkbox inner icon size |
| `icon_spacing` | `4.0` | Space between icon and text |
| `tooltip_width` | `500.0` | Tooltip wrap width |
| `menu_width` | `400.0` | Default menu width |

### Key Spacing Principles

1. **More horizontal than vertical spacing**: `item_spacing` is `vec2(8.0, 3.0)` - wider gaps between columns than rows
2. **Indent matches checkbox width**: `indent = 18.0` matches the checkbox/radio button width
3. **Icon + spacing + text pattern**: `14px (icon) + 4px (spacing) + text_width`
4. **Tighter vertical spacing**: Use `3.0` for related items in a list

### Layout Direction Patterns

```rust
// Horizontal layout (left to right)
ui.horizontal(|ui| {
    ui.label("A");
    ui.button("B");
});

// Vertical layout (top to bottom)
ui.vertical(|ui| {
    ui.label("A");
    ui.button("B");
});

// Centered content
ui.vertical_centered(|ui| {
    ui.button("Centered");
});

// Right-to-left layout
ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
    ui.label("Right to left");
});

// Horizontal wrapped
ui.horizontal_wrapped(|ui| {
    ui.label("Wraps to new row");
    ui.spacing_mut().item_spacing.x = 0.0;  // Tight packing
});
```

### Grid Layout Pattern

```rust
egui::Grid::new("my_grid")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .striped(true)
    .show(ui, |ui| {
        ui.label("Label:");
        ui.text_edit_singleline(&mut value);
        ui.end_row(); // Explicit row ending
    });
```

### Spacing Guidelines

- Use `ui.separator()` between major sections
- Use `ui.add_space(8.0)` for custom spacing needs
- Keep related items closer together (3px vertical)
- Separate unrelated items more (8px+)

---

## 3. Color System

### Dark Theme (Default)

| Purpose | Property | Color Value |
|---------|----------|-------------|
| Window background | `window_fill` | `from_gray(27)` |
| Panel background | `panel_fill` | `from_gray(27)` |
| TextEdit background | `extreme_bg_color` | `from_gray(10)` |
| Code background | `code_bg_color` | `from_gray(64)` |
| Faint overlay | `faint_bg_color` | `from_additive_luminance(5)` |
| Static text | `noninteractive_fg_stroke` | `from_gray(140)` |
| Inactive widget | `inactive_fg_stroke` | `from_gray(180)` |
| Hovered widget | `hovered_fg_stroke` | `from_gray(240)` |
| Active widget | `active_fg_stroke` | `WHITE` |
| Hyperlink | `hyperlink_color` | `from_rgb(90, 170, 255)` |
| Warning | `warn_fg_color` | `from_rgb(255, 143, 0)` |
| Error | `error_fg_color` | `from_rgb(255, 0, 0)` |
| Selection background | `selection_bg_fill` | `from_rgb(0, 92, 128)` |
| Selection border | `selection_stroke` | `from_rgb(192, 222, 255)` |
| Inactive weak bg | `inactive_weak_bg_fill` | `from_gray(60)` |
| Hovered weak bg | `hovered_weak_bg_fill` | `from_gray(70)` |

### Light Theme

| Purpose | Property | Color Value |
|---------|----------|-------------|
| Window background | `window_fill` | `from_gray(248)` |
| Panel background | `panel_fill` | `from_gray(248)` |
| TextEdit background | `extreme_bg_color` | `from_gray(255)` |
| Code background | `code_bg_color` | `from_gray(230)` |
| Static text | `noninteractive_fg_stroke` | `from_gray(80)` |
| Inactive widget | `inactive_fg_stroke` | `from_gray(60)` |
| Hyperlink | `hyperlink_color` | `from_rgb(0, 155, 255)` |
| Warning | `warn_fg_color` | `from_rgb(255, 100, 0)` |

### Color Usage Guidelines

1. **Use semantic colors**: `hyperlink_color`, `warn_fg_color`, `error_fg_color`
2. **Respect theme**: Get colors from `ui.style()` rather than hardcoding
3. **Gray scale for neutral**: Use `from_gray()` for non-accented elements
4. **High contrast for text**: Ensure text is readable against background

### Getting Colors Properly

```rust
// Always use style for theme-aware colors
let text_color = ui.style().visuals.text_color();
let window_fill = ui.style().visuals.window_fill();

// Interactive colors use interact()
let visuals = ui.style().interact(&response);
let bg_color = visuals.bg_fill;
let text_color = visuals.text_color();
let stroke = visuals.bg_stroke;
```

---

## 4. Widget Patterns

### Standard Widget Template (4-Step Pattern)

```rust
pub fn custom_widget_ui(ui: &mut egui::Ui, state: &mut bool) -> egui::Response {
    // 1. Decide size (base on interact_size)
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    
    // 2. Allocate space
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    // 3. Handle interactions
    if response.clicked() {
        *state = !*state;
        response.mark_changed();
    }
    
    // 4. Paint
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke);
        // ... more painting
    }
    
    response
}

// Wrapper for idiomatic usage
pub fn custom_widget(state: &mut bool) -> impl egui::Widget + '_ {
    move |ui: &mut egui::Ui| custom_widget_ui(ui, state)
}
```

### Widget Size Guidelines

| Widget | Size Pattern |
|--------|--------------|
| Checkbox/Radio | `interact_size` (40x18) |
| Small button | `interact_size` (40x18) |
| Medium button | `interact_size * vec2(2, 1)` (80x18) |
| Large button | `interact_size * vec2(3, 2)` (120x36) |
| Icon button | Square based on `icon_width` |
| Toggle switch | `interact_size.y * vec2(2, 1)` (36x18) |

### Widget State Management

```rust
pub struct MyWidget {
    // State fields (snake_case)
    enabled: bool,
    visible: bool,
    selected: bool,
    value: String,
}

// Always implement View trait for consistency
impl crate::View for MyWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // Widget content here
    }
}
```

### Widget Implementation Checklist

- [ ] Size based on `ui.spacing().interact_size`
- [ ] Allocate space with `allocate_exact_size` or `allocate_space`
- [ ] Use appropriate `Sense` (click, drag, hover)
- [ ] Handle all relevant interactions (click, hover, drag)
- [ ] Use `ui.style().interact()` for proper styling
- [ ] Call `response.mark_changed()` when state changes
- [ ] Include `WidgetInfo` for accessibility
- [ ] Use animation for smooth transitions when applicable

---

## 5. Frame Design

### Frame Structure

```rust
pub struct Frame {
    inner_margin: Margin,    // CSS: padding
    fill: Color32,           // Background color
    stroke: Stroke,          // Border (width + color)
    corner_radius: CornerRadius,
    outer_margin: Margin,    // CSS: margin
    shadow: Shadow,
}
```

### Size Calculation

```
total_size = content_size + inner_margin + 2*stroke.width + outer_margin
```

### Standard Frame Presets

| Preset | Inner Margin | Corner Radius | Stroke | Usage |
|--------|--------------|---------------|--------|-------|
| `Frame::NONE` | - | - | - | No frame at all |
| `Frame::group(style)` | `6` | `2` | `1.0, gray` | Group related widgets |
| `Frame::window(style)` | `6` | `6` | `1.0, gray(60)` | Window frame |
| `Frame::canvas(style)` | `2` | `2` | window_stroke | Drawing area |
| `Frame::menu(style)` | `6` | `6` | popup_shadow | Menu/Popup |

### Frame Usage Patterns

```rust
// No frame
Frame::none()
    .show(ui, |ui| {
        ui.label("Content");
    });

// Grouping related widgets
Frame::group(ui.style())
    .inner_margin(12)
    .show(ui, |ui| {
        ui.heading("Group Title");
        ui.label("Grouped content");
    });

// Canvas for drawing
Frame::canvas(ui.style())
    .fill(ui.style().visuals.extreme_bg_color)
    .show(ui, |ui| {
        ui.painter().circle_filled(center, radius, color);
    });

// Custom styled frame
Frame::default()
    .inner_margin(12)
    .outer_margin(24)
    .corner_radius(14)
    .shadow(egui::Shadow {
        offset: [8, 12],
        blur: 16,
        spread: 0,
        color: egui::Color32::from_black_alpha(180),
    })
    .show(ui, |ui| {
        // Content
    });
```

### Frame Guidelines

- Use `Frame::group()` for grouping related controls
- Use `Frame::canvas()` for custom painting
- Use `Frame::NONE` when you need full control over spacing
- Set `inner_margin` for internal padding (like CSS padding)
- Set `outer_margin` for external spacing (like CSS margin)
- Adjust `corner_radius` based on context (2-6px typical)

---

## 6. Window Patterns

### Window Builder Pattern

```rust
egui::Window::new("Title")
    .id(Id::new("unique_id"))     // Required if title changes
    .open(&mut is_open)           // Adds close button
    .resizable(true)              // Enable resize
    .collapsible(true)            // Allow collapse
    .default_width(280.0)
    .default_height(400.0)
    .min_size([100.0, 100.0])
    .max_size([800.0, 600.0])
    .fixed_pos([10.0, 10.0])      // Immoveable
    .anchor(Align2::RIGHT_TOP, [0.0, 0.0])  // Anchor to corner
    .frame(custom_frame)          // Custom styling
    .show(ctx, |ui| {
        // Content
    });
```

### Window Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `id()` | `Id` | Auto | Unique identifier (required if title changes) |
| `open()` | `&mut bool` | `true` | Window visibility state |
| `resizable()` | `bool` | `true` | Enable/disable resize |
| `collapsible()` | `bool` | `true` | Enable/disable collapse |
| `movable()` | `bool` | `true` | Enable/disable move |
| `default_width()` | `f32` | - | Initial width |
| `default_height()` | `f32` | - | Initial height |
| `min_size()` | `Vec2` | `[96, 32]` | Minimum size |
| `max_size()` | `Vec2` | - | Maximum size |
| `fixed_pos()` | `Pos2` | - | Fixed position (non-movable) |
| `anchor()` | `(Align2, Vec2)` | - | Anchor to corner with offset |
| `frame()` | `Frame` | `Frame::window()` | Custom frame styling |

### Window Size Defaults

| Property | Value |
|----------|-------|
| Min width | `96.0` |
| Min height | `32.0` |
| Default size | `[340.0, 420.0]` |
| Corner radius | `6` |
| Title bar height | Based on font + margins |

### Title Bar Patterns

- **Title height**: Based on font height + margins
- **Close button**: Right side, `icon_width` size (14px)
- **Collapse button**: Left side, same size
- **Double-click**: Toggles collapse
- **Drag**: Anywhere on title bar
- **Close on Escape**: Press Escape to close focused window

### Window Creation Pattern for Demos

```rust
impl crate::Demo for MyDemo {
    fn name(&self) -> &'static str {
        "📋 My Demo"
    }
    
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable([true, false])  // Can specify per-axis
            .default_width(280.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}
```

---

## 7. Panel Layouts

### Critical Panel Order (MUST Follow)

Panels MUST be added in this order:
1. **Top panel**
2. **Left panel**
3. **Right panel**
4. **Bottom panel**
5. **Central panel** (fills remaining space)

### Panel Configuration Pattern

```rust
// Top panel
egui::Panel::top("top_panel")
    .resizable(true)
    .min_size(32.0)
    .show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Top bar content");
        });
    });

// Left panel
egui::Panel::left("left_panel")
    .resizable(true)
    .default_size(150.0)
    .size_range(80.0..=200.0)
    .show_inside(ui, |ui| {
        ui.vertical(|ui| {
            ui.label("Sidebar");
        });
    });

// Right panel
egui::Panel::right("right_panel")
    .resizable(true)
    .default_size(200.0)
    .show_inside(ui, |ui| {
        ui.vertical(|ui| {
            ui.label("Properties");
        });
    });

// Bottom panel
egui::Panel::bottom("bottom_panel")
    .resizable(true)
    .min_height(32.0)
    .show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Status bar");
        });
    });

// Central panel (main content)
egui::CentralPanel::default().show_inside(ui, |ui| {
    ui.heading("Main content");
});
```

### Panel Size Guidelines

| Panel Type | Min Size | Typical Range | Notes |
|------------|----------|---------------|-------|
| Top/Bottom | `32.0` | 32-100px | Toolbars, status bars |
| Side panels | - | 80-200px | Sidebars, inspectors |

### Panel Tips

- Always use **unique IDs** for each panel
- Use `show_inside(ui)` instead of `show(ctx)` for proper nesting
- Use `resizable(true)` for flexible layouts
- Use `size_range()` to constrain panel sizes
- Order matters! Wrong order causes layout bugs

---

## 8. Interactive States

### Five Interactive States

| State | Description | Use For |
|-------|-------------|---------|
| **Noninteractive** | Static labels, separators | Non-interactive elements |
| **Inactive** | Resting state of buttons/checkboxes | Default widget state |
| **Hovered** | Mouse over widget | Interactive feedback |
| **Active** | Clicking/dragging | Pressed state |
| **Open** | Menu/ComboBox showing dropdown | Dropdown state |

### Getting Visuals for States

```rust
// For generic interactive widgets
let visuals = ui.style().interact(&response);

// For selectable widgets (checkbox, radio button)
let visuals = ui.style().interact_selectable(&response, is_selected);

// For disabled widgets
let visuals = ui.style().interact_disabled(&response);
```

### Visual Properties by State

| Property | Inactive | Hovered | Active |
|----------|----------|---------|--------|
| Corner Radius | `2` | `3` | `3` |
| Background | `inactive_weak_bg_fill` | `hovered_weak_bg_fill` | `fg_stroke.color` |
| Text Color | `inactive_fg_stroke` | `hovered_fg_stroke` | `active_fg_stroke` |
| Stroke Width | `1.0` | `1.0` | `1.0-2.0` |

### State Transition Guidelines

- Use `animate_bool_responsive()` for smooth state transitions
- Animation duration: ~0.1s (6 frames at 60fps)
- Animate corner radius, colors, positions
- Don't animate layout changes (causes jitter)

---

## 9. Common UI Patterns

### Form Layout

```rust
egui::Grid::new("form")
    .num_columns(2)
    .spacing([40.0, 4.0])
    .show(ui, |ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut name);
        ui.end_row();
        
        ui.label("Email:");
        ui.text_edit_singleline(&mut email);
        ui.end_row();
        
        ui.label("Age:");
        ui.add(egui::Slider::new(&mut age, 0..=120));
        ui.end_row();
    });
```

### Settings Panel

```rust
ui.vertical(|ui| {
    ui.heading("Settings");
    ui.separator();
    
    ui.checkbox(&mut setting1, "Setting 1");
    ui.checkbox(&mut setting2, "Setting 2");
    ui.checkbox(&mut setting3, "Setting 3");
    
    ui.add_space(10.0);
    
    ui.horizontal(|ui| {
        if ui.button("Save").clicked() {
            // Save settings
        }
        if ui.button("Cancel").clicked() {
            // Revert changes
        }
    });
});
```

### Modal Dialog

```rust
if modal_open {
    let modal = egui::Modal::new(Id::new("unique_modal")).show(ui.ctx(), |ui| {
        ui.set_width(250.0);
        ui.heading("Modal Title");
        ui.label("Are you sure?");
        
        ui.separator();
        
        ui.horizontal(|ui| {
            if ui.button("Confirm").clicked() {
                // Confirm action
                ui.close();
            }
            if ui.button("Cancel").clicked() {
                ui.close();
            }
        });
    });
    
    if modal.should_close() {
        modal_open = false;
    }
}
```

### Status Bar

```rust
egui::Panel::bottom("status_bar")
    .resizable(false)
    .min_size(0.0)
    .show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.label("Status: Ready");
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label("v1.0.0");
            });
        });
    });
```

### Tooltip

```rust
// Simple text tooltip
ui.label("Hover me")
    .on_hover_text("This is a tooltip");

// Rich tooltip with UI
ui.label("Hover me")
    .on_hover_ui(|ui| {
        ui.label("Rich content");
        ui.hyperlink("https://example.com");
        if ui.button("Action").clicked() {
            // Handle action
        }
    });

// Tooltip that follows cursor
ui.label("Move me around")
    .on_hover_text_at_pointer("Follows cursor position");
```

### Context Menu

```rust
ui.label("Right-click me")
    .context_menu(|ui| {
        if ui.button("Copy").clicked() {
            // Copy action
        }
        if ui.button("Paste").clicked() {
            // Paste action
        }
        ui.separator();
        if ui.button("Delete").clicked() {
            // Delete action
        }
    });
```

### Menu Button

```rust
ui.menu_button("Menu", |ui| {
    // Nested submenu
    ui.menu_button("Submenu", |ui| {
        if ui.button("Item 1").clicked() {
            ui.close();
        }
        if ui.button("Item 2").clicked() {
            ui.close();
        }
    });
    
    ui.separator();
    
    if ui.button("Item 3").clicked() {
        ui.close();
    }
    if ui.button("Item 4").clicked() {
        ui.close();
    }
});
```

### Scroll Area

```rust
egui::ScrollArea::vertical()
    .auto_shrink(true)            // Shrink to fit content
    .max_height(300.0)
    .scroll_bar_visibility(egui::ScrollBarVisibility::AsNeeded)
    .drag_to_scroll(true)         // Enable drag-to-scroll
    .stick_to_bottom(true)        // Auto-scroll to bottom
    .show(ui, |ui| {
        // Scrollable content here
    });
```

### Scope/Container with State

```rust
ui.scope_builder(ui_builder, |ui| {
    ui.multiply_opacity(0.5);
    ui.style_mut().visuals.override_text_color(Color32::GRAY);
    
    // All widgets here inherit the modified style
    ui.label("This label is semi-transparent gray");
});
```

---

## 10. Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Demo structs | PascalCase | `WidgetGallery`, `FrameDemo`, `ClipboardHistory` |
| State fields | snake_case | `enabled`, `visible`, `boolean`, `selected_index` |
| Demo methods | `ui(&mut self, ui: &mut egui::Ui)` | Standard View trait |
| Window IDs | snake_case strings | `"my_grid"`, `"form"`, `"settings_panel"` |
| Id::new() | snake_case strings | `Id::new("unique_id")` |
| Functions | snake_case | `show_widget()`, `handle_click()` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_ITEMS`, `DEFAULT_SIZE` |

### File Naming

- Demo files: `demo/widget_gallery.rs`, `demo/clipboard_history.rs`
- Module files: `mod.rs` for module declarations
- Test files: `tests/` directory with `*_test.rs` suffix

---

## 11. Animation & Motion

### Default Animation Values

| Property | Value |
|----------|-------|
| Animation time | `6.0 / 60.0` (~0.1s, 6 frames) |
| Scroll speed | `1000.0` points/sec |
| Scroll duration | `0.1 - 0.3` seconds |

### Animation Patterns

```rust
// Boolean state animation
let how_on = ui.ctx().animate_bool_responsive(response.id, *on);

// Value animation
let animated_value = ui.ctx().animate_value_with_time(
    response.id,
    current_value,
    target_value,
    0.2  // duration in seconds
);

// Smooth position
let animated_pos = ui.ctx().animate_value_with_time(
    response.id,
    current_pos,
    target_pos,
    0.15
);
```

### Animation Guidelines

- Use `animate_bool_responsive` for toggle states
- Keep animations short (0.1-0.3s)
- Don't animate layout changes
- Animate visual properties: opacity, position, color
- Use animation for state transitions, not continuous effects

---

## 12. Accessibility

### Always Include WidgetInfo

```rust
// For basic widgets
response.widget_info(|| {
    egui::WidgetInfo::labeled(
        egui::WidgetType::Button,
        ui.is_enabled(),
        "Button text"
    )
});

// For selectable widgets
response.widget_info(|| {
    egui::WidgetInfo::selected(
        egui::WidgetType::Checkbox,
        ui.is_enabled(),
        *checked,
        "Checkbox label"
    )
});

// For custom widgets
response.widget_info(|| {
    egui::WidgetInfo::new(
        egui::WidgetType::Other,
        ui.is_enabled(),
    ).label("Custom widget description")
});
```

### Accessibility Guidelines

- All interactive widgets must have `WidgetInfo`
- Use descriptive labels for all controls
- Provide keyboard alternatives where possible
- Ensure sufficient color contrast
- Support screen readers through proper widget info

---

## 13. Design Principles

### KISS (Keep It Simple, Stupid)

- Use standard widgets when possible
- Avoid custom widgets unless necessary
- Keep UI hierarchy flat (avoid deep nesting)
- Prefer built-in layouts over custom positioning

### DRY (Don't Repeat Yourself)

- Create reusable components for repeated patterns
- Use Frame presets consistently
- Define common spacing in one place
- Share widget implementations across demos

### YAGNI (You Aren't Gonna Need It)

- Implement only what's needed now
- Avoid "future-proofing" UI components
- Remove unused code and features
- Keep feature flags minimal

### Consistency

- Follow spacing constants from `ui.spacing()`
- Use corner radius guidelines (2 inactive, 3 hovered)
- Use proper colors from `ui.style().interact()`
- Maintain consistent naming conventions

### Clarity

- Clear visual hierarchy (Small → Body → Heading)
- Show interactive states clearly
- Provide feedback for all actions
- Use separators to group related content

### Performance

- Don't create widgets that won't be visible
- Use `ui.is_rect_visible()` before painting
- Avoid expensive operations in UI code
- Cache expensive computations

---

## 14. Component Checklist

### Before Creating a New UI Component

- [ ] **Planning**: Is this component really needed?
- [ ] **Standard widget**: Can an existing widget work?
- [ ] **Reuse**: Can I adapt an existing component?
- [ ] **Consistency**: Does it follow this spec?

### Implementation Checklist

- [ ] Uses standard spacing constants from `ui.spacing()`
- [ ] Follows corner radius guidelines (2 inactive, 3 hovered)
- [ ] Implements all 5 interactive states properly
- [ ] Uses proper colors from `ui.style().interact()`
- [ ] Includes `WidgetInfo` for accessibility
- [ ] Size based on `interact_size` (40x18)
- [ ] Uses proper text style hierarchy
- [ ] Follows naming conventions (PascalCase for types, snake_case for fields)
- [ ] Proper Frame selection (NONE/group/window/canvas/menu)
- [ ] Handles theme correctly (no hardcoded colors)
- [ ] Provides visual feedback for all interactions
- [ ] Uses animation for smooth state transitions

### Code Review Checklist

- [ ] No dead code or unused fields
- [ ] No deprecated API usage
- [ ] Proper error handling
- [ ] No hardcoded magic numbers (use constants)
- [ ] Comments are clear and necessary
- [ ] Code is readable and maintainable

---

## Appendix: Quick Reference

### Common Imports

```rust
use egui::{self, *};
use egui::Color32;
use egui::Stroke;
use egui::Vec2;
use egui::Pos2;
use egui::Rect;
use egui::Sense;
use egui::Layout;
use egui::Align;
use egui::Align2;
```

### Common Colors

```rust
Color32::BLACK
Color32::WHITE
Color32::RED
Color32::GREEN
Color32::BLUE
Color32::YELLOW
Color32::TRANSPARENT
Color32::from_gray(128)
Color32::from_rgb(r, g, b)
Color32::from_rgba_premultiplied(r, g, b, a)
```

### Common Layouts

```rust
Layout::left_to_right(Align::Center)
Layout::right_to_left(Align::TOP)
Layout::top_down(Align::LEFT)
Layout::bottom_up(Align::Center)
Layout::centered_and_justified(Direction::TopDown)
```

### Common Alignments

```rust
Align::LEFT
Align::Center
Align::RIGHT
Align::TOP
Align::BOTTOM

Align2::LEFT_TOP
Align2::CENTER_TOP
Align2::RIGHT_TOP
Align2::LEFT_CENTER
Align2::CENTER_CENTER
Align2::RIGHT_CENTER
Align2::LEFT_BOTTOM
Align2::CENTER_BOTTOM
Align2::RIGHT_BOTTOM
```

---

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-01-30 | Initial specification based on egui codebase analysis |

---

## References

- egui source code: `crates/egui/src/`
- egui examples: `crates/egui_demo_lib/src/demo/`
- egui style: `crates/egui/src/style.rs`
- egui containers: `crates/egui/src/containers/`
