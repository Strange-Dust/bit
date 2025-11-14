mod bit_viewer;
mod file_io;
mod operations;

use bit_viewer::{BitShape, BitViewer};
use bitvec::prelude::*;
use eframe::egui;
use file_io::{read_file_as_bits, write_bits_to_file};
use operations::OperationSequence;
use std::path::PathBuf;

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

struct BitApp {
    original_bits: BitVec<u8, Msb0>,
    processed_bits: BitVec<u8, Msb0>,
    viewer: BitViewer,
    operation_sequences: Vec<OperationSequence>,
    current_operation_input: String,
    current_file_path: Option<PathBuf>,
    error_message: Option<String>,
    dragging_index: Option<usize>,
    show_original: bool,
    show_settings: bool,
    font_size: f32,
}

impl Default for BitApp {
    fn default() -> Self {
        Self {
            original_bits: BitVec::new(),
            processed_bits: BitVec::new(),
            viewer: BitViewer::new(),
            operation_sequences: Vec::new(),
            current_operation_input: String::new(),
            current_file_path: None,
            error_message: None,
            dragging_index: None,
            show_original: true,
            show_settings: false,
            font_size: 14.0,
        }
    }
}

impl BitApp {
    fn load_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_file() {
            match read_file_as_bits(&path) {
                Ok(bits) => {
                    self.original_bits = bits.clone();
                    self.processed_bits = bits;
                    self.current_file_path = Some(path);
                    self.error_message = None;
                    self.update_viewer();
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to load file: {}", e));
                }
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let bits_to_save = if self.show_original {
                &self.original_bits
            } else {
                &self.processed_bits
            };

            match write_bits_to_file(&path, bits_to_save) {
                Ok(_) => {
                    self.error_message = None;
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save file: {}", e));
                }
            }
        }
    }

    fn apply_operations(&mut self) {
        if self.original_bits.is_empty() {
            return;
        }

        let mut result = self.original_bits.clone();
        
        for seq in &self.operation_sequences {
            result = seq.apply(&result);
        }

        self.processed_bits = result;
        self.update_viewer();
    }

    fn update_viewer(&mut self) {
        let bits_to_show = if self.show_original {
            &self.original_bits
        } else {
            &self.processed_bits
        };
        self.viewer.set_bits(bits_to_show.clone());
    }

    fn add_operation(&mut self) {
        if self.current_operation_input.is_empty() {
            return;
        }

        match OperationSequence::from_string(&self.current_operation_input) {
            Ok(seq) => {
                self.operation_sequences.push(seq);
                self.current_operation_input.clear();
                self.error_message = None;
                self.apply_operations();
            }
            Err(e) => {
                self.error_message = Some(format!("Invalid operation: {}", e));
            }
        }
    }
}

impl eframe::App for BitApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Ctrl+Shift+Mouse Wheel for font size adjustment
        // This needs to be checked before any UI elements consume the scroll
        let mut font_size_changed = false;
        ctx.input(|i| {
            // Check if BOTH Ctrl and Shift are being held down
            let ctrl_shift_held = i.modifiers.ctrl && i.modifiers.shift;
            
            if ctrl_shift_held {
                // Combine both scroll delta types
                let delta = i.smooth_scroll_delta.y + i.raw_scroll_delta.y;
                
                if delta.abs() > 0.01 {
                    // Positive scroll = zoom in, negative = zoom out
                    // Increased sensitivity for more responsive feel
                    let sensitivity = 0.1;
                    self.font_size = (self.font_size + delta * sensitivity).clamp(8.0, 24.0);
                    font_size_changed = true;
                }
            }
        });
        
        // Request repaint if font size changed for immediate visual feedback
        if font_size_changed {
            ctx.request_repaint();
        }

        // Apply font size to the context
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔧 B.I.T. - Bit Information Tool");
                
                if ui.button("📂 Open File").clicked() {
                    self.load_file();
                }

                if ui.button("💾 Save File").clicked() {
                    self.save_file();
                }

                ui.separator();

                ui.label("Frame Length:");
                ui.add(egui::Slider::new(&mut self.viewer.frame_length, 8..=512).logarithmic(true));

                ui.separator();

                ui.label("Zoom:");
                if ui.button("➕").clicked() {
                    self.viewer.zoom_in();
                }
                if ui.button("➖").clicked() {
                    self.viewer.zoom_out();
                }
                if ui.button("🔄").clicked() {
                    self.viewer.reset_zoom();
                }

                ui.separator();

                if ui.selectable_label(self.show_original, "Original").clicked() {
                    self.show_original = true;
                    self.update_viewer();
                }
                if ui.selectable_label(!self.show_original, "Processed").clicked() {
                    self.show_original = false;
                    self.update_viewer();
                }

                ui.separator();

                if ui.button("⚙ Settings").clicked() {
                    self.show_settings = !self.show_settings;
                }
            });
        });

        egui::SidePanel::left("operations_panel")
            .default_width(300.0)
            .show(ctx, |ui| {
                ui.heading("Operations");

                ui.horizontal(|ui| {
                    ui.label("Add operation:");
                    let response = ui.text_edit_singleline(&mut self.current_operation_input);
                    
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.add_operation();
                    }
                    
                    if ui.button("➕ Add").clicked() {
                        self.add_operation();
                    }
                });

                ui.label("Syntax: t4r3i8s1");
                ui.label("t=take, r=reverse, i=invert, s=skip");

                ui.separator();

                ui.heading("Operation Sequences");

                egui::ScrollArea::vertical()
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
                .show(ui, |ui| {
                    let mut to_remove = None;
                    let mut to_move = None;

                    for (i, seq) in self.operation_sequences.iter().enumerate() {
                        ui.horizontal(|ui| {
                            // Drag handle
                            if ui.button("☰").clicked() {
                                self.dragging_index = Some(i);
                            }

                            ui.label(format!("{}. {}", i + 1, seq.to_string()));

                            if ui.button("🗑").clicked() {
                                to_remove = Some(i);
                            }

                            if i > 0 && ui.button("⬆").clicked() {
                                to_move = Some((i, i - 1));
                            }

                            if i < self.operation_sequences.len() - 1 && ui.button("⬇").clicked() {
                                to_move = Some((i, i + 1));
                            }
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.operation_sequences.remove(idx);
                        self.apply_operations();
                    }

                    if let Some((from, to)) = to_move {
                        let seq = self.operation_sequences.remove(from);
                        self.operation_sequences.insert(to, seq);
                        self.apply_operations();
                    }
                });

                if !self.operation_sequences.is_empty() {
                    ui.separator();
                    if ui.button("🔄 Reapply All Operations").clicked() {
                        self.apply_operations();
                    }
                    if ui.button("🗑 Clear All Operations").clicked() {
                        self.operation_sequences.clear();
                        self.processed_bits = self.original_bits.clone();
                        self.update_viewer();
                    }
                }

                ui.separator();

                if let Some(path) = &self.current_file_path {
                    ui.label(format!("File: {}", path.display()));
                }
                ui.label(format!("Original bits: {}", self.original_bits.len()));
                ui.label(format!("Processed bits: {}", self.processed_bits.len()));
                ui.label(format!("Bit size: {:.1}px", self.viewer.bit_size));
            });

        // Settings Window
        if self.show_settings {
            egui::Window::new("⚙ Settings")
                .open(&mut self.show_settings)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Display Settings");
                    ui.separator();

                    // Shape selector
                    ui.label("Bit Shape:");
                    ui.horizontal(|ui| {
                        if ui.selectable_label(self.viewer.shape == BitShape::Square, "⬛ Square").clicked() {
                            self.viewer.shape = BitShape::Square;
                        }
                        if ui.selectable_label(self.viewer.shape == BitShape::Circle, "⚫ Circle").clicked() {
                            self.viewer.shape = BitShape::Circle;
                        }
                    });

                    ui.separator();

                    // Grid settings
                    ui.checkbox(&mut self.viewer.show_grid, "Show Grid Lines");
                    ui.label("Toggle the grid lines around each bit");

                    ui.add_space(8.0);

                    ui.label("Thick Grid Interval (Horizontal):");
                    ui.add(egui::Slider::new(&mut self.viewer.thick_grid_interval_horizontal, 0..=64)
                        .text("bits"));
                    ui.label("Thicker vertical lines every N bits horizontally (0 = off)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Interval (Vertical):");
                    ui.add(egui::Slider::new(&mut self.viewer.thick_grid_interval_vertical, 0..=64)
                        .text("bits"));
                    ui.label("Thicker horizontal lines every N bits vertically (0 = off)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Spacing (Horizontal):");
                    ui.add(egui::Slider::new(&mut self.viewer.thick_grid_spacing_horizontal, 0.0..=10.0)
                        .text("pixels"));
                    ui.label("Horizontal gap size (vertical line spacing)");

                    ui.add_space(4.0);

                    ui.label("Thick Grid Spacing (Vertical):");
                    ui.add(egui::Slider::new(&mut self.viewer.thick_grid_spacing_vertical, 0.0..=10.0)
                        .text("pixels"));
                    ui.label("Vertical gap size (horizontal line spacing)");

                    ui.separator();

                    // Font size setting
                    ui.label("GUI Font Size:");
                    ui.add(egui::Slider::new(&mut self.font_size, 8.0..=24.0)
                        .text("pixels"));
                    ui.label("Adjust the size of all interface text");

                    ui.separator();
                    
                    ui.label("💡 Tips:");
                    ui.label("• Grid lines help distinguish individual bits");
                    ui.label("• Thick intervals are useful for byte boundaries");
                    ui.label("• Try interval of 8 for byte alignment");
                    ui.label("• Increase spacing for more visible separation");
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            if self.original_bits.is_empty() {
                ui.centered_and_justified(|ui| {
                    ui.heading("Open a file to view its bits");
                });
            } else {
                self.viewer.show(ui);
            }
        });
    }
}
