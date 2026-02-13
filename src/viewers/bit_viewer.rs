use bitvec::prelude::*;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BitShape {
    Square,
    Circle,
    Octagon,
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
    last_bit_offset: Option<usize>,
}

impl BitViewer {
    pub fn new() -> Self {
        Self {
            bits: BitVec::new(),
            frame_length: 64,
            bit_size: 10.0,
            bit_spacing: 0.0,
            shape: BitShape::Square,
            show_grid: true,
            thick_grid_interval_horizontal: 8,
            thick_grid_interval_vertical: 8,
            thick_grid_spacing_horizontal: 3.0,
            thick_grid_spacing_vertical: 3.0,
            highlighted_bits: HashSet::new(),
            jump_to_bit: None,
            last_bit_offset: None,
        }
    }

    pub fn set_bits(&mut self, bits: BitVec<u8, Msb0>) {
        self.bits = bits;
    }
    
    pub fn clear_highlights(&mut self) {
        self.highlighted_bits.clear();
    }

    pub fn add_highlight_range(&mut self, start: usize, length: usize) {
        for i in start..(start + length) {
            self.highlighted_bits.insert(i);
        }
    }
    
    pub fn jump_to_position(&mut self, bit_position: usize) {
        self.jump_to_bit = Some(bit_position);
    }

    pub fn show(&mut self, ui: &mut egui::Ui, bit_offset: usize) -> usize {
        let frame_length = self.frame_length;
        let sub_row = bit_offset % frame_length;
        // Total rows covering all data (aligned to sub_row)
        let total_bits = self.bits.len().saturating_sub(sub_row);
        let total_rows = if total_bits > 0 { (total_bits + frame_length - 1) / frame_length } else { 0 };
        let offset_row = bit_offset / frame_length;
        let cell_size = self.bit_size + self.bit_spacing;
        // Add padding to prevent scrollbar from covering content
        let padding = 20.0;

        // Calculate extra spacing from thick grid intervals
        let extra_width_spacing = if self.thick_grid_interval_horizontal > 0 {
            ((frame_length / self.thick_grid_interval_horizontal) as f32) * self.thick_grid_spacing_horizontal
        } else {
            0.0
        };

        let extra_height_spacing = if self.thick_grid_interval_vertical > 0 && total_rows > 0 {
            ((total_rows / self.thick_grid_interval_vertical) as f32) * self.thick_grid_spacing_vertical
        } else {
            0.0
        };

        let content_width = (frame_length as f32) * cell_size + padding + extra_width_spacing;
        let content_height = (total_rows as f32) * cell_size + padding + extra_height_spacing;

        // Helper to convert row index to Y position (accounting for thick grid spacing)
        let calc_y_position = |row: usize| -> f32 {
            if self.thick_grid_interval_vertical > 0 && row > 0 {
                (row as f32) * cell_size + (row / self.thick_grid_interval_vertical) as f32 * self.thick_grid_spacing_vertical
            } else {
                (row as f32) * cell_size
            }
        };

        // Set scrollbar to always be expanded (no hover animation)
        ui.style_mut().spacing.scroll.bar_width = 8.0;
        ui.style_mut().spacing.scroll.floating_width = 8.0;
        ui.style_mut().spacing.scroll.bar_inner_margin = 4.0;
        ui.style_mut().spacing.scroll.bar_outer_margin = 0.0;
        ui.style_mut().spacing.scroll.floating = false;

        let mut scroll_area = egui::ScrollArea::both()
            .auto_shrink([false, false])
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible);

        // Handle jump to bit position (takes priority)
        if let Some(bit_pos) = self.jump_to_bit.take() {
            let target_row = if bit_pos > sub_row {
                (bit_pos - sub_row) / frame_length
            } else {
                0
            };
            scroll_area = scroll_area.vertical_scroll_offset(calc_y_position(target_row));
        } else if self.last_bit_offset != Some(bit_offset) {
            // bit_offset changed externally (keyboard, mouse wheel) — set scroll to match
            scroll_area = scroll_area.vertical_scroll_offset(calc_y_position(offset_row));
        }
        // Otherwise: don't set scroll, let the scroll area be free (user can drag scrollbar)

        let thick_v = self.thick_grid_interval_vertical;
        let thick_v_spacing = self.thick_grid_spacing_vertical;

        let output = scroll_area.show_viewport(ui, |ui, viewport| {
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
                let start_col = if frame_length == 0 {
                    0
                } else {
                    let mut low = 0;
                    let mut high = frame_length;
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
                let end_col = if frame_length == 0 {
                    0
                } else {
                    let mut col = start_col;
                    while col < frame_length {
                        let pos = calc_position(col, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal);
                        if pos > viewport.max.x + cell_size {
                            break;
                        }
                        col += 1;
                    }
                    col.min(frame_length)
                };

                // Only render visible bits
                for row in start_row..end_row {
                    for col in start_col..end_col {
                        let bit_index = sub_row + row * frame_length + col;
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
                            BitShape::Octagon => {
                                let center = Pos2::new(
                                    x + self.bit_size / 2.0,
                                    y + self.bit_size / 2.0,
                                );
                                let radius = self.bit_size / 2.0;
                                
                                // Calculate octagon vertices (8 points)
                                let angle_offset = std::f32::consts::PI / 8.0; // Start at 22.5 degrees for flat top/bottom
                                let mut points = Vec::new();
                                for i in 0..8 {
                                    let angle = angle_offset + (i as f32) * std::f32::consts::PI / 4.0;
                                    points.push(Pos2::new(
                                        center.x + radius * angle.cos(),
                                        center.y + radius * angle.sin(),
                                    ));
                                }
                                
                                // Draw filled octagon
                                painter.add(egui::Shape::convex_polygon(
                                    points.clone(),
                                    color,
                                    Stroke::NONE,
                                ));
                                
                                // Draw highlight overlay if this bit is highlighted
                                if self.highlighted_bits.contains(&bit_index) {
                                    painter.add(egui::Shape::convex_polygon(
                                        points.clone(),
                                        Color32::from_rgba_unmultiplied(255, 255, 0, 150),
                                        Stroke::NONE,
                                    ));
                                }
                                
                                if self.show_grid {
                                    // Draw octagon outline
                                    painter.add(egui::Shape::convex_polygon(
                                        points,
                                        Color32::TRANSPARENT,
                                        Stroke::new(1.0, Color32::GRAY),
                                    ));
                                }
                            }
                        }
                    }
                }
            });

        // Convert actual scroll Y back to a row index
        let actual_scroll_y = output.state.offset.y;
        let actual_row = if thick_v > 0 && thick_v_spacing > 0.0 {
            // Invert the thick grid spacing: each group of `interval` rows = interval*cell_size + spacing
            let group_height = thick_v as f32 * cell_size + thick_v_spacing;
            let group = (actual_scroll_y / group_height) as usize;
            let remainder = actual_scroll_y - group as f32 * group_height;
            let row_in_group = (remainder / cell_size).min(thick_v as f32 - 1.0).max(0.0) as usize;
            (group * thick_v + row_in_group).min(total_rows.saturating_sub(1))
        } else {
            (actual_scroll_y / cell_size).max(0.0) as usize
        };

        let effective_bit_offset = actual_row * frame_length + sub_row;
        self.last_bit_offset = Some(effective_bit_offset);
        effective_bit_offset
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
