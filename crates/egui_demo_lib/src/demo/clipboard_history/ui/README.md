# UI Component System

> 基于Props/State/Event模式的组件化UI系统，可组合、可扩展。

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                     Component System                        │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐    ┌──────────┐    ┌──────────┐             │
│  │  Props   │───▶│ Component│───▶│  Events  │             │
│  │ (Input)  │    │  (Logic) │    │ (Output) │             │
│  └──────────┘    └──────────┘    └──────────┘             │
│       │              │               │                     │
│       ▼              ▼               ▼                     │
│  • Config        • State        • Actions                 │
│  • Data          • UI           • Notifications           │
│  • Callbacks     • Validation   • State Changes           │
├─────────────────────────────────────────────────────────────┤
│              Component Library (Reusable)                   │
│  • Button • Input • Card • Icon • Badge • Divider          │
└─────────────────────────────────────────────────────────────┘
```

## 核心概念

### Component Trait

所有UI组件都实现 `Component` trait：

```rust
pub trait Component: 'static {
    type Props: Props;     // 输入属性
    type State: State;     // 内部状态
    type Event: Event;     // 输出事件

    fn render(&self, ui: &mut Ui, props: &Props, state: &mut State) -> Vec<Event>;
    fn init_state(&self, props: &Props) -> State { ... }
}
```

### 数据流

```
Parent Component
    │
    │ (Props)
    ▼
Child Component ──▶ (Events) ──▶ Parent
    │
    │ (State)
    ▼
Internal State
```

## 可复用组件库

### ButtonComponent

可配置的按钮组件，支持多种样式。

```rust
use clipboard_history::ui::{ButtonComponent, ButtonProps, ButtonVariant};

let button = ButtonComponent::new();
let props = ButtonProps {
    label: "Click Me".to_string(),
    variant: ButtonVariant::Primary,
    enabled: true,
    icon: Some("🎯".to_string()),
    ..Default::default()
};
```

**样式变体**：
- `Primary` - 主要操作（蓝色）
- `Secondary` - 次要操作（灰色）
- `Danger` - 危险操作（红色）
- `Success` - 成功状态（绿色）
- `Ghost` - 幽灵按钮（透明）

### InputComponent

带验证的文本输入组件。

```rust
use clipboard_history::ui::{InputComponent, InputProps, InputType};

let input = InputComponent::new();
let props = InputProps {
    value: "".to_string(),
    placeholder: Some("Enter text...".to_string()),
    input_type: InputType::Text,
    validator: Some(|s: &str| {
        if s.is_empty() { Err("Cannot be empty".to_string()) }
        else { Ok(()) }
    }),
    ..Default::default()
};
```

**输入类型**：
- `Text` - 单行文本
- `TextArea` - 多行文本
- `Password` - 密码输入
- `Email` - 邮箱格式
- `Number` - 数字输入

### CardComponent

容器卡片组件，支持多种主题。

```rust
use clipboard_history::ui::{CardComponent, CardProps, CardTheme};

let card = CardComponent::new();
let props = CardProps {
    theme: CardTheme::Elevated,
    shadow: true,
    ..Default::default()
};
```

**主题变体**：
- `Default` - 默认白卡带边框
- `Elevated` - 带阴影的提升效果
- `Flat` - 无边框扁平卡片
- `Outlined` - 突出边框的卡片
- `Interactive` - 交互效果卡片

### IconComponent

图标组件（使用emoji或unicode）。

```rust
use clipboard_history::ui::{IconComponent, IconProps, Icon, NamedIcon};

let icon = IconComponent::new();
let props = IconProps {
    icon: Icon::Named(NamedIcon::Edit),
    size: 24.0,
    color: Some(egui::Color32::BLUE),
    ..Default::default()
};
```

**预定义图标**：
- 操作：`Add`, `Edit`, `Delete`, `Copy`, `Paste`, `Cut`
- 导航：`Search`, `Filter`, `Settings`, `Undo`, `Redo`
- 文件：`Folder`, `File`, `Image`, `Text`, `Link`, `Code`
- 状态：`Checkmark`, `Cross`, `Warning`, `Info`, `Error`

### BadgeComponent

状态徽章组件。

```rust
use clipboard_history::ui::{BadgeComponent, BadgeProps, BadgeVariant};

let badge = BadgeComponent::new();
let props = BadgeProps {
    text: "New".to_string(),
    variant: BadgeVariant::Info,
    ..Default::default()
};
```

**徽章变体**：
- `Default` - 灰色
- `Info` - 蓝色
- `Success` - 绿色
- `Warning` - 黄色
- `Danger` - 红色

### DividerComponent

视觉分隔线组件。

```rust
use clipboard_history::ui::{DividerComponent, DividerProps, DividerOrientation};

let divider = DividerComponent::new();
let props = DividerProps {
    orientation: DividerOrientation::Horizontal,
    thickness: Some(1.0),
    ..Default::default()
};
```

## 创建自定义组件

### 示例：计数器组件

```rust
use clipboard_history::ui::component::{Component, Props, State, Event};

// 1. 定义 Props
#[derive(Clone, PartialEq, Default)]
pub struct CounterProps {
    pub initial_value: u32,
    pub min: u32,
    pub max: u32,
}

// 2. 定义 State
#[derive(Clone, Default)]
pub struct CounterState {
    value: u32,
}

// 3. 定义 Event
#[derive(Clone, Debug)]
pub enum CounterEvent {
    Changed(u32),
    MaxReached,
    MinReached,
}

// 4. 实现 Component
pub struct CounterComponent;

impl Component for CounterComponent {
    type Props = CounterProps;
    type State = CounterState;
    type Event = CounterEvent;

    fn render(&self, ui: &mut egui::Ui, props: &Self::Props, state: &mut Self::State) -> Vec<Self::Event> {
        let mut events = Vec::new();

        ui.horizontal(|ui| {
            // Decrement button
            if ui.button("-").clicked() {
                if state.value > props.min {
                    state.value -= 1;
                    events.push(CounterEvent::Changed(state.value));
                    if state.value == props.min {
                        events.push(CounterEvent::MinReached);
                    }
                }
            }

            // Value display
            ui.label(format!("{}", state.value));

            // Increment button
            if ui.button("+").clicked() {
                if state.value < props.max {
                    state.value += 1;
                    events.push(CounterEvent::Changed(state.value));
                    if state.value == props.max {
                        events.push(CounterEvent::MaxReached);
                    }
                }
            }
        });

        events
    }

    fn init_state(&self, props: &Self::Props) -> Self::State {
        CounterState {
            value: props.initial_value,
        }
    }
}
```

### 使用自定义组件

```rust
use clipboard_history::ui::{ComponentBuilder, CounterComponent, CounterProps};

fn render_ui(ui: &mut egui::Ui) {
    let component = CounterComponent;
    let builder = ComponentBuilder::new(&component);

    let props = CounterProps {
        initial_value: 0,
        min: 0,
        max: 100,
    };

    let events = builder.render(ui, &props);

    for event in events {
        match event {
            CounterEvent::Changed(value) => {
                println!("Counter changed: {}", value);
            }
            CounterEvent::MaxReached => {
                println!("Maximum value reached!");
            }
            CounterEvent::MinReached => {
                println!("Minimum value reached!");
            }
        }
    }
}
```

## 主题系统

### 使用主题

```rust
use clipboard_history::ui::{Theme, LightTheme, DarkTheme};

fn apply_theme<T: Theme>(theme: &T, ui: &mut egui::Ui) {
    ui.style_mut().visuals.dark_bg_color = theme.background_color();
    ui.style_mut().visuals.override_text_color = Some(theme.text_color());
}
```

### 自定义主题

```rust
use clipboard_history::ui::Theme;

#[derive(Clone, Debug)]
pub struct CustomTheme;

impl Theme for CustomTheme {
    fn primary_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(128, 0, 128) // Purple
    }

    fn background_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(245, 240, 255) // Light purple
    }

    fn text_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(64, 0, 64)
    }

    fn border_radius(&self) -> f32 {
        8.0
    }

    fn spacing(&self) -> egui::Vec2 {
        egui::Vec2::new(12.0, 6.0)
    }

    fn font_size(&self) -> f32 {
        16.0
    }
}
```

## 组件组合

### 嵌套组件

```rust
pub struct FormComponent;

impl Component for FormComponent {
    type Props = FormProps;
    type State = FormState;
    type Event = FormEvent;

    fn render(&self, ui: &mut Ui, props: &Props, state: &mut State) -> Vec<Event> {
        let mut all_events = Vec::new();

        // 渲染子组件并收集事件
        let input = InputComponent::new();
        let input_events = input.render(ui, &state.input_props);
        all_events.extend(input_events.into_iter().map(|e| match e {
            InputEvent::Changed(s) => FormEvent::FieldChanged("input".to_string(), s),
            _ => FormEvent::Ignored,
        }));

        // 渲染按钮
        let button = ButtonComponent::new();
        let button_events = button.render(ui, &state.button_props);
        all_events.extend(button_events.into_iter().map(|e| match e {
            ButtonEvent::Clicked => FormEvent::Submit,
            _ => FormEvent::Ignored,
        }));

        all_events
    }
}
```

## 组件注册表

### 动态组件加载

```rust
use clipboard_history::ui::{ComponentRegistry, Component};

fn setup_registry() -> ComponentRegistry {
    let mut registry = ComponentRegistry::new();

    // 注册组件
    registry.register("counter".to_string(), CounterComponent);
    registry.register("toggle".to_string(), ToggleComponent);
    registry.register("slider".to_string(), SliderComponent);

    registry
}

fn render_by_name(registry: &ComponentRegistry, name: &str, ui: &mut egui::Ui) {
    if let Some(component) = registry.get::<CounterComponent>(name) {
        let builder = ComponentBuilder::new(component);
        builder.render(ui, &CounterProps::default());
    }
}
```

## 最佳实践

### 1. 保持组件单一职责

每个组件应该只做一件事：
- ✅ `ButtonComponent` - 只负责按钮
- ✅ `InputComponent` - 只负责输入
- ❌ `FormComponent` - 不要包含表单外的逻辑

### 2. Props应该是不可变的

```rust
// ✅ 好
#[derive(Clone, PartialEq)]
pub struct ButtonProps {
    pub label: String,
}

// ❌ 不好
pub struct ButtonProps {
    pub label: RefCell<String>, // 不要用可变内部状态
}
```

### 3. State应该是私有的

```rust
// ✅ 好
pub struct CounterState {
    value: u32, // 私有
}

// ❌ 不好
pub struct CounterState {
    pub value: u32, // 公开让外部修改破坏封装
}
```

### 4. Events应该是描述性的

```rust
// ✅ 好 - 描述性
pub enum ButtonEvent {
    Clicked,
    Hovered,
}

// ❌ 不好 - 不清晰
pub enum ButtonEvent {
    Something,
}
```

### 5. 使用默认值

```rust
impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            label: "Button".to_string(),
            variant: ButtonVariant::Primary,
            enabled: true,
            icon: None,
            width: None,
            small: false,
        }
    }
}
```

## 迁移指南

### 从旧的card.rs迁移到组件系统

**旧代码**：
```rust
let renderer = ItemCardRenderer::new();
renderer.render(ui, &item, index, &mut actions, &mut cache, None);
```

**新代码**：
```rust
use clipboard_history::ui::{ItemCardComponent, ItemCardProps};

let card = ItemCardComponent::new();
let props = ItemCardProps {
    item: item.clone(),
    index,
    ..Default::default()
};
let events = card.render(ui, &props, &mut state);
```

## 扩展点

### 添加新的组件类型

1. 在 `components.rs` 中添加新组件模块
2. 实现 `Component` trait
3. 在 `mod.rs` 中导出
4. 添加文档和示例

### 自定义组件样式

1. 定义样式配置结构
2. 实现 `Theme` trait
3. 在组件中应用样式

### 创建组件库

```rust
// 在你的模块中
pub mod my_components {
    use clipboard_history::ui::component::{Component, Props, State, Event};

    pub mod widgets {
        // 你的自定义组件
    }

    pub mod layouts {
        // 布局组件
    }
}
```

## 性能优化

### 1. 组件复用

```rust
// ✅ 好 - 复用组件实例
let button = ButtonComponent::new();
for _ in 0..100 {
    button.render(ui, &props);
}

// ❌ 不好 - 每次创建新实例
for _ in 0..100 {
    let button = ButtonComponent::new();
    button.render(ui, &props);
}
```

### 2. 延迟渲染

```rust
fn render(&self, ui: &mut Ui, props: &Props, state: &mut State) -> Vec<Event> {
    // 只在可见时渲染
    if !props.visible {
        return Vec::new();
    }
    // ...
}
```

### 3. 事件合并

```rust
fn render(&self, ui: &mut Ui, props: &Props, state: &mut State) -> Vec<Event> {
    let mut events = Vec::new();

    // 收集所有事件
    for child in &props.children {
        events.extend(child.render(ui));
    }

    // 合并重复事件
    events.dedup();
    events
}
```
