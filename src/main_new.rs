// Main entry point - kept minimal

mod app;
mod bit_viewer;
mod byte_viewer;
mod core;
mod file_io;
mod operations;
mod pattern_locator;
mod session;
mod settings;
mod ui;
mod worksheet;

use app::BitApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("B.I.T. - Bit Information Tool"),
        ..Default::default()
    };

    eframe::run_native(
        "B.I.T.",
        options,
        Box::new(|_cc| Ok(Box::new(BitApp::default()))),
    )
}

// Implement the eframe::App trait for BitApp
// The UI implementation is in the ui module
impl eframe::App for BitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // For now, keep the existing update logic from the old main.rs
        // TODO: This will be migrated to the ui module
        
        // Render top panel
        self.render_top_panel(ctx);
        
        // Central panel with bit viewer
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Bit Viewer");
            ui.label("UI migration in progress - full implementation coming soon");
            
            if let Some(err) = &self.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }
        });
    }
}
