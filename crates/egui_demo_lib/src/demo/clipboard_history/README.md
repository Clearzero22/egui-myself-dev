# Clipboard History - Component Architecture

> 模块化剪贴板历史管理器，展示 egui 应用的组件化开发最佳实践。

## 架构概览

```
┌─────────────────────────────────────────────────────────────┐
│                    ClipboardHistory                         │
│                     (mod.rs)                                │
│  状态管理 · 事件协调 · 层次组合                              │
├──────────┬────────────────┬────────────────┬────────────────┤
│   Core   │   Clipboard    │      UI        │   (Extensibility)│
│──────────│────────────────│────────────────│─────────────────│
│• Item    │• Backend trait │• DialogManager │ Store impls:    │
│• Store   │• ArboardBackend│• CardRenderer  │ • MemoryStore   │
│• Filter  │                │• Action types  │ • SqliteStore   │
│• Manager │                │                │ • AsyncStore    │
└──────────┴────────────────┴────────────────┴─────────────────┘
```

## 层次说明

### Core Layer (`core/`)

**职责**：数据模型和领域逻辑，完全独立于UI和具体实现。

| 模块 | 功能 | 扩展点 |
|------|------|--------|
| `item.rs` | 数据模型 `ClipboardItem`, `ContentType` | 自定义内容类型 |
| `store.rs` | 存储抽象 `Store` trait | 不同存储后端 |
| `sqlite_store.rs` | SQLite持久化实现 | 自定义数据库配置 |
| `filter.rs` | 过滤器抽象 `Filter` trait | 自定义过滤策略 |
| `image_manager.rs` | 图片缓存管理 | 自定义图片存储 |

**设计原则**：
- ✅ 零外部依赖（除标准库）
- ✅ 使用trait定义扩展点
- ✅ 所有public API有文档和示例
- ✅ 包含单元测试

### Clipboard Layer (`clipboard/`)

**职责**：剪贴板后端适配，抽象平台差异。

| 模块 | 功能 | 扩展点 |
|------|------|--------|
| `backend.rs` | `Backend` trait定义 | 不同剪贴板实现 |
| `arboard.rs` | arboard库适配 | 其他平台实现 |

**设计原则**：
- ✅ 依赖Core层
- ✅ 使用trait抽象平台差异
- ✅ 错误处理统一

### UI Layer (`ui/`)

**职责**：界面渲染和用户交互组件。

| 模块 | 功能 | 扩展点 |
|------|------|--------|
| `dialog.rs` | 对话框管理器 | 自定义对话框 |
| `card.rs` | 项目卡片渲染器 | 自定义卡片样式 |

**设计原则**：
- ✅ 依赖Core层定义的数据类型
- ✅ 不包含业务逻辑
- ✅ 通过Action枚举与主模块通信

### Module Layer (`mod.rs`)

**职责**：组合各层，管理应用状态和事件流。

```rust
pub struct ClipboardHistory {
    // Core: 数据存储
    store: SqliteStore,

    // Clipboard: 后端适配
    clipboard: ArboardBackend,

    // UI: 组件状态
    dialogs: DialogManager,
    pending_actions: Vec<CardAction>,

    // 应用状态
    search_query: String,
    filter_mode: FilterMode,
    current_page: usize,
}
```

## 扩展点使用示例

### 1. 自定义存储后端

```rust
use clipboard_history::core::store::{Store, ClipboardItem};

pub struct RedisStore {
    client: redis::Client,
}

impl Store for RedisStore {
    fn add(&mut self, item: ClipboardItem) -> Result<()> {
        // Redis实现
    }

    fn get_all(&self) -> Vec<ClipboardItem> {
        // Redis实现
    }

    // ... 其他方法
}
```

### 2. 自定义过滤器

```rust
use clipboard_history::core::filter::{Filter, FilterBox};

pub struct DateRangeFilter {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl Filter for DateRangeFilter {
    fn matches(&self, item: &ClipboardItem) -> bool {
        // 日期范围过滤逻辑
    }
}
```

### 3. 自定义UI组件

```rust
use clipboard_history::ui::card::CardAction;

pub struct GridCardRenderer {
    columns: usize,
}

impl GridCardRenderer {
    pub fn render(
        &self,
        ui: &mut egui::Ui,
        item: &ClipboardItem,
        actions: &mut Vec<CardAction>,
    ) {
        // 网格布局渲染
    }
}
```

## 添加新功能的工作流

### 场景：添加"云同步"功能

1. **Core层**：定义云存储抽象
```rust
// core/cloud_store.rs
pub trait CloudStore: Send + Sync {
    fn sync(&self) -> Result<()>;
    fn upload(&self, items: Vec<ClipboardItem>) -> Result<()>;
    fn download(&self) -> Result<Vec<ClipboardItem>>;
}
```

2. **Backend层**：实现具体云服务
```rust
// clipboard/cloud/aws_sync.rs
pub struct AwsSync {
    bucket: S3Bucket,
}

impl CloudStore for AwsSync {
    // AWS S3实现
}
```

3. **Module层**：集成到主应用
```rust
// mod.rs
pub struct ClipboardHistory {
    // ...
    cloud_sync: Option<Box<dyn CloudStore>>,
}
```

## 测试策略

```bash
# Core层测试（无UI依赖）
cargo test --lib clipboard_history::core

# 集成测试
cargo test --lib clipboard_history

# 示例运行
cargo run --features "persistence,arboard"
```

## 性能优化记录

- ✅ 懒加载图片（仅加载当前页）
- ✅ 分页查询（SQL LIMIT/OFFSET）
- ✅ 纹理缓存（避免重复解码）
- 🚧 异步存储（AsyncStore包装器，待集成）

## 未来扩展方向

- [ ] 完整集成AsyncStore
- [ ] 添加标签/文件夹系统
- [ ] 云同步功能
- [ ] 全文搜索索引
- [ ] 导出/导入功能
