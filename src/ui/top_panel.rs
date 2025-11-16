// Top panel with main navigation and controls

use crate::app::BitApp;
use crate::core::ViewMode;
use eframe::egui;

pub fn render(app: &mut BitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("🔧 B.I.T. - Bit Information Tool");
            
            ui.separator();

            if ui.button("💾 Save File").clicked() {
                app.save_file();
            }

            ui.separator();

            if ui.button("⚙ Settings").clicked() {
                app.show_settings = !app.show_settings;
            }

            if ui.button("🔍 Pattern Locator").clicked() {
                app.show_pattern_locator = !app.show_pattern_locator;
            }

            ui.separator();

            // View mode toggle
            ui.label("View:");
            if ui.selectable_label(app.view_mode == ViewMode::Bit, "⬛ Bit").clicked() {
                app.view_mode = ViewMode::Bit;
            }
            if ui.selectable_label(app.view_mode == ViewMode::Byte, "📊 Byte").clicked() {
                app.view_mode = ViewMode::Byte;
            }
            if ui.selectable_label(app.view_mode == ViewMode::Ascii, "🔤 ASCII").clicked() {
                app.view_mode = ViewMode::Ascii;
            }

            ui.separator();

            ui.label("Zoom:");
            if ui.button("➕").clicked() {
                app.viewer.zoom_in();
            }
            if ui.button("➖").clicked() {
                app.viewer.zoom_out();
            }
            if ui.button("🔄").clicked() {
                app.viewer.reset_zoom();
            }

            ui.separator();

            if ui.selectable_label(app.show_original, "Original").clicked() {
                app.show_original = true;
                if app.view_mode == ViewMode::Bit {
                    app.update_viewer();
                }
            }
            if ui.selectable_label(!app.show_original, "Processed").clicked() {
                app.show_original = false;
                if app.view_mode == ViewMode::Bit {
                    app.update_viewer();
                }
            }
        });
    });
}

