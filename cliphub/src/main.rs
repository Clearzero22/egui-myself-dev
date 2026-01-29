use eframe::egui;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

mod types;
mod storage;
mod clipboard;
mod ui;

use ui::MainWindow;
use storage::Database;
use clipboard::ClipboardMonitor;
use types::ClipboardItem;

struct ClipHubApp {
    main_window: MainWindow,
    receiver: mpsc::Receiver<ClipboardItem>,
}

impl Default for ClipHubApp {
    fn default() -> Self {
        let db = Database::new("cliphub.db").expect("Failed to open database");
        let main_window = MainWindow::new(db);

        let (sender, receiver) = mpsc::channel();

        // Start monitor thread
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let db = Database::new("cliphub.db").expect("Failed to open database");
            let mut monitor = ClipboardMonitor::new(Duration::from_secs(1));

            rt.block_on(async {
                let _ = monitor.run_with_channel(&db, sender).await;
            });
        });

        Self { main_window, receiver }
    }
}

impl eframe::App for ClipHubApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for new clipboard items
        if let Ok(_item) = self.receiver.try_recv() {
            self.main_window.refresh_items();
        }

        // Request repaint for continuous updates
        ctx.request_repaint();
        self.main_window.show(ctx);
    }
}

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
