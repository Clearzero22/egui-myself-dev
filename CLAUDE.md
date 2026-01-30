# Claude AI 开发指令

> 本文档定义 AI 助手在此项目中开发时必须遵循的组件化开发规范。

## 核心原则

### 组件化开发 (Component-Based Development)

此项目采用**模块化组件架构**，所有新功能必须按照组件化方式开发和组织。

```
┌─────────────────────────────────────────────────────────┐
│                    Application Layer                    │
├─────────────────────────────────────────────────────────┤
│                    Feature Module                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │   Core   │  │Clipboard │  │    UI    │              │
│  │ (Data)   │  │(Backend) │  │(Render) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
├─────────────────────────────────────────────────────────┤
│                    egui Framework                       │
└─────────────────────────────────────────────────────────┘
```

## 组件化架构规范

### 1. 模块层次结构

每个功能模块必须按照以下层次组织：

```
feature_name/
├── core/           # 核心数据层：数据模型、领域逻辑
│   ├── item.rs     # 数据实体
│   ├── store.rs    # 存储抽象
│   ├── filter.rs   # 业务逻辑
│   └── mod.rs
├── backend/        # 后端适配层：外部服务集成
│   ├── adapter.rs  # 适配器实现
│   └── mod.rs
├── ui/             # UI层：界面渲染
│   ├── card.rs     # UI组件
│   ├── dialog.rs   # 对话框组件
│   └── mod.rs
└── mod.rs          # 模块入口：组合层
```

### 2. 层次职责

| 层次 | 职责 | 依赖 | 禁止 |
|------|------|------|------|
| **Core** | 数据模型、领域逻辑、存储抽象 | 无外部依赖 | 不依赖UI、不依赖具体后端 |
| **Backend** | 外部服务适配、平台集成 | Core | 不依赖UI |
| **UI** | 界面渲染、用户交互 | Core、Backend | 不包含业务逻辑 |
| **Module** | 模块编排、状态管理 | Core、Backend、UI | - |

### 3. 扩展点设计

每个组件必须明确定义扩展点（Extension Points）：

```rust
//! # Extension Points
//!
//! - Implement [`core::store::Store`] for different storage backends
//! - Implement [`clipboard::backend::Backend`] for platform-specific clipboards
//! - Implement [`core::filter::Filter`] for custom filtering strategies
```

## 开发工作流

### 新功能开发步骤

当用户要求添加新功能时，AI必须：

1. **分析需求** → 识别涉及的层次
2. **设计接口** → 在对应层次定义trait/抽象
3. **实现核心** → 先实现Core层的数据和逻辑
4. **实现适配** → 再实现Backend/UI层
5. **集成测试** → 在Module层组合测试

### 代码审查检查点

AI在提交代码前必须检查：

- [ ] 每个层次职责清晰，无越界依赖
- [ ] 扩展点已文档化
- [ ] Core层可独立测试（无UI依赖）
- [ ] 使用trait抽象外部依赖
- [ ] 遵循SOLID原则

## 示例：clipboard_history 模块

参考 `crates/egui_demo_lib/src/demo/clipboard_history/` 作为标准组件化实现：

```
clipboard_history/
├── core/           # 数据模型、存储抽象、过滤逻辑
├── clipboard/      # 剪贴板后端抽象
├── ui/             # UI组件（卡片、对话框）
└── mod.rs          # 主应用：组合各层
```

## 命令速查

### 查看现有组件结构
```
Show me the module structure of clipboard_history
```

### 创建新组件
```
Create a new feature module following the component architecture
```

### 检查架构合规性
```
Review this code for component architecture compliance
```

## 参考文档

- [clipboard_history 模块文档](./crates/egui_demo_lib/src/demo/clipboard_history/README.md)
- [项目架构文档](./ARCHITECTURE.md)
- [贡献指南](./CONTRIBUTING.md)
