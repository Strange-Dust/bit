// Top panel with main navigation and controls

use crate::app::BitApp;
use crate::core::ViewMode;
use eframe::egui;

pub fn render(app: &mut BitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.heading("B.I.T. - Bit Information Tool");

            // Show mini progress indicator when loading or processing
            if app.is_loading() {
                ui.add_space(10.0);
                ui.add(
                    egui::ProgressBar::new(app.loading_state.progress)
                        .desired_width(100.0)
                        .show_percentage()
                ).on_hover_text(format!("Loading file... {}", app.loading_state.eta_string()));
            } else if app.is_processing_operations() {
                ui.add_space(10.0);
                ui.add(
                    egui::ProgressBar::new(app.operation_processing_state.progress)
                        .desired_width(100.0)
                        .show_percentage()
                ).on_hover_text(format!("{}\nETA: {}",
                    app.operation_processing_state.progress_message,
                    app.operation_processing_state.eta_string()
                ));
            }

            ui.separator();

            // Recent files dropdown
            ui.menu_button("File", |ui| {
                if ui.button("Save File")
                    .on_hover_text("Save processed file (Ctrl+S)")
                    .clicked()
                {
                    app.save_file();
                    ui.close();
                }

                ui.separator();
                ui.label("Export Data:");

                if ui.button("Export as Hex Dump")
                    .on_hover_text("Export processed data as hexadecimal text")
                    .clicked()
                {
                    app.export_as_hex();
                    ui.close();
                }

                if ui.button("Export as Base64")
                    .on_hover_text("Export processed data as Base64 encoded text")
                    .clicked()
                {
                    app.export_as_base64();
                    ui.close();
                }

                ui.separator();
                ui.label("Recent Files:");

                let recent_count = app.settings.recent_files.list().len();
                if recent_count == 0 {
                    ui.label("(No recent files)");
                } else {
                    let mut to_open = None;
                    let mut to_remove = None;

                    for (idx, recent) in app.settings.recent_files.list().iter().enumerate() {
                        ui.horizontal(|ui| {
                            let file_name = recent.file_name();
                            let file_size = recent.file_size_string();
                            let label = format!("{} ({})", file_name, file_size);

                            if ui.button(&label)
                                .on_hover_text(recent.path.display().to_string())
                                .clicked()
                            {
                                to_open = Some(idx);
                            }

                            if ui.small_button("x")
                                .on_hover_text("Remove from recent files")
                                .clicked()
                            {
                                to_remove = Some(idx);
                            }
                        });
                    }

                    if let Some(idx) = to_open {
                        if let Some(path) = app.settings.recent_files.get_path(idx) {
                            app.load_file_from_path(path);
                            ui.close();
                        }
                    }

                    if let Some(idx) = to_remove {
                        app.settings.recent_files.remove(idx);
                        app.settings.auto_save();
                    }

                    ui.separator();
                    if ui.button("Clear Recent Files")
                        .on_hover_text("Remove all files from recent files list")
                        .clicked()
                    {
                        app.settings.recent_files.clear();
                        app.settings.auto_save();
                        ui.close();
                    }
                }
            });

            ui.separator();

            if ui.button("Settings")
                .on_hover_text("Open settings panel (Ctrl+,)\nConfigure visualization, grid, and notification preferences")
                .clicked()
            {
                app.show_settings = !app.show_settings;
            }

            if ui.button("Pattern Locator")
                .on_hover_text("Open pattern locator (Ctrl+F)\nSearch for bit patterns in your data")
                .clicked()
            {
                app.show_pattern_locator = !app.show_pattern_locator;
            }

            if ui.button("Frame Width Finder")
                .on_hover_text("Open frame width finder\nAnalyze optimal frame width for structured data")
                .clicked()
            {
                app.show_frame_width_finder = !app.show_frame_width_finder;
            }

            ui.separator();

            // View mode toggle
            ui.label("View:");
            if ui.selectable_label(app.view_mode == ViewMode::Bit, "Bit")
                .on_hover_text("View data as individual bits (Press 1)")
                .clicked()
            {
                app.view_mode = ViewMode::Bit;
            }
            if ui.selectable_label(app.view_mode == ViewMode::Byte, "Byte")
                .on_hover_text("View data as bytes in hexadecimal (Press 2)")
                .clicked()
            {
                app.view_mode = ViewMode::Byte;
            }
            if ui.selectable_label(app.view_mode == ViewMode::Ascii, "ASCII")
                .on_hover_text("View data as ASCII characters (Press 3)")
                .clicked()
            {
                app.view_mode = ViewMode::Ascii;
            }

            ui.separator();

            ui.label("Zoom:");
            if ui.button("+")
                .on_hover_text("Zoom in (Ctrl++)")
                .clicked()
            {
                app.viewer.zoom_in();
            }
            if ui.button("-")
                .on_hover_text("Zoom out (Ctrl+-)")
                .clicked()
            {
                app.viewer.zoom_out();
            }
            if ui.button("Reset")
                .on_hover_text("Reset zoom (Ctrl+0)")
                .clicked()
            {
                app.viewer.reset_zoom();
            }

            ui.separator();

            if ui.selectable_label(app.show_original, "Original")
                .on_hover_text("Show original data before operations")
                .clicked()
            {
                app.show_original = true;
                if app.view_mode == ViewMode::Bit {
                    app.update_viewer();
                }
            }
            if ui.selectable_label(!app.show_original, "Processed")
                .on_hover_text("Show processed data after operations")
                .clicked()
            {
                app.show_original = false;
                if app.view_mode == ViewMode::Bit {
                    app.update_viewer();
                }
            }
        });
    });
}

