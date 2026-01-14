# egui 自定义窗口框架开发记录

## 项目信息

- **项目**: egui
- **分支**: custom-window-frame-warm-theme
- **开发时间**: 2026-01-14
- **远程仓库**: https://github.com/Clearzero22/egui-myself-dev

---

## 任务概述

为 egui_demo_app 添加自定义窗口框架，去除系统默认边框，实现统一的暖黄色主题界面，并支持窗口拖拽、最大化等交互功能。

---

## 技术实现

### 1. 窗口配置 (main.rs)

**文件**: `crates/egui_demo_app/src/main.rs`

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false)    // 隐藏 OS 窗口边框
        .with_transparent(true)     // 启用透明支持圆角
        .with_inner_size([1280.0, 1024.0])
        .with_drag_and_drop(true),
    ..Default::default()
};
```

**关键点**:
- `with_decorations(false)`: 移除操作系统的标题栏和边框
- `with_transparent(true)`: 启用透明背景，支持圆角效果

---

### 2. 透明背景色 (wrap_app.rs)

**方法**: `eframe::App::clear_color`

```rust
fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
    egui::Rgba::TRANSPARENT.to_array()  // 返回透明色 [0.0, 0.0, 0.0, 0.0]
}
```

**作用**: 每帧绘制前清空画布时使用透明色，避免圆角外出现白边

---

### 3. 自定义窗口框架

**文件**: `crates/egui_demo_app/src/wrap_app.rs`

#### 主窗口框架

```rust
let panel_frame = egui::Frame::new()
    .fill(egui::Color32::from_rgb(249, 243, 224))  // 主背景色 #F9F3E0
    .corner_radius(12)
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 213, 184)))  // 边框色 #E0D5B8
    .inner_margin(4)
    .outer_margin(2);
```

#### 顶部标题栏

```rust
let title_bar_frame = egui::Frame::new()
    .fill(egui::Color32::from_rgb(245, 233, 200))  // 标签栏背景色 #F5E9C8
    .inner_margin(egui::Margin::symmetric(8, 8));
```

---

### 4. 窗口交互功能

#### 拖拽移动窗口

```rust
// 检测鼠标按下位置和当前位置来判断拖拽
let (press_origin, current_pos) = ui.input(|i| {
    (i.pointer.press_origin(), i.pointer.interact_pos())
});

if let (Some(origin), Some(current)) = (press_origin, current_pos) {
    if title_bar_rect.contains(origin) {
        let delta = (current - origin).length();
        if delta > 3.0 && ui.input(|i| i.pointer.primary_down()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }
}
```

#### 双击最大化/还原

```rust
let double_click = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
if double_click {
    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
    if let Some(pos) = pointer_pos {
        if title_bar_rect.contains(pos) {
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
        }
    }
}
```

---

### 5. 窗口控制按钮

**方法**: `window_controls`

```rust
/// 窗口控制按钮：关闭、最小化、最大化
fn window_controls(&self, ui: &mut egui::Ui) {
    let button_size = egui::Vec2::new(32.0, 32.0);

    // 关闭按钮
    if ui.add_sized(button_size, Button::new(RichText::new("❌").size(14.0)))
        .on_hover_text("关闭窗口").clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
    }

    // 最小化按钮
    if ui.add_sized(button_size, Button::new(RichText::new("🗕").size(14.0)))
        .on_hover_text("最小化").clicked() {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
    }

    // 最大化/还原按钮
    let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    if is_maximized {
        if ui.add_sized(button_size, Button::new(RichText::new("🗗").size(14.0)))
            .on_hover_text("还原").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(false));
        }
    } else {
        if ui.add_sized(button_size, Button::new(RichText::new("🗗").size(14.0)))
            .on_hover_text("最大化").clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(true));
        }
    }
}
```

---

## 配色方案

| 用途 | 颜色名称 | RGB | 十六进制 | 描述 |
|-----|---------|-----|---------|------|
| 主背景色 | 最浅暖黄色 | 249, 243, 224 | `#F9F3E0` | 终端主要背景 |
| 标签栏背景 | 暖黄色 | 245, 233, 200 | `#F5E9C8` | 顶部标签区域 |
| 边框/分隔线 | 浅棕色 | 224, 213, 184 | `#E0D5B8` | 区域分隔 |

---

## ViewportCommand 命令参考

| 命令 | 作用 |
|-----|------|
| `ViewportCommand::StartDrag` | 开始拖拽窗口 |
| `ViewportCommand::Close` | 关闭窗口 |
| `ViewportCommand::Minimized(true)` | 最小化窗口 |
| `ViewportCommand::Maximized(true)` | 最大化窗口 |
| `ViewportCommand::Maximized(false)` | 还原窗口 |
| `ViewportCommand::Fullscreen(bool)` | 全屏切换 |

---

## Wayland 环境编译问题

### 问题
在 Wayland 环境下编译失败，提示平台不支持。

### 解决方案
运行时启用 wayland feature:
```bash
cargo run --features wayland
```

### 原因分析
winit 使用条件编译支持不同平台，默认未启用 wayland feature。

---

## Git 提交记录

### 分支
```
custom-window-frame-warm-theme
```

### 提交

| 提交 ID | 描述 |
|---------|------|
| `3feb1c6c` | Add custom window frame with warm yellow theme to egui_demo_app |
| `83848be2` | Add fullscreen toggle support to confirm_exit example |
| `4eef1c38` | Add Wayland compilation fix analysis document |

### 远程仓库
```
https://github.com/Clearzero22/egui-myself-dev/tree/custom-window-frame-warm-theme
```

---

## 参考资源

- [egui custom_window_frame 示例](https://github.com/emilk/egui/tree/main/examples/custom_window_frame)
- [egui ViewportBuilder 文档](https://docs.rs/egui/latest/egui/struct.ViewportBuilder.html)
- [ViewportCommand 参考](https://docs.rs/egui/latest/egui/enum.ViewportCommand.html)

---

## 开发命令

### 编译运行
```bash
# Wayland 环境
cargo run --manifest-path crates/egui_demo_app/Cargo.toml --features wayland

# X11 环境
cargo run --manifest-path crates/egui_demo_app/Cargo.toml --features x11
```

### Git 操作
```bash
# 推送到远程
git push myfork custom-window-frame-warm-theme

# 查看提交历史
git log --oneline -3
```

---

## 技术要点总结

1. **无边框窗口**: 使用 `with_decorations(false)` 移除系统边框
2. **透明背景**: `with_transparent(true)` + `clear_color` 返回透明色
3. **自定义 Frame**: 使用 `egui::Frame` 定义自定义窗口外观
4. **窗口拖拽**: 通过检测鼠标按下位置和移动距离，发送 `StartDrag` 命令
5. **窗口控制**: 使用 `ViewportCommand` 控制窗口状态
6. **交互区域**: 先绘制 UI 元素，再检测鼠标位置，避免按钮与拖拽冲突

---

## 后续优化方向

1. 添加窗口大小调整功能（拖拽边缘调整大小）
2. 支持多显示器环境
3. 添加窗口位置记忆功能
4. 优化拖拽性能，减少命令发送频率
5. 添加更多主题颜色选项
