use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("ClipHub"),
        ..Default::default()
    };

    eframe::run_native(
        "ClipHub",
        options,
        Box::new(|_cc| Ok(Box::<ClipHubApp>::default())),
    )
}

struct ClipHubApp;

impl Default for ClipHubApp {
    fn default() -> Self {
        Self
    }
}

impl eframe::App for ClipHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("📋 ClipHub");
            ui.label("Clipboard history manager - coming soon!");
        });
    }
}
