# 🤖 egui Android 实现技术深度解析

## 📋 目录

1. [架构概览](#架构概览)
2. [核心技术栈](#核心技术栈)
3. [实现原理](#实现原理)
4. [代码结构](#代码结构)
5. [编译配置](#编译配置)
6. [运行机制](#运行机制)
7. [与桌面版差异](#与桌面版差异)
8. [开发指南](#开发指南)

---

## 架构概览

### 完整技术栈

```
┌─────────────────────────────────────────────┐
│  你的 Rust 应用 (eframe::App)              │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  eframe (应用框架)                          │
│  - 应用生命周期管理                         │
│  - 状态持久化                               │
│  - 原生平台集成                             │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  egui-winit (平台绑定)                      │
│  - 事件转换                                 │
│  - 输入处理                                 │
│  - 窗口管理                                 │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  winit (跨平台窗口抽象)                     │
│  - Android Game Activity 绑定              │
│  - 事件循环                                 │
│  - 触摸/键盘输入                            │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  android-activity (Rust FFI)               │
│  - game-activity backend                    │
│  - JNI 桥接                                 │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  Android NDK (C/C++)                        │
│  - NativeActivity                           │
│  - 游戏活动生命周期                         │
└──────────────┬──────────────────────────────┘
               ↓
┌─────────────────────────────────────────────┐
│  Android Framework (Java/Kotlin)           │
│  - ActivityManager                         │
│  - WindowManager                            │
│  - InputMethodManager                      │
└─────────────────────────────────────────────┘
```

---

## 核心技术栈

### 1. Android Activity Backend

**两种后端选择**:

```toml
# 方案 A: Game Activity (推荐)
android-game-activity = ["winit/android-game-activity"]

# 方案 B: Native Activity
android-native-activity = ["winit/android-native-activity"]
```

| 特性 | Game Activity | Native Activity |
|-----|--------------|-----------------|
| **IM 支持** | ✅ 完整 | ❌ 有限 |
| **生命周期** | ✅ 标准 | ⚠️ 自定义 |
| **兼容性** | ✅ Android 12+ | ✅ Android 2.3+ |
| **文档** | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| **推荐度** | ✅ 推荐 | ⚠️ 高级用户 |

### 2. eframe Android 集成

**Cargo.toml 配置**:

```toml
[lib]
# 关键！必须同时支持 cdylib 和 lib
crate-type = ["cdylib", "lib"]

[dependencies]
eframe = { workspace = true, default-features = false, features = [
  "default_fonts",
  "glow",              # 渲染后端
  "android-game-activity",  # Android 后端
] }
```

**为什么需要两种 crate 类型？**
- `cdylib`: Android 需要动态系统库 (C Dynamic Library)
- `lib`: 桌面测试和文档生成需要 Rust 库

### 3. Android SDK 配置

```toml
[package.metadata.android]
build_targets = ["armv7-linux-androideabi", "aarch64-linux-android"]

[package.metadata.android.sdk]
min_sdk_version = 23      # Android 6.0
target_sdk_version = 35   # Android 15
```

---

## 实现原理

### 入口点：android_main 函数

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // 1. 初始化日志
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug),
    );

    // 2. 配置 NativeOptions
    let options = eframe::NativeOptions {
        android_app: Some(app),  // ⬅️ 关键！
        ..Default::default()
    };

    // 3. 运行应用
    eframe::run_native(
        "App Name",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    );
}
```

**关键点**:
- `#[unsafe(no_mangle)]` - 禁用名称修饰，让 JNI 能找到函数
- `fn android_main` - Android 特定的入口点
- `android_app: Some(app)` - 将 Android 应用实例传递给 winit

### AndroidApp 的作用

```rust
// winit::platform::android::activity::AndroidApp
pub struct AndroidApp {
    // 内部包含：
    // - JNI 环境指针
    // - NativeActivity 实例
    // - 输入队列
    // - 生命周期回调
}
```

**功能**:
1. **事件循环集成**: 提供事件循环源
2. **输入管理**: 触摸、键盘、手柄输入
3. **窗口管理**: Surface 和 View 管理
4. **生命周期**: onPause/onResume 等

### NativeOptions 的 Android 特殊处理

```rust
pub struct NativeOptions {
    // ... 其他字段 ...

    /// Android 应用实例
    #[cfg(target_os = "android")]
    pub android_app: Option<winit::platform::android::activity::AndroidApp>,
}
```

**为什么使用 Option？**
- Android: `Some(app)` - 必须提供
- 其他平台: `None` - 不适用

---

## 代码结构

### 完整示例结构

```
examples/hello_android_game_ime/
├── Cargo.toml              ← Android 配置
├── src/
│   └── main.rs              ← 应用代码
└── build.rs                ← 构建脚本（可选）
```

### main.rs 完整结构

```rust
use eframe::{CreationContext, egui};
use winit::platform::android::activity::AndroidApp;

// ============================================================
// Android 入口点
// ============================================================
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // 初始化、配置、运行
}

// ============================================================
// 应用结构
// ============================================================
pub struct GameApp {
    text: String,
}

impl GameApp {
    pub fn new(cc: &CreationContext) -> Self {
        Self { text: "Hello".to_string() }
    }
}

// ============================================================
// 应用逻辑
// ============================================================
impl eframe::App for GameApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // UI 渲染
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Android App");
            ui.text_edit_singleline(&mut self.text);
        });
    }
}

// ============================================================
// 桌面入口点（用于开发测试）
// ============================================================
#[cfg(not(target_os = "android"))]
fn main() {
    eframe::run_native(
        "App Name",
        Default::default(),
        Box::new(|cc| Ok(Box::new(GameApp::new(cc)))),
    )
    .unwrap()
}
```

---

## 编译配置

### 1. crate-type 配置

```toml
[lib]
crate-type = ["cdylib", "lib"]
```

**详细说明**:

| 类型 | 生成文件 | 用途 |
|-----|---------|------|
| `cdylib` | `libhello_android.so` | Android 动态库 |
| `lib` | `libhello_android.rlib` | Rust 库元数据 |

**为什么两个都要？**
```bash
# Android 编译时需要 .so
cargo build --target aarch64-linux-android

# 桌面测试时需要 .rlib
cargo build

# 文档生成时需要 .rlib
cargo doc
```

### 2. 特性配置

```toml
[dependencies]
eframe = { workspace = true, default-features = false, features = [
  "default_fonts",        # 内置字体
  "glow",                 # OpenGL 渲染
  "android-game-activity", # ⬅️ Android 后端
] }
```

**default-features = false 的重要性**:
- 不启用 Wayland/X11 支持（Android 不需要）
- 减少编译时间
- 减小最终 APK 大小

### 3. 目标三元组

```toml
[package.metadata.android]
build_targets = [
    "armv7-linux-androideabi",   # 32位 ARM (老设备)
    "aarch64-linux-android",      # 64位 ARM (新设备)
]
```

**编译命令**:
```bash
# 64位 (推荐)
cargo build --target aarch64-linux-android --release

# 32位
cargo build --target armv7-linux-androideabi --release
```

---

## 运行机制

### 1. 应用启动流程

```
Android APK 启动
    ↓
Android Framework
    ↓
NativeActivity 加载
    ↓
JNI: 调用 android_main()
    ↓
Rust: android_main(app)
    ↓
创建 EventLoop (with_android_app)
    ↓
eframe::run_native()
    ↓
创建 Window 和 Surface
    ↓
启动事件循环
    ↓
持续渲染 UI
```

### 2. 事件循环

```rust
// winit 内部实现（简化）
impl ApplicationHandler for WinitAppWrapper {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Android: 每次回到前台时调用
        // 创建或重建窗口
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        // 处理输入事件
        // 触摸、键盘、生命周期等
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // 请求下一帧渲染
        event_loop.request_redraw();
    }
}
```

### 3. 事件处理流程

```
用户触摸屏幕
    ↓
Android InputQueue
    ↓
JNI 转换为 Rust 事件
    ↓
winit::event::Event::Touch
    ↓
egui-winit 转换
    ↓
egui::Event::Pointer
    ↓
你的 App::update()
    ↓
UI 渲染
```

---

## 与桌面版差异

### 1. 入口点差异

| 平台 | 入口点 | 函数签名 |
|-----|--------|---------|
| **桌面** | `main()` | `fn main() -> eframe::Result<()>` |
| **Android** | `android_main()` | `fn android_main(app: AndroidApp)` |

### 2. NativeOptions 差异

```rust
// 桌面版
let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default()
        .with_inner_size([800.0, 600.0]),
    ..Default::default()
};

// Android 版本
let options = eframe::NativeOptions {
    android_app: Some(app),  // ⬅️ 必须提供
    // viewport 由系统管理，不应手动设置
    ..Default::default()
};
```

### 3. 窗口管理差异

| 特性 | 桌面 | Android |
|-----|------|---------|
| **窗口创建** | 应用创建 | 系统管理 |
| **窗口大小** | 可调整 | 固定（屏幕尺寸） |
| **多窗口** | 支持 | 通常单窗口 |
| **窗口装饰** | 系统/自定义 | 无标题栏 |

### 4. 输入处理差异

```rust
// 桌面：鼠标事件
if response.hovered() { ... }
if response.clicked() { ... }

// Android：触摸事件
if response.hovered() { ... }  // 手指悬停
if response.clicked() { ... }   // 触摸点击
```

**关键差异**:
- 桌面有鼠标悬停，Android 没有
- Android 有点触事件
- 需要处理虚拟键盘

### 5. 生命周期差异

```rust
// Android 特有的生命周期事件
#[cfg(target_os = "android")]
fn handle_lifecycle(event: &winit::event::Event) {
    match event {
        winit::event::Event::Suspended => {
            // 应用进入后台
            // 保存状态、暂停渲染
        }
        winit::event::Event::Resumed => {
            // 应用回到前台
            // 恢复渲染
        }
        _ => {}
    }
}
```

---

## 开发指南

### 1. 创建新的 Android 项目

**步骤 1: 创建目录**
```bash
mkdir my_android_app
cd my_android_app
```

**步骤 2: 创建 Cargo.toml**
```toml
[package]
name = "my_android_app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "lib"]

[dependencies]
eframe = { path = "../../crates/eframe", default-features = false, features = [
    "default_fonts",
    "glow",
    "android-game-activity",
]}
egui = { path = "../../crates/egui", default-features = false }
log = "0.4"
android_logger = "0.13"
```

**步骤 3: 创建 main.rs**
```rust
use eframe::{CreationContext, egui};
use winit::platform::android::activity::AndroidApp;

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug),
    );

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };

    eframe::run_native(
        "My Android App",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap()
}

pub struct MyApp {
    text: String,
}

impl MyApp {
    pub fn new(_cc: &CreationContext) -> Self {
        Self {
            text: "Hello Android!".to_string(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My Android App");
            ui.text_edit_singleline(&mut self.text);
        });
    }
}

#[cfg(not(target_os = "android"))]
fn main() {
    eframe::run_native(
        "My Android App",
        Default::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
    .unwrap()
}
```

### 2. 在桌面测试

**为什么需要桌面测试？**
- 快速迭代，不需要每次都部署到 Android
- 更好的调试工具
- 更快的编译速度

**方法**: 使用 `#[cfg(not(target_os = "android"))]` 提供 main 函数

### 3. 编译 Android APK

**方法 1: 使用 cargo-apk**
```bash
# 安装 cargo-apk
cargo install cargo-apk

# 构建 APK
cargo apk build --example my_android_app --release

# 安装到设备
cargo apk install --example my_android_app
```

**方法 2: 使用 gradle**
```bash
# 需要配置 gradle 构建脚本
./gradlew assembleDebug
./gradlew installDebug
```

### 4. 调试

**查看日志**:
```bash
# 使用 adb logcat
adb logcat | grep "Rust"

# 或使用 Android Studio 的 Logcat
```

**在代码中添加日志**:
```rust
log::info!("Information message");
log::warn!("Warning message");
log::error!("Error message");
log::debug!("Debug message");
```

---

## 🎯 关键技术点总结

### 1. android_main 函数

```rust
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    // ⬅️ 这是 Android 应用的真正入口点
}
```

**要点**:
- `#[unsafe(no_mangle)]` 必须保留
- 函数名必须是 `android_main`
- `AndroidApp` 参数由系统提供

### 2. NativeOptions.android_app

```rust
let options = eframe::NativeOptions {
    android_app: Some(app),  // ⬅️ 必须提供
    ..Default::default()
};
```

**如果忘记提供？**
```rust
// ❌ 编译通过，但运行时崩溃
// ❌ 错误: "android_app is required on Android"
```

### 3. 条件编译

```rust
#[cfg(target_os = "android")]
// Android 特定代码

#[cfg(not(target_os = "android"))]
// 非 Android 代码（桌面等）
```

### 4. 虚拟键盘处理

```rust
// 自动显示虚拟键盘
ui.text_edit_singleline(&mut self.text);

// 监听 IME 输出
ctx.output(|o| {
    if let Some(ime) = &o.ime {
        // IME 信息
    }
});
```

---

## 📊 性能考虑

### 1. 渲染性能

**建议**:
```rust
// ❌ 不推荐：每帧都创建大量 widget
fn update(&mut self, ctx: &egui::Context) {
    for i in 0..10000 {
        ui.label(format!("Item {}", i));
    }
}

// ✅ 推荐：使用 ScrollArea 虚拟化
egui::ScrollArea::vertical().show(ctx, |ui| {
    for i in 0..10000 {
        ui.label(format!("Item {}", i));
    }
});
```

### 2. 内存管理

```rust
// ❌ 不推荐：每帧都分配新字符串
let text = format!("Frame: {}", frame_count);

// ✅ 推荐：重用字符串
self.text.clear();
write!(self.text, "Frame: {}", frame_count);
```

### 3. 编译优化

```toml
[profile.release]
opt-level = "z"       # 优化大小
lto = true           # 链接时优化
codegen-units = 1    # 单个编译单元（更小体积）
```

---

## 🐛 常见问题

### Q1: 编译错误 "undefined reference to `android_main`"

**原因**: 函数名或签名不正确

**解决**:
```rust
// ✅ 正确
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) { }

// ❌ 错误
fn main() { }  // 在 Android 上找不到
```

### Q2: 运行时崩溃 "android_app is required"

**原因**: 未提供 android_app 参数

**解决**:
```rust
let options = eframe::NativeOptions {
    android_app: Some(app),  // ← 必须有这一行
    ..Default::default()
};
```

### Q3: 虚拟键盘不显示

**原因**: 文本框没有获得焦点

**解决**:
```rust
// 方法 1: 自动显示（推荐）
ui.text_edit_singleline(&mut self.text);  // 点击自动显示

// 方法 2: 手动请求焦点
let response = ui.text_edit_singleline(&mut self.text);
if response.has_focus() {
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
```

### Q4: 桌面测试时出错

**原因**: 缺少桌面平台的 main 函数

**解决**:
```rust
#[cfg(not(target_os = "android"))]
fn main() {
    eframe::run_native(
        "App",
        Default::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    ).unwrap()
}
```

---

## 📚 参考资料

### 官方文档
- [winit Android 支持](https://docs.rs/winit/latest/winit/platform/android/index.html)
- [android-activity 文档](https://github.com/rust-windowing/android-activity)
- [egui 示例代码](https://github.com/emilk/egui/tree/main/examples/hello_android_game_ime)

### 相关文档
- [eframe Android 集成](./SYSTEM_TITLEBAR_TECH.md)
- [中文字体支持](./TECHNICAL_ANALYSIS.md)

---

*文档生成时间: 2026-01-29*
*egui 版本: 0.33.3*
