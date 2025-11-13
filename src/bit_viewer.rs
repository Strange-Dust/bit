use bitvec::prelude::*;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};

#[derive(Debug, Clone, Copy, PartialEq)]
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
}

impl BitViewer {
    pub fn new() -> Self {
        Self {
            bits: BitVec::new(),
            frame_length: 64,
            bit_size: 10.0,
            bit_spacing: 2.0,
            shape: BitShape::Square,
        }
    }

    pub fn set_bits(&mut self, bits: BitVec<u8, Msb0>) {
        self.bits = bits;
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // Calculate total content size
        let total_rows = (self.bits.len() + self.frame_length - 1) / self.frame_length;
        let cell_size = self.bit_size + self.bit_spacing;
        // Add padding to prevent scrollbar from covering content
        let padding = 20.0;
        let content_width = (self.frame_length as f32) * cell_size + padding;
        let content_height = (total_rows as f32) * cell_size + padding;

        // Set scrollbar to always be expanded (no hover animation)
        ui.style_mut().spacing.scroll.bar_width = 8.0;
        ui.style_mut().spacing.scroll.floating_width = 8.0;
        ui.style_mut().spacing.scroll.bar_inner_margin = 4.0;
        ui.style_mut().spacing.scroll.bar_outer_margin = 0.0;
        ui.style_mut().spacing.scroll.floating = false;


        egui::ScrollArea::both()
            .auto_shrink([false, false])

            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
            .drag_to_scroll(true)
            .show_viewport(ui, |ui, viewport| {
                // Set the content size
                ui.set_width(content_width);
                ui.set_height(content_height);

                let (response, painter) = ui.allocate_painter(
                    Vec2::new(content_width, content_height),
                    Sense::hover(),
                );

                let start_row = (viewport.min.y / cell_size).floor() as usize;
                let end_row = ((viewport.max.y / cell_size).ceil() as usize + 1).min(total_rows);
                let start_col = (viewport.min.x / cell_size).floor() as usize;
                let end_col = ((viewport.max.x / cell_size).ceil() as usize + 1).min(self.frame_length);

                // Only render visible bits
                for row in start_row..end_row {
                    for col in start_col..end_col {
                        let bit_index = row * self.frame_length + col;
                        if bit_index >= self.bits.len() {
                            break;
                        }

                        let bit = self.bits[bit_index];
                        let color = if bit { Color32::BLACK } else { Color32::WHITE };

                        let x = response.rect.min.x + (col as f32) * cell_size;
                        let y = response.rect.min.y + (row as f32) * cell_size;

                        match self.shape {
                            BitShape::Square => {
                                let rect = Rect::from_min_size(
                                    Pos2::new(x, y),
                                    Vec2::new(self.bit_size, self.bit_size),
                                );
                                painter.rect_filled(rect, 0.0, color);
                                painter.rect_stroke(rect, 0.0, Stroke::new(1.0, Color32::GRAY));
                            }
                            BitShape::Circle => {
                                let center = Pos2::new(
                                    x + self.bit_size / 2.0,
                                    y + self.bit_size / 2.0,
                                );
                                painter.circle_filled(center, self.bit_size / 2.0, color);
                                painter.circle_stroke(
                                    center,
                                    self.bit_size / 2.0,
                                    Stroke::new(1.0, Color32::GRAY),
                                );
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
