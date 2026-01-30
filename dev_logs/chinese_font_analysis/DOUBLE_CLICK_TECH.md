# egui 双击标题栏最大化技术深度分析

## 📋 目录

1. [实现原理概述](#实现原理概述)
2. [事件检测流程](#事件检测流程)
3. [源码分析](#源码分析)
4. [完整实现示例](#完整实现示例)
5. [关键技术点](#关键技术点)
6. [调试技巧](#调试技巧)

---

## 实现原理概述

### 核心概念

双击标题栏最大化涉及三个层面：

```
┌─────────────────────────────────────────────┐
│  应用层 (egui)                              │
│  - 检测双击事件                             │
│  - 发送最大化命令                           │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  框架层 (eframe/egui-winit)                │
│  - 处理 ViewportCommand                    │
│  - 调用底层窗口 API                         │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  系统层 (winit/操作系统)                    │
│  - 实际执行窗口操作                         │
│  - 改变窗口状态和尺寸                       │
└─────────────────────────────────────────────┘
```

---

## 事件检测流程

### 1. 用户交互层

```
用户双击标题栏区域
    ↓
egui 检测到两次鼠标点击
    ↓
判断时间间隔和位置
    ↓
识别为双击事件
```

### 2. egui 事件处理

```
PointerState 收集鼠标事件
    ↓
计算 click_count (点击计数)
    ↓
创建 Click 对象 (count=2)
    ↓
生成 PointerEvent::Released
    ↓
Response::double_clicked() 返回 true
```

### 3. 命令发送

```
应用代码检测 double_clicked()
    ↓
发送 ViewportCommand::Maximized(true/false)
    ↓
egui-winit 接收命令
    ↓
调用 window.set_maximized()
    ↓
winit 调用平台 API
    ↓
窗口状态改变
```

---

## 源码分析

### 1. 双击检测结构

**文件**: `crates/egui/src/input_state/mod.rs:919-940`

```rust
/// A pointer (mouse or touch) click.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Click {
    pub pos: Pos2,

    /// 1 or 2 (double-click) or 3 (triple-click)
    pub count: u32,

    /// Allows you to check for e.g. shift-click
    pub modifiers: Modifiers,
}

impl Click {
    pub fn is_double(&self) -> bool {
        self.count == 2  // ⬅️ 关键：count == 2 表示双击
    }

    pub fn is_triple(&self) -> bool {
        self.count == 3
    }
}
```

### 2. PointerEvent 枚举

**文件**: `crates/egui/src/input_state/mod.rs:942-960`

```rust
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum PointerEvent {
    Moved(Pos2),
    Pressed {
        position: Pos2,
        button: PointerButton,
    },
    Released {
        click: Option<Click>,  // ⬅️ 包含点击计数信息
        button: PointerButton,
    },
}
```

### 3. 双击检测方法

**文件**: `crates/egui/src/input_state/mod.rs:1401-1412`

```rust
/// Was the button given double clicked this frame?
pub fn button_double_clicked(&self, button: PointerButton) -> bool {
    self.pointer_events.iter().any(|event| {
        matches!(
            &event,
            PointerEvent::Released {
                click: Some(click),
                button: b,
            } if *b == button && click.is_double()  // ⬅️ 检查 click.count == 2
        )
    })
}
```

### 4. Response 层 API

**文件**: `crates/egui/src/response.rs:196-213`

```rust
/// Returns true if this widget was double-clicked this frame by the primary button.
#[inline]
pub fn double_clicked(&self) -> bool {
    self.double_clicked_by(PointerButton::Primary)
}

/// Returns true if this widget was double-clicked this frame by the given button.
#[inline]
pub fn double_clicked_by(&self, button: PointerButton) -> bool {
    self.flags.contains(Flags::CLICKED)
        && self.ctx.input(|i| i.pointer.button_double_clicked(button))
}
```

### 5. ViewportCommand 处理

**文件**: `crates/egui-winit/src/lib.rs:1516-1519`

```rust
ViewportCommand::Maximized(v) => {
    window.set_maximized(v);  // ⬅️ 调用 winit 的 set_maximized
    info.maximized = Some(v);
}
```

---

## 完整实现示例

### 方式 1: 简单实现（推荐）

```rust
use egui::{self, CentralPanel, Id, Sense, ViewportCommand};

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let app_rect = ui.max_rect();

            // 定义标题栏区域
            let title_bar_height = 32.0;
            let title_bar_rect = {
                let mut rect = app_rect;
                rect.max.y = rect.min.y + title_bar_height;
                rect
            };

            // 为标题栏区域创建交互响应
            let title_bar_response = ui.interact(
                title_bar_rect,
                Id::new("title_bar"),
                Sense::click(),  // ⬅️ 只需要点击检测
            );

            // 检测双击
            if title_bar_response.double_clicked() {
                let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
            }

            // 绘制标题栏...
        });
    }
}
```

### 方式 2: 完整自定义标题栏

```rust
use egui::{
    self, Align2, CentralPanel, Color32, FontId, Id, PointerButton, Sense,
    Stroke, Vec2, ViewportCommand,
};

fn custom_window_frame(ctx: &egui::Context, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    CentralPanel::default().show(ctx, |ui| {
        let app_rect = ui.max_rect();
        let title_bar_height = 32.0;

        // 标题栏矩形区域
        let title_bar_rect = {
            let mut rect = app_rect;
            rect.max.y = rect.min.y + title_bar_height;
            rect
        };

        // 创建交互区域（支持点击和拖拽）
        let title_bar_response = ui.interact(
            title_bar_rect,
            Id::new("title_bar"),
            Sense::click_and_drag(),  // ⬅️ 同时支持点击和拖拽
        );

        // 绘制标题栏背景
        ui.painter().rect_filled(
            title_bar_rect,
            0.0,
            Color32::from_rgb(240, 240, 240),
        );

        // 绘制标题文本
        ui.painter().text(
            title_bar_rect.center(),
            Align2::CENTER_CENTER,
            title,
            FontId::proportional(16.0),
            Color32::BLACK,
        );

        // 绘制底部边框
        ui.painter().line_segment(
            [
                title_bar_rect.left_bottom(),
                title_bar_rect.right_bottom(),
            ],
            Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
        );

        // ========== 双击最大化 ==========
        if title_bar_response.double_clicked() {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
        }

        // ========== 拖拽移动窗口 ==========
        if title_bar_response.drag_started_by(PointerButton::Primary) {
            ctx.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        // ========== 窗口控制按钮 ==========
        let buttons_rect = {
            let mut rect = title_bar_rect;
            rect.min.x = rect.max.x - 100.0;
            rect
        };

        ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(buttons_rect)
                .layout(egui::Layout::right_to_left(egui::Align::Center)),
            |ui| {
                ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                // 关闭按钮
                if ui.button("❌").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Close);
                }

                // 最大化/还原按钮
                let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                if is_maximized {
                    if ui.button("🗗").on_hover_text("还原").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(false));
                    }
                } else {
                    if ui.button("🗗").on_hover_text("最大化").clicked() {
                        ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
                    }
                }

                // 最小化按钮
                if ui.button("🗕").on_hover_text("最小化").clicked() {
                    ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
                }
            },
        );

        // 内容区域
        let content_rect = app_rect.shrink(4.0);
        let mut content_ui = ui.new_child(egui::UiBuilder::new().max_rect(content_rect));
        add_contents(&mut content_ui);
    });
}
```

### 方式 3: 添加按钮点击和双击共存

```rust
// 如果标题栏上有按钮，需要确保按钮优先响应
let title_bar_response = ui.interact(
    title_bar_rect,
    Id::new("title_bar"),
    Sense::click_and_drag(),
);

// 先处理按钮点击
if button_response.clicked() {
    // 按钮逻辑
    return;
}

// 如果没有点击按钮，再处理标题栏的双击
if title_bar_response.double_clicked() {
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
}
```

---

## 关键技术点

### 1. Sense 类型选择

```rust
pub enum Sense {
    Nothing,              // 不感知任何交互
    Click,                // 只感知点击
    ClickAndDrag,         // 感知点击和拖拽
}
```

**选择指南：**
- 只需要双击：使用 `Sense::click()`
- 需要双击 + 拖拽：使用 `Sense::click_and_drag()`

### 2. 交互区域定义

```rust
// 方式 1: 使用 interact
let response = ui.interact(rect, Id::new("id"), Sense::click());

// 方式 2: 使用 allocate_response
let response = ui.allocate_response(rect.size(), Sense::click());

// 方式 3: 直接使用 widget 的 Response
let response = ui.button("点击我");
```

### 3. 状态获取

```rust
// 获取当前窗口状态
let viewport_info = ui.input(|i| i.viewport());

let is_maximized = viewport_info.maximized.unwrap_or(false);
let is_fullscreen = viewport_info.fullscreen.unwrap_or(false);
let is_minimized = viewport_info.minimized.unwrap_or(false);

// 获取窗口位置和大小
let inner_size = viewport_info.inner_size;
let outer_size = viewport_info.outer_size;
let position = viewport_info.position;
```

### 4. ViewportCommand 常用命令

```rust
pub enum ViewportCommand {
    Close,                          // 关闭窗口
    Minimized(bool),                // 设置最小化状态
    Maximized(bool),                // 设置最大化状态
    Fullscreen(bool),               // 设置全屏状态
    StartDrag,                      // 开始拖拽窗口
    StartResize(Area),              // 开始调整窗口大小
    InnerSize(Vec2),                // 设置窗口内部大小
    OuterSize(Vec2),                // 设置窗口外部大小
    Visible(bool),                  // 设置窗口可见性
    Focus(bool),                    // 设置窗口焦点
    Title(String),                  // 设置窗口标题
    // ... 更多命令
}
```

---

## 调试技巧

### 1. 查看双击事件

```rust
if title_bar_response.double_clicked() {
    eprintln!("检测到双击！");
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    eprintln!("当前最大化状态: {}", is_maximized);
    eprintln!("将切换到: {}", !is_maximized);
}
```

### 2. 可视化交互区域

```rust
// 调试时绘制交互区域边界
ui.painter().rect_stroke(
    title_bar_rect,
    0.0,
    egui::Stroke::new(2.0, egui::Color32::RED),
);

// 显示区域信息
ui.painter().text(
    title_bar_rect.left_top(),
    egui::Align2::LEFT_TOP,
    format!("{:?}", title_bar_response),
    egui::FontId::monospace(10.0),
    egui::Color32::RED,
);
```

### 3. 鼠标事件追踪

```rust
// 在每一帧打印鼠标信息
if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
    eprintln!("鼠标位置: {:?}", pos);
}

if ui.input(|i| i.pointer.any_click()) {
    eprintln!("检测到点击！");
}

if ui.input(|i| i.pointer.any_pressed()) {
    eprintln!("鼠标按下！");
}
```

### 4. 完整调试代码

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("双击调试工具");

            // 显示当前窗口状态
            let viewport = ui.input(|i| i.viewport());
            ui.label(format!("最大化: {:?}", viewport.maximized));
            ui.label(format!("全屏: {:?}", viewport.fullscreen));
            ui.label(format!("最小化: {:?}", viewport.minimized));

            ui.separator();

            // 创建测试区域
            let rect = egui::Rect::from_min_max(
                egui::pos2(100.0, 100.0),
                egui::pos2(300.0, 150.0),
            );

            let response = ui.interact(
                rect,
                egui::Id::new("test_area"),
                egui::Sense::click(),
            );

            // 绘制测试区域
            ui.painter().rect_filled(
                rect,
                0.0,
                if response.hovered() {
                    egui::Color32::from_rgb(200, 200, 255)
                } else {
                    egui::Color32::from_rgb(150, 150, 150)
                },
            );

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                if response.double_clicked() {
                    "双击！"
                } else if response.clicked() {
                    "点击！"
                } else if response.hovered() {
                    "悬停"
                } else {
                    "测试区域"
                },
                egui::FontId::proportional(14.0),
                egui::Color32::BLACK,
            );

            // 处理双击
            if response.double_clicked() {
                let is_maximized = viewport.maximized.unwrap_or(false);
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }
        });
    }
}
```

---

## 性能优化

### 1. 避免重复获取状态

```rust
// ❌ 不推荐：重复获取
if response.double_clicked() {
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
}

if response.clicked() {
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));  // 重复
}

// ✅ 推荐：一次获取
let viewport_info = ui.input(|i| i.viewport());
let is_maximized = viewport_info.maximized.unwrap_or(false);

if response.double_clicked() {
    ctx.send_viewport_cmd(ViewportCommand::Maximized(!is_maximized));
}
```

### 2. 使用常量 ID

```rust
// ✅ 推荐
use egui::Id;

const TITLE_BAR_ID: Id = Id::new("title_bar");
let response = ui.interact(rect, TITLE_BAR_ID, Sense::click());

// ❌ 不推荐（每次创建新 ID）
let response = ui.interact(
    rect,
    Id::new(format!("title_bar_{}", frame_count)),  // 每帧都变
    Sense::click(),
);
```

---

## 常见问题

### Q1: 双击不响应？

**可能原因：**
1. Sense 设置不正确
2. 交互区域被其他 widget 覆盖
3. 鼠标点击位置不在交互区域内

**解决方法：**
```rust
// 调试：绘制交互区域
ui.painter().rect_stroke(rect, 0.0, Stroke::new(2.0, Color32::RED));

// 检查 Sense
let response = ui.interact(rect, id, Sense::click_and_drag());

// 检查是否有重叠
eprintln!("Hovered: {}, Clicked: {}", response.hovered(), response.clicked());
```

### Q2: 双击变成单击？

**原因：** 两次点击时间间隔超过系统双击间隔

**解决：** 系统设置中调整双击速度，或者使用单击切换

### Q3: 如何区分单击和双击？

```rust
// egui 会自动区分，双击事件在第二下点击时触发
if response.double_clicked() {
    // 双击逻辑
} else if response.clicked() {
    // 单击逻辑（注意：双击时也会触发 clicked）
}

// 更好的方式：使用计数
let click_count = ui.input(|i| {
    i.pointer
        .button_click_count(PointerButton::Primary)
        .unwrap_or(0)
});

match click_count {
    1 => { /* 单击 */ }
    2 => { /* 双击 */ }
    3 => { /* 三击 */ }
    _ => {}
}
```

---

## 参考资料

- [egui ViewportCommand 文档](https://docs.rs/egui/latest/egui/enum.ViewportCommand.html)
- [egui Sense 文档](https://docs.rs/egui/latest/egui/struct.Sense.html)
- [egui Response 文档](https://docs.rs/egui/latest/egui/struct.Response.html)
- [custom_window_frame 示例源码](https://github.com/emilk/egui/tree/main/examples/custom_window_frame)

---

*文档生成时间: 2026-01-29*
*egui 版本: 0.33.3*
