use bitvec::prelude::*;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};

/// Represents a labeled column in the byte view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteColumn {
    pub label: String,
    pub bit_start: usize,  // Start bit within the byte row (0-based)
    pub bit_end: usize,    // End bit within the byte row (inclusive)
    pub color: [u8; 3],    // RGB color
}

impl ByteColumn {
    pub fn new(label: String, bit_start: usize, bit_end: usize, color: [u8; 3]) -> Self {
        Self {
            label,
            bit_start,
            bit_end,
            color,
        }
    }

    pub fn color32(&self) -> Color32 {
        Color32::from_rgb(self.color[0], self.color[1], self.color[2])
    }

    pub fn byte_range(&self, bytes_per_row: usize) -> (usize, usize) {
        let start_byte = self.bit_start / 8;
        let end_byte = self.bit_end / 8;
        (start_byte.min(bytes_per_row - 1), end_byte.min(bytes_per_row - 1))
    }
}

/// Configuration for the byte viewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteViewConfig {
    pub bytes_per_row: usize,
    pub columns: Vec<ByteColumn>,
    pub show_hex_offset: bool,
}

impl Default for ByteViewConfig {
    fn default() -> Self {
        Self {
            bytes_per_row: 16,
            columns: Vec::new(),
            show_hex_offset: true,
        }
    }
}

/// The byte viewer component
pub struct ByteViewer {
    pub config: ByteViewConfig,
    pub byte_size: f32,
}

impl Default for ByteViewer {
    fn default() -> Self {
        Self {
            config: ByteViewConfig::default(),
            byte_size: 20.0,
        }
    }
}

impl ByteViewer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_bytes_per_row(&mut self, bytes_per_row: usize) {
        self.config.bytes_per_row = bytes_per_row.max(1).min(64);
    }

    pub fn add_column(&mut self, column: ByteColumn) {
        self.config.columns.push(column);
    }

    pub fn remove_column(&mut self, index: usize) {
        if index < self.config.columns.len() {
            self.config.columns.remove(index);
        }
    }

    pub fn update_column(&mut self, index: usize, column: ByteColumn) {
        if index < self.config.columns.len() {
            self.config.columns[index] = column;
        }
    }

    /// Convert bits to bytes for display
    fn bits_to_bytes(bits: &BitVec<u8, Msb0>) -> Vec<u8> {
        let mut bytes = Vec::new();
        for chunk in bits.chunks(8) {
            let mut byte = 0u8;
            for (i, bit) in chunk.iter().enumerate() {
                if *bit {
                    byte |= 1 << (7 - i);
                }
            }
            bytes.push(byte);
        }
        bytes
    }

    /// Render the byte view
    pub fn render(&mut self, ui: &mut egui::Ui, bits: &BitVec<u8, Msb0>) {
        if bits.is_empty() {
            ui.label("No data to display");
            return;
        }

        let bytes = Self::bits_to_bytes(bits);
        let bytes_per_row = self.config.bytes_per_row;
        let total_rows = (bytes.len() + bytes_per_row - 1) / bytes_per_row;

        // Calculate layout dimensions
        let byte_width = self.byte_size * 2.5;
        let byte_height = self.byte_size * 1.5;
        let header_height = 30.0;
        let offset_width = if self.config.show_hex_offset { 80.0 } else { 0.0 };

        egui::ScrollArea::vertical()
            .id_salt("byte_viewer_scroll")
            .show(ui, |ui| {
                // Draw column headers
                self.render_column_headers(ui, bytes_per_row, byte_width, offset_width, header_height);

                // Draw each row
                for row in 0..total_rows {
                    ui.horizontal(|ui| {
                        // Show offset
                        if self.config.show_hex_offset {
                            let offset = row * bytes_per_row;
                            ui.add_sized(
                                [offset_width, byte_height],
                                egui::Label::new(
                                    egui::RichText::new(format!("{:08X}", offset))
                                        .monospace()
                                        .color(Color32::GRAY)
                                )
                            );
                        }

                        // Draw bytes
                        let row_start = row * bytes_per_row;
                        let row_end = (row_start + bytes_per_row).min(bytes.len());
                        
                        for byte_idx in row_start..row_end {
                            let byte = bytes[byte_idx];
                            let local_byte_idx = byte_idx - row_start;
                            let bit_offset = local_byte_idx * 8;

                            // Find which column this byte belongs to
                            let column_color = self.find_column_color(bit_offset);

                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(byte_width, byte_height),
                                Sense::hover(),
                            );

                            // Draw background color if in a column
                            if let Some(color) = column_color {
                                ui.painter().rect_filled(
                                    rect,
                                    2.0,
                                    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 40)
                                );
                            }

                            // Draw byte value
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                format!("{:02X}", byte),
                                egui::FontId::monospace(self.byte_size),
                                if column_color.is_some() { Color32::BLACK } else { Color32::DARK_GRAY }
                            );

                            // Draw border
                            ui.painter().rect_stroke(
                                rect,
                                2.0,
                                Stroke::new(1.0, Color32::from_gray(100)),
                                egui::epaint::StrokeKind::Middle
                            );

                            // Show tooltip with bit offset
                            if response.hovered() {
                                response.on_hover_ui(|ui| {
                                    ui.label(format!("Byte: {}\nBit offset: {}", byte_idx, byte_idx * 8));
                                    ui.label(format!("Value: 0x{:02X} ({})", byte, byte));
                                    ui.label(format!("Binary: {:08b}", byte));
                                });
                            }
                        }
                    });
                }
            });
    }

    fn render_column_headers(&self, ui: &mut egui::Ui, bytes_per_row: usize, byte_width: f32, offset_width: f32, header_height: f32) {
        if self.config.columns.is_empty() {
            return;
        }

        ui.horizontal(|ui| {
            // Offset spacer
            if offset_width > 0.0 {
                ui.add_space(offset_width);
            }

            // Calculate total width for headers
            let total_width = bytes_per_row as f32 * byte_width;

            let (rect, _) = ui.allocate_exact_size(
                Vec2::new(total_width, header_height),
                Sense::hover(),
            );

            // Draw each column header
            for column in &self.config.columns {
                let (start_byte, end_byte) = column.byte_range(bytes_per_row);
                
                if start_byte < bytes_per_row {
                    let x_start = rect.min.x + start_byte as f32 * byte_width;
                    let x_end = rect.min.x + (end_byte + 1) as f32 * byte_width;
                    
                    let header_rect = Rect::from_min_max(
                        Pos2::new(x_start, rect.min.y),
                        Pos2::new(x_end, rect.max.y)
                    );

                    // Draw background
                    ui.painter().rect_filled(
                        header_rect,
                        2.0,
                        Color32::from_rgba_unmultiplied(
                            column.color[0],
                            column.color[1],
                            column.color[2],
                            100
                        )
                    );

                    // Draw label
                    ui.painter().text(
                        header_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &column.label,
                        egui::FontId::proportional(12.0),
                        Color32::WHITE
                    );

                    // Draw border
                    ui.painter().rect_stroke(
                        header_rect,
                        2.0,
                        Stroke::new(2.0, column.color32()),
                        egui::epaint::StrokeKind::Middle
                    );
                }
            }
        });

        ui.add_space(5.0);
    }

    fn find_column_color(&self, bit_offset: usize) -> Option<Color32> {
        for column in &self.config.columns {
            if bit_offset >= column.bit_start && bit_offset < column.bit_end {
                return Some(column.color32());
            }
        }
        None
    }

    pub fn get_config(&self) -> &ByteViewConfig {
        &self.config
    }

    pub fn set_config(&mut self, config: ByteViewConfig) {
        self.config = config;
    }
}
