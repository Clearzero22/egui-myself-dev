# 组件模板 - 用于创建新的功能模块

> 复制此模板创建新模块，确保遵循项目组件化架构规范。

## 使用步骤

1. 复制模板：`cp -r COMPONENT_TEMPLATE your_feature_name/`
2. 替换占位符：`__FEATURE_NAME__` → 实际功能名
3. 实现各层代码
4. 更新 `mod.rs` 导出新模块

## 目录结构模板

```
__FEATURE_NAME__/
├── core/                   # 数据层
│   ├── item.rs            # 数据实体
│   ├── store.rs           # 存储抽象
│   ├── manager.rs         # 业务逻辑
│   └── mod.rs
├── backend/               # 后端层
│   ├── adapter.rs         # 适配器实现
│   └── mod.rs
├── ui/                    # UI层
│   ├── widget.rs          # UI组件
│   ├── dialog.rs          # 对话框
│   └── mod.rs
├── mod.rs                 # 模块入口
└── README.md              # 模块文档
```

## 代码模板

### core/item.rs - 数据实体

```rust
//! Data models for __FEATURE_NAME__.
//!
//! This module provides the core data structures used throughout
//! the __FEATURE_NAME__ feature.

use std::time::{SystemTime, UNIX_EPOCH};

/// Main data entity for __FEATURE_NAME__.
#[derive(Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct __FEATURE_NAME__Item {
    /// Unique identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Content/data
    pub content: String,
    /// Creation timestamp
    pub timestamp: String,
}

impl __FEATURE_NAME__Item {
    /// Create a new item with generated ID and timestamp.
    pub fn new(title: String, content: String) -> Self {
        Self {
            id: Self::generate_id(),
            title,
            content,
            timestamp: Self::current_timestamp(),
        }
    }

    fn generate_id() -> String {
        format!("{:x}", md5::compute(UUID::new_v4().to_bytes()))
    }

    fn current_timestamp() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!(
            "{:02}:{:02}",
            (now % 86400 / 3600) as u32,
            (now % 3600 / 60) as u32
        )
    }
}

impl Default for __FEATURE_NAME__Item {
    fn default() -> Self {
        Self::new("Untitled".to_string(), String::new())
    }
}
```

### core/store.rs - 存储抽象

```rust
//! Storage abstraction for __FEATURE_NAME__.
//!
//! # Extension Points
//!
//! Implement [`Store`] trait for different storage backends:
//! - In-memory: `MemoryStore`
//! - Database: `SqliteStore`, `PostgresStore`
//! - Cloud: `AwsStore`, `GcpStore`

use crate::demo::__FEATURE_NAME__::core::item::__FEATURE_NAME__Item;

/// Error type for store operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotFound(String),
    CapacityExceeded,
    Io(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Storage abstraction for __FEATURE_NAME__ items.
///
/// This trait defines the interface for storing and managing data.
pub trait Store: Send + Sync {
    /// Add a new item.
    fn add(&mut self, item: __FEATURE_NAME__Item) -> Result<()>;

    /// Remove item by ID.
    fn remove(&mut self, id: &str) -> Result<()>;

    /// Get all items.
    fn get_all(&self) -> Vec<__FEATURE_NAME__Item>;

    /// Get item by ID.
    fn get(&self, id: &str) -> Option<__FEATURE_NAME__Item>;

    /// Update existing item.
    fn update(&mut self, id: &str, item: __FEATURE_NAME__Item) -> Result<()>;

    /// Clear all items.
    fn clear(&mut self) -> Result<()>;

    /// Get item count.
    fn len(&self) -> usize;

    /// Check if empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
```

### core/mod.rs - 核心模块入口

```rust
//! Core data models and business logic for __FEATURE_NAME__.
//!
//! This module provides:
//! - [`item`]: Data models and entities
//! - [`store`]: Storage abstraction layer
//! - [`manager`]: Business logic and orchestration
//!
//! # Extension Points
//!
//! - Implement [`store::Store`] for different storage backends
//! - Extend [`item::__FEATURE_NAME__Item`] with custom fields

pub mod item;
pub mod store;
pub mod manager;

// Re-export commonly used types
pub use item::__FEATURE_NAME__Item;
pub use store::{Store, MemoryStore, Error as StoreError, Result as StoreResult};
pub use manager::Manager;
```

### backend/adapter.rs - 后端适配器

```rust
//! Backend adapter for external service integration.
//!
//! # Extension Points
//!
//! Implement [`Adapter`] trait for different external services.

use crate::demo::__FEATURE_NAME__::core::item::__FEATURE_NAME__Item;

/// Error type for adapter operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    NotAvailable,
    Unauthorized,
    Failed(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Adapter trait for external service integration.
pub trait Adapter: Send + Sync {
    /// Sync data from external service.
    fn sync(&self) -> Result<Vec<__FEATURE_NAME__Item>>;

    /// Push data to external service.
    fn push(&self, items: Vec<__FEATURE_NAME__Item>) -> Result<()>;

    /// Check if adapter is available.
    fn is_available(&self) -> bool {
        true
    }
}

/// Default no-op adapter implementation.
pub struct NoOpAdapter;

impl Adapter for NoOpAdapter {
    fn sync(&self) -> Result<Vec<__FEATURE_NAME__Item>> {
        Ok(Vec::new())
    }

    fn push(&self, _items: Vec<__FEATURE_NAME__Item>) -> Result<()> {
        Ok(())
    }

    fn is_available(&self) -> bool {
        false
    }
}
```

### ui/widget.rs - UI组件

```rust
//! UI widgets for __FEATURE_NAME__.

use crate::demo::__FEATURE_NAME__::core::item::__FEATURE_NAME__Item;
use egui::{self, Ui};

/// Action that can be triggered from the widget.
#[derive(Clone, Debug)]
pub enum WidgetAction {
    /// View item details
    View(String),
    /// Edit item
    Edit(String),
    /// Delete item
    Delete(String),
}

/// Renderer for __FEATURE_NAME__ item widget.
pub struct WidgetRenderer {
    // Configuration
    show_timestamp: bool,
    max_content_length: usize,
}

impl WidgetRenderer {
    pub fn new() -> Self {
        Self {
            show_timestamp: true,
            max_content_length: 100,
        }
    }

    /// Render a single item widget.
    pub fn render(
        &self,
        ui: &mut Ui,
        item: &__FEATURE_NAME__Item,
        actions: &mut Vec<WidgetAction>,
    ) {
        egui::Frame::none()
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                self.render_header(ui, item);
                self.render_content(ui, item);
                self.render_actions(ui, item, actions);
            });
    }

    fn render_header(&self, ui: &mut Ui, item: &__FEATURE_NAME__Item) {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(&item.title).strong());
            if self.show_timestamp {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(&item.timestamp).small().weak());
                });
            }
        });
    }

    fn render_content(&self, ui: &mut Ui, item: &__FEATURE_NAME__Item) {
        let preview = if item.content.len() > self.max_content_length {
            format!("{}...", &item.content[..self.max_content_length])
        } else {
            item.content.clone()
        };
        ui.label(egui::RichText::new(preview).small().weak());
    }

    fn render_actions(
        &self,
        ui: &mut Ui,
        item: &__FEATURE_NAME__Item,
        actions: &mut Vec<WidgetAction>,
    ) {
        ui.horizontal(|ui| {
            if ui.button("View").clicked() {
                actions.push(WidgetAction::View(item.id.clone()));
            }
            if ui.button("Edit").clicked() {
                actions.push(WidgetAction::Edit(item.id.clone()));
            }
            if ui.button("Delete").clicked() {
                actions.push(WidgetAction::Delete(item.id.clone()));
            }
        });
    }
}

impl Default for WidgetRenderer {
    fn default() -> Self {
        Self::new()
    }
}
```

### mod.rs - 模块入口

```rust
//! __FEATURE_NAME__ Demo
//!
//! This demo showcases __FEATURE_NAME__ with:
//!
//! - Core data management
//! - Backend integration
//! - UI components
//!
//! # Architecture
//!
//! The __FEATURE_NAME__ system is organized into modules:
//!
//! - [`core`]: Data models, storage abstraction, and business logic
//! - [`backend`]: External service adapters and implementations
//! - [`ui`]: UI components for rendering widgets and dialogs
//!
//! # Extension Points
//!
//! - Implement [`core::store::Store`] for different storage backends
//! - Implement [`backend::Adapter`] for external service integration
//! - Implement custom UI widgets in [`ui`]

pub mod core;
pub mod backend;
pub mod ui;

use core::{Store, MemoryStore, __FEATURE_NAME__Item};
use backend::{Adapter, NoOpAdapter};
use ui::{WidgetRenderer, WidgetAction};
use std::collections::HashMap;

/// Filter mode for the UI.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    #[default]
    All,
    Active,
    Archived,
}

/// __FEATURE_NAME__ Demo Application.
pub struct __FEATURE_NAME__ {
    /// Storage for items
    store: MemoryStore,
    /// Backend adapter
    adapter: NoOpAdapter,
    /// Search query
    search_query: String,
    /// Current filter mode
    filter_mode: FilterMode,
    /// Selected item ID
    selected_id: Option<String>,
    /// Pending actions from widgets
    pending_actions: Vec<WidgetAction>,
}

impl __FEATURE_NAME__ {
    pub fn new() -> Self {
        Self {
            store: MemoryStore::new(),
            adapter: NoOpAdapter,
            search_query: String::default(),
            filter_mode: FilterMode::default(),
            selected_id: None,
            pending_actions: Vec::new(),
        }
    }

    fn add_item(&mut self, title: String, content: String) {
        let item = __FEATURE_NAME__Item::new(title, content);
        let _ = self.store.add(item);
    }

    fn execute_actions(&mut self) {
        for action in self.pending_actions.drain(..) {
            match action {
                WidgetAction::View(id) => {
                    // Handle view action
                }
                WidgetAction::Edit(id) => {
                    // Handle edit action
                }
                WidgetAction::Delete(id) => {
                    let _ = self.store.remove(&id);
                }
            }
        }
    }
}

impl Default for __FEATURE_NAME__ {
    fn default() -> Self {
        Self::new()
    }
}

// -----------------------------------------------------------------------------
// Demo trait implementation
// -----------------------------------------------------------------------------

impl crate::Demo for __FEATURE_NAME__ {
    fn name(&self) -> &'static str {
        "__FEATURE_NAME__"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(700.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

// -----------------------------------------------------------------------------
// View trait implementation
// -----------------------------------------------------------------------------

impl crate::View for __FEATURE_NAME__ {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("__FEATURE_NAME__");
        ui.separator();

        // Action bar
        ui.horizontal(|ui| {
            if ui.button("Add Item").clicked() {
                self.add_item("New Item".to_string(), "Content".to_string());
            }
        });

        ui.separator();

        // Item list
        let items = self.store.get_all();
        let renderer = WidgetRenderer::new();

        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .max_height(400.0)
            .show(ui, |ui| {
                for item in items {
                    renderer.render(ui, &item, &mut self.pending_actions);
                }
            });

        // Execute pending actions
        self.execute_actions();
    }
}
```

## 占位符替换清单

在使用此模板前，请全局替换以下占位符：

| 占位符 | 说明 | 示例值 |
|--------|------|--------|
| `__FEATURE_NAME__` | 功能名称（大写下划线） | `NOTE_TAKING` |
| `__feature_name__` | 功能名称（小写下划线） | `note_taking` |
| `__FeatureName__` | 功能名称（PascalCase） | `NoteTaking` |

## 检查清单

创建新模块时，确保：

- [ ] 所有模块都有 `//!` 级别文档
- [ ] 所有public类型有文档注释
- [ ] Core层无外部依赖
- [ ] 使用trait定义扩展点
- [ ] UI层不包含业务逻辑
- [ ] 包含单元测试
- [ ] 更新本目录的 README.md
