use egui::{Color32, Ui};

/// My Custom Demo - A simple interactive demo
#[derive(PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct MyDemo {
    counter: i32,
    enabled: bool,
    value: f32,
    text: String,
    color: Color32,
}

impl Default for MyDemo {
    fn default() -> Self {
        Self {
            counter: 0,
            enabled: true,
            value: 50.0,
            text: "Hello, egui!".to_string(),
            color: Color32::LIGHT_BLUE,
        }
    }
}

impl crate::Demo for MyDemo {
    fn name(&self) -> &'static str {
        "🎯 My Demo"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .resizable(true)
            .default_width(300.0)
            .show(ctx, |ui| {
                use crate::View as _;
                self.ui(ui);
            });
    }
}

impl crate::View for MyDemo {
    fn ui(&mut self, ui: &mut Ui) {
        ui.heading("My Custom Demo - Updated!");

        ui.separator();

        // Counter section
        ui.horizontal(|ui| {
            ui.label("Counter:");
            ui.label(format!("{}", self.counter));
            if ui.button("+").clicked() {
                self.counter += 1;
            }
            if ui.button("-").clicked() && self.counter > 0 {
                self.counter -= 1;
            }
        });

        ui.separator();

        // Checkbox
        ui.checkbox(&mut self.enabled, "Enable features");

        if self.enabled {
            // Slider
            ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));

            // Color picker
            ui.horizontal(|ui| {
                ui.label("Color:");
                egui::color_picker::color_edit_button_srgba(ui, &mut self.color, egui::color_picker::Alpha::OnlyBlend);
            });

            // Text input
            ui.horizontal(|ui| {
                ui.label("Text:");
                ui.text_edit_singleline(&mut self.text);
            });

            // Display text with color
            ui.horizontal(|ui| {
                ui.label("Preview:");
                ui.colored_label(self.color, &self.text);
            });

            // Progress bar
            ui.add_space(10.0);
            let progress = self.value / 100.0;
            ui.add(egui::ProgressBar::new(progress).show_percentage().animate(true));
        }

        ui.separator();

        // Reset button and GitHub link
        ui.vertical_centered(|ui| {
            egui::reset_button(ui, self, "Reset");
            ui.add_space(5.0);
            ui.add(crate::egui_github_link_file!());
        });
    }
}
