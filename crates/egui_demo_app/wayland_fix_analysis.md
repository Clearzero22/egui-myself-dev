# Wayland 环境下 egui_demo_app 编译错误分析与解决方案

## 问题描述

在 Manjaro Linux (Wayland 环境) 下运行 `cargo run` 时，egui_demo_app 编译失败。

### 错误信息

```
error: The platform you're compiling for is not supported by winit
  --> /home/clearzero22/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/winit-0.30.12/src/platform_impl/mod.rs:78:1
   |
78 | compile_error!("The platform you're compiling for is not supported by winit");
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0432]: unresolved import `self::platform`
  --> /home/clearzero22/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/winit-0.30.12/src/platform_impl/mod.rs:34:15
   |
34 | pub use self::platform::*;
   |               ^^^^^^^^ could not find `platform` in `self`
```

## 问题分析

### 根本原因

1. **平台检测失败**：winit 0.30.12 在编译时未能检测到正确的 Linux 后端支持
2. **环境不匹配**：系统运行在 Wayland 环境下，但项目默认配置未启用 Wayland feature
3. **模块缺失**：由于平台不支持，`platform` 模块未正确编译，导致一系列连锁导入错误

### 系统环境

- **操作系统**: Manjaro Linux (Kernel 6.12.61-1-MANJARO)
- **显示服务器**: Wayland
- **Rust 版本**: 1.88.0
- **Cargo 版本**: 1.88.0
- **GPU**: NVIDIA GeForce RTX 4060 Ti (NVIDIA 575.64.05)

### Cargo.toml 配置分析

查看 `crates/egui_demo_app/Cargo.toml`：

```toml
[features]
default = ["wgpu", "persistence"]
wayland = ["eframe/wayland"]
x11 = ["eframe/x11"]
```

**问题所在**：`default` features 中未包含 `wayland`，导致在 Wayland 环境下编译失败。

## 解决方案

### 临时解决方案

运行时显式启用 wayland feature：

```bash
cargo run --features wayland
```

### 验证结果

编译成功，应用程序正常启动：

```
[2026-01-13T07:08:45Z DEBUG eframe] Using the wgpu renderer
[2026-01-13T07:08:45Z DEBUG sctk] Bound new global [5] wl_output v4
[2026-01-13T07:08:45Z DEBUG sctk] Bound new global [6] zxdg_output_manager_v1 v3
[2026-01-13T07:08:45Z DEBUG egui_wgpu] Picked wgpu adapter: backend: Vulkan, device_type: DiscreteGpu, name: "NVIDIA GeForce RTX 4060 Ti"
```

### 永久解决方案

#### 方案 1: 修改默认 features

修改 `Cargo.toml` 中的 default features：

```toml
[features]
default = ["wgpu", "persistence", "wayland"]  # 添加 wayland
```

**注意**：这会影响所有用户，包括 X11 用户。

#### 方案 2: 配置 Cargo 默认参数

在 `~/.cargo/config.toml` 中添加：

```toml
[build]
jobs = 4

[target.x86_64-unknown-linux-gnu]
egui_demo_app = ["wayland"]
```

#### 方案 3: 创建 Shell 别名

在 `~/.bashrc` 或 `~/.zshrc` 中添加：

```bash
alias cargo-run-wayland='cargo run --manifest-path=path/to/egui_demo_app/Cargo.toml --features wayland'
```

#### 方案 4: 使用环境变量

```bash
export CARGO_FEATURES="wayland"
cargo run --features $CARGO_FEATURES
```

## 技术细节

### winit 平台支持机制

winit 使用条件编译来支持不同平台：

- **Linux X11**: `platform_impl/x11.rs`
- **Linux Wayland**: `platform_impl/wayland.rs`
- **Windows**: `platform_impl/windows.rs`
- **macOS**: `platform_impl/macos.rs`

当未启用对应 feature 时，平台模块不会被编译，导致 `platform` 模块找不到。

### 依赖链分析

```
egui_demo_app
  └── eframe (default-features = false, features = ["web_screen_reader"])
       └── winit (未启用平台 features)
```

需要在 `egui_demo_app` 的 features 中显式启用 `wayland`，它会传递给 `eframe`，进而传递给 `winit`。

### NVIDIA GPU + Wayland 兼容性

运行日志显示：

- **Vulkan 后端**: 正常工作，被选为主要渲染后端
- **OpenGL ES 后端**: 存在一些警告，但不影响运行
- **EGL 错误**: `EGL_BAD_ATTRIBUTE` 错误，但应用程序能正常运行

## 最佳实践建议

### 1. 检测显示服务器类型

在编译或运行前自动检测：

```bash
if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    cargo run --features wayland
else
    cargo run --features x11
fi
```

### 2. 同时支持两种显示服务器

修改 `Cargo.toml`：

```toml
[features]
default = ["wgpu", "persistence", "linux-linux"]
linux-wayland = ["eframe/wayland"]
linux-x11 = ["eframe/x11"]
```

### 3. 文档说明

在项目 README 中明确说明：

```markdown
## Linux 用户

如果您使用 Wayland 显示服务器，请运行：
```bash
cargo run --features wayland
```

如果您使用 X11，请运行：
```bash
cargo run --features x11
```

检测显示服务器类型：
```bash
echo $XDG_SESSION_TYPE
```
```

## 总结

| 项目 | 说明 |
|------|------|
| **问题类型** | 平台依赖未配置 |
| **影响范围** | Wayland 环境下的 Linux 用户 |
| **解决难度** | 低 (单个参数即可) |
| **推荐方案** | 使用 `--features wayland` 或修改默认配置 |
| **长期建议** | 在项目中添加自动检测逻辑或明确的文档说明 |

## 相关资源

- [winit Documentation](https://docs.rs/winit/)
- [Wayland Documentation](https://wayland.freedesktop.org/)
- [eframe GitHub](https://github.com/emilk/egui)
- [EGL on Wayland](https://www.khronos.org/egl/)
