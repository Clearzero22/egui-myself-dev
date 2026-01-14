use egui_demo_lib::{DemoWindows, is_mobile};

#[cfg(feature = "glow")]
use eframe::glow;

#[cfg(target_arch = "wasm32")]
use core::any::Any;

use crate::DemoApp;

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
struct EasyMarkApp {
    editor: egui_demo_lib::easy_mark::EasyMarkEditor,
}

impl DemoApp for EasyMarkApp {
    fn demo_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.editor.panels(ui);
    }
}

// ----------------------------------------------------------------------------

impl DemoApp for DemoWindows {
    fn demo_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.ui(ui);
    }
}

// ----------------------------------------------------------------------------

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct FractalClockApp {
    fractal_clock: crate::apps::FractalClock,
    pub mock_time: Option<f64>,
}

impl DemoApp for FractalClockApp {
    fn demo_ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Frame::dark_canvas(ui.style())
            .stroke(egui::Stroke::NONE)
            .corner_radius(0)
            .show(ui, |ui| {
                self.fractal_clock.ui(
                    ui,
                    self.mock_time
                        .or_else(|| Some(crate::seconds_since_midnight())),
                );
            });
    }
}

// ----------------------------------------------------------------------------

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ColorTestApp {
    color_test: egui_demo_lib::ColorTest,
}

impl DemoApp for ColorTestApp {
    fn demo_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if frame.is_web() {
                ui.label(
                        "NOTE: Some old browsers stuck on WebGL1 without sRGB support will not pass the color test.",
                    );
                ui.separator();
            }
            egui::ScrollArea::both().auto_shrink(false).show(ui, |ui| {
                self.color_test.ui(ui);
            });
        });
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum Anchor {
    #[default]
    Demo,

    EasyMarkEditor,

    #[cfg(feature = "http")]
    Http,

    #[cfg(feature = "image_viewer")]
    ImageViewer,

    Clock,

    #[cfg(any(feature = "glow", feature = "wgpu"))]
    Custom3d,

    /// Rendering test
    Rendering,
}

impl Anchor {
    #[cfg(target_arch = "wasm32")]
    fn all() -> Vec<Self> {
        vec![
            Self::Demo,
            Self::EasyMarkEditor,
            #[cfg(feature = "http")]
            Self::Http,
            Self::Clock,
            #[cfg(any(feature = "glow", feature = "wgpu"))]
            Self::Custom3d,
            Self::Rendering,
        ]
    }

    #[cfg(target_arch = "wasm32")]
    fn from_str_case_insensitive(anchor: &str) -> Option<Self> {
        let anchor = anchor.to_lowercase();
        Self::all().into_iter().find(|x| x.to_string() == anchor)
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut name = format!("{self:?}");
        name.make_ascii_lowercase();
        f.write_str(&name)
    }
}

impl From<Anchor> for egui::WidgetText {
    fn from(value: Anchor) -> Self {
        Self::from(value.to_string())
    }
}

// ----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
#[must_use]
enum Command {
    Nothing,
    ResetEverything,
}

// ----------------------------------------------------------------------------

/// The state that we persist (serialize).
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    demo: DemoWindows,
    easy_mark_editor: EasyMarkApp,
    #[cfg(feature = "http")]
    http: crate::apps::HttpApp,
    #[cfg(feature = "image_viewer")]
    image_viewer: crate::apps::ImageViewer,
    pub clock: FractalClockApp,
    rendering_test: ColorTestApp,

    selected_anchor: Anchor,
    backend_panel: super::backend_panel::BackendPanel,
}

/// Wraps many demo/test apps into one.
pub struct WrapApp {
    pub state: State,

    #[cfg(any(feature = "glow", feature = "wgpu"))]
    custom3d: Option<crate::apps::Custom3d>,

    dropped_files: Vec<egui::DroppedFile>,
}

impl WrapApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This gives us image support:
        egui_extras::install_image_loaders(&cc.egui_ctx);

        #[cfg(feature = "accessibility_inspector")]
        cc.egui_ctx
            .add_plugin(crate::accessibility_inspector::AccessibilityInspectorPlugin::default());

        #[allow(unused_mut, clippy::allow_attributes)]
        let mut slf = Self {
            state: State::default(),

            #[cfg(any(feature = "glow", feature = "wgpu"))]
            custom3d: crate::apps::Custom3d::new(cc),

            dropped_files: Default::default(),
        };

        #[cfg(feature = "persistence")]
        if let Some(storage) = cc.storage
            && let Some(state) = eframe::get_value(storage, eframe::APP_KEY)
        {
            slf.state = state;
        }

        slf
    }

    pub fn apps_iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&'static str, Anchor, &mut dyn DemoApp)> {
        let mut vec = vec![
            (
                "✨ Demos",
                Anchor::Demo,
                &mut self.state.demo as &mut dyn DemoApp,
            ),
            (
                "🖹 EasyMark editor",
                Anchor::EasyMarkEditor,
                &mut self.state.easy_mark_editor as &mut dyn DemoApp,
            ),
            #[cfg(feature = "http")]
            (
                "⬇ HTTP",
                Anchor::Http,
                &mut self.state.http as &mut dyn DemoApp,
            ),
            (
                "🕑 Fractal Clock",
                Anchor::Clock,
                &mut self.state.clock as &mut dyn DemoApp,
            ),
            #[cfg(feature = "image_viewer")]
            (
                "🖼 Image Viewer",
                Anchor::ImageViewer,
                &mut self.state.image_viewer as &mut dyn DemoApp,
            ),
        ];

        #[cfg(any(feature = "glow", feature = "wgpu"))]
        if let Some(custom3d) = &mut self.custom3d {
            vec.push((
                "🔺 3D painting",
                Anchor::Custom3d,
                custom3d as &mut dyn DemoApp,
            ));
        }

        vec.push((
            "🎨 Rendering test",
            Anchor::Rendering,
            &mut self.state.rendering_test as &mut dyn DemoApp,
        ));

        vec.into_iter()
    }
}

impl eframe::App for WrapApp {
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.state);
    }

    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        // 使用透明背景以支持无边框窗口的圆角效果
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        if let Some(anchor) = frame
            .info()
            .web_info
            .location
            .hash
            .strip_prefix('#')
            .and_then(Anchor::from_str_case_insensitive)
        {
            self.state.selected_anchor = anchor;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }

        let mut cmd = Command::Nothing;

        // 自定义窗口框架 - 暖黄色主题
        let panel_frame = egui::Frame::new()
            .fill(egui::Color32::from_rgb(249, 243, 224))  // 主背景色 #F9F3E0
            .corner_radius(12)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 213, 184)))  // 边框色 #E0D5B8
            .inner_margin(4)
            .outer_margin(2);

        egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            // 顶部栏 - 支持拖拽窗口，使用稍深的暖黄色
            let title_bar_frame = egui::Frame::new()
                .fill(egui::Color32::from_rgb(245, 233, 200))  // 标签栏背景色 #F5E9C8
                .inner_margin(egui::Margin::symmetric(8, 8));

            let _title_bar_inner = title_bar_frame.show(ui, |ui| {
                // 先绘制内容，让按钮先注册它们的交互
                ui.horizontal(|ui| {
                    ui.visuals_mut().button_frame = false;
                    self.bar_contents(ui, frame, &mut cmd);
                });

                // 获取标题栏区域的矩形
                let title_bar_rect = ui.min_rect();

                // 检测鼠标按下位置和当前位置来判断拖拽
                let (press_origin, current_pos) = ui.input(|i| {
                    (i.pointer.press_origin(), i.pointer.interact_pos())
                });

                if let (Some(origin), Some(current)) = (press_origin, current_pos) {
                    // 如果按下位置在标题栏区域内
                    if title_bar_rect.contains(origin) {
                        // 计算移动距离
                        let delta = (current - origin).length();

                        // 如果移动距离超过阈值，开始拖拽窗口
                        if delta > 3.0 && ui.input(|i| i.pointer.primary_down()) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                        }
                    }
                }

                // 检测双击最大化/还原
                let double_click = ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary));
                if double_click {
                    let pointer_pos = ui.input(|i| i.pointer.interact_pos());
                    if let Some(pos) = pointer_pos {
                        if title_bar_rect.contains(pos) {
                            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
                        }
                    }
                }
            });

            // 添加分隔线，使用边框色
            ui.add_space(4.0);
            let separator_rect = ui.available_rect_before_wrap();
            let painter = ui.painter();
            painter.line_segment(
                [separator_rect.left_top(), separator_rect.right_top()],
                egui::Stroke::new(1.0, egui::Color32::from_rgb(224, 213, 184)),  // 边框色 #E0D5B8
            );
            ui.add_space(4.0);

            self.state.backend_panel.update(ctx, frame);

            // 内容区域
            if !is_mobile(ctx) {
                ui.horizontal(|ui| {
                    if self.state.backend_panel.open || ui.memory(|mem| mem.everything_is_visible()) {
                        // 左侧后端面板区域
                        ui.vertical(|ui| {
                            ui.add_space(4.0);
                            ui.vertical_centered(|ui| {
                                ui.heading("💻 Backend");
                            });
                            ui.separator();
                            self.backend_panel_contents(ui, frame, &mut cmd);
                        });
                        ui.separator();
                    }

                    // 主内容区域
                    ui.vertical(|ui| {
                        self.show_selected_app(ui, frame);
                    });
                });
            } else {
                self.show_selected_app(ui, frame);
            }
        });

        self.state.backend_panel.end_of_frame(ctx);

        self.ui_file_drag_and_drop(ctx);

        self.run_cmd(ctx, cmd);
    }

    #[cfg(feature = "glow")]
    fn on_exit(&mut self, gl: Option<&glow::Context>) {
        if let Some(custom3d) = &mut self.custom3d {
            custom3d.on_exit(gl);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(&mut *self)
    }
}

impl WrapApp {
    fn run_cmd(&mut self, ctx: &egui::Context, cmd: Command) {
        match cmd {
            Command::Nothing => {}
            Command::ResetEverything => {
                self.state = Default::default();
                ctx.memory_mut(|mem| *mem = Default::default());
            }
        }
    }

    fn backend_panel_contents(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
        cmd: &mut Command,
    ) {
        self.state.backend_panel.ui(ui, frame);

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button("Reset egui")
                .on_hover_text("Forget scroll, positions, sizes etc")
                .clicked()
            {
                ui.ctx().memory_mut(|mem| *mem = Default::default());
                ui.close();
            }

            if ui.button("Reset everything").clicked() {
                *cmd = Command::ResetEverything;
                ui.close();
            }
        });
    }

    fn show_selected_app(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let selected_anchor = self.state.selected_anchor;
        for (_name, anchor, app) in self.apps_iter_mut() {
            if anchor == selected_anchor || ui.memory(|mem| mem.everything_is_visible()) {
                app.demo_ui(ui, frame);
            }
        }
    }

    fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cmd: &mut Command) {
        // 窗口控制按钮 (关闭、最小化、最大化)
        self.window_controls(ui);

        ui.separator();

        egui::widgets::global_theme_preference_switch(ui);

        ui.separator();

        if is_mobile(ui.ctx()) {
            ui.menu_button("💻 Backend", |ui| {
                ui.set_style(ui.ctx().style()); // ignore the "menu" style set by `menu_button`.
                self.backend_panel_contents(ui, frame, cmd);
            });
        } else {
            ui.toggle_value(&mut self.state.backend_panel.open, "💻 Backend");
        }

        ui.separator();

        let mut selected_anchor = self.state.selected_anchor;
        for (name, anchor, _app) in self.apps_iter_mut() {
            if ui
                .selectable_label(selected_anchor == anchor, name)
                .clicked()
            {
                selected_anchor = anchor;
                if frame.is_web() {
                    ui.ctx()
                        .open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
                }
            }
        }
        self.state.selected_anchor = selected_anchor;

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if false {
                // TODO(emilk): fix the overlap on small screens
                if clock_button(ui, crate::seconds_since_midnight()).clicked() {
                    self.state.selected_anchor = Anchor::Clock;
                    if frame.is_web() {
                        ui.ctx().open_url(egui::OpenUrl::same_tab("#clock"));
                    }
                }
            }

            egui::warn_if_debug_build(ui);
        });
    }

    /// 窗口控制按钮：关闭、最小化、最大化
    fn window_controls(&self, ui: &mut egui::Ui) {
        use egui::{Button, RichText};

        ui.spacing_mut().item_spacing.x = 0.0;
        ui.visuals_mut().button_frame = false;

        let button_size = egui::Vec2::new(32.0, 32.0);

        // 关闭按钮
        let close_response = ui
            .add_sized(
                button_size,
                Button::new(RichText::new("❌").size(14.0))
            )
            .on_hover_text("关闭窗口");
        if close_response.clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // 最小化按钮
        let minimize_response = ui
            .add_sized(
                button_size,
                Button::new(RichText::new("🗕").size(14.0))
            )
            .on_hover_text("最小化");
        if minimize_response.clicked() {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }

        // 最大化/还原按钮
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if is_maximized {
            let restore_response = ui
                .add_sized(
                    button_size,
                    Button::new(RichText::new("🗗").size(14.0))
                )
                .on_hover_text("还原");
            if restore_response.clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(false));
            }
        } else {
            let maximize_response = ui
                .add_sized(
                    button_size,
                    Button::new(RichText::new("🗗").size(14.0))
                )
                .on_hover_text("最大化");
            if maximize_response.clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
        }
    }

    fn ui_file_drag_and_drop(&mut self, ctx: &egui::Context) {
        use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};
        use std::fmt::Write as _;

        // Preview hovering files:
        if !ctx.input(|i| i.raw.hovered_files.is_empty()) {
            let text = ctx.input(|i| {
                let mut text = "Dropping files:\n".to_owned();
                for file in &i.raw.hovered_files {
                    if let Some(path) = &file.path {
                        write!(text, "\n{}", path.display()).ok();
                    } else if !file.mime.is_empty() {
                        write!(text, "\n{}", file.mime).ok();
                    } else {
                        text += "\n???";
                    }
                }
                text
            });

            let painter =
                ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

            let content_rect = ctx.content_rect();
            painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
            painter.text(
                content_rect.center(),
                Align2::CENTER_CENTER,
                text,
                TextStyle::Heading.resolve(&ctx.style()),
                Color32::WHITE,
            );
        }

        // Collect dropped files:
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                self.dropped_files.clone_from(&i.raw.dropped_files);
            }
        });

        // Show dropped files (if any):
        if !self.dropped_files.is_empty() {
            let mut open = true;
            egui::Window::new("Dropped files")
                .open(&mut open)
                .show(ctx, |ui| {
                    for file in &self.dropped_files {
                        let mut info = if let Some(path) = &file.path {
                            path.display().to_string()
                        } else if !file.name.is_empty() {
                            file.name.clone()
                        } else {
                            "???".to_owned()
                        };

                        let mut additional_info = vec![];
                        if !file.mime.is_empty() {
                            additional_info.push(format!("type: {}", file.mime));
                        }
                        if let Some(bytes) = &file.bytes {
                            additional_info.push(format!("{} bytes", bytes.len()));
                        }
                        if !additional_info.is_empty() {
                            info += &format!(" ({})", additional_info.join(", "));
                        }

                        ui.label(info);
                    }
                });
            if !open {
                self.dropped_files.clear();
            }
        }
    }
}

fn clock_button(ui: &mut egui::Ui, seconds_since_midnight: f64) -> egui::Response {
    let time = seconds_since_midnight;
    let time = format!(
        "{:02}:{:02}:{:02}.{:02}",
        (time % (24.0 * 60.0 * 60.0) / 3600.0).floor(),
        (time % (60.0 * 60.0) / 60.0).floor(),
        (time % 60.0).floor(),
        (time % 1.0 * 100.0).floor()
    );

    ui.button(egui::RichText::new(time).monospace())
}
