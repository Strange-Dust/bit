// Window rendering functions for pattern locator and operation editors

use crate::analysis::{Pattern, PatternFormat};
use crate::app::BitApp;
use crate::operations::{EditorAction, EditorContext};
use eframe::egui;

pub fn render_pattern_locator_window(app: &mut BitApp, ctx: &egui::Context) {
    if !app.show_pattern_locator {
        return;
    }

    const PAGE_SIZE: usize = 100;

    // Collect deferred actions to avoid borrow conflicts
    let mut do_search = false;
    let mut navigate_to: Option<(usize, usize)> = None; // (position, pattern_bits_len)
    let mut select_and_search_pattern: Option<usize> = None;
    let mut remove_pattern: Option<usize> = None;

    egui::Window::new("Pattern Locator")
        .open(&mut app.show_pattern_locator)
        .default_width(480.0)
        .default_height(550.0)
        .resizable(true)
        .collapsible(false)
        .show(ctx, |ui| {
            // ── Section 1: Search Form ──
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Format:");
                    ui.selectable_value(&mut app.pattern_format, PatternFormat::Bits, "Bits");
                    ui.selectable_value(&mut app.pattern_format, PatternFormat::Hex, "Hex");
                    ui.selectable_value(&mut app.pattern_format, PatternFormat::Ascii, "ASCII");
                });

                ui.horizontal(|ui| {
                    ui.label("Pattern:");
                    let response = ui.text_edit_singleline(&mut app.pattern_input);
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        do_search = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Garbles:");
                    ui.add(egui::Slider::new(&mut app.pattern_garbles, 0..=16));
                });

                ui.horizontal(|ui| {
                    if ui.button("Search").clicked() {
                        do_search = true;
                    }
                    if ui.button("Clear").clicked() {
                        app.pattern_input.clear();
                        app.pattern_garbles = 0;
                    }
                });
            });

            // ── Section 2: Saved Patterns ──
            if !app.patterns.is_empty() {
                ui.add_space(2.0);
                egui::CollapsingHeader::new(format!("Saved Patterns ({})", app.patterns.len()))
                    .default_open(true)
                    .show(ui, |ui| {
                        for (idx, pattern) in app.patterns.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let selected = app.selected_pattern == Some(idx);
                                let label_text = format!(
                                    "{}    {} matches",
                                    pattern.name,
                                    pattern.matches.len()
                                );
                                if ui.selectable_label(selected, label_text).clicked() {
                                    select_and_search_pattern = Some(idx);
                                }

                                if ui.small_button("x").clicked() {
                                    remove_pattern = Some(idx);
                                }
                            });
                        }
                    });
            }

            // ── Section 3: Results ──
            if let Some(pattern_idx) = app.selected_pattern {
                if pattern_idx < app.patterns.len() {
                    let pattern = &app.patterns[pattern_idx];
                    let match_count = pattern.matches.len();

                    if match_count > 0 {
                        ui.separator();
                        ui.group(|ui| {
                            ui.strong(format!("\"{}\" -- {} matches", pattern.name, match_count));

                            // Navigation bar: Prev / Match N of M / Next
                            ui.horizontal(|ui| {
                                if ui.button("<< Prev").clicked() {
                                    let new_idx = match app.current_match_index {
                                        Some(i) if i > 0 => i - 1,
                                        Some(_) => match_count - 1,
                                        None => match_count - 1,
                                    };
                                    app.current_match_index = Some(new_idx);
                                    app.match_page = new_idx / PAGE_SIZE;
                                    let m = &pattern.matches[new_idx];
                                    navigate_to = Some((m.position, pattern.bits.len()));
                                }

                                let display_idx = app.current_match_index.map(|i| i + 1).unwrap_or(0);
                                ui.label(format!("Match {} of {}", display_idx, match_count));

                                if ui.button("Next >>").clicked() {
                                    let new_idx = match app.current_match_index {
                                        Some(i) if i + 1 < match_count => i + 1,
                                        Some(_) => 0,
                                        None => 0,
                                    };
                                    app.current_match_index = Some(new_idx);
                                    app.match_page = new_idx / PAGE_SIZE;
                                    let m = &pattern.matches[new_idx];
                                    navigate_to = Some((m.position, pattern.bits.len()));
                                }
                            });

                            ui.horizontal(|ui| {
                                if ui.button("Highlight All").clicked() {
                                    app.viewer.clear_highlights();
                                    for m in &pattern.matches {
                                        app.viewer.add_highlight_range(m.position, pattern.bits.len());
                                    }
                                }
                                if ui.button("Clear Highlights").clicked() {
                                    app.viewer.clear_highlights();
                                }
                            });

                            ui.separator();

                            // Paginated table
                            let total_pages = (match_count + PAGE_SIZE - 1) / PAGE_SIZE;
                            let page = app.match_page.min(total_pages.saturating_sub(1));
                            let page_start = page * PAGE_SIZE;
                            let page_end = (page_start + PAGE_SIZE).min(match_count);

                            // Pagination controls
                            if total_pages > 1 {
                                ui.horizontal(|ui| {
                                    ui.add_enabled_ui(page > 0, |ui| {
                                        if ui.small_button("<<").clicked() {
                                            app.match_page = 0;
                                        }
                                        if ui.small_button("<").clicked() {
                                            app.match_page = page.saturating_sub(1);
                                        }
                                    });

                                    ui.label(format!(
                                        "Page {} of {}  (#{} - #{})",
                                        page + 1,
                                        total_pages,
                                        page_start + 1,
                                        page_end,
                                    ));

                                    ui.add_enabled_ui(page + 1 < total_pages, |ui| {
                                        if ui.small_button(">").clicked() {
                                            app.match_page = (page + 1).min(total_pages - 1);
                                        }
                                        if ui.small_button(">>").clicked() {
                                            app.match_page = total_pages - 1;
                                        }
                                    });
                                });
                            }

                            // Table
                            use egui_extras::{TableBuilder, Column};
                            let current_idx = app.current_match_index;
                            let bits_len = pattern.bits.len();
                            let has_garbles = pattern.garbles > 0;

                            let mut table = TableBuilder::new(ui)
                                .striped(true)
                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                .column(Column::auto().at_least(35.0))   // #
                                .column(Column::auto().at_least(80.0))   // Position
                                .column(Column::auto().at_least(70.0));  // Delta
                            if has_garbles {
                                table = table.column(Column::remainder().at_least(50.0)); // Garbles
                            } else {
                                table = table.column(Column::remainder()); // spacer
                            }

                            table
                                .header(20.0, |mut header| {
                                    header.col(|ui| { ui.strong("#"); });
                                    header.col(|ui| { ui.strong("Position"); });
                                    header.col(|ui| { ui.strong("Delta"); });
                                    if has_garbles {
                                        header.col(|ui| { ui.strong("Garbles"); });
                                    } else {
                                        header.col(|_| {});
                                    }
                                })
                                .body(|body| {
                                    let row_count = page_end - page_start;
                                    body.rows(20.0, row_count, |mut row| {
                                        let match_idx = page_start + row.index();
                                        let m = &pattern.matches[match_idx];
                                        let is_current = current_idx == Some(match_idx);

                                        if is_current {
                                            row.set_selected(true);
                                        }

                                        row.col(|ui| {
                                            if ui.selectable_label(is_current, format!("{}", match_idx + 1)).clicked() {
                                                app.current_match_index = Some(match_idx);
                                                navigate_to = Some((m.position, bits_len));
                                            }
                                        });
                                        row.col(|ui| {
                                            if ui.selectable_label(is_current, format!("{}", m.position)).clicked() {
                                                app.current_match_index = Some(match_idx);
                                                navigate_to = Some((m.position, bits_len));
                                            }
                                        });
                                        row.col(|ui| {
                                            let text = m.delta.map(|d| format!("{}", d)).unwrap_or_default();
                                            if ui.selectable_label(is_current, text).clicked() {
                                                app.current_match_index = Some(match_idx);
                                                navigate_to = Some((m.position, bits_len));
                                            }
                                        });
                                        row.col(|ui| {
                                            if has_garbles {
                                                let text = if m.mismatches > 0 { format!("{}", m.mismatches) } else { String::new() };
                                                if ui.selectable_label(is_current, text).clicked() {
                                                    app.current_match_index = Some(match_idx);
                                                    navigate_to = Some((m.position, bits_len));
                                                }
                                            }
                                        });
                                    });
                                });
                        });
                    } else {
                        ui.separator();
                        ui.label(format!("\"{}\" -- 0 matches", pattern.name));
                    }
                }
            }
        });

    // ── Deferred actions ──

    // Handle search button / Enter
    if do_search && !app.pattern_input.is_empty() {
        let name = format!("Pattern {}", app.patterns.len() + 1);
        match Pattern::new(name, app.pattern_format, app.pattern_input.clone(), app.pattern_garbles) {
            Ok(mut pattern) => {
                let bits_to_search = if app.show_original {
                    &app.original_bits
                } else {
                    &app.processed_bits
                };
                pattern.search(bits_to_search);
                app.patterns.push(pattern);
                let new_idx = app.patterns.len() - 1;
                app.selected_pattern = Some(new_idx);
                app.current_match_index = None;
                app.match_page = 0;
                app.error_message = None;
            }
            Err(e) => {
                app.error_message = Some(format!("Invalid pattern: {}", e));
            }
        }
    }

    // Handle select + search from saved patterns list
    if let Some(idx) = select_and_search_pattern {
        let bits_to_search = if app.show_original {
            &app.original_bits
        } else {
            &app.processed_bits
        };
        app.patterns[idx].search(bits_to_search);
        app.selected_pattern = Some(idx);
        app.current_match_index = None;
        app.match_page = 0;
    }

    // Handle pattern removal
    if let Some(idx) = remove_pattern {
        app.patterns.remove(idx);
        if app.selected_pattern == Some(idx) {
            app.selected_pattern = None;
            app.current_match_index = None;
            app.match_page = 0;
        } else if let Some(sel) = app.selected_pattern {
            if sel > idx {
                app.selected_pattern = Some(sel - 1);
            }
        }
        app.viewer.clear_highlights();
    }

    // Handle navigation (cross-view fix: set both bit_offset AND jump_to_position)
    if let Some((position, bits_len)) = navigate_to {
        app.viewer.clear_highlights();
        app.viewer.add_highlight_range(position, bits_len);
        app.view_bit_offset = position;
        app.viewer.jump_to_position(position);
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

