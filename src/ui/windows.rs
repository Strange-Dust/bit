// Window rendering functions for pattern locator and operation editors

use crate::analysis::PatternFormat;
use crate::analysis::pattern_locator::MatchFilter;
use crate::app::BitApp;
use crate::operations::{EditorAction, EditorContext};
use eframe::egui;

pub fn render_pattern_locator_window(app: &mut BitApp, ctx: &egui::Context) {
    if !app.pattern_locator.show {
        return;
    }

    const PAGE_SIZE: usize = 200;

    // Deferred actions
    let mut navigate_to: Option<usize> = None;
    let mut do_search = false;
    let mut do_export = false;
    let mut delete_pattern_idx: Option<usize> = None;

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("pattern_locator"),
        egui::ViewportBuilder::default()
            .with_title("Pattern Locator")
            .with_inner_size([750.0, 850.0]),
        |vp_ctx, _class| {
            // Detect viewport close
            if vp_ctx.input(|i| i.viewport().close_requested()) {
                app.pattern_locator.show = false;
            }

            // ── Bottom panel: SEARCH button (always pinned) ──
            egui::TopBottomPanel::bottom("pl_search_panel").show(vp_ctx, |ui| {
                ui.add_space(4.0);
                let search_button = egui::Button::new(
                    egui::RichText::new("SEARCH").strong().size(16.0)
                ).min_size(egui::vec2(ui.available_width(), 36.0));
                if ui.add(search_button).clicked() {
                    do_search = true;
                }
                ui.add_space(4.0);
            });

            egui::CentralPanel::default().show(vp_ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("pattern_locator_scroll")
                    .show(ui, |ui| {
                        // ── Section 1: Pattern Grid ──
                        ui.heading("Patterns");
                        ui.label("Add multiple patterns before running search");
                        ui.add_space(4.0);

                        // Grid buttons
                        ui.horizontal(|ui| {
                            if ui.button("+ Row").clicked() {
                                app.pattern_locator.add_pattern();
                            }
                            let can_remove = !app.pattern_locator.patterns.is_empty();
                            ui.add_enabled_ui(can_remove, |ui| {
                                if ui.button("- Row").clicked() {
                                    let last = app.pattern_locator.patterns.len().saturating_sub(1);
                                    app.pattern_locator.remove_pattern(last);
                                }
                                if ui.button("Duplicate").clicked() {
                                    let last = app.pattern_locator.patterns.len().saturating_sub(1);
                                    app.pattern_locator.duplicate_pattern(last);
                                }
                            });
                            if ui.button("Enable All").clicked() {
                                app.pattern_locator.set_all_enabled(true);
                            }
                            if ui.button("Disable All").clicked() {
                                app.pattern_locator.set_all_enabled(false);
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Toggle Inverted").clicked() {
                                app.pattern_locator.toggle_all_inverted();
                            }
                            if ui.button("Toggle Highlight").clicked() {
                                app.pattern_locator.toggle_all_highlight();
                            }
                            if ui.button("Validate Patterns").clicked() {
                                app.pattern_locator.validate_all();
                            }
                        });

                        ui.add_space(4.0);

                        // Pattern table
                        if !app.pattern_locator.patterns.is_empty() {
                            use egui_extras::{TableBuilder, Column};

                            TableBuilder::new(ui)
                                .id_salt("pl_pattern_grid")
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .min_scrolled_height(0.0)
                                .max_scroll_height(f32::INFINITY)
                                .column(Column::exact(30.0))             // ID
                                .column(Column::exact(100.0))            // Name
                                .column(Column::exact(70.0))             // Type
                                .column(Column::remainder().at_least(120.0)) // Value
                                .column(Column::exact(28.0))             // EN
                                .column(Column::exact(28.0))             // INV
                                .column(Column::exact(28.0))             // HIL
                                .column(Column::exact(24.0))             // X (delete)
                                .header(22.0, |mut header| {
                                    header.col(|ui| { ui.strong("ID"); });
                                    header.col(|ui| { ui.strong("Name"); });
                                    header.col(|ui| { ui.strong("Type"); });
                                    header.col(|ui| { ui.strong("Value"); });
                                    header.col(|ui| { ui.strong("EN"); });
                                    header.col(|ui| { ui.strong("INV"); });
                                    header.col(|ui| { ui.strong("HIL"); });
                                    header.col(|_| {});
                                })
                                .body(|body| {
                                    let count = app.pattern_locator.patterns.len();
                                    body.rows(26.0, count, |mut row| {
                                        let idx = row.index();
                                        let pattern = &mut app.pattern_locator.patterns[idx];
                                        let pat_id = pattern.id;

                                        // ID
                                        row.col(|ui| {
                                            ui.label(format!("{}", pat_id));
                                        });

                                        // Name
                                        row.col(|ui| {
                                            let te = egui::TextEdit::singleline(&mut pattern.name)
                                                .id(egui::Id::new(("pl_name", pat_id)));
                                            ui.add(te);
                                        });

                                        // Type (format combo)
                                        row.col(|ui| {
                                            let current_label = pattern.format.label();
                                            egui::ComboBox::from_id_salt(("pl_fmt", pat_id))
                                                .selected_text(current_label)
                                                .width(50.0)
                                                .show_ui(ui, |ui| {
                                                    for &fmt in PatternFormat::all() {
                                                        ui.selectable_value(&mut pattern.format, fmt, fmt.label());
                                                    }
                                                });
                                        });

                                        // Value (with red outline on validation error)
                                        row.col(|ui| {
                                            let has_error = pattern.validation_error.is_some();
                                            let te = egui::TextEdit::singleline(&mut pattern.input)
                                                .id(egui::Id::new(("pl_val", pat_id)));
                                            if has_error {
                                                let frame = egui::Frame::NONE
                                                    .stroke(egui::Stroke::new(2.0, egui::Color32::RED))
                                                    .inner_margin(egui::Margin::same(2));
                                                frame.show(ui, |ui| {
                                                    ui.add(te);
                                                });
                                                if let Some(ref err) = pattern.validation_error {
                                                    ui.label(egui::RichText::new(err).color(egui::Color32::RED).small());
                                                }
                                            } else {
                                                ui.add(te);
                                            }
                                        });

                                        // EN
                                        row.col(|ui| {
                                            ui.checkbox(&mut pattern.enabled, "");
                                        });

                                        // INV
                                        row.col(|ui| {
                                            ui.checkbox(&mut pattern.search_inverted, "");
                                        });

                                        // HIL
                                        row.col(|ui| {
                                            ui.checkbox(&mut pattern.highlight, "");
                                        });

                                        // X (delete)
                                        row.col(|ui| {
                                            if ui.small_button("x").clicked() {
                                                delete_pattern_idx = Some(idx);
                                            }
                                        });
                                    });
                                });
                        } else {
                            ui.label("No patterns defined. Click '+ Row' to add one.");
                        }

                        ui.add_space(8.0);

                        // ── Section 2: Counts per Pattern (always visible) ──
                        ui.separator();
                        ui.heading("Pattern Match Counts");

                        {
                            use egui_extras::{TableBuilder, Column};

                            TableBuilder::new(ui)
                                .id_salt("pl_counts_table")
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .min_scrolled_height(0.0)
                                .max_scroll_height(f32::INFINITY)
                                .column(Column::exact(30.0))             // ID
                                .column(Column::remainder().at_least(100.0)) // Name
                                .column(Column::exact(50.0))             // Enabled
                                .column(Column::exact(80.0))             // Normal Count
                                .column(Column::exact(80.0))             // Inverted Count
                                .header(22.0, |mut header| {
                                    header.col(|ui| { ui.strong("ID"); });
                                    header.col(|ui| { ui.strong("Name"); });
                                    header.col(|ui| { ui.strong("EN"); });
                                    header.col(|ui| { ui.strong("Normal"); });
                                    header.col(|ui| { ui.strong("Inverted"); });
                                })
                                .body(|body| {
                                    let count = app.pattern_locator.patterns.len();
                                    body.rows(20.0, count, |mut row| {
                                        let idx = row.index();
                                        let pattern = &app.pattern_locator.patterns[idx];

                                        row.col(|ui| { ui.label(format!("{}", pattern.id)); });
                                        row.col(|ui| { ui.label(&pattern.name); });
                                        row.col(|ui| { ui.label(if pattern.enabled { "Y" } else { "N" }); });
                                        row.col(|ui| {
                                            if app.pattern_locator.has_searched && pattern.enabled {
                                                ui.label(format!("{}", pattern.normal_matches.len()));
                                            } else {
                                                ui.label("-");
                                            }
                                        });
                                        row.col(|ui| {
                                            if app.pattern_locator.has_searched && pattern.enabled && pattern.search_inverted {
                                                ui.label(format!("{}", pattern.inverted_matches.len()));
                                            } else {
                                                ui.label("-");
                                            }
                                        });
                                    });
                                });
                        }

                        ui.add_space(8.0);

                        // ── Section 3: All Matches (always visible) ──
                        ui.separator();
                        ui.heading("All Matches");

                        // Filter bar
                        ui.horizontal(|ui| {
                            ui.label("Filter ID:");
                            egui::ComboBox::from_id_salt("pl_filter_pattern_id")
                                .selected_text(
                                    app.pattern_locator.filter_pattern_id
                                        .map(|id| format!("{}", id))
                                        .unwrap_or_else(|| "All".to_string())
                                )
                                .show_ui(ui, |ui| {
                                    if ui.selectable_value(&mut app.pattern_locator.filter_pattern_id, None, "All").clicked() {}
                                    for pattern in &app.pattern_locator.patterns {
                                        let id = Some(pattern.id);
                                        if ui.selectable_value(&mut app.pattern_locator.filter_pattern_id, id, format!("{} - {}", pattern.id, pattern.name)).clicked() {}
                                    }
                                });

                            ui.label("Show:");
                            ui.selectable_value(&mut app.pattern_locator.filter_mode, MatchFilter::All, "All");
                            ui.selectable_value(&mut app.pattern_locator.filter_mode, MatchFilter::NormalOnly, "Normal");
                            ui.selectable_value(&mut app.pattern_locator.filter_mode, MatchFilter::InvertedOnly, "Inverted");

                            if ui.button("Export CSV...").clicked() {
                                do_export = true;
                            }
                        });

                        // Collect filtered matches into owned data to release borrow
                        let all_filtered: Vec<(usize, usize, bool, Option<usize>)> = app.pattern_locator.filtered_matches()
                            .iter()
                            .map(|m| (m.position, m.pattern_id, m.is_inverted, m.delta_next_same_id))
                            .collect();
                        let match_count = all_filtered.len();

                        ui.label(format!("{} matches", match_count));

                        if match_count > 0 {
                            // Pagination
                            let total_pages = (match_count + PAGE_SIZE - 1) / PAGE_SIZE;
                            let page = app.pattern_locator.match_page.min(total_pages.saturating_sub(1));
                            let page_start = page * PAGE_SIZE;
                            let page_end = (page_start + PAGE_SIZE).min(match_count);

                            if total_pages > 1 {
                                ui.horizontal(|ui| {
                                    ui.add_enabled_ui(page > 0, |ui| {
                                        if ui.small_button("<<").clicked() {
                                            app.pattern_locator.match_page = 0;
                                        }
                                        if ui.small_button("<").clicked() {
                                            app.pattern_locator.match_page = page.saturating_sub(1);
                                        }
                                    });
                                    ui.label(format!(
                                        "Page {} of {} (#{}-#{})",
                                        page + 1, total_pages, page_start + 1, page_end
                                    ));
                                    ui.add_enabled_ui(page + 1 < total_pages, |ui| {
                                        if ui.small_button(">").clicked() {
                                            app.pattern_locator.match_page = (page + 1).min(total_pages - 1);
                                        }
                                        if ui.small_button(">>").clicked() {
                                            app.pattern_locator.match_page = total_pages - 1;
                                        }
                                    });
                                });
                            }

                            // Matches table
                            use egui_extras::{TableBuilder, Column};

                            let selected_idx = app.pattern_locator.selected_match_index;

                            let page_matches = &all_filtered[page_start..page_end];

                            TableBuilder::new(ui)
                                .id_salt("pl_matches_table")
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .min_scrolled_height(0.0)
                                .max_scroll_height(f32::INFINITY)
                                .column(Column::remainder().at_least(80.0))  // Bit Location
                                .column(Column::exact(50.0))             // ID
                                .column(Column::exact(35.0))             // Inv
                                .column(Column::exact(100.0))            // Delta Next
                                .header(22.0, |mut header| {
                                    header.col(|ui| { ui.strong("Bit Location"); });
                                    header.col(|ui| { ui.strong("ID"); });
                                    header.col(|ui| { ui.strong("Inv"); });
                                    header.col(|ui| { ui.strong("Delta Next"); });
                                })
                                .body(|body| {
                                    body.rows(20.0, page_matches.len(), |mut row| {
                                        let global_idx = page_start + row.index();
                                        let (position, pattern_id, is_inverted, delta_next) = &page_matches[row.index()];
                                        let is_selected = selected_idx == Some(global_idx);

                                        if is_selected {
                                            row.set_selected(true);
                                        }

                                        row.col(|ui| {
                                            if ui.selectable_label(is_selected, format!("{}", position)).clicked() {
                                                app.pattern_locator.selected_match_index = Some(global_idx);
                                                navigate_to = Some(*position);
                                            }
                                        });
                                        row.col(|ui| {
                                            if ui.selectable_label(is_selected, format!("{}", pattern_id)).clicked() {
                                                app.pattern_locator.selected_match_index = Some(global_idx);
                                                navigate_to = Some(*position);
                                            }
                                        });
                                        row.col(|ui| {
                                            let text = if *is_inverted { "Y" } else { "" };
                                            if ui.selectable_label(is_selected, text).clicked() {
                                                app.pattern_locator.selected_match_index = Some(global_idx);
                                                navigate_to = Some(*position);
                                            }
                                        });
                                        row.col(|ui| {
                                            let text = delta_next.map(|d| format!("{}", d)).unwrap_or_default();
                                            if ui.selectable_label(is_selected, text).clicked() {
                                                app.pattern_locator.selected_match_index = Some(global_idx);
                                                navigate_to = Some(*position);
                                            }
                                        });
                                    });
                                });
                        } else {
                            // Empty matches table with just headers
                            use egui_extras::{TableBuilder, Column};

                            TableBuilder::new(ui)
                                .id_salt("pl_matches_table_empty")
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .min_scrolled_height(0.0)
                                .max_scroll_height(f32::INFINITY)
                                .column(Column::remainder().at_least(80.0))
                                .column(Column::exact(50.0))
                                .column(Column::exact(35.0))
                                .column(Column::exact(100.0))
                                .header(22.0, |mut header| {
                                    header.col(|ui| { ui.strong("Bit Location"); });
                                    header.col(|ui| { ui.strong("ID"); });
                                    header.col(|ui| { ui.strong("Inv"); });
                                    header.col(|ui| { ui.strong("Delta Next"); });
                                })
                                .body(|_body| {});
                        }

                        ui.add_space(8.0);
                    });
            });
        },
    );

    // ── Deferred actions (full &mut BitApp access) ──

    if let Some(idx) = delete_pattern_idx {
        app.pattern_locator.remove_pattern(idx);
    }

    if do_search {
        app.pattern_locator.validate_all();
        let bits_to_search = if app.show_original {
            &app.original_bits
        } else {
            &app.processed_bits
        };
        // Can't pass reference directly due to borrow checker, clone the bits
        let haystack = bits_to_search.clone();
        app.pattern_locator.search_all(&haystack);

        // Update viewer highlights for patterns with HIL enabled
        app.viewer.clear_highlights();
        for pattern in &app.pattern_locator.patterns {
            if pattern.highlight && pattern.enabled {
                for m in &pattern.normal_matches {
                    app.viewer.add_highlight_range(m.position, pattern.bits.len());
                }
                for m in &pattern.inverted_matches {
                    app.viewer.add_highlight_range(m.position, pattern.bits.len());
                }
            }
        }
    }

    if let Some(position) = navigate_to {
        app.viewer.clear_highlights();
        app.viewer.add_highlight_range(position, 1);
        app.view_bit_offset = position;
        app.viewer.jump_to_position(position);
    }

    if do_export {
        let filtered = app.pattern_locator.filtered_matches();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name("pattern_matches.csv")
            .save_file()
        {
            let mut csv = String::from("Bit Location,Pattern ID,Inverted,Delta Next Same ID\n");
            for m in &filtered {
                csv.push_str(&format!(
                    "{},{},{},{}\n",
                    m.position,
                    m.pattern_id,
                    if m.is_inverted { "Y" } else { "N" },
                    m.delta_next_same_id.map(|d| d.to_string()).unwrap_or_default(),
                ));
            }
            match std::fs::write(&path, csv) {
                Ok(_) => app.show_success(format!(
                    "Exported {} matches to {}",
                    filtered.len(),
                    path.file_name().unwrap_or_default().to_string_lossy()
                )),
                Err(e) => app.show_error(format!("Failed to export: {}", e), true),
            }
        }
    }
}

pub fn render_operation_windows(app: &mut BitApp, ctx: &egui::Context) {
    if let Some(op_type) = app.show_operation_menu {
        let title = if app.editing_operation_index.is_some() {
            format!("Edit {}", op_type.name())
        } else {
            format!("Create {}", op_type.name())
        };

        let mut open = true;
        let mut action = EditorAction::None;

        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                // Create editor context with worksheet info
                let editor_ctx = EditorContext::new(&app.worksheets, app.current_worksheet_index);

                // Render the editor and get the action
                if let Some(ref mut editor_state) = app.editor_state {
                    action = editor_state.render(&editor_ctx, ui);
                }
            });

        // Handle editor actions
        match action {
            EditorAction::Save => app.save_current_operation(),
            EditorAction::Cancel => app.cancel_operation_edit(),
            EditorAction::None => {
                if !open {
                    app.cancel_operation_edit();
                }
            }
        }
    }
}

pub fn render_column_editor_window(app: &mut BitApp, ctx: &egui::Context) {
    if app.show_column_editor {
        let mut open = true;
        egui::Window::new("+ Add Protocol Column")
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Define Protocol Column");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Label:");
                    ui.text_edit_singleline(&mut app.column_editor_label);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    ui.label("Start bit:");
                    ui.text_edit_singleline(&mut app.column_editor_bit_start);
                });
                
                ui.horizontal(|ui| {
                    ui.label("End bit:");
                    ui.text_edit_singleline(&mut app.column_editor_bit_end);
                });
                
                ui.add_space(8.0);
                
                ui.label("Color:");
                ui.horizontal(|ui| {
                    ui.label("R:");
                    ui.add(egui::Slider::new(&mut app.column_editor_color[0], 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("G:");
                    ui.add(egui::Slider::new(&mut app.column_editor_color[1], 0..=255));
                });
                ui.horizontal(|ui| {
                    ui.label("B:");
                    ui.add(egui::Slider::new(&mut app.column_editor_color[2], 0..=255));
                });
                
                let color = egui::Color32::from_rgb(
                    app.column_editor_color[0],
                    app.column_editor_color[1],
                    app.column_editor_color[2]
                );
                ui.horizontal(|ui| {
                    ui.label("Preview:");
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(100.0, 30.0),
                        egui::Sense::hover()
                    );
                    ui.painter().rect_filled(rect, 3.0, color);
                });
                
                ui.add_space(8.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Add Column").clicked() {
                        if let (Ok(start), Ok(end)) = (
                            app.column_editor_bit_start.parse::<usize>(),
                            app.column_editor_bit_end.parse::<usize>()
                        ) {
                            if start <= end {
                                let label = if app.column_editor_label.is_empty() {
                                    format!("Column {}", app.byte_viewer.config.columns.len() + 1)
                                } else {
                                    app.column_editor_label.clone()
                                };
                                
                                app.byte_viewer.add_column(
                                    crate::viewers::ByteColumn::new(
                                        label,
                                        start,
                                        end,
                                        app.column_editor_color
                                    )
                                );
                                
                                app.column_editor_label.clear();
                                app.column_editor_bit_start = format!("{}", end + 1);
                                app.column_editor_bit_end = format!("{}", end + 8);
                                app.show_column_editor = false;
                            } else {
                                app.error_message = Some("Start bit must be <= end bit".to_string());
                            }
                        } else {
                            app.error_message = Some("Invalid bit range values".to_string());
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        app.show_column_editor = false;
                    }
                });
            });
        
        if !open {
            app.show_column_editor = false;
        }
    }
}

pub fn render_frame_width_finder_window(app: &mut BitApp, ctx: &egui::Context) {
    use egui_plot::{Line, Plot, PlotPoints, Bar, BarChart, Points};
    
    if !app.show_frame_width_finder {
        return;
    }
    
    // Clone values before entering the window to avoid borrow checker issues
    let analysis = app.frame_width_analysis.clone();
    let mut frame_width_min = app.frame_width_min;
    let mut frame_width_max = app.frame_width_max;
    let mut frame_width_delta = app.frame_width_delta;
    let mut sort_by_score = app.frame_width_sort_by_score;
    let mut selected_width = app.frame_width_selected;
    let mut run_analysis = false;
    let mut apply_width: Option<usize> = None;
    let mut keep_open = true;
    
    egui::Window::new("Find Frame Width")
        .open(&mut keep_open)
        .default_width(800.0)
        .default_height(600.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Automatic Frame Width Detection");
            ui.label("Analyzes bit patterns to detect the most probable frame width");
            ui.separator();
            
            // Controls
            ui.horizontal(|ui| {
                ui.label("Min Width:");
                ui.add(egui::DragValue::new(&mut frame_width_min)
                    .range(1..=1024)
                    .speed(1.0));
                
                ui.add_space(10.0);
                
                ui.label("Max Width:");
                ui.add(egui::DragValue::new(&mut frame_width_max)
                    .range(1..=1024)
                    .speed(1.0));
                
                ui.add_space(10.0);
                
                ui.label("Delta:")
                    .on_hover_text("Warning: Delta measures REPETITION PERIOD, not frame width!\n\n\
                        • Delta = 0: Find frame width by bit position consistency (recommended)\n\
                        • Delta > 0: Find repetition period (e.g., delta=5 finds patterns every 5 frames)\n\
                        • For ASCII/binary frame detection, keep Delta = 0");
                ui.add(egui::DragValue::new(&mut frame_width_delta)
                    .range(0..=100)
                    .speed(1.0));
            });
            
            ui.add_space(5.0);
            
            if ui.button("Analyze").clicked() {
                run_analysis = true;
            }
            
            ui.separator();
            
            // Show results if analysis has been run
            if let Some(ref analysis) = analysis {
                ui.horizontal(|ui| {
                    ui.heading(format!("Best Width: {}", analysis.best_width));
                    ui.label(format!("(score: {:.4})", analysis.best_score));
                    
                    ui.add_space(20.0);
                    
                    if ui.button("Apply Width to Viewer").clicked() {
                        apply_width = Some(analysis.best_width);
                    }
                });
                
                ui.separator();
                    
                // Width scores line chart
                ui.heading("Width Scores");
                ui.label("Higher scores indicate more consistent bit patterns at that width");
                ui.label("Tip: Click on the graph to select a width");
                
                let plot_response = Plot::new("width_scores_plot")
                    .view_aspect(2.5)
                    .legend(egui_plot::Legend::default())
                    .allow_drag(false)  // Disable dragging so clicks are easier
                    .label_formatter(|name, value| {
                        if !name.is_empty() {
                            format!("{}\nWidth: {}\nScore: {:.4}", name, value.x as usize, value.y)
                        } else {
                            format!("Width: {}\nScore: {:.4}", value.x as usize, value.y)
                        }
                    })
                    .show(ui, |plot_ui| {
                        // Convert scores to plot points
                        let points: PlotPoints = analysis.width_scores
                            .iter()
                            .map(|(width, score)| [*width as f64, *score])
                            .collect();
                        
                        plot_ui.line(
                            Line::new("consistency", points)
                                .width(2.0)
                        );
                        
                        // Highlight the best width
                        let best_point = PlotPoints::new(vec![
                            [analysis.best_width as f64, analysis.best_score]
                        ]);
                        
                        plot_ui.points(
                            Points::new("best", best_point)
                                .radius(6.0)
                        );
                    });
                
                // Handle clicks on the plot to select a width
                let mut hover_width: Option<usize> = None;
                if let Some(pointer_pos) = plot_response.response.hover_pos() {
                    let plot_pos = plot_response.transform.value_from_position(pointer_pos);
                    
                    // Find the nearest width to the hover position
                    let hovered_width = plot_pos.x.round() as usize;
                    
                    // Find the closest actual width that was tested
                    if let Some((nearest_width, _)) = analysis.width_scores
                        .iter()
                        .min_by_key(|(w, _)| (*w as i32 - hovered_width as i32).abs())
                    {
                        hover_width = Some(*nearest_width);
                        
                        if plot_response.response.clicked() {
                            apply_width = Some(*nearest_width);
                        }
                    }
                }
                
                // Show hover feedback
                if let Some(width) = hover_width {
                    let score = analysis.width_scores
                        .iter()
                        .find(|(w, _)| *w == width)
                        .map(|(_, s)| *s)
                        .unwrap_or(0.0);
                    ui.label(format!("Hovering: Width {} (score: {:.6}) - Click to apply", width, score));
                }
                
                ui.add_space(10.0);
                    ui.separator();
                    
                    // Bit position consistency for best width
                    ui.heading(format!("Bit Position Consistency (Width {})", analysis.best_width));
                    ui.label("Shows which bit positions have consistent patterns");
                    
                    let best_width_idx = analysis.width_scores
                        .iter()
                        .position(|(w, _)| *w == analysis.best_width)
                        .unwrap_or(0);
                    
                    if best_width_idx < analysis.bit_position_patterns.len() {
                        let bit_patterns = &analysis.bit_position_patterns[best_width_idx];
                        
                        Plot::new("bit_position_heatmap")
                            .view_aspect(3.0)
                            .legend(egui_plot::Legend::default())
                            .label_formatter(|name, value| {
                                if !name.is_empty() {
                                    format!("{}\nBit Position: {}\nConsistency: {:.4}", 
                                        name, value.x as usize, value.y)
                                } else {
                                    format!("Bit Position: {}\nConsistency: {:.4}", 
                                        value.x as usize, value.y)
                                }
                            })
                            .show(ui, |plot_ui| {
                                // Create bars for each bit position
                                let bars: Vec<_> = bit_patterns
                                    .iter()
                                    .enumerate()
                                    .map(|(pos, &score)| {
                                        Bar::new(pos as f64, score)
                                            .width(0.8)
                                    })
                                    .collect();
                                
                                plot_ui.bar_chart(BarChart::new("bit_positions", bars));
                            });
                    }
                    
                    ui.add_space(10.0);
                    ui.separator();
                    
                    // Top 100 candidate widths in a scrollable, sortable table
                    ui.heading("Candidate Widths");
                    
                    let mut sorted_widths = analysis.width_scores.clone();
                    
                    // Apply current sort
                    if sort_by_score {
                        sorted_widths.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                    } else {
                        sorted_widths.sort_by_key(|(w, _)| *w);
                    }
                    
                    use egui_extras::{TableBuilder, Column};
                    
                    TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::auto().at_least(80.0))  // Width
                        .column(Column::remainder())             // Score
                        .header(25.0, |mut header| {
                            header.col(|ui| {
                                if ui.selectable_label(!sort_by_score, "Width").clicked() {
                                    sort_by_score = false;
                                }
                            });
                            header.col(|ui| {
                                if ui.selectable_label(sort_by_score, "Score").clicked() {
                                    sort_by_score = true;
                                }
                            });
                        })
                        .body(|body| {
                            body.rows(20.0, sorted_widths.len().min(100), |mut row| {
                                let row_idx = row.index();
                                let (width, score) = sorted_widths[row_idx];
                                let is_selected = selected_width == Some(width);
                                
                                row.col(|ui| {
                                    let (rect, response) = ui.allocate_exact_size(
                                        ui.available_size(),
                                        egui::Sense::click()
                                    );
                                    
                                    if is_selected {
                                        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(60, 100, 180));
                                    }
                                    
                                    ui.painter().text(
                                        rect.left_center() + egui::vec2(5.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        format!("{}", width),
                                        egui::FontId::default(),
                                        if is_selected { egui::Color32::WHITE } else { ui.visuals().text_color() }
                                    );
                                    
                                    if response.clicked() {
                                        apply_width = Some(width);
                                        selected_width = Some(width);
                                    }
                                });
                                
                                row.col(|ui| {
                                    let (rect, response) = ui.allocate_exact_size(
                                        ui.available_size(),
                                        egui::Sense::click()
                                    );
                                    
                                    if is_selected {
                                        ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(60, 100, 180));
                                    }
                                    
                                    ui.painter().text(
                                        rect.left_center() + egui::vec2(5.0, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        format!("{:.6}", score),
                                        egui::FontId::default(),
                                        if is_selected { egui::Color32::WHITE } else { ui.visuals().text_color() }
                                    );
                                    
                                    if response.clicked() {
                                        apply_width = Some(width);
                                        selected_width = Some(width);
                                    }
                                });
                            });
                        });
                } else {
                    ui.label("Click 'Analyze' to detect frame width");
                }
            });
    
    // Update app state from window
    app.show_frame_width_finder = keep_open;
    app.frame_width_min = frame_width_min;
    app.frame_width_max = frame_width_max;
    app.frame_width_delta = frame_width_delta;
    app.frame_width_sort_by_score = sort_by_score;
    app.frame_width_selected = selected_width;
    
    // Run analysis if requested
    if run_analysis {
        app.run_frame_width_analysis();
    }
    
    // Apply width after window to avoid borrow issues
    if let Some(width) = apply_width {
        app.viewer.frame_length = width;
        app.update_viewer();
    }
}

pub fn render_goto_offset_dialog(app: &mut BitApp, ctx: &egui::Context) {
    if !app.show_goto_offset_dialog {
        return;
    }

    let mut open = true;
    let mut apply = false;

    egui::Window::new("Go to Offset")
        .open(&mut open)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Format:");
                ui.selectable_value(
                    &mut app.goto_offset_format,
                    crate::app::GotoOffsetFormat::Bit,
                    "Bit",
                );
                ui.selectable_value(
                    &mut app.goto_offset_format,
                    crate::app::GotoOffsetFormat::Byte,
                    "Byte (dec)",
                );
                ui.selectable_value(
                    &mut app.goto_offset_format,
                    crate::app::GotoOffsetFormat::Hex,
                    "Byte (hex)",
                );
            });

            ui.horizontal(|ui| {
                ui.label("Offset:");
                let response = ui.text_edit_singleline(&mut app.goto_offset_input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    apply = true;
                }
            });

            ui.add_space(5.0);

            ui.horizontal(|ui| {
                if ui.button("Go").clicked() {
                    apply = true;
                }
                if ui.button("Cancel").clicked() {
                    app.show_goto_offset_dialog = false;
                }
            });
        });

    if !open {
        app.show_goto_offset_dialog = false;
    }

    if apply {
        apply_goto_offset(app);
    }
}

fn apply_goto_offset(app: &mut BitApp) {
    use crate::app::GotoOffsetFormat;

    let input = app.goto_offset_input.trim();
    let bit_position = match app.goto_offset_format {
        GotoOffsetFormat::Bit => input.parse::<usize>().ok(),
        GotoOffsetFormat::Byte => input.parse::<usize>().ok().map(|b| b * 8),
        GotoOffsetFormat::Hex => usize::from_str_radix(
            input.trim_start_matches("0x").trim_start_matches("0X"),
            16,
        )
        .ok()
        .map(|b| b * 8),
    };

    if let Some(pos) = bit_position {
        app.view_bit_offset = pos;
        app.show_goto_offset_dialog = false;
    } else {
        app.show_toast(
            "Invalid offset value".to_string(),
            crate::app::ToastType::Warning,
        );
    }
}

