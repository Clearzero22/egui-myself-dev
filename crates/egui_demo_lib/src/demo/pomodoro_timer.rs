use egui::{CollapsingHeader, Color32, Context, Ui, RichText, vec2};

/// Pomodoro Timer - 番茄钟倒计时
#[derive(PartialEq, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TimerMode {
    #[default]
    Work,
    Break,
}

impl TimerMode {
    pub fn emoji(&self) -> &str {
        match self {
            TimerMode::Work => "🍅",
            TimerMode::Break => "☕",
        }
    }

    pub fn title(&self) -> &str {
        match self {
            TimerMode::Work => "工作时间",
            TimerMode::Break => "休息时间",
        }
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub enum TimerState {
    #[default]
    Idle,
    Running,
    Paused,
}

/// Pomodoro Timer state
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct PomodoroTimer {
    mode: TimerMode,
    state: TimerState,
    duration: f32,           // 当前模式总时长（秒）
    remaining: f32,          // 剩余时间（秒）
    start_time: Option<f64>,  // 开始时间戳
    work_duration: f32,      // 工作时长（分钟）
    break_duration: f32,      // 休息时长（分钟）
    completed_count: u32,     // 完成的番茄数
    show_settings: bool,      // 是否显示设置面板
}

impl Default for PomodoroTimer {
    fn default() -> Self {
        Self {
            mode: TimerMode::Work,
            state: TimerState::Idle,
            duration: 25.0 * 60.0,
            remaining: 25.0 * 60.0,
            start_time: None,
            work_duration: 25.0,
            break_duration: 5.0,
            completed_count: 0,
            show_settings: false,
        }
    }
}

impl PomodoroTimer {
    /// 获取当前时间（秒）
    fn get_current_time(&self, ctx: &Context) -> f64 {
        ctx.input(|i| i.time)
    }

    /// 更新剩余时间
    fn update_remaining(&mut self, ctx: &Context) {
        if let Some(start) = self.start_time {
            if self.state == TimerState::Running {
                let current = self.get_current_time(ctx);
                let elapsed = (current - start) as f32;
                self.remaining = (self.duration - elapsed).max(0.0);

                // 时间到
                if self.remaining <= 0.0 {
                    self.complete_timer();
                }
            }
        }
    }

    /// 计时器完成
    fn complete_timer(&mut self) {
        match self.mode {
            TimerMode::Work => {
                self.completed_count += 1;
                self.switch_to_break();
            }
            TimerMode::Break => {
                self.switch_to_work();
            }
        }
    }

    /// 切换到工作时间
    fn switch_to_work(&mut self) {
        self.mode = TimerMode::Work;
        self.duration = self.work_duration * 60.0;
        self.remaining = self.duration;
        self.start_time = None;
        self.state = TimerState::Idle;
    }

    /// 切换到休息时间
    fn switch_to_break(&mut self) {
        self.mode = TimerMode::Break;
        self.duration = self.break_duration * 60.0;
        self.remaining = self.duration;
        self.start_time = None;
        self.state = TimerState::Idle;
    }

    /// 开始计时
    fn start(&mut self, ctx: &Context) {
        self.state = TimerState::Running;
        let current = self.get_current_time(ctx);
        self.start_time = Some(current - (self.duration - self.remaining) as f64);
    }

    /// 暂停计时
    fn pause(&mut self) {
        self.state = TimerState::Paused;
        self.start_time = None;
    }

    /// 重置计时器
    fn reset(&mut self) {
        self.state = TimerState::Idle;
        self.remaining = self.duration;
        self.start_time = None;
    }

    /// 跳过当前阶段
    fn skip(&mut self) {
        match self.mode {
            TimerMode::Work => self.switch_to_break(),
            TimerMode::Break => self.switch_to_work(),
        }
    }

    /// 格式化时间为 MM:SS
    fn format_time(seconds: f32) -> String {
        let mins = (seconds / 60.0) as u32;
        let secs = (seconds % 60.0) as u32;
        format!("{:02}:{:02}", mins, secs)
    }
}

impl crate::Demo for PomodoroTimer {
    fn name(&self) -> &'static str {
        "🍅 番茄钟"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(400.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

impl crate::View for PomodoroTimer {
    fn ui(&mut self, ui: &mut Ui) {
        // 更新时间
        self.update_remaining(ui.ctx());

        // 请求重绘（如果正在运行）
        if self.state == TimerState::Running {
            ui.ctx().request_repaint();
        }

        ui.vertical_centered(|ui| {
            // 标题
            ui.heading(format!("{} {}", self.mode.emoji(), self.mode.title()));

            ui.add_space(20.0);

            // 大字体时间显示
            let time_text = Self::format_time(self.remaining);
            ui.label(RichText::new(time_text).size(80.0).strong());

            ui.add_space(10.0);

            // 进度条
            let progress = self.remaining / self.duration;
            let progress_color = match self.mode {
                TimerMode::Work => Color32::from_rgb(255, 100, 100),  // 红色
                TimerMode::Break => Color32::from_rgb(100, 200, 100), // 绿色
            };
            ui.add(
                egui::ProgressBar::new(progress)
                    .show_percentage()
                    .fill(progress_color)
                    .animate(self.state == TimerState::Running)
                    .desired_width(300.0),
            );

            ui.add_space(20.0);

            // 控制按钮
            ui.horizontal(|ui| {
                ui.set_width(300.0);
                ui.spacing_mut().item_spacing = vec2(10.0, 10.0);

                if self.state == TimerState::Running {
                    if ui.button(egui::RichText::new("⏸ 暂停").size(18.0)).clicked() {
                        self.pause();
                    }
                } else {
                    let button_text = if self.state == TimerState::Idle {
                        "▶ 开始"
                    } else {
                        "▶ 继续"
                    };
                    if ui.button(egui::RichText::new(button_text).size(18.0)).clicked() {
                        self.start(ui.ctx());
                    }
                }

                if ui.button(egui::RichText::new("↺ 重置").size(18.0)).clicked() {
                    self.reset();
                }

                if ui.button(egui::RichText::new("⏭ 跳过").size(18.0)).clicked() {
                    self.skip();
                }
            });

            ui.add_space(20.0);

            // 设置面板
            CollapsingHeader::new("⚙️ 设置")
                .default_open(false)
                .show(ui, |ui| {
                    ui.add_space(10.0);

                    ui.horizontal(|ui| {
                        ui.label("工作时长:");
                        ui.add(egui::DragValue::new(&mut self.work_duration)
                            .speed(0.1)
                            .range(1.0..=60.0));
                        ui.label("分钟");
                    });

                    ui.horizontal(|ui| {
                        ui.label("休息时长:");
                        ui.add(egui::DragValue::new(&mut self.break_duration)
                            .speed(0.1)
                            .range(1.0..=30.0));
                        ui.label("分钟");
                    });

                    // 应用设置（如果当前是空闲状态）
                    if self.state == TimerState::Idle {
                        match self.mode {
                            TimerMode::Work => {
                                self.duration = self.work_duration * 60.0;
                                self.remaining = self.duration;
                            }
                            TimerMode::Break => {
                                self.duration = self.break_duration * 60.0;
                                self.remaining = self.duration;
                            }
                        }
                    }
                });

            ui.separator();

            // 统计信息
            ui.horizontal(|ui| {
                ui.label("已完成的番茄数:");
                ui.label(RichText::new(format!("{}", self.completed_count))
                    .size(24.0)
                    .color(Color32::LIGHT_YELLOW)
                    .strong());
            });

            // 番茄图标展示
            ui.add_space(10.0);
            self.show_completed_tomatoes(ui);
        });

        // 底部 GitHub 链接
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);
            ui.add(crate::egui_github_link_file!());
        });
    }
}

impl PomodoroTimer {
    /// 显示已完成的番茄图标
    fn show_completed_tomatoes(&self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.style_mut().spacing.item_spacing = vec2(5.0, 5.0);

            for i in 0..=4 {
                let is_completed = i < self.completed_count;
                let emoji = if is_completed { "🍅" } else { "⬜" };
                ui.label(RichText::new(emoji).size(20.0));
            }

            if self.completed_count > 5 {
                ui.label(RichText::new(format!("+{}", self.completed_count - 5)).size(16.0));
            }
        });
    }
}
