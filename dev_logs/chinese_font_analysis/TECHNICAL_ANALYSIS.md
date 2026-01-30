# egui 中文字符显示问题技术分析

## 📋 目录

1. [问题现象](#问题现象)
2. [根本原因](#根本原因)
3. [源码分析](#源码分析)
4. [解决方案](#解决方案)
5. [实现细节](#实现细节)
6. [最佳实践](#最佳实践)

---

## 问题现象

### 预期行为
```
你好，世界！
```

### 实际行为（未配置中文字体时）
```
◻◻◻◻◻◻◻
```

或者部分显示为方块：
```
你好◻◻◻
```

---

## 根本原因

### 1. 默认字体不包含 CJK 字符

egui 默认加载的字体文件（`crates/epaint_default_fonts/src/lib.rs`）：

| 字体 | 文件 | 字符集支持 | CJK支持 |
|-----|------|----------|--------|
| Hack | `Hack-Regular.ttf` | Latin, Greek, Cyrillic | ❌ |
| Ubuntu-Light | `Ubuntu-Light.ttf` | Latin, Greek | ❌ |
| NotoEmoji-Regular | `NotoEmoji-Regular.ttf` | Emoji | ❌ |
| emoji-icon-font | `emoji-icon-font.ttf` | Icons | ❌ |

**结论：所有默认字体都不包含中日韩统一表意文字区块**

### 2. Unicode 范围对比

| 字符集 | Unicode范围 | 默认字体支持 |
|--------|------------|------------|
| ASCII | U+0000 - U+007F | ✅ |
| Latin-1 | U+0080 - U+00FF | ✅ |
| CJK统一汉字 | U+4E00 - U+9FFF | ❌ |
| CJK扩展A | U+3400 - U+4DBF | ❌ |
| CJK扩展B-F | U+20000 - U+2EBEF | ❌ |

---

## 源码分析

### 3.1 字体定义结构 (`fonts.rs:288-301`)

```rust
pub struct FontDefinitions {
    /// 字体数据映射：名称 -> 字体数据
    pub font_data: BTreeMap<String, Arc<FontData>>,

    /// 字体家族映射：字体家族 -> 字体名称列表（按优先级）
    pub families: BTreeMap<FontFamily, Vec<String>>,
}
```

### 3.2 默认字体配置 (`fonts.rs:357-414`)

```rust
impl Default for FontDefinitions {
    fn default() -> Self {
        let mut font_data: BTreeMap<String, Arc<FontData>> = BTreeMap::new();
        let mut families = BTreeMap::new();

        // 加载默认字体
        font_data.insert("Hack".to_owned(), Arc::new(FontData::from_static(HACK_REGULAR)));
        font_data.insert("Ubuntu-Light".to_owned(), Arc::new(FontData::from_static(UBUNTU_LIGHT)));
        font_data.insert("NotoEmoji-Regular".to_owned(), Arc::new(FontData::from_static(NOTO_EMOJI_REGULAR)));
        font_data.insert("emoji-icon-font".to_owned(), Arc::new(FontData::from_static(EMOJI_ICON)));

        // 配置 Proportional 字体家族
        families.insert(
            FontFamily::Proportional,
            vec![
                "Ubuntu-Light".to_owned(),      // 优先级 1（最高）
                "NotoEmoji-Regular".to_owned(),  // 优先级 2
                "emoji-icon-font".to_owned(),    // 优先级 3（最低）
            ],
        );

        // ... Monospace 配置类似
    }
}
```

### 3.3 字符查找流程 (`fonts.rs:516-529`)

```rust
pub(crate) fn glyph_info_no_cache_or_fallback(
    &mut self,
    c: char,                                    // 要查找的字符
    fonts_by_id: &mut nohash_hasher::IntMap<FontFaceKey, FontFace>,
) -> Option<(FontFaceKey, GlyphInfo)> {
    // 按优先级顺序遍历字体列表
    for font_key in &self.fonts {
        let font_face = fonts_by_id.get_mut(font_key).expect("Nonexistent font ID");

        // 检查当前字体是否包含该字符的字形
        if let Some(glyph_info) = font_face.glyph_info(c) {
            // 找到了！缓存并返回
            self.glyph_info_cache.insert(c, (*font_key, glyph_info));
            return Some((*font_key, glyph_info));
        }
    }

    // 所有字体都没有该字符
    None
}
```

### 3.4 替换字符 (`fonts.rs:498-514`)

```rust
impl CachedFamily {
    fn new(
        fonts: Vec<FontFaceKey>,
        fonts_by_id: &mut nohash_hasher::IntMap<FontFaceKey, FontFace>,
    ) -> Self {
        const PRIMARY_REPLACEMENT_CHAR: char = '◻';  // ⬅️ 这就是你看到的方块！
        const FALLBACK_REPLACEMENT_CHAR: char = '?';

        let replacement_glyph = slf
            .glyph_info_no_cache_or_fallback(PRIMARY_REPLACEMENT_CHAR, fonts_by_id)
            .or_else(|| slf.glyph_info_no_cache_or_fallback(FALLBACK_REPLACEMENT_CHAR, fonts_by_id))
            .unwrap_or_else(|| {
                log::warn!(
                    "Failed to find replacement characters {:?} or {:?}. Will use empty glyph.",
                    PRIMARY_REPLACEMENT_CHAR, FALLBACK_REPLACEMENT_CHAR
                );
                (FontFaceKey::INVALID, GlyphInfo::INVISIBLE)
            });

        slf.replacement_glyph = replacement_glyph;
        slf
    }
}
```

---

## 解决方案

### 4.1 完整代码示例

```rust
use eframe::egui;

fn setup_chinese_fonts(ctx: &egui::Context) {
    // 步骤 1: 获取默认字体定义
    let mut fonts = egui::FontDefinitions::default();

    // 步骤 2: 加载中文字体文件
    fonts.font_data.insert(
        "NotoSansCJK".to_owned(),
        egui::FontData::from_static(include_bytes!("fonts/NotoSansCJK-Regular.ttc")),
    );

    // 步骤 3: 将中文字体添加到 Proportional 字体家族的最高优先级
    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "NotoSansCJK".to_owned());
    }

    // 步骤 4: （可选）为 Monospace 也添加中文支持
    if let Some(monospace) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        monospace.push("NotoSansCJK".to_owned());
    }

    // 步骤 5: 应用字体配置
    ctx.set_fonts(fonts);
}
```

### 4.2 字体加载流程图

```
FontDefinitions::default()
         ↓
    加载默认字体
         ↓
┌────────────────────────────────┐
│  font_data: {                  │
│    "Hack": <bytes>,            │
│    "Ubuntu-Light": <bytes>,    │
│    "NotoEmoji": <bytes>,       │
│  }                             │
│  families: {                   │
│    Proportional: [             │
│      "Ubuntu-Light",    ← 1    │
│      "NotoEmoji",       ← 2    │
│    ]                           │
│  }                             │
└────────────────────────────────┘
         ↓
    insert 中文字体
         ↓
┌────────────────────────────────┐
│  font_data: {                  │
│    ...原有字体...,             │
│    "NotoSansCJK": <bytes>, ← 新增 │
│  }                             │
│  families: {                   │
│    Proportional: [             │
│      "NotoSansCJK",    ← 1 (新增) │
│      "Ubuntu-Light",    ← 2    │
│      "NotoEmoji",       ← 3    │
│    ]                           │
│  }                             │
└────────────────────────────────┘
         ↓
    ctx.set_fonts(fonts)
         ↓
    应用到渲染系统
```

---

## 实现细节

### 5.1 FontData 结构

```rust
pub struct FontData {
    /// TTF/OTF 文件内容
    pub font: Cow<'static, [u8]>,

    /// 字体索引（TTC 文件可能包含多个字体）
    pub index: u32,

    /// 字体微调参数
    pub tweak: FontTweak,

    /// 字重 (100-900)，如果可用
    pub weight: Option<u16>,
}
```

### 5.2 从文件加载字体

```rust
// 方法 1: 编译时嵌入（推荐用于示例程序）
egui::FontData::from_static(include_bytes!("fonts/NotoSansCJK-Regular.ttc"))

// 方法 2: 运行时加载（推荐用于应用程序）
let font_bytes = std::fs::read("fonts/NotoSansCJK-Regular.ttc")?;
egui::FontData::from_owned(font_bytes)

// 方法 3: 使用 Arc 避免复制
let font_bytes = std::fs::read("fonts/NotoSansCJK-Regular.ttc")?;
egui::FontData {
    font: std::borrow::Cow::Owned(font_bytes),
    index: 0,
    tweak: Default::default(),
    weight: None,
}
```

### 5.3 TTC 文件处理

TTC (TrueType Collection) 文件包含多个字体：

```rust
// NotoSansCJK.ttc 包含：
// - index 0: NotoSansCJK-Regular
// - index 1: NotoSansCJK-Bold
// - ...

// 指定使用哪个字体
egui::FontData {
    font: Cow::Borrowed(include_bytes!("NotoSansCJK.ttc")),
    index: 0,  // 使用 Regular 字体
    ..Default::default()
}
```

---

## 最佳实践

### 6.1 字体选择建议

| 使用场景 | 推荐字体 | 大小 | 优点 |
|---------|---------|------|------|
| 生产环境 | Noto Sans CJK | ~100MB | 完整CJK支持，OFL授权 |
| 开发/测试 | 文泉驿微米黑 | ~4MB | 体积小，基本够用 |
| 设计/美观 | 站酷系列 | ~5MB | 美观，OFL授权 |
| 极端轻量 | 只嵌入常用字 | 自定义 | 最小化体积 |

### 6.2 性能优化

```rust
// ❌ 不推荐：每次都重新加载
fn update(&mut self, ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    // ...
    ctx.set_fonts(fonts);  // 每帧都重设字体！
}

// ✅ 推荐：只设置一次
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "App",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);  // 只在启动时设置一次
            Ok(Box::new(MyApp::default()))
        }),
    )
}
```

### 6.3 内存管理

```rust
// 对于大型字体文件，使用 Arc 共享
use std::sync::Arc;

let font_data = Arc::new(egui::FontData::from_static(include_bytes!("LargeFont.ttf")));
fonts.font_data.insert("LargeFont".to_owned(), font_data);
```

### 6.4 多字体配置示例

```rust
fn setup_multilingual_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 中文
    fonts.font_data.insert(
        "NotoSansCJK".to_owned(),
        egui::FontData::from_static(include_bytes!("fonts/NotoSansCJK.ttc")),
    );

    // 日文
    fonts.font_data.insert(
        "NotoSansJP".to_owned(),
        egui::FontData::from_static(include_bytes!("fonts/NotoSansJP.ttf")),
    );

    // 韩文
    fonts.font_data.insert(
        "NotoSansKR".to_owned(),
        egui::FontData::from_static(include_bytes!("fonts/NotoSansKR.ttf")),
    );

    // 配置优先级
    if let Some(proportional) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        proportional.insert(0, "NotoSansCJK".to_owned());
        proportional.insert(1, "NotoSansJP".to_owned());
        proportional.insert(2, "NotoSansKR".to_owned());
    }

    ctx.set_fonts(fonts);
}
```

---

## 参考资料

### 官方文档
- [egui::FontDefinitions](https://docs.rs/egui/latest/egui/struct.FontDefinitions.html)
- [egui::FontData](https://docs.rs/egui/latest/egui/struct.FontData.html)
- [epaint::text::fonts](https://docs.rs/epaint/latest/epaint/text/fonts/index.html)

### 字体资源
- [Noto Sans CJK (Google)](https://github.com/googlefonts/noto-cjk)
- [文泉驿](https://wenq.org/)
- [站酷字体](https://zcool.com.cn/fonts)

### Unicode 标准
- [CJK Unified Ideographs](https://en.wikipedia.org/wiki/CJK_Unified_Ideographs)
- [中文编码](https://zh.wikipedia.org/wiki/汉字内码)

---

## 总结

中文显示为方块的问题根源在于 **egui 默认字体不包含 CJK 字符集**。通过 `FontDefinitions` 加载中文字体并设置正确的优先级，可以完美解决这个问题。

**关键要点：**
1. 使用 `FontData::from_static()` 或 `FontData::from_owned()` 加载字体
2. 通过 `insert(0, ...)` 将中文字体设为最高优先级
3. 在应用启动时设置一次即可，无需每帧更新
4. 根据需求选择合适的字体文件（大小vs完整性）

---

*文档生成时间: 2026-01-29*
*egui 版本: 0.33.3*
