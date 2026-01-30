# 中文字体配置 - 快速参考卡

## 🚀 30秒快速配置

```rust
fn main() -> eframe::Result<()> {
    eframe::run_native("我的应用", options, Box::new(|cc| {
        // 加载中文字体
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "中文".to_owned(),
            egui::FontData::from_static(include_bytes!("NotoSansCJK.ttc")),
        );
        fonts.families.get_mut(&egui::FontFamily::Proportional)
            .unwrap().insert(0, "中文".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        Ok(Box::new(MyApp::default()))
    }))
}
```

---

## 📊 字体查找流程

```
字符 '中'
    ↓
┌─────────────────────────────┐
│ Proportional 字体列表:      │
│ 1. NotoSansCJK     ✅ 找到! │
│ 2. Ubuntu-Light             │
│ 3. NotoEmoji                │
└─────────────────────────────┘
    ↓
显示: 中 ✅
```

---

## ⚠️ 常见问题

### Q: 为什么显示方块 ◻？
**A:** 没有加载中文字体，系统找不到字符字形

### Q: 如何知道字体是否正确加载？
**A:** 运行示例程序，检查中文是否正常显示

### Q: 字体文件太大怎么办？
**A:** 使用文泉驿微米黑 (~4MB) 或子集化字体

### Q: 可以使用系统字体吗？
**A:** 可以，但需要通过 `FontData::from_owned()` 从系统路径加载

---

## 📦 推荐字体

| 字体 | 大小 | 下载 |
|-----|------|-----|
| Noto Sans CJK | ~100MB | [GitHub](https://github.com/googlefonts/noto-cjk/releases) |
| 文泉驿微米黑 | ~4MB | [GitHub](https://github.com/lidongered753/WQY-MicroHei) |
| 站酷小薇 | ~5MB | [GitHub](https://github.com/FZSS/ZCOOLXiaoWei) |

---

## 📝 完整示例位置

```
egui/examples/chinese_font_support/
├── Cargo.toml
├── main.rs          ← 完整示例代码
├── README.md        ← 使用说明
└── fonts/           ← 放置字体文件
    └── NotoSansCJK.ttc
```

---

## 🔗 相关链接

- 完整技术分析: `dev_logs/chinese_font_analysis/TECHNICAL_ANALYSIS.md`
- egui 官方文档: https://docs.rs/egui
- 字体下载: https://github.com/googlefonts/noto-cjk
