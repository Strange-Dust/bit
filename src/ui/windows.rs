// Window rendering functions for pattern locator and operation editors

use crate::analysis::{Pattern, PatternFormat};
use crate::app::BitApp;
use crate::core::OperationType;
use crate::utils::eval_expression;
use eframe::egui;

pub fn render_pattern_locator_window(app: &mut BitApp, ctx: &egui::Context) {
    if app.show_pattern_locator {
        egui::Window::new("🔍 Pattern Locator")
            .open(&mut app.show_pattern_locator)
            .default_width(450.0)
            .default_height(700.0)
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.heading("Search for Bit Patterns");
                        ui.separator();
                        
                        ui.group(|ui| {
                            ui.heading("Add Pattern");
                            
                            ui.horizontal(|ui| {
                                ui.label("Name:");
                                ui.text_edit_singleline(&mut app.pattern_name_input);
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Format:");
                                ui.selectable_value(&mut app.pattern_format, PatternFormat::Bits, "Bits");
                                ui.selectable_value(&mut app.pattern_format, PatternFormat::Hex, "Hex");
                                ui.selectable_value(&mut app.pattern_format, PatternFormat::Ascii, "ASCII");
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Pattern:");
                                ui.text_edit_singleline(&mut app.pattern_input);
                            });
                            
                            ui.horizontal(|ui| {
                                ui.label("Garbles:");
                                ui.add(egui::Slider::new(&mut app.pattern_garbles, 0..=16));
                            });
                            
                            ui.horizontal(|ui| {
                                if ui.button("➕ Add Pattern").clicked() {
                                    let name = if app.pattern_name_input.is_empty() {
                                        format!("Pattern {}", app.patterns.len() + 1)
                                    } else {
                                        app.pattern_name_input.clone()
                                    };
                                    
                                    match Pattern::new(name, app.pattern_format, app.pattern_input.clone(), app.pattern_garbles) {
                                        Ok(pattern) => {
                                            app.patterns.push(pattern);
                                            app.pattern_name_input.clear();
                                            app.pattern_input.clear();
                                            app.error_message = None;
                                        }
                                        Err(e) => {
                                            app.error_message = Some(format!("Invalid pattern: {}", e));
                                        }
                                    }
                                }
                                
                                if ui.button("🔄 Clear").clicked() {
                                    app.pattern_name_input.clear();
                                    app.pattern_input.clear();
                                    app.pattern_garbles = 0;
                                }
                            });
                        });
                        
                        ui.separator();
                        ui.heading("Patterns");
                        
                        if app.patterns.is_empty() {
                            ui.label("No patterns added yet");
                        } else {
                            let mut to_remove = None;
                            let mut to_search = None;
                            
                            for (idx, pattern) in app.patterns.iter().enumerate() {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let selected = app.selected_pattern == Some(idx);
                                        if ui.selectable_label(selected, &pattern.name).clicked() {
                                            app.selected_pattern = Some(idx);
                                        }
                                        
                                        if ui.button("🔍 Search").clicked() {
                                            to_search = Some(idx);
                                        }
                                        
                                        if ui.button("❌").clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                    
                                    ui.label(format!("Pattern: {}", pattern.input));
                                    ui.label(format!("Garbles: {} | Matches: {}", pattern.garbles, pattern.matches.len()));
                                });
                            }
                            
                            if let Some(idx) = to_remove {
                                app.patterns.remove(idx);
                                if app.selected_pattern == Some(idx) {
                                    app.selected_pattern = None;
                                }
                            }
                            
                            if let Some(idx) = to_search {
                                let bits_to_search = if app.show_original {
                                    &app.original_bits
                                } else {
                                    &app.processed_bits
                                };
                                app.patterns[idx].search(bits_to_search);
                                app.selected_pattern = Some(idx);
                            }
                        }
                        
                        ui.separator();
                        
                        if let Some(pattern_idx) = app.selected_pattern {
                            if pattern_idx < app.patterns.len() {
                                let pattern = &app.patterns[pattern_idx];
                                
                                ui.heading(format!("Results for '{}'", pattern.name));
                                ui.label(format!("Found {} matches", pattern.matches.len()));
                                
                                if !pattern.matches.is_empty() {
                                    ui.horizontal(|ui| {
                                        if ui.button("🎯 Highlight All").clicked() {
                                            app.viewer.clear_highlights();
                                            for m in &pattern.matches {
                                                app.viewer.add_highlight_range(m.position, pattern.bits.len());
                                            }
                                        }
                                        
                                        if ui.button("🔲 Clear Highlights").clicked() {
                                            app.viewer.clear_highlights();
                                        }
                                    });
                                    
                                    ui.separator();
                                    
                                    egui::ScrollArea::vertical()
                                        .max_height(300.0)
                                        .show(ui, |ui| {
                                            for (idx, m) in pattern.matches.iter().enumerate() {
                                                ui.horizontal(|ui| {
                                                    if ui.button(format!("#{}", idx + 1)).clicked() {
                                                        app.viewer.clear_highlights();
                                                        app.viewer.add_highlight_range(m.position, pattern.bits.len());
                                                        app.viewer.jump_to_position(m.position);
                                                    }
                                                    
                                                    ui.label(format!("@{}", m.position));
                                                    
                                                    if let Some(delta) = m.delta {
                                                        ui.label(format!("Δ{}", delta));
                                                    }
                                                    
                                                    if m.mismatches > 0 {
                                                        ui.label(format!("~{}", m.mismatches));
                                                    }
                                                });
                                            }
                                        });
                                }
                            }
                        }
                    });
            });
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
        egui::Window::new(title)
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                match op_type {
                    OperationType::LoadFile => render_loadfile_editor(app, ui),
                    OperationType::TakeSkipSequence => render_takeskip_editor(app, ui),
                    OperationType::InvertBits => render_invert_editor(app, ui),
                    OperationType::MultiWorksheetLoad => render_multiworksheet_editor(app, ui),
                    OperationType::TruncateBits => render_truncate_editor(app, ui),
                    OperationType::InterleaveBits => render_interleave_editor(app, ui),
                }
            });
        
        if !open {
            app.cancel_operation_edit();
        }
    }
}

fn render_loadfile_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Load File");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.loadfile_name);
    });
    
    ui.add_space(8.0);
    
    if let Some(path) = &app.loadfile_path {
        ui.label(format!("📄 Selected: {}", path.display()));
    } else {
        ui.label("No file selected");
    }
    
    if ui.button("📂 Browse...").clicked() {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            app.loadfile_path = Some(path);
        }
    }
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

fn render_takeskip_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Take/Skip Sequence");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.takeskip_name);
    });
    
    ui.add_space(8.0);
    
    ui.label("Enter a sequence of operations:");
    ui.label("• t = take N bits");
    ui.label("• r = reverse N bits");
    ui.label("• i = invert N bits");
    ui.label("• s = skip N bits");
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        ui.label("Sequence:");
        let response = ui.text_edit_singleline(&mut app.takeskip_input);
        
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            app.save_current_operation();
        }
    });
    
    ui.label("Example: t4r3i8s1");
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

fn render_invert_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Invert All Bits");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.invert_name);
    });
    
    ui.add_space(8.0);
    
    ui.label("This operation will invert all bits:");
    ui.label("• 0 → 1");
    ui.label("• 1 → 0");
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

fn render_truncate_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Truncate Bits");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.truncate_name);
    });
    
    ui.add_space(8.0);
    
    ui.label("Specify the range of bits to keep:");
    ui.add_space(4.0);
    
    ui.horizontal(|ui| {
        ui.label("Start (inclusive):");
        let start_response = ui.text_edit_singleline(&mut app.truncate_start);
        
        // Evaluate math expression on Enter key
        if start_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if let Ok(result) = eval_expression(&app.truncate_start) {
                app.truncate_start = result.to_string();
            }
        }
    });
    
    ui.horizontal(|ui| {
        ui.label("End (exclusive):  ");
        let end_response = ui.text_edit_singleline(&mut app.truncate_end);
        
        // Evaluate math expression on Enter key
        if end_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            if !app.truncate_end.is_empty() {
                if let Ok(result) = eval_expression(&app.truncate_end) {
                    app.truncate_end = result.to_string();
                }
            }
        }
    });
    
    ui.add_space(4.0);
    ui.label("💡 Tips:");
    ui.label("• Leave end empty to keep until the end");
    ui.label("• You can use math: 8*8, 100+50, 200-10, 64/2");
    ui.label("• Example: Start=0, End=250 keeps bits 0-249");
    ui.label("• Example: Start=100, End=250 keeps bits 100-249");
    ui.label("• Example: Start=0, End=empty keeps all bits from 0");
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

fn render_multiworksheet_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Multi-Worksheet Load");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.multiworksheet_name);
    });
    
    ui.add_space(8.0);
    
    ui.group(|ui| {
        ui.label("Add Worksheet Operation:");
        
        ui.horizontal(|ui| {
            ui.label("Worksheet:");
            egui::ComboBox::from_id_salt("worksheet_selector")
                .selected_text(if app.multiworksheet_selected_worksheet < app.worksheets.len() {
                    &app.worksheets[app.multiworksheet_selected_worksheet].name
                } else {
                    "Select..."
                })
                .show_ui(ui, |ui| {
                    for (idx, worksheet) in app.worksheets.iter().enumerate() {
                        if idx != app.current_worksheet_index {
                            ui.selectable_value(&mut app.multiworksheet_selected_worksheet, idx, &worksheet.name);
                        }
                    }
                });
        });
        
        ui.horizontal(|ui| {
            ui.label("Sequence:");
            ui.text_edit_singleline(&mut app.multiworksheet_input);
        });
        
        if ui.button("➕ Add").clicked() {
            if !app.multiworksheet_input.is_empty() {
                app.multiworksheet_ops.push((
                    app.multiworksheet_selected_worksheet,
                    app.multiworksheet_input.clone()
                ));
                app.multiworksheet_input.clear();
            }
        }
    });
    
    ui.add_space(8.0);
    
    if app.multiworksheet_ops.is_empty() {
        ui.label("No worksheets added yet");
    } else {
        let mut to_remove = None;
        for (idx, (ws_idx, seq)) in app.multiworksheet_ops.iter().enumerate() {
            ui.horizontal(|ui| {
                let ws_name = if *ws_idx < app.worksheets.len() {
                    &app.worksheets[*ws_idx].name
                } else {
                    "Unknown"
                };
                ui.label(format!("{}. {} → {}", idx + 1, ws_name, seq));
                if ui.button("❌").clicked() {
                    to_remove = Some(idx);
                }
            });
        }
        
        if let Some(idx) = to_remove {
            app.multiworksheet_ops.remove(idx);
        }
    }
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

pub fn render_column_editor_window(app: &mut BitApp, ctx: &egui::Context) {
    if app.show_column_editor {
        let mut open = true;
        egui::Window::new("➕ Add Protocol Column")
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
                    if ui.button("✓ Add Column").clicked() {
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
                    
                    if ui.button("✗ Cancel").clicked() {
                        app.show_column_editor = false;
                    }
                });
            });
        
        if !open {
            app.show_column_editor = false;
        }
    }
}

fn render_interleave_editor(app: &mut BitApp, ui: &mut egui::Ui) {
    ui.heading("Bit Interleaving / De-interleaving");
    ui.separator();
    
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut app.interleave_name);
    });
    
    ui.add_space(8.0);
    
    // Type selection
    ui.horizontal(|ui| {
        ui.label("Interleaver Type:");
        ui.radio_value(&mut app.interleave_type, crate::processing::InterleaverType::Block, "Block");
        ui.radio_value(&mut app.interleave_type, crate::processing::InterleaverType::Convolutional, "Convolutional");
        ui.radio_value(&mut app.interleave_type, crate::processing::InterleaverType::Symbol, "Symbol");
    });
    
    ui.add_space(4.0);
    
    // Direction selection
    ui.horizontal(|ui| {
        ui.label("Direction:");
        ui.radio_value(&mut app.interleave_direction, crate::processing::InterleaverDirection::Interleave, "Interleave");
        ui.radio_value(&mut app.interleave_direction, crate::processing::InterleaverDirection::Deinterleave, "De-interleave");
    });
    
    ui.add_space(8.0);
    ui.separator();
    
    // Parameters based on type
    match app.interleave_type {
        crate::processing::InterleaverType::Block => {
            ui.label("📦 Block Interleaver Parameters:");
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Block Size (columns):");
                let block_response = ui.text_edit_singleline(&mut app.interleave_block_size);
                
                if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_block_size) {
                        app.interleave_block_size = result.to_string();
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Depth (rows):      ");
                let depth_response = ui.text_edit_singleline(&mut app.interleave_depth);
                
                if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_depth) {
                        app.interleave_depth = result.to_string();
                    }
                }
            });
            
            ui.add_space(8.0);
            
            // Visual preview
            ui.group(|ui| {
                ui.label("📊 Visual Preview:");
                ui.add_space(4.0);
                
                if let (Ok(block_size), Ok(depth)) = (
                    app.interleave_block_size.parse::<usize>(),
                    app.interleave_depth.parse::<usize>()
                ) {
                    if block_size > 0 && depth > 0 && block_size <= 16 && depth <= 16 {
                        ui.label(format!("Matrix: {}×{} = {} bits per block", block_size, depth, block_size * depth));
                        ui.add_space(4.0);
                        
                        match app.interleave_direction {
                            crate::processing::InterleaverDirection::Interleave => {
                                ui.label("Write row-wise → Read column-wise:");
                                ui.add_space(2.0);
                                render_block_matrix_preview(ui, block_size, depth);
                            }
                            crate::processing::InterleaverDirection::Deinterleave => {
                                ui.label("Write column-wise → Read row-wise:");
                                ui.add_space(2.0);
                                render_block_matrix_preview(ui, block_size, depth);
                            }
                        }
                    } else {
                        ui.label("⚠ Invalid dimensions (max 16×16)");
                    }
                } else {
                    ui.label("⚠ Enter valid numbers for preview");
                }
            });
            
            ui.add_space(4.0);
            ui.label("💡 Tips:");
            ui.label("• Block interleaver rearranges data in matrix blocks");
            ui.label("• Interleave: Write row-wise, read column-wise");
            ui.label("• De-interleave: Reverses the process");
            ui.label("• Example: 8×4 matrix handles 32 bits at a time");
            ui.label("• Math supported: 2*4, 16/2, 8+4, etc.");
        }
        
        crate::processing::InterleaverType::Convolutional => {
            ui.label("🔄 Convolutional Interleaver Parameters:");
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Branches (B):       ");
                let branches_response = ui.text_edit_singleline(&mut app.interleave_branches);
                
                if branches_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_branches) {
                        app.interleave_branches = result.to_string();
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Delay Increment (M):");
                let delay_response = ui.text_edit_singleline(&mut app.interleave_delay_increment);
                
                if delay_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_delay_increment) {
                        app.interleave_delay_increment = result.to_string();
                    }
                }
            });
            
            ui.add_space(8.0);
            
            // Visual preview
            ui.group(|ui| {
                ui.label("📊 Visual Preview:");
                ui.add_space(4.0);
                
                if let (Ok(branches), Ok(delay_inc)) = (
                    app.interleave_branches.parse::<usize>(),
                    app.interleave_delay_increment.parse::<usize>()
                ) {
                    if branches > 0 && branches <= 16 && delay_inc > 0 && delay_inc <= 16 {
                        ui.label(format!("Branches: {}, Delay Increment: {}", branches, delay_inc));
                        ui.add_space(4.0);
                        
                        match app.interleave_direction {
                            crate::processing::InterleaverDirection::Interleave => {
                                ui.label("Round-robin distribution with delay:");
                                ui.add_space(2.0);
                                render_convolutional_preview(ui, branches, delay_inc);
                            }
                            crate::processing::InterleaverDirection::Deinterleave => {
                                ui.label("Reverse round-robin with delay:");
                                ui.add_space(2.0);
                                render_convolutional_preview(ui, branches, delay_inc);
                            }
                        }
                    } else {
                        ui.label("⚠ Invalid parameters (max B=16, M=16)");
                    }
                } else {
                    ui.label("⚠ Enter valid numbers for preview");
                }
            });
            
            ui.add_space(4.0);
            ui.label("💡 Tips:");
            ui.label("• Convolutional uses delay lines for each branch");
            ui.label("• Branch i has delay = i × M symbols");
            ui.label("• Distributes bits round-robin across branches");
            ui.label("• Example: B=4, M=1 → delays [0,1,2,3]");
            ui.label("• Provides time-diversity for burst errors");
        }
        
        crate::processing::InterleaverType::Symbol => {
            ui.label("🔤 Symbol Interleaver Parameters:");
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                ui.label("Symbol Size (bits):");
                let symbol_response = ui.text_edit_singleline(&mut app.interleave_symbol_size);
                
                if symbol_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_symbol_size) {
                        app.interleave_symbol_size = result.to_string();
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Block Size (columns):");
                let block_response = ui.text_edit_singleline(&mut app.interleave_block_size);
                
                if block_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_block_size) {
                        app.interleave_block_size = result.to_string();
                    }
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Depth (rows):       ");
                let depth_response = ui.text_edit_singleline(&mut app.interleave_depth);
                
                if depth_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Ok(result) = eval_expression(&app.interleave_depth) {
                        app.interleave_depth = result.to_string();
                    }
                }
            });
            
            ui.add_space(8.0);
            
            ui.label("💡 Tips:");
            ui.label("• Symbol interleaver treats multi-bit symbols as atomic units");
            ui.label("• Common symbol sizes: 8 (bytes), 4 (nibbles), 16 (words)");
            ui.label("• Use for AABBCCDD → ABCDABCD transformations");
            ui.label("• Matrix: Write symbols row-wise, read column-wise");
            ui.label("• Example: Symbol=8, Block=2, Depth=4 → AABBCCDD → ABCDABCD");
        }
    }
    
    ui.add_space(8.0);
    
    ui.horizontal(|ui| {
        if ui.button("✓ Save").clicked() {
            app.save_current_operation();
        }
        
        if ui.button("✗ Cancel").clicked() {
            app.cancel_operation_edit();
        }
    });
}

fn render_block_matrix_preview(ui: &mut egui::Ui, cols: usize, rows: usize) {
    use egui::{Color32, Rect, Stroke};
    
    let cell_size = 24.0;
    let spacing = 2.0;
    let total_width = cols as f32 * (cell_size + spacing);
    let total_height = rows as f32 * (cell_size + spacing);
    
    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover()
    );
    
    let base_pos = response.rect.min;
    
    for row in 0..rows.min(8) {
        for col in 0..cols.min(8) {
            let x = base_pos.x + col as f32 * (cell_size + spacing);
            let y = base_pos.y + row as f32 * (cell_size + spacing);
            
            let rect = Rect::from_min_size(
                egui::pos2(x, y),
                egui::vec2(cell_size, cell_size)
            );
            
            let bit_index = row * cols + col;
            let color = Color32::from_rgb(
                (100 + bit_index * 15).min(255) as u8,
                (150 - bit_index * 5).max(50) as u8,
                200
            );
            
            painter.rect_filled(rect, 2.0, color);
            painter.rect_stroke(
                rect,
                2.0,
                Stroke::new(1.0, Color32::from_gray(80)),
                egui::epaint::StrokeKind::Outside
            );
            
            let text = format!("{}", bit_index);
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                text,
                egui::FontId::monospace(10.0),
                Color32::WHITE
            );
        }
    }
    
    if rows > 8 || cols > 8 {
        ui.label("  (showing first 8×8)");
    }
}

fn render_convolutional_preview(ui: &mut egui::Ui, branches: usize, delay_inc: usize) {
    use egui::{Color32, Rect, Stroke};
    
    let branch_width = 80.0;
    let branch_height = 30.0;
    let spacing = 10.0;
    
    let total_width = branch_width + 150.0;
    let total_height = branches.min(8) as f32 * (branch_height + spacing);
    
    let (response, painter) = ui.allocate_painter(
        egui::vec2(total_width, total_height),
        egui::Sense::hover()
    );
    
    let base_pos = response.rect.min;
    
    for i in 0..branches.min(8) {
        let y = base_pos.y + i as f32 * (branch_height + spacing);
        let delay = i * delay_inc;
        
        // Draw branch box
        let rect = Rect::from_min_size(
            egui::pos2(base_pos.x, y),
            egui::vec2(branch_width, branch_height)
        );
        
        let color = Color32::from_rgb(
            (100 + i * 20).min(255) as u8,
            (150 - i * 10).max(50) as u8,
            200
        );
        
        painter.rect_filled(rect, 3.0, color);
        painter.rect_stroke(
            rect,
            3.0,
            Stroke::new(1.5, Color32::from_gray(80)),
            egui::epaint::StrokeKind::Outside
        );
        
        let text = format!("Branch {}", i);
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(11.0),
            Color32::WHITE
        );
        
        // Draw delay indicator
        let delay_text = format!("Delay: {} symbols", delay);
        painter.text(
            egui::pos2(base_pos.x + branch_width + 15.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            delay_text,
            egui::FontId::monospace(10.0),
            ui.style().visuals.text_color()
        );
    }
    
    if branches > 8 {
        ui.label("  (showing first 8 branches)");
    }
}

