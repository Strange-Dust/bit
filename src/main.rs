mod analysis;
mod app;
mod app_state;
mod core;
mod operations;
mod storage;
mod ui;
mod utils;
mod viewers;

use crate::app::BitApp;
use crate::core::ViewMode;
use crate::operations::OperationType;
use crate::storage::AppSession;
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

/// Handle keyboard shortcuts for the application
fn handle_keyboard_shortcuts(app: &mut BitApp, ctx: &egui::Context) {
    // Only handle shortcuts when no text input is focused
    let text_edit_focused = ctx.memory(|mem| mem.focused().is_some());

    ctx.input(|i| {
        let ctrl = i.modifiers.ctrl || i.modifiers.command; // Command on macOS, Ctrl elsewhere
        let shift = i.modifiers.shift;

        // Ctrl+O - Open file
        if ctrl && !shift && i.key_pressed(egui::Key::O) {
            if let Some(path) = rfd::FileDialog::new().pick_file() {
                app.start_loading_file(path);
            }
        }

        // Ctrl+S - Save file
        if ctrl && !shift && i.key_pressed(egui::Key::S) {
            app.save_file();
        }

        // Ctrl+W - Close a popup window
        if ctrl && !shift && i.key_pressed(egui::Key::W) {
            // Close any open window in priority order
            if app.show_settings {
                app.show_settings = false;
            } else if app.show_pattern_locator {
                app.show_pattern_locator = false;
            } else if app.show_frame_width_finder {
                app.show_frame_width_finder = false;
            } else if app.show_operation_menu.is_some() {
                app.cancel_operation_edit();
            } else if app.show_column_editor {
                app.show_column_editor = false;
            }
        }

        // Ctrl+T - New worksheet
        if ctrl && !shift && i.key_pressed(egui::Key::T) {
            let new_name = format!("Worksheet {}", app.worksheets.len() + 1);
            app.sync_to_worksheet();
            app.worksheets.push(crate::storage::Worksheet::new(new_name));
            app.current_worksheet_index = app.worksheets.len() - 1;
            app.load_from_worksheet();
        }

        // Ctrl+Tab - Next worksheet
        if ctrl && !shift && i.key_pressed(egui::Key::Tab) {
            if !app.worksheets.is_empty() {
                let next_index = (app.current_worksheet_index + 1) % app.worksheets.len();
                app.switch_worksheet(next_index);
            }
        }

        // Ctrl+Shift+Tab - Previous worksheet
        if ctrl && shift && i.key_pressed(egui::Key::Tab) {
            if !app.worksheets.is_empty() {
                let prev_index = if app.current_worksheet_index == 0 {
                    app.worksheets.len() - 1
                } else {
                    app.current_worksheet_index - 1
                };
                app.switch_worksheet(prev_index);
            }
        }

        // Ctrl+F - Open Pattern Locator
        if ctrl && !shift && i.key_pressed(egui::Key::F) {
            app.show_pattern_locator = !app.show_pattern_locator;
        }

        // Ctrl+Comma - Open Settings
        if ctrl && !shift && i.key_pressed(egui::Key::Comma) {
            app.show_settings = !app.show_settings;
        }

        // Ctrl+Plus/Equals - Zoom in
        if ctrl && !shift && (i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) {
            app.viewer.zoom_in();
        }

        // Ctrl+Minus - Zoom out
        if ctrl && !shift && i.key_pressed(egui::Key::Minus) {
            app.viewer.zoom_out();
        }

        // Ctrl+0 - Reset zoom
        if ctrl && !shift && i.key_pressed(egui::Key::Num0) {
            app.viewer.reset_zoom();
        }

        // Only handle non-text shortcuts when text input is not focused
        if !text_edit_focused {
            // 1, 2, 3 - Switch view modes (Bit, Byte, ASCII)
            if !ctrl && !shift && i.key_pressed(egui::Key::Num1) {
                app.view_mode = ViewMode::Bit;
                app.update_viewer();
            }

            if !ctrl && !shift && i.key_pressed(egui::Key::Num2) {
                app.view_mode = ViewMode::Byte;
            }

            if !ctrl && !shift && i.key_pressed(egui::Key::Num3) {
                app.view_mode = ViewMode::Ascii;
            }

            // Delete - Delete selected operation (if we have a selection mechanism in the future)
            // For now, this is a placeholder for when we add operation selection
            if i.key_pressed(egui::Key::Delete) {
                // TODO: Delete selected operation when selection is implemented
            }

            // Space - Toggle operation enabled/disabled (if we have a selection mechanism)
            // For now, this is a placeholder for when we add operation selection
            if i.key_pressed(egui::Key::Space) {
                // TODO: Toggle selected operation when selection is implemented
            }
        }
    });
}

impl eframe::App for BitApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // Auto-save session when closing
        self.save_session();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for close request and handle unsaved changes
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.has_unsaved_changes && !self.close_requested {
                // Prevent close and show confirmation
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                if !self.show_confirmation_dialog {
                    self.request_confirmation(
                        crate::app::PendingConfirmation::CloseApp,
                        "You have unsaved changes.\n\nAre you sure you want to close?".to_string()
                    );
                }
            }
        }
        // Disable text selection while dragging
        if self.dragging_operation.is_some() {
            ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
        }

        // Handle keyboard shortcuts
        handle_keyboard_shortcuts(self, ctx);

        // Clean up expired toasts
        self.clean_expired_toasts();

        // Handle close request after confirmation
        if self.close_requested {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Handle Ctrl+Shift+Mouse Wheel for font size adjustment
        let mut font_size_changed = false;
        ctx.input(|i| {
            let ctrl_shift_held = i.modifiers.ctrl && i.modifiers.shift;

            if ctrl_shift_held {
                let delta = i.smooth_scroll_delta.y + i.raw_scroll_delta.y;

                if delta.abs() > 0.01 {
                    let sensitivity = 0.1;
                    self.font_size = (self.font_size + delta * sensitivity).clamp(8.0, 24.0);
                    font_size_changed = true;
                }
            }
        });

        if font_size_changed {
            ctx.request_repaint();
        }

        // Apply font size to the context (only if changed)
        if self.font_size != self.last_applied_font_size {
            let mut style = (*ctx.style()).clone();
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Button,
                egui::FontId::new(self.font_size, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::new(self.font_size * 0.85, egui::FontFamily::Proportional),
            );
            style.text_styles.insert(
                egui::TextStyle::Heading,
                egui::FontId::new(self.font_size * 1.3, egui::FontFamily::Proportional),
            );
            ctx.set_style(style);
            self.last_applied_font_size = self.font_size;
        }

        // Show restore session dialog
        if self.show_restore_dialog {
            egui::Window::new("Restore Previous Session?")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("A previous session was found.");
                        ui.label("Would you like to restore it or start fresh?");
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            if ui.button("🔄 Restore Session").clicked() {
                                if let Some(session) = self.pending_session.take() {
                                    self.restore_session(session);
                                }
                                self.show_restore_dialog = false;
                            }

                            if ui.button("🆕 Start Fresh").clicked() {
                                let _ = AppSession::delete();
                                self.pending_session = None;
                                self.show_restore_dialog = false;
                            }
                        });
                    });
                });
        }

        // Show confirmation dialog
        if self.show_confirmation_dialog {
            egui::Window::new("Confirmation Required")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(5.0);
                        ui.label(&self.confirmation_message);
                        ui.add_space(15.0);

                        ui.horizontal(|ui| {
                            let yes_button = egui::Button::new(
                                egui::RichText::new("Yes").color(egui::Color32::from_rgb(255, 80, 80))
                            );
                            if ui.add(yes_button).clicked() {
                                self.execute_confirmed_action();
                            }

                            if ui.button("No").clicked() {
                                self.cancel_confirmation();
                            }
                        });
                        ui.add_space(5.0);
                    });
                });
        }

        // Show notification dialog (when using Dialog mode)
        if self.show_notification_dialog {
            if let Some(toast_type) = self.notification_dialog_type {
                use crate::app::ToastType;
                let title = match toast_type {
                    ToastType::Info => "Information",
                    ToastType::Success => "Success",
                    ToastType::Warning => "Warning",
                    ToastType::Error => "Error",
                };

                egui::Window::new(title)
                    .collapsible(false)
                    .resizable(false)
                    .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                    .show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(5.0);
                            ui.label(&self.notification_dialog_message);
                            ui.add_space(15.0);

                            if ui.button("OK").clicked() {
                                self.show_notification_dialog = false;
                                self.notification_dialog_type = None;
                                self.notification_dialog_message.clear();
                            }
                            ui.add_space(5.0);
                        });
                    });
            }
        }
        
        // Update loading progress
        self.update_loading_progress();
        
        // Update operation processing progress
        self.update_operation_progress();
        
        // Show loading dialog
        if self.is_loading() {
            egui::Window::new("Loading File...")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        if let Some(path) = &self.loading_state.file_path {
                            ui.label(format!("Loading: {}", path.file_name().unwrap_or_default().to_string_lossy()));
                        }
                        ui.add_space(10.0);

                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(self.loading_state.progress)
                            .show_percentage()
                            .desired_width(300.0);
                        ui.add(progress_bar);

                        ui.add_space(5.0);

                        // Show loaded/total bytes
                        let loaded_mb = (self.loading_state.total as f64 * self.loading_state.progress as f64) / (1024.0 * 1024.0);
                        let total_mb = self.loading_state.total as f64 / (1024.0 * 1024.0);
                        ui.label(format!("{:.2} MB / {:.2} MB", loaded_mb, total_mb));

                        // Show estimated time remaining
                        ui.add_space(3.0);
                        ui.label(format!("⏱ ETA: {}", self.loading_state.eta_string()));
                    });
                });
            
            // Request continuous repaints while loading
            ctx.request_repaint();
        }
        
        // Show operation processing dialog
        if self.is_processing_operations() {
            egui::Window::new("Processing Operations...")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(&self.operation_processing_state.progress_message);
                        ui.add_space(10.0);

                        // Progress bar
                        let progress_bar = egui::ProgressBar::new(self.operation_processing_state.progress)
                            .show_percentage()
                            .desired_width(300.0);
                        ui.add(progress_bar);

                        // Show estimated time remaining
                        ui.add_space(5.0);
                        ui.label(format!("⏱ ETA: {}", self.operation_processing_state.eta_string()));
                    });
                });
            
            // Request continuous repaints while processing
            ctx.request_repaint();
        }
        
        // Show rendering preparation dialog and defer render if needed
        if self.defer_first_render {
            egui::Window::new("Preparing View...")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(&self.render_progress_message);
                        ui.add_space(10.0);
                        
                        // Spinner animation
                        ui.spinner();
                        
                        ui.add_space(5.0);
                        ui.label("This may take a moment for large files...");
                    });
                });
            
            // Clear the flag and trigger the actual render on the next frame
            self.defer_first_render = false;
            self.update_viewer();
            
            // Request a repaint to show the view immediately after preparation
            ctx.request_repaint();
            
            // Skip rendering the main UI this frame - just show the "preparing" dialog
            return;
        }

        // Render UI panels
        ui::top_panel::render(self, ctx);
        render_left_panels(self, ctx);
        render_bottom_panel(self, ctx);
        
        // Render windows
        render_settings_window(self, ctx);
        render_pattern_locator_window(self, ctx);
        render_frame_width_finder_window(self, ctx);
        render_operation_windows(self, ctx);
        render_column_editor_window(self, ctx);
        render_save_template_dialog(self, ctx);

        // Render central panel
        render_central_panel(self, ctx);

        // Render toast notifications on top of everything
        render_toasts(self, ctx);
    }
}

fn render_left_panels(app: &mut BitApp, ctx: &egui::Context) {
    // Leftmost panel: Available Operations
    let available_ops_response = egui::SidePanel::left("available_operations_panel")
        .default_width(app.settings.available_operations_panel_width)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Available Operations");
            ui.separator();

            // Search box
            ui.horizontal(|ui| {
                ui.label("🔍")
                    .on_hover_text("Search operations and templates by name or description");
                let response = ui.text_edit_singleline(&mut app.operations_search_text)
                    .on_hover_text("Type to filter operations and templates");
                if response.changed() {
                    // Search text changed, filter will update automatically
                }
                if !app.operations_search_text.is_empty() {
                    if ui.small_button("❌")
                        .on_hover_text("Clear search")
                        .clicked()
                    {
                        app.operations_search_text.clear();
                    }
                }
            });
            ui.add_space(5.0);

            egui::ScrollArea::vertical()
                .id_salt("available_ops")
                .show(ui, |ui| {
                    let all_operations = OperationType::all();

                    // Filter operations based on search text
                    let search_lower = app.operations_search_text.to_lowercase();
                    let filtered_operations: Vec<_> = all_operations
                        .iter()
                        .filter(|op_type| {
                            if search_lower.is_empty() {
                                true
                            } else {
                                op_type.name().to_lowercase().contains(&search_lower) ||
                                op_type.description().to_lowercase().contains(&search_lower)
                            }
                        })
                        .collect();

                    if filtered_operations.is_empty() {
                        ui.label("No matching operations");
                    } else {
                        for &op_type in &filtered_operations {
                        let available_width = ui.available_width();
                        let (rect, response) = ui.allocate_exact_size(
                            egui::vec2(available_width, ui.spacing().interact_size.y),
                            egui::Sense::click()
                        );
                        
                        if ui.is_rect_visible(rect) {
                            let visuals = ui.style().interact(&response);
                            ui.painter().rect_filled(
                                rect,
                                3.0,
                                visuals.bg_fill,
                            );
                            if visuals.bg_stroke.width > 0.0 {
                                ui.painter().rect_stroke(
                                    rect,
                                    3.0,
                                    visuals.bg_stroke,
                                    egui::epaint::StrokeKind::Outside
                                );
                            }

                            // Draw category color indicator
                            let category_color = op_type.category().color();
                            let indicator_rect = egui::Rect::from_min_size(
                                rect.left_top() + egui::vec2(4.0, 4.0),
                                egui::vec2(4.0, rect.height() - 8.0)
                            );
                            ui.painter().rect_filled(
                                indicator_rect,
                                2.0,
                                category_color,
                            );

                            let text_pos = rect.left_center() + egui::vec2(12.0, 0.0);
                            ui.painter().text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                format!("{} {}", op_type.icon(), op_type.name()),
                                egui::FontId::default(),
                                visuals.text_color(),
                            );
                        }
                        
                        if response.on_hover_text(op_type.description()).clicked() {
                            app.open_operation_creator(*op_type);
                        }
                        ui.add_space(4.0);
                        }
                    }

                    ui.separator();

                    // Templates section
                    ui.heading("📚 Templates");
                    ui.separator();

                    let template_count = app.settings.template_library.templates.len();
                    if template_count == 0 {
                        ui.label("No templates saved");
                    } else {
                        let mut to_instantiate = None;
                        let mut to_delete = None;

                        // Filter templates based on search text
                        let filtered_templates: Vec<(usize, &crate::storage::OperationTemplate)> =
                            app.settings.template_library.templates.iter().enumerate()
                                .filter(|(_, template)| {
                                    if search_lower.is_empty() {
                                        true
                                    } else {
                                        template.template_name.to_lowercase().contains(&search_lower) ||
                                        template.operation.description().to_lowercase().contains(&search_lower)
                                    }
                                })
                                .collect();

                        if filtered_templates.is_empty() && !search_lower.is_empty() {
                            ui.label("No matching templates");
                        } else {
                            for (idx, template) in filtered_templates {
                                ui.horizontal(|ui| {
                                    if ui.button(format!("{} {}", template.icon(), template.template_name))
                                        .on_hover_text(template.operation.description())
                                        .clicked()
                                    {
                                        to_instantiate = Some(idx);
                                    }

                                    if ui.small_button("🗑").clicked() {
                                        to_delete = Some(idx);
                                    }
                                });
                            }
                        }

                        if let Some(idx) = to_instantiate {
                            if let Some(template) = app.settings.template_library.get_template(idx) {
                                let operation = template.instantiate();
                                app.operations.push(operation);
                                app.mark_unsaved();
                                app.apply_operations();
                            }
                        }

                        if let Some(idx) = to_delete {
                            app.settings.template_library.remove_template(idx);
                            app.settings.auto_save();
                        }
                    }

                    ui.separator();
                    ui.label("💡 Click an operation to add");
                    ui.label("💡 Click a template to use");
                    ui.label("💡 Use search above to filter");

                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("❓");
                        ui.label("Drag operations to reorder")
                            .on_hover_text("Click and drag operations in the active list to change their order");
                    });
                });
        });

    // Save panel width if it changed
    let available_ops_width = available_ops_response.response.rect.width();
    if (available_ops_width - app.settings.available_operations_panel_width).abs() > 1.0 {
        app.settings.available_operations_panel_width = available_ops_width;
        app.settings.auto_save();
    }

    // Middle panel: Worksheets and Active Operations
    let active_ops_response = egui::SidePanel::left("active_operations_panel")
        .default_width(app.settings.active_operations_panel_width)
        .resizable(true)
        .show(ctx, |ui| {
            render_worksheets_section(app, ui);
            ui.separator();
            render_active_operations_section(app, ui);
            render_byte_view_config_section(app, ui);
            render_info_section(app, ui);
        });

    // Save panel width if it changed
    let active_ops_width = active_ops_response.response.rect.width();
    if (active_ops_width - app.settings.active_operations_panel_width).abs() > 1.0 {
        app.settings.active_operations_panel_width = active_ops_width;
        app.settings.auto_save();
    }
}

fn render_worksheets_section(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Worksheets");
    ui.separator();
    
    ui.horizontal(|ui| {
        if ui.button("➕")
            .on_hover_text("Add new worksheet (Ctrl+T)")
            .clicked()
        {
            let new_name = format!("Worksheet {}", app.worksheets.len() + 1);
            app.sync_to_worksheet();
            app.worksheets.push(crate::storage::Worksheet::new(new_name));
            app.current_worksheet_index = app.worksheets.len() - 1;
            app.load_from_worksheet();
        }

        if ui.button("💾 Save")
            .on_hover_text("Save worksheet to file\nStores operations and settings")
            .clicked()
        {
            app.save_worksheet_to_file();
        }

        if ui.button("📂 Load")
            .on_hover_text("Load worksheet from file\nRestores saved operations")
            .clicked()
        {
            app.load_worksheet_from_file();
        }
    });
    
    ui.add_space(4.0);
    
    egui::ScrollArea::vertical()
        .id_salt("worksheets")
        .max_height(120.0)
        .show(ui, |ui| {
            let mut to_switch = None;
            let mut to_delete = None;
            let num_worksheets = app.worksheets.len();
            
            for i in 0..num_worksheets {
                let is_current = i == app.current_worksheet_index;
                let worksheet_name = app.worksheets[i].name.clone();
                let is_modified = app.worksheets[i].is_modified();

                ui.horizontal(|ui| {
                    let mut response = ui.selectable_label(is_current, "");
                    if response.clicked() && !is_current {
                        to_switch = Some(i);
                    }

                    if app.renaming_worksheet == Some(i) {
                        let text_response = ui.text_edit_singleline(&mut app.worksheet_name_buffer);
                        if text_response.lost_focus() {
                            app.worksheets[i].name = app.worksheet_name_buffer.clone();
                            app.worksheets[i].mark_modified();
                            app.renaming_worksheet = None;
                        }
                    } else {
                        // Show modified indicator if worksheet has unsaved changes
                        let display_name = if is_modified {
                            format!("● {}", worksheet_name)
                        } else {
                            worksheet_name.clone()
                        };
                        response = ui.label(&display_name)
                            .on_hover_text(if is_modified { "Worksheet has unsaved changes" } else { "" });
                        if response.clicked() && !is_current {
                            to_switch = Some(i);
                        }
                        if response.double_clicked() {
                            app.renaming_worksheet = Some(i);
                            app.worksheet_name_buffer = worksheet_name.clone();
                        }
                    }
                    
                    if num_worksheets > 1 && ui.small_button("🗑")
                        .on_hover_text("Delete worksheet")
                        .clicked()
                    {
                        to_delete = Some(i);
                    }
                });
            }
            
            if let Some(idx) = to_switch {
                app.switch_worksheet(idx);
            }

            if let Some(idx) = to_delete {
                // Check if worksheet has operations before deleting
                let has_operations = !app.worksheets[idx].operations.is_empty();
                if has_operations {
                    let worksheet_name = &app.worksheets[idx].name;
                    app.request_confirmation(
                        crate::app::PendingConfirmation::DeleteWorksheet(idx),
                        format!("Delete worksheet \"{}\"?\n\nThis worksheet has {} operation(s) that will be lost.",
                            worksheet_name,
                            app.worksheets[idx].operations.len())
                    );
                } else {
                    // No operations, delete without confirmation
                    app.worksheets.remove(idx);
                    if app.current_worksheet_index >= app.worksheets.len() {
                        app.current_worksheet_index = app.worksheets.len() - 1;
                    }
                    if app.current_worksheet_index == idx || idx < app.current_worksheet_index {
                        app.load_from_worksheet();
                    }
                }
            }
        });
}

fn render_active_operations_section(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Active Operations");
    ui.separator();

    let mut to_remove: Option<usize> = None;
    let mut to_edit: Option<usize> = None;
    let mut to_save_as_template: Option<usize> = None;
    let mut toggled_operation: Option<usize> = None;

    egui::ScrollArea::vertical()
        .id_salt("active_ops")
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .show(ui, |ui| {

            if app.operations.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.label("No operations added");
                });
            } else {
                let mut drop_target_idx = None;
                
                for (i, op) in app.operations.iter().enumerate() {
                    let is_being_dragged = app.dragging_operation == Some(i);
                    let is_enabled = op.is_enabled();
                    let alpha = if is_being_dragged { 0.3 } else if !is_enabled { 0.5 } else { 1.0 };
                    
                    let mut show_drop_indicator_above = false;
                    let mut show_drop_indicator_below = false;
                    
                    ui.scope(|ui| {
                        if is_being_dragged || !is_enabled {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = 
                                ui.style().visuals.widgets.inactive.bg_fill.linear_multiply(alpha);
                            ui.style_mut().visuals.widgets.noninteractive.bg_fill = 
                                ui.style().visuals.widgets.noninteractive.bg_fill.linear_multiply(alpha);
                        }
                        
                        let response = ui.group(|ui| {
                            ui.set_min_width(ui.available_width());
                            ui.style_mut().interaction.selectable_labels = false;
                            
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let drag_handle = ui.label("☰").interact(egui::Sense::click_and_drag());
                                    
                                    // Checkbox to enable/disable operation
                                    let mut enabled = is_enabled;
                                    if ui.checkbox(&mut enabled, "").changed() {
                                        toggled_operation = Some(i);
                                    }
                                    
                                    ui.label(format!("{}.", i + 1));
                                    ui.vertical(|ui| {
                                        let name_color = if is_enabled { ui.style().visuals.text_color() } else { ui.style().visuals.weak_text_color() };
                                        ui.colored_label(name_color, op.name());
                                        ui.small(op.description());
                                    });
                                    
                                    if drag_handle.dragged() {
                                        app.dragging_operation = Some(i);
                                    }
                                });
                                
                                ui.horizontal(|ui| {
                                    if ui.button("✏ Edit").clicked() {
                                        to_edit = Some(i);
                                    }
                                    if ui.button("💾 Template").on_hover_text("Save as template").clicked() {
                                        to_save_as_template = Some(i);
                                    }
                                    if ui.button("🗑").clicked() {
                                        to_remove = Some(i);
                                    }
                                });
                            });
                        }).response;
                        
                        if let Some(dragged_idx) = app.dragging_operation {
                            if dragged_idx != i {
                                if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                    let rect = response.rect;
                                    if rect.contains(pointer_pos) {
                                        let mid_y = rect.center().y;
                                        if pointer_pos.y < mid_y {
                                            drop_target_idx = Some(i);
                                            show_drop_indicator_above = true;
                                        } else {
                                            drop_target_idx = Some(i + 1);
                                            show_drop_indicator_below = true;
                                        }
                                    }
                                }
                            }
                        }
                        
                        if show_drop_indicator_above {
                            let rect = response.rect;
                            let line_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x - 5.0, rect.min.y - 3.0),
                                egui::vec2(rect.width() + 10.0, 6.0)
                            );
                            ui.painter().rect_filled(line_rect, 3.0, egui::Color32::from_rgb(70, 130, 255));
                        }
                        
                        if show_drop_indicator_below {
                            let rect = response.rect;
                            let line_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.min.x - 5.0, rect.max.y - 1.0),
                                egui::vec2(rect.width() + 10.0, 6.0)
                            );
                            ui.painter().rect_filled(line_rect, 3.0, egui::Color32::from_rgb(70, 130, 255));
                        }
                        
                        if is_being_dragged {
                            ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                            
                            if let Some(pointer_pos) = ui.ctx().pointer_hover_pos() {
                                let preview_rect = egui::Rect::from_min_size(
                                    pointer_pos + egui::vec2(10.0, 10.0),
                                    egui::vec2(200.0, 40.0)
                                );
                                ui.painter().rect_filled(preview_rect, 4.0, egui::Color32::from_rgba_unmultiplied(60, 60, 80, 230));
                                ui.painter().text(
                                    preview_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    format!("{}. {}", i + 1, op.name()),
                                    egui::FontId::default(),
                                    egui::Color32::WHITE
                                );
                            }
                        }
                    });
                    
                    ui.add_space(4.0);
                }
                
                if ui.input(|i| i.pointer.primary_released()) {
                    if let Some(from) = app.dragging_operation {
                        if let Some(to) = drop_target_idx {
                            if from != to {
                                let op = app.operations.remove(from);
                                let insert_pos = if to > from { to - 1 } else { to };
                                app.operations.insert(insert_pos, op);
                                app.clear_pattern_matches(); // Operation order changed, clear patterns
                                app.apply_operations();
                            }
                        }
                    }
                    app.dragging_operation = None;
                }
            }
        });

    if let Some(idx) = to_remove {
        if let Some(op) = app.operations.get(idx) {
            app.request_confirmation(
                crate::app::PendingConfirmation::DeleteOperation(idx),
                format!("Delete operation \"{}\"?\n\nThis action cannot be undone.", op.name())
            );
        }
    }

    if let Some(idx) = to_edit {
        app.open_operation_editor(idx);
    }

    if let Some(idx) = to_save_as_template {
        if let Some(op) = app.operations.get(idx) {
            app.template_name_buffer = op.name().to_string();
            app.show_save_template_dialog = true;
            app.editing_operation_index = Some(idx); // Reuse this field to track which op to save
        }
    }

    if let Some(idx) = toggled_operation {
        if let Some(op) = app.operations.get_mut(idx) {
            let new_enabled = !op.is_enabled();
            op.set_enabled(new_enabled);
            app.apply_operations();
        }
    }

    if !app.operations.is_empty() {
        ui.separator();
        if ui.button("🔄 Reapply All").clicked() {
            app.apply_operations();
        }
        if ui.button("🗑 Clear All").clicked() {
            app.request_confirmation(
                crate::app::PendingConfirmation::ClearAllOperations,
                format!("Clear all {} operation(s)?\n\nThis action cannot be undone.", app.operations.len())
            );
        }
    }
}

fn render_byte_view_config_section(app: &mut BitApp, ui: &mut egui::Ui) {
    if app.view_mode == ViewMode::Byte {
        ui.separator();
        ui.heading("📊 Byte View Config");
        
        ui.horizontal(|ui| {
            ui.label("Bytes per row:");
            let mut bytes_per_row = app.byte_viewer.config.bytes_per_row;
            if ui.add(egui::Slider::new(&mut bytes_per_row, 1..=64)).changed() {
                app.byte_viewer.set_bytes_per_row(bytes_per_row);
            }
        });
        
        ui.checkbox(&mut app.byte_viewer.config.show_hex_offset, "Show hex offset");
        
        ui.add_space(8.0);
        ui.strong("Protocol Columns");
        
        egui::ScrollArea::vertical()
            .id_salt("byte_columns")
            .max_height(200.0)
            .show(ui, |ui| {
                let mut to_remove = None;
                
                for (idx, column) in app.byte_viewer.config.columns.iter().enumerate() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            let color_rect = ui.allocate_rect(
                                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(20.0, 20.0)),
                                egui::Sense::hover()
                            );
                            ui.painter().rect_filled(color_rect.rect, 3.0, column.color32());
                            
                            ui.vertical(|ui| {
                                ui.label(&column.label);
                                ui.small(format!("Bits {}..{}", column.bit_start, column.bit_end));
                            });
                            
                            if ui.button("🗑").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    });
                }
                
                if let Some(idx) = to_remove {
                    app.byte_viewer.remove_column(idx);
                }
            });
        
        ui.horizontal(|ui| {
            if ui.button("➕ Add Column").clicked() {
                app.show_column_editor = true;
            }
            
            if ui.button("💾 Save Config").clicked() {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("protocol_config.json")
                    .save_file() {
                    match serde_json::to_string_pretty(&app.byte_viewer.config) {
                        Ok(json) => {
                            if let Err(e) = std::fs::write(&file_path, json) {
                                app.error_message = Some(format!("Failed to save config: {}", e));
                            }
                        }
                        Err(e) => {
                            app.error_message = Some(format!("Failed to serialize config: {}", e));
                        }
                    }
                }
            }
            
            if ui.button("📂 Load Config").clicked() {
                if let Some(file_path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .pick_file() {
                    match std::fs::read_to_string(&file_path) {
                        Ok(json) => {
                            match serde_json::from_str(&json) {
                                Ok(config) => {
                                    app.byte_viewer.config = config;
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("Failed to parse config: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            app.error_message = Some(format!("Failed to read config file: {}", e));
                        }
                    }
                }
            }
        });
        
        if ui.button("📄 Export Documentation").clicked() {
            if let Some(file_path) = rfd::FileDialog::new()
                .add_filter("Text", &["txt"])
                .set_file_name("protocol_documentation.txt")
                .save_file() {
                let mut doc = String::new();
                doc.push_str("Protocol Documentation\n");
                doc.push_str("=====================\n\n");
                doc.push_str(&format!("Bytes per row: {}\n\n", app.byte_viewer.config.bytes_per_row));
                doc.push_str("Field Definitions:\n");
                doc.push_str("------------------\n\n");
                
                for (idx, column) in app.byte_viewer.config.columns.iter().enumerate() {
                    doc.push_str(&format!("{}. {}\n", idx + 1, column.label));
                    doc.push_str(&format!("   Bit Range: {} - {}\n", column.bit_start, column.bit_end));
                    let (start_byte, end_byte) = column.byte_range(app.byte_viewer.config.bytes_per_row);
                    doc.push_str(&format!("   Byte Range: {} - {}\n", start_byte, end_byte));
                    doc.push_str(&format!("   Color: RGB({}, {}, {})\n\n", 
                        column.color[0], column.color[1], column.color[2]));
                }
                
                if let Err(e) = std::fs::write(&file_path, doc) {
                    app.error_message = Some(format!("Failed to export documentation: {}", e));
                }
            }
        }
    }
}

fn render_info_section(app: &BitApp, ui: &mut egui::Ui) {
    ui.separator();
    if let Some(path) = &app.current_file_path {
        ui.label(format!("📄 {}", path.file_name().unwrap_or_default().to_string_lossy()));
    }
    ui.label(format!("Original: {} bits", app.original_bits.len()));
    ui.label(format!("Processed: {} bits", app.processed_bits.len()));
    ui.label(format!("Bit size: {:.1}px", app.viewer.bit_size));
}

fn render_bottom_panel(app: &mut BitApp, ctx: &egui::Context) {
    egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Frame Length (bits per row):");
            if ui.add(egui::Slider::new(&mut app.viewer.frame_length, 8..=512).logarithmic(true)).changed() {
                app.settings.frame_length = app.viewer.frame_length;
                app.settings.auto_save();
            }
            ui.label(format!("({} chars in ASCII view)", app.viewer.frame_length / 8));
        });
    });
}

fn render_central_panel(app: &mut BitApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Show critical errors prominently
        let mut dismiss_error = false;
        if let Some(error) = &app.error_message {
            let error_text = error.clone();
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                // Error icon
                ui.label(egui::RichText::new("⚠").size(24.0).color(egui::Color32::from_rgb(255, 100, 100)));

                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Error").strong().color(egui::Color32::from_rgb(255, 100, 100)));
                    ui.label(&error_text);

                    ui.horizontal(|ui| {
                        if ui.small_button("📋 Copy Error").clicked() {
                            ui.ctx().copy_text(error_text.clone());
                        }
                        if ui.small_button("✕ Dismiss").clicked() {
                            dismiss_error = true;
                        }
                    });
                });
            });
            ui.separator();
        }

        if dismiss_error {
            app.error_message = None;
        }

        if app.original_bits.is_empty() && app.processed_bits.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.heading("Open a file to view its bits");
            });
        } else {
            let bits_to_display = if app.show_original {
                &app.original_bits
            } else {
                &app.processed_bits
            };
            
            match app.view_mode {
                ViewMode::Bit => {
                    app.viewer.show(ui);
                }
                ViewMode::Byte => {
                    app.byte_viewer.render_with_patterns(ui, bits_to_display, &app.patterns);
                }
                ViewMode::Ascii => {
                    app.render_ascii_view(ui, bits_to_display);
                }
            }
        }
    });
}

fn render_settings_window(app: &mut BitApp, ctx: &egui::Context) {
    let mut layout_reset = false;

    if app.show_settings {
        egui::Window::new("⚙ Settings")
            .open(&mut app.show_settings)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Display Settings");
                ui.separator();

                ui.label("Bit Shape:");
                ui.horizontal(|ui| {
                    if ui.selectable_label(app.viewer.shape == crate::viewers::BitShape::Square, "⬛ Square").clicked() {
                        app.viewer.shape = crate::viewers::BitShape::Square;
                        app.settings.bit_shape = crate::viewers::BitShape::Square;
                        app.settings.auto_save();
                    }
                    if ui.selectable_label(app.viewer.shape == crate::viewers::BitShape::Circle, "⚫ Circle").clicked() {
                        app.viewer.shape = crate::viewers::BitShape::Circle;
                        app.settings.bit_shape = crate::viewers::BitShape::Circle;
                        app.settings.auto_save();
                    }
                    if ui.selectable_label(app.viewer.shape == crate::viewers::BitShape::Octagon, "⬢ Octagon").clicked() {
                        app.viewer.shape = crate::viewers::BitShape::Octagon;
                        app.settings.bit_shape = crate::viewers::BitShape::Octagon;
                        app.settings.auto_save();
                    }
                });

                ui.separator();

                if ui.checkbox(&mut app.viewer.show_grid, "Show Grid Lines").changed() {
                    app.settings.show_grid = app.viewer.show_grid;
                    app.settings.auto_save();
                }

                ui.add_space(8.0);

                ui.label("Thick Grid Interval (Horizontal):");
                if ui.add(egui::Slider::new(&mut app.viewer.thick_grid_interval_horizontal, 0..=64)).changed() {
                    app.settings.thick_grid_interval_horizontal = app.viewer.thick_grid_interval_horizontal;
                    app.settings.auto_save();
                }

                ui.add_space(4.0);

                ui.label("Thick Grid Interval (Vertical):");
                if ui.add(egui::Slider::new(&mut app.viewer.thick_grid_interval_vertical, 0..=64)).changed() {
                    app.settings.thick_grid_interval_vertical = app.viewer.thick_grid_interval_vertical;
                    app.settings.auto_save();
                }

                ui.add_space(4.0);

                ui.label("Thick Grid Spacing (Horizontal):");
                if ui.add(egui::Slider::new(&mut app.viewer.thick_grid_spacing_horizontal, 0.0..=10.0)).changed() {
                    app.settings.thick_grid_spacing_horizontal = app.viewer.thick_grid_spacing_horizontal;
                    app.settings.auto_save();
                }

                ui.add_space(4.0);

                ui.label("Thick Grid Spacing (Vertical):");
                if ui.add(egui::Slider::new(&mut app.viewer.thick_grid_spacing_vertical, 0.0..=10.0)).changed() {
                    app.settings.thick_grid_spacing_vertical = app.viewer.thick_grid_spacing_vertical;
                    app.settings.auto_save();
                }

                ui.separator();

                ui.label("GUI Font Size:");
                if ui.add(egui::Slider::new(&mut app.font_size, 8.0..=24.0)).changed() {
                    app.settings.font_size = app.font_size;
                    app.settings.auto_save();
                }

                ui.separator();

                ui.label("Notification Display:");
                ui.horizontal(|ui| {
                    use crate::storage::NotificationDisplayMode;
                    if ui.selectable_label(app.settings.notification_display_mode == NotificationDisplayMode::Toast, "🔔 Toast").clicked() {
                        app.settings.notification_display_mode = NotificationDisplayMode::Toast;
                        app.settings.auto_save();
                    }
                    if ui.selectable_label(app.settings.notification_display_mode == NotificationDisplayMode::Dialog, "💬 Dialog").clicked() {
                        app.settings.notification_display_mode = NotificationDisplayMode::Dialog;
                        app.settings.auto_save();
                    }
                    if ui.selectable_label(app.settings.notification_display_mode == NotificationDisplayMode::None, "🔇 None").clicked() {
                        app.settings.notification_display_mode = NotificationDisplayMode::None;
                        app.settings.auto_save();
                    }
                });

                ui.separator();
                ui.heading("Workspace Layout");
                ui.separator();

                ui.label(format!(
                    "Panel widths: Available Ops: {:.0}px, Active Ops: {:.0}px",
                    app.settings.available_operations_panel_width,
                    app.settings.active_operations_panel_width
                ));

                if ui.button("🔄 Reset Layout")
                    .on_hover_text("Reset panel widths to default values")
                    .clicked()
                {
                    app.settings.available_operations_panel_width = 200.0;
                    app.settings.active_operations_panel_width = 300.0;
                    app.settings.auto_save();
                    layout_reset = true;
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("💾 Save Settings").clicked() {
                        if let Some(file_path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .set_file_name("settings.json")
                            .save_file() {
                            if let Err(e) = app.settings.save_to_file(&file_path) {
                                app.error_message = Some(format!("Failed to save settings: {}", e));
                            }
                        }
                    }
                    if ui.button("📂 Load Settings").clicked() {
                        if let Some(file_path) = rfd::FileDialog::new()
                            .add_filter("JSON", &["json"])
                            .pick_file() {
                            match crate::storage::AppSettings::load_from_file(&file_path) {
                                Ok(loaded_settings) => {
                                    app.settings = loaded_settings;
                                    app.viewer.shape = app.settings.bit_shape;
                                    app.viewer.show_grid = app.settings.show_grid;
                                    app.viewer.thick_grid_interval_horizontal = app.settings.thick_grid_interval_horizontal;
                                    app.viewer.thick_grid_interval_vertical = app.settings.thick_grid_interval_vertical;
                                    app.viewer.thick_grid_spacing_horizontal = app.settings.thick_grid_spacing_horizontal;
                                    app.viewer.thick_grid_spacing_vertical = app.settings.thick_grid_spacing_vertical;
                                    app.viewer.frame_length = app.settings.frame_length;
                                    app.font_size = app.settings.font_size;
                                }
                                Err(e) => {
                                    app.error_message = Some(format!("Failed to load settings: {}", e));
                                }
                            }
                        }
                    }
                    if ui.button("🔄 Reset to Defaults").clicked() {
                        app.settings = crate::storage::AppSettings::default();
                        app.viewer.shape = app.settings.bit_shape;
                        app.viewer.show_grid = app.settings.show_grid;
                        app.viewer.thick_grid_interval_horizontal = app.settings.thick_grid_interval_horizontal;
                        app.viewer.thick_grid_interval_vertical = app.settings.thick_grid_interval_vertical;
                        app.viewer.thick_grid_spacing_horizontal = app.settings.thick_grid_spacing_horizontal;
                        app.viewer.thick_grid_spacing_vertical = app.settings.thick_grid_spacing_vertical;
                        app.viewer.frame_length = app.settings.frame_length;
                        app.font_size = app.settings.font_size;
                        app.settings.auto_save();
                    }
                });
            });
    }

    if layout_reset {
        app.show_success("Workspace layout reset to defaults".to_string());
    }
}

// Continued in next part due to size...

fn render_pattern_locator_window(app: &mut BitApp, ctx: &egui::Context) {
    crate::ui::windows::render_pattern_locator_window(app, ctx);
}

fn render_frame_width_finder_window(app: &mut BitApp, ctx: &egui::Context) {
    crate::ui::windows::render_frame_width_finder_window(app, ctx);
}

fn render_operation_windows(app: &mut BitApp, ctx: &egui::Context) {
    crate::ui::windows::render_operation_windows(app, ctx);
}

fn render_column_editor_window(app: &mut BitApp, ctx: &egui::Context) {
    crate::ui::windows::render_column_editor_window(app, ctx);
}

fn render_save_template_dialog(app: &mut BitApp, ctx: &egui::Context) {
    if app.show_save_template_dialog {
        egui::Window::new("💾 Save as Template")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.label("Enter a name for this template:");
                ui.add_space(5.0);

                let response = ui.text_edit_singleline(&mut app.template_name_buffer);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(idx) = app.editing_operation_index {
                        if let Some(op) = app.operations.get(idx).cloned() {
                            let template_name = if app.template_name_buffer.is_empty() {
                                op.name().to_string()
                            } else {
                                app.template_name_buffer.clone()
                            };

                            let template = crate::storage::OperationTemplate::new(template_name, op);
                            app.settings.template_library.add_template(template);
                            app.settings.auto_save();
                            app.show_toast("Template saved".to_string(), crate::app::ToastType::Success);
                            app.show_save_template_dialog = false;
                            app.editing_operation_index = None;
                        }
                    }
                }

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("✓ Save").clicked() {
                        if let Some(idx) = app.editing_operation_index {
                            if let Some(op) = app.operations.get(idx).cloned() {
                                let template_name = if app.template_name_buffer.is_empty() {
                                    op.name().to_string()
                                } else {
                                    app.template_name_buffer.clone()
                                };

                                let template = crate::storage::OperationTemplate::new(template_name, op);
                                app.settings.template_library.add_template(template);
                                app.settings.auto_save();
                                app.show_toast("Template saved".to_string(), crate::app::ToastType::Success);
                                app.show_save_template_dialog = false;
                                app.editing_operation_index = None;
                            }
                        }
                    }

                    if ui.button("✗ Cancel").clicked() {
                        app.show_save_template_dialog = false;
                        app.editing_operation_index = None;
                    }
                });
            });
    }
}

fn render_toasts(app: &BitApp, ctx: &egui::Context) {
    use crate::app::ToastType;

    let screen_rect = ctx.input(|i| i.screen_rect());
    let toast_width = 350.0;
    let toast_spacing = 10.0;
    let mut y_offset = screen_rect.max.y - 60.0; // Start from bottom

    for toast in app.toasts.iter().rev() {
        let opacity = toast.opacity();
        let alpha = (opacity * 255.0) as u8;

        // Determine color based on toast type
        let (bg_color, text_color, icon) = match toast.toast_type {
            ToastType::Info => (
                egui::Color32::from_rgba_unmultiplied(60, 120, 200, alpha),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                "ℹ",
            ),
            ToastType::Success => (
                egui::Color32::from_rgba_unmultiplied(80, 180, 100, alpha),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                "✓",
            ),
            ToastType::Warning => (
                egui::Color32::from_rgba_unmultiplied(220, 180, 60, alpha),
                egui::Color32::from_rgba_unmultiplied(40, 40, 40, alpha),
                "⚠",
            ),
            ToastType::Error => (
                egui::Color32::from_rgba_unmultiplied(220, 80, 80, alpha),
                egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha),
                "✗",
            ),
        };

        let toast_height = 60.0;
        let toast_rect = egui::Rect::from_min_size(
            egui::pos2(screen_rect.max.x - toast_width - 20.0, y_offset - toast_height),
            egui::vec2(toast_width, toast_height),
        );

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("toast_layer"),
        ));

        // Draw shadow
        painter.rect_filled(
            toast_rect.translate(egui::vec2(2.0, 2.0)),
            8.0,
            egui::Color32::from_rgba_unmultiplied(0, 0, 0, (alpha as f32 * 0.3) as u8),
        );

        // Draw background
        painter.rect_filled(toast_rect, 8.0, bg_color);

        // Draw border
        painter.rect_stroke(
            toast_rect,
            8.0,
            egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(255, 255, 255, (alpha as f32 * 0.3) as u8)),
            egui::epaint::StrokeKind::Outside,
        );

        // Draw icon
        painter.text(
            toast_rect.left_center() + egui::vec2(15.0, 0.0),
            egui::Align2::LEFT_CENTER,
            icon,
            egui::FontId::proportional(24.0),
            text_color,
        );

        // Draw message
        let text_rect = egui::Rect::from_min_max(
            toast_rect.left_center() + egui::vec2(45.0, -20.0),
            toast_rect.right_center() + egui::vec2(-10.0, 20.0),
        );

        painter.text(
            text_rect.left_center(),
            egui::Align2::LEFT_CENTER,
            &toast.message,
            egui::FontId::proportional(14.0),
            text_color,
        );

        y_offset -= toast_height + toast_spacing;
    }

    // Request repaint if there are active toasts
    if !app.toasts.is_empty() {
        ctx.request_repaint();
    }
}

