# ClipHub - 剪贴板历史管理与同步工具

**Date:** 2025-01-30
**Status:** Design Approved
**Author:** Claude + User

---

## 概述

ClipHub 是一个基于 egui 的桌面应用，用于自动捕获剪贴板内容并提供多平台同步功能。

### 核心功能

1. **自动监控剪贴板** - 后台监听，自动保存复制内容
2. **全类型支持** - 文本、代码、图片、链接
3. **本地优先** - SQLite 存储，离线可用
4. **多平台同步** - Notion、自建服务器、Obsidian
5. **智能管理** - 搜索、标签、批量操作

### 使用场景

工作资料收集 + 临时缓存稍后整理

---

## 数据结构

### 数据库表

```sql
CREATE TABLE clipboard_items (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    content_type    TEXT NOT NULL,           -- 'text' | 'code' | 'image' | 'url'
    title           TEXT,
    content         TEXT NOT NULL,
    metadata        TEXT,                    -- JSON: 语言、URL、图片尺寸等
    source_app      TEXT,
    created_at      DATETIME DEFAULT NOW(),
    is_synced       BOOLEAN DEFAULT 0,
    sync_targets    TEXT,                    -- JSON: ['notion', 'server', 'obsidian']
    tags            TEXT,                    -- JSON: ['工作', '参考']
    is_deleted      BOOLEAN DEFAULT 0
);

CREATE TABLE sync_status (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    item_id         INTEGER REFERENCES clipboard_items(id),
    target          TEXT NOT NULL,
    status          TEXT NOT NULL,           -- 'pending' | 'synced' | 'failed'
    error_message   TEXT,
    last_sync_at    DATETIME
);
```

### Rust 数据结构

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Code { language: String },
    Image { path: String, width: u32, height: u32 },
    Url { url: String, metadata: Option<UrlMetadata> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: Option<i64>,
    pub content_type: ContentType,
    pub title: Option<String>,
    pub content: String,
    pub source_app: Option<String>,
    pub created_at: DateTime<Utc>,
    pub is_synced: bool,
    pub sync_targets: Vec<String>,
    pub tags: Vec<String>,
}
```

---

## UI 设计

### 主窗口

```
┌─────────────────────────────────────────────────────────────────┐
│  📋 ClipHub                    [+ 批量同步] [⚙️ 设置] [🗑️ 清理]  │
├─────────────────────────────────────────────────────────────────┤
│  🔍 [搜索框........................................] [类型筛选▼]  │
├─────────────────────────────────────────────────────────────────┤
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ 📄 代码片段 - Rust                     2025-01-30 14:23     │ │
│ │ ┌─────────────────────────────────────────────────────────┐ │ │
│ │ │ fn hello() {                                          │ │ │
│ │ │     println!("Hello");                                │ │ │
│ │ │ }                                                     │ │ │
│ │ └─────────────────────────────────────────────────────────┘ │ │
│ │ 标签: [Rust] [示例]      来源: VSCode                      │ │
│ │ [☐ Notion] [☑ 服务器] [☐ Obsidian]                       │ │
│ └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 右侧详情面板

```
┌─────────────────────────────────────────────┐
│  详细信息                                    │
├─────────────────────────────────────────────┤
│  📝 标题: [代码片段 - Rust........]        │
│  🏷️ 标签: [+ 添加] [Rust] [x] [示例] [x]   │
│  📅 创建: 2025-01-30 14:23                 │
│  📱 来源: VSCode                            │
│                                             │
│  同步目标:                                  │
│  ☑ 我的服务器     ✅ 已同步 14:25          │
│  ☐ Notion                                  │
│  ☐ Obsidian                                │
│                                             │
│  [🔄 立即同步] [📋 复制] [🗑️ 删除]         │
└─────────────────────────────────────────────┘
```

---

## 模块架构

```
cliphub/
├── clipboard/
│   ├── monitor.rs        # 剪贴板监听器
│   ├── detector.rs       # 内容类型检测
│   └── processor.rs      # 内容预处理
│
├── storage/
│   ├── database.rs       # SQLite 操作
│   ├── image_store.rs    # 图片存储
│   └── query.rs          # 搜索过滤
│
├── sync/
│   ├── mod.rs            # 同步管理器
│   ├── notion.rs         # Notion API
│   ├── server.rs         # 服务器 API
│   └── obsidian.rs       # Obsidian 文件写入
│
├── ui/
│   ├── main_window.rs    # 主窗口
│   ├── item_card.rs      # 项目卡片
│   ├── detail_panel.rs   # 详情面板
│   └── tray_icon.rs      # 系统托盘
│
└── app.rs                # 应用入口
```

---

## 数据流

### 自动捕获流程

```
用户复制 → Clipboard Monitor → 内容检测 → 保存到数据库
```

### 同步流程

```
选择项目 → 选择目标 → 执行同步 → 更新状态
         (批量支持)  (多目标)   (并发)    (错误处理)
```

---

## 错误处理

### 错误分类

```rust
pub enum ClipHubError {
    ClipboardAccess(String),
    DatabaseError(sqlite::Error),
    NetworkError(reqwest::Error),
    ApiError { target: String, code: u16, message: String },
    AuthError(String),
    ConfigNotFound(String),
}
```

### 重试策略

- 可重试错误：网络错误、5xx 错误
- 指数退避：1s → 2s → 4s → 最大 30s
- 最大重试：3 次

---

## 测试策略

### 覆盖率目标

| 模块 | 目标 |
|------|------|
| clipboard/ | 90% |
| storage/ | 85% |
| sync/ | 80% |
| ui/ | 60% |

### 测试类型

- **单元测试** - 函数级测试
- **集成测试** - 模块交互测试
- **UI 测试** - egui_kittest
- **E2E 测试** - 完整流程测试

---

## 技术栈

- **UI**: egui + eframe
- **剪贴板**: arboard
- **数据库**: sqlite + rusqlite
- **HTTP**: reqwest
- **异步**: tokio
- **序列化**: serde
