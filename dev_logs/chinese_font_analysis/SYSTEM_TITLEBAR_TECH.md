# 系统标题栏双击最大化技术深度解析

## 📋 你问的是

```
┌────────────────────────────────────────────────┐
│ 👁  egui 中文字体支持示例              ─ □ ✕ │  ← 这一层！
├────────────────────────────────────────────────┤
│                                                │
│              应用程序内容区域                   │
│                                                │
└────────────────────────────────────────────────┘
```

**不是**应用内自定义的标题栏，而是**操作系统提供的原生标题栏**。

---

## 🔍 技术原理

### 1. 系统标题栏的归属

系统标题栏**不属于应用程序**，而是由**窗口管理器**渲染和管理的。

```
┌─────────────────────────────────────────────────┐
│  操作系统内核 (Kernel)                          │
│  - 管理硬件资源                                │
└──────────────┬──────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────┐
│  窗口系统                                      │
│  • Windows: User32/Win32                       │
│  • macOS: Cocoa/AppKit                         │
│  • Linux X11: X Server + Window Manager        │
│  • Linux Wayland: Compositor                   │
└──────────────┬──────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────┐
│  窗口管理器 (Window Manager)                   │
│  • Windows: dwm.exe (Desktop Window Manager)   │
│  • macOS: WindowServer                         │
│  • Linux: KWin, Mutter, i3, etc.               │
│  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ │
│  ⭐ 渲染和管理标题栏                            │
│  - 绘制标题、图标、按钮                        │
│  - 处理标题栏交互事件                          │
│  - 管理窗口位置和大小                          │
└──────────────┬──────────────────────────────────┘
               ↓
┌─────────────────────────────────────────────────┐
│  你的应用程序 (egui)                            │
│  • 只能看到和响应客户区内容                    │
│  • 无法直接访问系统标题栏                      │
│  • 只能通过 API 间接控制窗口                   │
└─────────────────────────────────────────────────┘
```

---

## 🎯 双击最大化的完整流程

### Windows 系统

```
用户双击系统标题栏
    ↓
Win32 子系统检测到事件
    ↓
User32.dll 处理
    ↓
发送 WM_NCLBUTTONDBLCLK 消息
    ↓
DefWindowProc 处理消息
    ↓
┌──────────────────────────────────┐
│ 检查点击位置是否在标题栏区域     │
│ - HTCAPTION (标题栏)             │
│ - HTSYSMENU (系统菜单)           │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│ 调用 ShowWindow 函数             │
│ - SW_MAXIMIZE (最大化)           │
│ - SW_RESTORE (还原)              │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│ 窗口管理器执行操作               │
│ - 改变窗口尺寸                   │
│ - 调整窗口位置                   │
│ - 更新窗口状态                   │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│ 通知应用程序                     │
│ - WM_SIZE 消息                   │
│ - WM_GETMINMAXINFO 消息          │
└──────────────┬───────────────────┘
               ↓
应用程序收到窗口大小变化事件
```

**关键 Win32 消息：**
```c
// 窗口非客户区双击消息
WM_NCLBUTTONDBLCLK
  WPARAM: 虚拟键码 (通常为 NULL)
  LPARAM: 光标位置的低 16 位为点击测试码

// 点击测试码 (Hit Test Codes)
#define HTCAPTION       2  // 标题栏区域
#define HTSYSMENU      3  // 系统菜单或关闭按钮
#define HTMINBUTTON    8  // 最小化按钮
#define HTMAXBUTTON    9  // 最大化按钮
#define HTCLOSE        20 // 关闭按钮

// 窗口大小变化消息
WM_SIZE
  WPARAM: SIZE_MAXIMIZED, SIZE_RESTORED, etc.
  LPARAM: 新的窗口尺寸

// 获取窗口最小/最大尺寸限制
WM_GETMINMAXINFO
```

### macOS (Cocoa)

```
用户双击系统标题栏
    ↓
WindowServer 检测到事件
    ↓
Cocoa 框架处理
    ↓
NSWindow 方法调用
    ↓
┌──────────────────────────────────┐
│ shouldZoom() 方法                │
│ - 判断是否允许最大化             │
└──────────────┬───────────────────┘
               ↓
┌──────────────────────────────────┐
│ zoom() 方法                      │
│ - 在标准和用户状态之间切换       │
└──────────────┬───────────────────┘
               ↓
窗口管理器执行操作
    ↓
通知应用程序 (NSWindowDelegate)
    ↓
windowDidResize() 等回调
```

**Objective-C 代码示例：**
```objc
// NSWindow 双击标题栏时的处理流程
- (BOOL)windowShouldZoom:(NSWindow *)window toFrame:(NSRect)frame {
    // 允许最大化
    return YES;
}

- (void)windowDidResize:(NSNotification *)notification {
    // 窗口大小改变后的回调
    NSLog(@"Window resized to: %@", NSStringFromRect(window.frame));
}
```

### Linux X11

```
用户双击系统标题栏
    ↓
X Server 检测到事件
    ↓
_XCB_BUTTON_PRESS 事件
    detail = 1 (首次点击)
    detail = 2 (双击)  ← 关键！
    ↓
窗口管理器处理 (KWin, Mutter, etc.)
    ↓
检查事件类型和窗口区域
    ↓
┌──────────────────────────────────┐
│ 发送 _NET_WM_STATE 请求          │
│  - _NET_WM_STATE_MAXIMIZED       │
│    (原子属性)                    │
└──────────────┬───────────────────┘
               ↓
窗口管理器执行最大化
    ↓
发送 ConfigureNotify 事件给应用
    ↓
应用程序收到窗口配置变化
```

**X11 相关代码：**
```c
// X11 事件类型
#define ButtonPress 4
#define ButtonRelease 5

// 双击检测：detail 字段
// 1 = 单击
// 2 = 双击
// 3 = 三击

// EWMH (Extended Window Manager Hints) 原子
#define _NET_WM_STATE_REMOVE 0  // 移除状态
#define _NET_WM_STATE_ADD    1  // 添加状态
#define _NET_WM_STATE_TOGGLE 2  // 切换状态

// 最大化状态
#define _NET_WM_STATE_MAXIMIZED_HORZ  // 水平最大化
#define _NET_WM_STATE_MAXIMIZED_VERT  // 垂直最大化
```

### Linux Wayland

```
用户双击系统标题栏
    ↓
Wayland 协议事件
    ↓
xdg_surface 事件
    ↓
合成器处理 (KWin, Weston, GNOME Shell, etc.)
    ↓
┌──────────────────────────────────┐
│ 根据 xdg-shell 协议处理          │
│ - 检查是否配置了双击行为         │
│ - 某些合成器可能不支持           │
└──────────────┬───────────────────┘
               ↓
可能的结果：
✅ 正常最大化（如 KDE Wayland）
❌ 无响应（合成器未实现）
⚠️ 不一致的行为
```

**为什么 Wayland 上经常不起作用？**

1. **协议限制**：
   - Wayland 协议没有明确规定双击标题栏的行为
   - 每个合成器可以自由实现或不实现

2. **安全隔离**：
   - 应用程序无法检测系统标题栏的事件
   - 所有交互都由合成器处理

3. **实现差异**：
   - 不同合成器的行为不一致
   - KDE Plasma Wayland: 通常支持
   - GNOME Wayland: 可能支持但不保证
   - Sway/Wlroots: 需要额外配置

---

## 🔧 如何在不同平台上启用双击最大化

### Windows

通常默认启用。如不起作用，检查注册表：

```powershell
# 查看当前设置
Get-ItemProperty -Path "HKCU:\Control Panel\Desktop" -Name "DoubleClickHeight"

# 或通过系统设置
设置 → 辅助功能 → 鼠标 → 双击速度
```

### macOS

系统偏好设置 → Dock → "双击窗口标题栏以最小化"（可以改为最大化）

### Linux X11

**KDE Plasma:**
```bash
# 方法 1: 系统设置
系统设置 → 工作区行为 → 窗口管理 → 标题栏操作
→ "双击标题栏: 最大化"

# 方法 2: 通过配置文件
kwriteconfig5 --file kwinrc --group Windows --key TitlebarDoubleClickCommand 3
# 3 = 最大化, 4 = 最大化到垂直等
```

**GNOME:**
```bash
# 通常默认启用
# 检查 GNOME 设置
gsettings get org.gnome.desktop.wm.preferences mouse-button-modifier
```

**i3/Sway (平铺窗口管理器):**
```bash
# 平铺窗口管理器通常没有传统的"最大化"
# 但可以切换到浮动模式
bindsym --whole-window button2 floating toggle
# 双击切换浮动/平铺
```

### Linux Wayland

**KDE Plasma (Wayland):**
- 通常与 X11 行为一致
- 在系统设置中检查 "窗口行为" → "标题栏和边框"

**GNOME (Wayland):**
- 可能在扩展中设置
- 或使用 dconf-editor:
```bash
gsettings set org.gnome.mutter action-double-click-titlebar 'toggle-maximize'
```

---

## 📊 平台对比总结

| 平台 | 双击最大化支持 | 可配置性 | 一致性 | 备注 |
|-----|--------------|---------|--------|------|
| **Windows** | ✅ 原生支持 | ⚠️ 有限 | ⭐⭐⭐⭐⭐ | 行为一致，可调整双击速度 |
| **macOS** | ✅ 原生支持 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | 可在系统设置中配置 |
| **Linux X11** | ✅ 支持 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | 依赖窗口管理器 |
| **Linux Wayland** | ⚠️ 不确定 | ⭐⭐⭐ | ⭐⭐ | 完全依赖合成器实现 |

---

## 💡 为什么你现在可能没有这个功能？

根据你的环境：
```bash
XDG_SESSION_TYPE=wayland
WAYLAND_DISPLAY=wayland-0
```

**可能原因：**

1. **Wayland 合成器未实现**
   - 你使用的 Wayland 合成器可能没有实现双击标题栏功能

2. **配置未启用**
   - 需要在合成器设置中手动启用

3. **主题/装饰问题**
   - 某些 GTK/Qt 主题可能覆盖了默认行为

**解决方法：**

```bash
# 如果使用 KDE Plasma Wayland
# 系统设置 → 工作区行为 → 窗口管理
# → "标题栏操作" → "双击标题栏: 最大化窗口"

# 如果使用 GNOME Wayland
gsettings set org.gnome.mutter action-double-click-titlebar 'toggle-maximize'

# 如果以上都不起作用
# 使用我们添加的 F11 快捷键作为替代
```

---

## 🎓 应用程序层面的限制

**重要概念：应用程序无法直接控制系统标题栏**

```rust
// ❌ 这是不可能做到的
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 应用程序无法检测到系统标题栏的双击事件
        // 因为这些事件在应用程序看到之前就被窗口管理器处理了

        // 只能通过以下方式间接获取窗口状态变化：
        let viewport = ctx.input(|i| i.viewport());
        let is_maximized = viewport.maximized.unwrap_or(false);
        // 这是窗口管理器处理完后的结果
    }
}
```

**事件流向对比：**

```
系统标题栏事件 (在应用之外)
    ↓
窗口管理器直接处理
    ↓
应用程序只能看到结果（窗口大小变化）

自定义标题栏事件 (在应用内)
    ↓
egui 检测并处理
    ↓
应用程序主动发送命令
    ↓
窗口管理器执行操作
```

---

## 🚀 如果想要完全控制标题栏

唯一的办法是**移除系统标题栏**，自己实现：

```rust
// main.rs
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_decorations(false)  // ❌ 移除系统标题栏
        .with_transparent(true),   // 支持透明和圆角
        ..Default::default()
};
```

**然后自己实现所有功能：**
- 双击最大化
- 拖拽移动
- 关闭/最小化/最大化按钮
- 窗口图标和标题

---

## 📚 参考资料

### Windows
- [WM_NCLBUTTONDBLCLK message](https://docs.microsoft.com/en-us/windows/win32/inputdev/wm-nclbuttondblclk)
- [Window Features](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features)

### macOS
- [NSWindow Class Reference](https://developer.apple.com/documentation/appkit/nswindow)
- [Window Management Guide](https://developer.apple.com/documentation/appkit/nswindow_management)

### Linux X11
- [EWMH (Extended Window Manager Hints)](https://specifications.freedesktop.org/wm-spec/wm-spec-latest.html)
- [_NET_WM_STATE](https://specifications.freedesktop.org/wm-spec/1.3/ar01s03.html)

### Linux Wayland
- [Wayland Protocol](https://wayland.freedesktop.org/docs/html/apa.html)
- [xdg-shell protocol](https://wayland.freedesktop.org/docs/html/protocol-spec-wl_surface.html)

---

*文档生成时间: 2026-01-29*
*egui 版本: 0.33.3*
