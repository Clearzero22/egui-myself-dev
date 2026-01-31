# Pomodoro Timer - Component Architecture

> 番茄计时器 - 支持自动翻页的专注阅读工具

## 功能特性

- 标准25/5分钟番茄钟
- 环形进度条可视化
- 自动翻页触发
- 会话统计
- 手动翻页控制

## 架构

```
pomodoro_timer/
├── core/      # 计时器核心逻辑
├── content/   # 内容分页
├── ui/        # UI组件
└── mod.rs     # 主应用
```

## 扩展点

- 实现 `core::store::Store` 支持持久化存储
- 实现 `content::pager::Pager` 支持不同内容类型
- 自定义 `ui` 组件样式

## 使用

在 egui_demo 中选择 "番茄计时器" 查看演示。
