use bitvec::prelude::*;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BitShape {
    Square,
    Circle,
}

pub struct BitViewer {
    pub bits: BitVec<u8, Msb0>,
    pub frame_length: usize,
    pub bit_size: f32,
    pub bit_spacing: f32,
    pub shape: BitShape,
    pub show_grid: bool,
    pub thick_grid_interval_horizontal: usize,
    pub thick_grid_interval_vertical: usize,
    pub thick_grid_spacing_horizontal: f32,
    pub thick_grid_spacing_vertical: f32,
    pub highlighted_bits: HashSet<usize>,
    pub jump_to_bit: Option<usize>,
}

impl BitViewer {
    pub fn new() -> Self {
        Self {
            bits: BitVec::new(),
            frame_length: 64,
            bit_size: 10.0,
            bit_spacing: 2.0,
            shape: BitShape::Square,
            show_grid: true,
            thick_grid_interval_horizontal: 8,
            thick_grid_interval_vertical: 8,
            thick_grid_spacing_horizontal: 3.0,
            thick_grid_spacing_vertical: 3.0,
            highlighted_bits: HashSet::new(),
            jump_to_bit: None,
        }
    }

    pub fn set_bits(&mut self, bits: BitVec<u8, Msb0>) {
        self.bits = bits;
    }
    
    pub fn clear_highlights(&mut self) {
        self.highlighted_bits.clear();
    }
    
    pub fn add_highlight(&mut self, bit_index: usize) {
        self.highlighted_bits.insert(bit_index);
    }
    
    pub fn add_highlight_range(&mut self, start: usize, length: usize) {
        for i in start..(start + length) {
            self.highlighted_bits.insert(i);
        }
    }
    
    pub fn jump_to_position(&mut self, bit_position: usize) {
        self.jump_to_bit = Some(bit_position);
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Calculate total content size
        let total_rows = (self.bits.len() + self.frame_length - 1) / self.frame_length;
        let cell_size = self.bit_size + self.bit_spacing;
        // Add padding to prevent scrollbar from covering content
        let padding = 20.0;
        
        // Calculate extra spacing from thick grid intervals
        let extra_width_spacing = if self.thick_grid_interval_horizontal > 0 {
            ((self.frame_length / self.thick_grid_interval_horizontal) as f32) * self.thick_grid_spacing_horizontal
        } else {
            0.0
        };
        
        let extra_height_spacing = if self.thick_grid_interval_vertical > 0 {
            ((total_rows / self.thick_grid_interval_vertical) as f32) * self.thick_grid_spacing_vertical
        } else {
            0.0
        };
        
        let content_width = (self.frame_length as f32) * cell_size + padding + extra_width_spacing;
        let content_height = (total_rows as f32) * cell_size + padding + extra_height_spacing;

        // Set scrollbar to always be expanded (no hover animation)
        ui.style_mut().spacing.scroll.bar_width = 8.0;
        ui.style_mut().spacing.scroll.floating_width = 8.0;
        ui.style_mut().spacing.scroll.bar_inner_margin = 4.0;
        ui.style_mut().spacing.scroll.bar_outer_margin = 0.0;
        ui.style_mut().spacing.scroll.floating = false;

        let mut scroll_area = egui::ScrollArea::both()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .drag_to_scroll(true);
        
        // Handle jump to bit position
        if let Some(bit_pos) = self.jump_to_bit.take() {
            let row = bit_pos / self.frame_length;
            let y_offset = (row as f32) * cell_size;
            scroll_area = scroll_area.vertical_scroll_offset(y_offset);
        }

        scroll_area.show_viewport(ui, |ui, viewport| {
                // Set the content size
                ui.set_width(content_width);
                ui.set_height(content_height);

                let (response, painter) = ui.allocate_painter(
                    Vec2::new(content_width, content_height),
                    Sense::hover(),
                );

                // Helper function to calculate position with spacing
                let calc_position = |index: usize, interval: usize, spacing: f32| -> f32 {
                    if interval > 0 && index > 0 {
                        (index as f32) * cell_size + (index / interval) as f32 * spacing
                    } else {
                        (index as f32) * cell_size
                    }
                };

                // Binary search for start row
                let start_row = if total_rows == 0 {
                    0
                } else {
                    let mut low = 0;
                    let mut high = total_rows;
                    while low < high {
                        let mid = (low + high) / 2;
                        let pos = calc_position(mid, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical);
                        if pos < viewport.min.y - cell_size {
                            low = mid + 1;
                        } else {
                            high = mid;
                        }
                    }
                    low.saturating_sub(1)
                };

                // Find end row
                let end_row = if total_rows == 0 {
                    0
                } else {
                    let mut row = start_row;
                    while row < total_rows {
                        let pos = calc_position(row, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical);
                        if pos > viewport.max.y + cell_size {
                            break;
                        }
                        row += 1;
                    }
                    row.min(total_rows)
                };

                // Binary search for start col
                let start_col = if self.frame_length == 0 {
                    0
                } else {
                    let mut low = 0;
                    let mut high = self.frame_length;
                    while low < high {
                        let mid = (low + high) / 2;
                        let pos = calc_position(mid, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal);
                        if pos < viewport.min.x - cell_size {
                            low = mid + 1;
                        } else {
                            high = mid;
                        }
                    }
                    low.saturating_sub(1)
                };

                // Find end col
                let end_col = if self.frame_length == 0 {
                    0
                } else {
                    let mut col = start_col;
                    while col < self.frame_length {
                        let pos = calc_position(col, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal);
                        if pos > viewport.max.x + cell_size {
                            break;
                        }
                        col += 1;
                    }
                    col.min(self.frame_length)
                };

                // Only render visible bits
                for row in start_row..end_row {
                    for col in start_col..end_col {
                        let bit_index = row * self.frame_length + col;
                        if bit_index >= self.bits.len() {
                            break;
                        }

                        let bit = self.bits[bit_index];
                        let color = if bit { Color32::BLACK } else { Color32::WHITE };

                        // Calculate accumulated extra spacing for thick grid boundaries
                        let accumulated_x_spacing = if self.thick_grid_interval_horizontal > 0 && col > 0 {
                            (col / self.thick_grid_interval_horizontal) as f32 * self.thick_grid_spacing_horizontal
                        } else {
                            0.0
                        };
                        
                        let accumulated_y_spacing = if self.thick_grid_interval_vertical > 0 && row > 0 {
                            (row / self.thick_grid_interval_vertical) as f32 * self.thick_grid_spacing_vertical
                        } else {
                            0.0
                        };

                        let x = response.rect.min.x + (col as f32) * cell_size + accumulated_x_spacing;
                        let y = response.rect.min.y + (row as f32) * cell_size + accumulated_y_spacing;

                        // Determine if this bit is on a thick grid boundary
                        let is_thick_horizontal = self.thick_grid_interval_horizontal > 0 
                            && col % self.thick_grid_interval_horizontal == 0;
                        let is_thick_vertical = self.thick_grid_interval_vertical > 0 
                            && row % self.thick_grid_interval_vertical == 0;

                        match self.shape {
                            BitShape::Square => {
                                let rect = Rect::from_min_size(
                                    Pos2::new(x, y),
                                    Vec2::new(self.bit_size, self.bit_size),
                                );
                                painter.rect_filled(rect, 0.0, color);
                                
                                // Draw highlight overlay if this bit is highlighted
                                if self.highlighted_bits.contains(&bit_index) {
                                    painter.rect_filled(rect, 0.0, Color32::from_rgba_unmultiplied(255, 255, 0, 150));
                                }
                                
                                if self.show_grid {
                                    // Draw edges individually to support different thicknesses
                                    let left_width = if is_thick_horizontal { 2.0 } else { 1.0 };
                                    let top_width = if is_thick_vertical { 2.0 } else { 1.0 };
                                    let right_width = 1.0;
                                    let bottom_width = 1.0;
                                    
                                    // Left edge
                                    painter.line_segment(
                                        [rect.left_top(), rect.left_bottom()],
                                        Stroke::new(left_width, Color32::GRAY),
                                    );
                                    // Top edge
                                    painter.line_segment(
                                        [rect.left_top(), rect.right_top()],
                                        Stroke::new(top_width, Color32::GRAY),
                                    );
                                    // Right edge
                                    painter.line_segment(
                                        [rect.right_top(), rect.right_bottom()],
                                        Stroke::new(right_width, Color32::GRAY),
                                    );
                                    // Bottom edge
                                    painter.line_segment(
                                        [rect.left_bottom(), rect.right_bottom()],
                                        Stroke::new(bottom_width, Color32::GRAY),
                                    );
                                }
                            }
                            BitShape::Circle => {
                                let center = Pos2::new(
                                    x + self.bit_size / 2.0,
                                    y + self.bit_size / 2.0,
                                );
                                painter.circle_filled(center, self.bit_size / 2.0, color);
                                
                                // Draw highlight overlay if this bit is highlighted
                                if self.highlighted_bits.contains(&bit_index) {
                                    painter.circle_filled(center, self.bit_size / 2.0, Color32::from_rgba_unmultiplied(255, 255, 0, 150));
                                }
                                
                                if self.show_grid {
                                    // Use normal thin stroke for circles - spacing makes boundaries clear
                                    painter.circle_stroke(
                                        center,
                                        self.bit_size / 2.0,
                                        Stroke::new(1.0, Color32::GRAY),
                                    );
                                }
                            }
                        }
                    }
                }
            });
    }

    pub fn zoom_in(&mut self) {
        self.bit_size = (self.bit_size * 1.2).min(100.0);
    }

    pub fn zoom_out(&mut self) {
        self.bit_size = (self.bit_size / 1.2).max(2.0);
    }

    pub fn reset_zoom(&mut self) {
        self.bit_size = 10.0;
    }
}
