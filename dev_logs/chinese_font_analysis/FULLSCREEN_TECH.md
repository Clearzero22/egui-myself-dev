# egui 窗口全屏功能技术原理

## 📋 问题

点击系统标题栏双击全屏的技术原理是什么？为什么有时不起作用？

---

## 🔍 技术原理

### 1. 系统标题栏（默认情况）

当使用系统默认标题栏时：

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0])
        // with_decorations 默认为 true
        ..Default::default()
};
```

**事件流程：**
```
┌─────────────────────────────────────────┐
│  用户双击系统标题栏                      │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  窗口管理器检测事件                      │
│  (Windows: dwm.exe)                     │
│  (macOS: WindowServer)                  │
│  (Linux X11: 窗口管理器)                │
│  (Linux Wayland: 合成器)                │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  窗口管理器切换窗口状态                  │
│  - 普通窗口 → 最大化                     │
│  - 最大化 → 普通窗口                     │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  发送窗口尺寸变更事件给应用程序          │
└──────────────┬──────────────────────────┘
               ↓
┌─────────────────────────────────────────┐
│  egui 收到 resize 事件                   │
│  重新渲染 UI                             │
└─────────────────────────────────────────┘
```

**关键特点：**
- ✅ **应用程序无需编写任何代码**
- ✅ 由操作系统/窗口系统自动处理
- ✅ 行为一致，符合用户习惯

### 2. 自定义标题栏（移除系统装饰）

当使用自定义窗口框架时：

```rust
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false)  // ❌ 移除系统标题栏
        .with_transparent(true)
        ..Default::default()
};
```

**需要手动实现所有交互：**

```rust
// 检测标题栏区域的交互事件
fn handle_title_bar_interaction(ui: &mut egui::Ui, title_bar_rect: egui::Rect) {
    let response = ui.allocate_rect(title_bar_rect, egui::Sense::click_and_drag());

    // 双击最大化/还原
    if response.double_clicked() {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    }

    // 拖拽移动窗口
    if response.drag_started_by(egui::PointerButton::Primary) {
        ui.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }
}
```

**实现要点：**
1. 创建标题栏区域
2. 检测鼠标交互（点击、双击、拖拽）
3. 发送相应的 `ViewportCommand` 命令

---

## 🎯 ViewportCommand 命令列表

egui 提供的窗口控制命令：

| 命令 | 作用 | 使用场景 |
|-----|------|---------|
| `ViewportCommand::Close` | 关闭窗口 | 关闭按钮 |
| `ViewportCommand::Minimized(true)` | 最小化窗口 | 最小化按钮 |
| `ViewportCommand::Maximized(true)` | 最大化窗口 | 最大化按钮或双击标题栏 |
| `ViewportCommand::Maximized(false)` | 还原窗口 | 还原按钮或双击标题栏 |
| `ViewportCommand::Fullscreen(true)` | 进入全屏 | F11快捷键 |
| `ViewportCommand::Fullscreen(false)` | 退出全屏 | ESC或F11 |
| `ViewportCommand::StartDrag` | 开始拖拽窗口 | 拖拽标题栏 |
| `ViewportCommand::InnerSize(size)` | 设置窗口大小 | 调整大小 |

---

## 🖥️ 平台差异

### Windows (Win32)
```
双击标题栏
    ↓
WM_NCLBUTTONDBLCLK 消息
    ↓
DefWindowProc 处理
    ↓
ShowWindow(SW_MAXIMIZE) 或 ShowWindow(SW_RESTORE)
```
- ✅ 双击标题栏全屏：原生支持
- ✅ 行为一致

### macOS (Cocoa)
```
双击标题栏
    ↓
NSWindow 的标准行为
    ↓
调用 zoom: 方法
    ↓
窗口在标准和用户状态之间切换
```
- ✅ 双击标题栏全屏：原生支持
- ✅ 可在系统设置中配置

### Linux X11
```
双击标题栏
    ↓
_XCB_BUTTON_PRESS 事件 (count = 2)
    ↓
窗口管理器处理 (KWin, Mutter, etc.)
    ↓
_NET_WM_STATE 请求
```
- ✅ 双击标题栏全屏：依赖窗口管理器
- ⚠️ 不同窗口管理器行为可能不同

### Linux Wayland
```
双击标题栏
    ↓
Wayland 协议事件
    ↓
合成器处理 (KWin, Weston, GNOME Shell, etc.)
    ↓
xdg_shell 请求
```
- ⚠️ 双击标题栏全屏：依赖合成器实现
- ⚠️ **行为一致性较差**
- ⚠️ 可能不支持或需要额外配置

---

## 🐛 为什么有时不起作用？

### 原因1：Wayland 环境
你当前的环境：
```bash
XDG_SESSION_TYPE=wayland
WAYLAND_DISPLAY=wayland-0
```

**问题：**
- Wayland 合成器对窗口装饰的处理不统一
- 某些合成器可能不支持双击标题栏功能
- 需要在合成器设置中启用

### 原因2：窗口管理器配置
```bash
# KDE Plasma
系统设置 → 工作区行为 → 桌面效果
→ "双击标题栏最大化" 选项

# GNOME
通常默认启用，但某些版本可能需要：
gsettings set org.gnome.desktop.wm.preferences mouse-button-modifier '<Super>'
```

### 原因3：自定义框架覆盖
如果使用 `with_decorations(false)`，系统标题栏被移除，双击事件需要手动处理。

---

## 💡 解决方案

### 方案1：使用快捷键（推荐）

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // F11 切换全屏
        if ctx.input(|i| i.key_pressed(egui::Key::F11)) {
            let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!is_fullscreen));
        }

        // ESC 退出全屏
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            let is_fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            if is_fullscreen {
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
            }
        }
    }
}
```

### 方案2：检查窗口管理器设置

在 Wayland 环境下，检查你的合成器设置：

**KDE Plasma (Wayland):**
```bash
# 检查当前设置
kreadconfig5 --file kwinrc --group Windows --key TitlebarDoubleClickCommand

# 设置为最大化
kwriteconfig5 --file kwinrc --group Windows --key TitlebarDoubleClickCommand 3
# 3 = 最大化，其他值见文档
```

**GNOME (Wayland):**
```bash
# GNOME 通常默认支持双击标题栏
# 如果不起作用，检查扩展是否冲突
```

### 方案3：实现自定义标题栏（完全控制）

```rust
fn custom_window_frame(ctx: &egui::Context, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    egui::CentralPanel::default()
        .frame(egui::Frame::none())
        .show(ctx, |ui| {
            // 标题栏
            let title_bar_response = ui.allocate_response(
                egui::vec2(ui.available_width(), 32.0),
                egui::Sense::click_and_drag(),
            );

            // 双击最大化
            if title_bar_response.double_clicked() {
                let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }

            // 拖拽移动
            if title_bar_response.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }

            // 标题栏内容...
        });
}
```

---

## 📊 对比总结

| 特性 | 系统标题栏 | 自定义标题栏 |
|-----|----------|------------|
| 双击全屏 | 窗口管理器处理 | 需要手动实现 |
| 拖拽移动 | 窗口管理器处理 | 需要手动实现 |
| 平台一致性 | 高 | 完全控制 |
| 代码复杂度 | 低 | 高 |
| 外观定制 | 受限 | 完全自由 |

---

## 🎓 最佳实践

1. **优先使用系统标题栏**
   - 除非有特殊需求（如透明窗口、圆角等）
   - 用户习惯、行为一致

2. **提供快捷键备选方案**
   ```rust
   // F11 全屏
   // Ctrl+Q 退出
   // Ctrl+, 打开设置
   ```

3. **跨平台测试**
   - 在不同平台测试窗口交互
   - 特别注意 Wayland 环境的差异

4. **清晰的视觉反馈**
   ```rust
   ui.label("提示: F11 切换全屏 | 双击标题栏最大化");
   ```

---

## 📚 参考资料

- [egui ViewportCommand 文档](https://docs.rs/egui/latest/egui/enum.ViewportCommand.html)
- [Wayland xdg-shell 协议](https://wayland.freedesktop.org/docs/html/apa.html#protocol-spec-wl_surface)
- [X11 窗口管理规范](https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html)

---

*文档生成时间: 2026-01-29*
*egui 版本: 0.33.3*
