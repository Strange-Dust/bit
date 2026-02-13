use bitvec::prelude::*;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::core::BitSelection;

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

    pub fn show(&mut self, ui: &mut egui::Ui, bit_offset: usize, selection: &mut BitSelection) -> usize {
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

        // Selection state captured from inside the viewport closure
        let mut drag_started: Option<usize> = None;
        let mut drag_updated: Option<usize> = None;
        let mut drag_stopped = false;

        // Pre-compute selection rect to avoid borrowing selection inside closure
        let sel_rect = if selection.is_active || selection.is_dragging {
            Some(selection.rect(frame_length))
        } else {
            None
        };

        let output = scroll_area.show_viewport(ui, |ui, viewport| {
                // Set the content size
                ui.set_width(content_width);
                ui.set_height(content_height);

                let (response, painter) = ui.allocate_painter(
                    Vec2::new(content_width, content_height),
                    Sense::click_and_drag(),
                );

                // Helper function to calculate position with spacing
                let calc_position = |index: usize, interval: usize, spacing: f32| -> f32 {
                    if interval > 0 && index > 0 {
                        (index as f32) * cell_size + (index / interval) as f32 * spacing
                    } else {
                        (index as f32) * cell_size
                    }
                };

                // Helper to invert position back to index (accounting for thick grid spacing)
                let invert_position = |pos: f32, interval: usize, spacing: f32| -> usize {
                    if interval == 0 || spacing <= 0.0 {
                        return (pos / cell_size).floor().max(0.0) as usize;
                    }
                    let group_width = interval as f32 * cell_size + spacing;
                    let group = (pos / group_width).floor().max(0.0) as usize;
                    let remainder = pos - group as f32 * group_width;
                    let in_group = (remainder / cell_size).floor().clamp(0.0, interval as f32 - 1.0) as usize;
                    group * interval + in_group
                };

                // Handle pointer-to-bit conversion for drag events
                if let Some(pointer_pos) = response.interact_pointer_pos() {
                    let rel_x = pointer_pos.x - response.rect.min.x;
                    let rel_y = pointer_pos.y - response.rect.min.y;
                    let col = invert_position(rel_x, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal).min(frame_length.saturating_sub(1));
                    let row = invert_position(rel_y, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical);
                    let bit_idx = sub_row + row * frame_length + col;

                    if response.drag_started_by(egui::PointerButton::Primary) {
                        drag_started = Some(bit_idx);
                    } else if response.dragged_by(egui::PointerButton::Primary) {
                        drag_updated = Some(bit_idx);
                    }
                }
                if response.drag_stopped_by(egui::PointerButton::Primary) {
                    drag_stopped = true;
                }

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

                // Pre-compute shape geometry
                let half = self.bit_size / 2.0;

                const N_CIRCLE_SEG: usize = 16;
                let circle_offsets: [(f32, f32); N_CIRCLE_SEG] = {
                    let mut o = [(0.0f32, 0.0f32); N_CIRCLE_SEG];
                    for i in 0..N_CIRCLE_SEG {
                        let a = (i as f32) * std::f32::consts::TAU / N_CIRCLE_SEG as f32;
                        o[i] = (half * a.cos(), half * a.sin());
                    }
                    o
                };

                let octagon_offsets: [(f32, f32); 8] = {
                    let ao = std::f32::consts::PI / 8.0;
                    let mut o = [(0.0f32, 0.0f32); 8];
                    for i in 0..8 {
                        let a = ao + (i as f32) * std::f32::consts::PI / 4.0;
                        o[i] = (half * a.cos(), half * a.sin());
                    }
                    o
                };

                // Pre-compute outline ring offsets (outer/inner vertices for ring mesh)
                let circle_outline_ring: [(f32, f32, f32, f32); N_CIRCLE_SEG] = {
                    let hw = 0.5;
                    let ro = half + hw;
                    let ri = (half - hw).max(0.0);
                    let mut r = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); N_CIRCLE_SEG];
                    for i in 0..N_CIRCLE_SEG {
                        let a = (i as f32) * std::f32::consts::TAU / N_CIRCLE_SEG as f32;
                        let (c, s) = (a.cos(), a.sin());
                        r[i] = (ro * c, ro * s, ri * c, ri * s);
                    }
                    r
                };

                let octagon_outline_ring: [(f32, f32, f32, f32); 8] = {
                    let hw = 0.5;
                    let ro = half + hw;
                    let ri = (half - hw).max(0.0);
                    let ao = std::f32::consts::PI / 8.0;
                    let mut r = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 8];
                    for i in 0..8 {
                        let a = ao + (i as f32) * std::f32::consts::PI / 4.0;
                        let (c, s) = (a.cos(), a.sin());
                        r[i] = (ro * c, ro * s, ri * c, ri * s);
                    }
                    r
                };

                let grid_stroke = Stroke::new(1.0, Color32::GRAY);
                let thick_grid_stroke = Stroke::new(2.0, Color32::GRAY);
                let has_highlights = !self.highlighted_bits.is_empty();
                let highlight_color = Color32::from_rgba_unmultiplied(255, 255, 0, 150);
                let outline_color = Color32::GRAY;

                // Batched meshes: 1-3 draw calls instead of 41K+ individual shapes
                let vis_bits = (end_row.saturating_sub(start_row)) * (end_col.saturating_sub(start_col));
                let mut fill_mesh = egui::Mesh::default();
                let mut highlight_mesh = egui::Mesh::default();
                let mut outline_mesh = egui::Mesh::default();

                match self.shape {
                    BitShape::Square => {
                        fill_mesh.vertices.reserve(vis_bits * 4);
                        fill_mesh.indices.reserve(vis_bits * 6);
                    }
                    BitShape::Circle => {
                        fill_mesh.vertices.reserve(vis_bits * (N_CIRCLE_SEG + 1));
                        fill_mesh.indices.reserve(vis_bits * N_CIRCLE_SEG * 3);
                        if self.show_grid {
                            outline_mesh.vertices.reserve(vis_bits * N_CIRCLE_SEG * 2);
                            outline_mesh.indices.reserve(vis_bits * N_CIRCLE_SEG * 6);
                        }
                    }
                    BitShape::Octagon => {
                        fill_mesh.vertices.reserve(vis_bits * 9);
                        fill_mesh.indices.reserve(vis_bits * 24);
                        if self.show_grid {
                            outline_mesh.vertices.reserve(vis_bits * 16);
                            outline_mesh.indices.reserve(vis_bits * 48);
                        }
                    }
                }

                // Build meshes for all visible bits
                for row in start_row..end_row {
                    for col in start_col..end_col {
                        let bit_index = sub_row + row * frame_length + col;
                        if bit_index >= self.bits.len() {
                            break;
                        }

                        let bit = self.bits[bit_index];
                        let color = if bit { Color32::BLACK } else { Color32::WHITE };

                        let x = response.rect.min.x + calc_position(col, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal);
                        let y = response.rect.min.y + calc_position(row, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical);

                        let is_highlighted = has_highlights && self.highlighted_bits.contains(&bit_index);

                        match self.shape {
                            BitShape::Square => {
                                let base = fill_mesh.vertices.len() as u32;
                                fill_mesh.colored_vertex(Pos2::new(x, y), color);
                                fill_mesh.colored_vertex(Pos2::new(x + self.bit_size, y), color);
                                fill_mesh.colored_vertex(Pos2::new(x + self.bit_size, y + self.bit_size), color);
                                fill_mesh.colored_vertex(Pos2::new(x, y + self.bit_size), color);
                                fill_mesh.add_triangle(base, base + 1, base + 2);
                                fill_mesh.add_triangle(base, base + 2, base + 3);

                                if is_highlighted {
                                    let hb = highlight_mesh.vertices.len() as u32;
                                    highlight_mesh.colored_vertex(Pos2::new(x, y), highlight_color);
                                    highlight_mesh.colored_vertex(Pos2::new(x + self.bit_size, y), highlight_color);
                                    highlight_mesh.colored_vertex(Pos2::new(x + self.bit_size, y + self.bit_size), highlight_color);
                                    highlight_mesh.colored_vertex(Pos2::new(x, y + self.bit_size), highlight_color);
                                    highlight_mesh.add_triangle(hb, hb + 1, hb + 2);
                                    highlight_mesh.add_triangle(hb, hb + 2, hb + 3);
                                }
                            }
                            BitShape::Circle => {
                                let cx = x + half;
                                let cy = y + half;

                                // Fan: center + 16 perimeter vertices
                                let base = fill_mesh.vertices.len() as u32;
                                fill_mesh.colored_vertex(Pos2::new(cx, cy), color);
                                for &(dx, dy) in &circle_offsets {
                                    fill_mesh.colored_vertex(Pos2::new(cx + dx, cy + dy), color);
                                }
                                for i in 0..N_CIRCLE_SEG as u32 {
                                    fill_mesh.add_triangle(base, base + 1 + i, base + 1 + (i + 1) % N_CIRCLE_SEG as u32);
                                }

                                if is_highlighted {
                                    let hb = highlight_mesh.vertices.len() as u32;
                                    highlight_mesh.colored_vertex(Pos2::new(cx, cy), highlight_color);
                                    for &(dx, dy) in &circle_offsets {
                                        highlight_mesh.colored_vertex(Pos2::new(cx + dx, cy + dy), highlight_color);
                                    }
                                    for i in 0..N_CIRCLE_SEG as u32 {
                                        highlight_mesh.add_triangle(hb, hb + 1 + i, hb + 1 + (i + 1) % N_CIRCLE_SEG as u32);
                                    }
                                }

                                if self.show_grid {
                                    // Ring mesh: 32 vertices, 32 triangles per circle outline
                                    let ob = outline_mesh.vertices.len() as u32;
                                    for &(ox, oy, ix, iy) in &circle_outline_ring {
                                        outline_mesh.colored_vertex(Pos2::new(cx + ox, cy + oy), outline_color);
                                        outline_mesh.colored_vertex(Pos2::new(cx + ix, cy + iy), outline_color);
                                    }
                                    for i in 0..N_CIRCLE_SEG as u32 {
                                        let next = (i + 1) % N_CIRCLE_SEG as u32;
                                        outline_mesh.add_triangle(ob + i * 2, ob + i * 2 + 1, ob + next * 2 + 1);
                                        outline_mesh.add_triangle(ob + i * 2, ob + next * 2 + 1, ob + next * 2);
                                    }
                                }
                            }
                            BitShape::Octagon => {
                                let cx = x + half;
                                let cy = y + half;

                                // Fan: center + 8 vertices
                                let base = fill_mesh.vertices.len() as u32;
                                fill_mesh.colored_vertex(Pos2::new(cx, cy), color);
                                for &(dx, dy) in &octagon_offsets {
                                    fill_mesh.colored_vertex(Pos2::new(cx + dx, cy + dy), color);
                                }
                                for i in 0..8u32 {
                                    fill_mesh.add_triangle(base, base + 1 + i, base + 1 + (i + 1) % 8);
                                }

                                if is_highlighted {
                                    let hb = highlight_mesh.vertices.len() as u32;
                                    highlight_mesh.colored_vertex(Pos2::new(cx, cy), highlight_color);
                                    for &(dx, dy) in &octagon_offsets {
                                        highlight_mesh.colored_vertex(Pos2::new(cx + dx, cy + dy), highlight_color);
                                    }
                                    for i in 0..8u32 {
                                        highlight_mesh.add_triangle(hb, hb + 1 + i, hb + 1 + (i + 1) % 8);
                                    }
                                }

                                if self.show_grid {
                                    // Ring mesh: 16 vertices, 16 triangles per octagon outline
                                    let ob = outline_mesh.vertices.len() as u32;
                                    for &(ox, oy, ix, iy) in &octagon_outline_ring {
                                        outline_mesh.colored_vertex(Pos2::new(cx + ox, cy + oy), outline_color);
                                        outline_mesh.colored_vertex(Pos2::new(cx + ix, cy + iy), outline_color);
                                    }
                                    for i in 0..8u32 {
                                        let next = (i + 1) % 8;
                                        outline_mesh.add_triangle(ob + i * 2, ob + i * 2 + 1, ob + next * 2 + 1);
                                        outline_mesh.add_triangle(ob + i * 2, ob + next * 2 + 1, ob + next * 2);
                                    }
                                }
                            }
                        }
                    }
                }

                // Submit batched meshes (1-3 draw calls for ALL visible shapes)
                if !fill_mesh.vertices.is_empty() {
                    painter.add(egui::Shape::mesh(fill_mesh));
                }
                if !highlight_mesh.vertices.is_empty() {
                    painter.add(egui::Shape::mesh(highlight_mesh));
                }
                if !outline_mesh.vertices.is_empty() {
                    painter.add(egui::Shape::mesh(outline_mesh));
                }

                // Draw grid as segmented lines for Square shape
                // Lines are segmented per group so they don't cross through thick_grid_spacing gaps
                if self.show_grid && matches!(self.shape, BitShape::Square) && end_row > start_row && end_col > start_col {
                    let h_interval = self.thick_grid_interval_horizontal;
                    let v_interval = self.thick_grid_interval_vertical;
                    let h_spacing = self.thick_grid_spacing_horizontal;
                    let v_spacing = self.thick_grid_spacing_vertical;

                    // Collect row group ranges (start_row..end_row segmented by v_interval)
                    let row_groups: Vec<(usize, usize)> = if v_interval > 0 && v_spacing > 0.0 {
                        let mut groups = Vec::new();
                        let mut r = start_row;
                        while r < end_row {
                            let group_end = if v_interval > 0 {
                                // Next group boundary: next multiple of v_interval
                                let next_boundary = ((r / v_interval) + 1) * v_interval;
                                next_boundary.min(end_row)
                            } else {
                                end_row
                            };
                            groups.push((r, group_end));
                            r = group_end;
                        }
                        groups
                    } else {
                        vec![(start_row, end_row)]
                    };

                    // Collect col group ranges (start_col..end_col segmented by h_interval)
                    let col_groups: Vec<(usize, usize)> = if h_interval > 0 && h_spacing > 0.0 {
                        let mut groups = Vec::new();
                        let mut c = start_col;
                        while c < end_col {
                            let group_end = if h_interval > 0 {
                                let next_boundary = ((c / h_interval) + 1) * h_interval;
                                next_boundary.min(end_col)
                            } else {
                                end_col
                            };
                            groups.push((c, group_end));
                            c = group_end;
                        }
                        groups
                    } else {
                        vec![(start_col, end_col)]
                    };

                    // Vertical lines: left edge of each visible column, segmented per row group
                    for col in start_col..end_col {
                        let x = response.rect.min.x + calc_position(col, h_interval, h_spacing);
                        let stroke = if h_interval > 0 && col % h_interval == 0 { thick_grid_stroke } else { grid_stroke };
                        for &(grp_start, grp_end) in &row_groups {
                            let y_top = response.rect.min.y + calc_position(grp_start, v_interval, v_spacing);
                            let y_bottom = response.rect.min.y + calc_position(grp_end - 1, v_interval, v_spacing) + self.bit_size;
                            painter.line_segment([Pos2::new(x, y_top), Pos2::new(x, y_bottom)], stroke);
                        }
                    }
                    // Right edge of last visible column, segmented per row group
                    {
                        let x_right = response.rect.min.x + calc_position(end_col - 1, h_interval, h_spacing) + self.bit_size;
                        for &(grp_start, grp_end) in &row_groups {
                            let y_top = response.rect.min.y + calc_position(grp_start, v_interval, v_spacing);
                            let y_bottom = response.rect.min.y + calc_position(grp_end - 1, v_interval, v_spacing) + self.bit_size;
                            painter.line_segment([Pos2::new(x_right, y_top), Pos2::new(x_right, y_bottom)], grid_stroke);
                        }
                    }

                    // Horizontal lines: top edge of each visible row, segmented per col group
                    for row in start_row..end_row {
                        let y = response.rect.min.y + calc_position(row, v_interval, v_spacing);
                        let stroke = if v_interval > 0 && row % v_interval == 0 { thick_grid_stroke } else { grid_stroke };
                        for &(grp_start, grp_end) in &col_groups {
                            let x_left = response.rect.min.x + calc_position(grp_start, h_interval, h_spacing);
                            let x_right = response.rect.min.x + calc_position(grp_end - 1, h_interval, h_spacing) + self.bit_size;
                            painter.line_segment([Pos2::new(x_left, y), Pos2::new(x_right, y)], stroke);
                        }
                    }
                    // Bottom edge of last visible row, segmented per col group
                    {
                        let y_bottom = response.rect.min.y + calc_position(end_row - 1, v_interval, v_spacing) + self.bit_size;
                        for &(grp_start, grp_end) in &col_groups {
                            let x_left = response.rect.min.x + calc_position(grp_start, h_interval, h_spacing);
                            let x_right = response.rect.min.x + calc_position(grp_end - 1, h_interval, h_spacing) + self.bit_size;
                            painter.line_segment([Pos2::new(x_left, y_bottom), Pos2::new(x_right, y_bottom)], grid_stroke);
                        }
                    }
                }

                // Draw selection overlay as a single rectangle
                if let Some(ref sr) = sel_rect {
                    // Clamp selection to visible rows/cols
                    let vis_col_start = sr.col_start.max(start_col);
                    let vis_col_end = (sr.col_end + 1).min(end_col);
                    let vis_row_start = sr.row_start.max(start_row);
                    let vis_row_end = (sr.row_end + 1).min(end_row);

                    if vis_col_start < vis_col_end && vis_row_start < vis_row_end {
                        let x_start = response.rect.min.x + calc_position(vis_col_start, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal);
                        let y_start = response.rect.min.y + calc_position(vis_row_start, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical);
                        let x_end = response.rect.min.x + calc_position(vis_col_end - 1, self.thick_grid_interval_horizontal, self.thick_grid_spacing_horizontal) + self.bit_size;
                        let y_end = response.rect.min.y + calc_position(vis_row_end - 1, self.thick_grid_interval_vertical, self.thick_grid_spacing_vertical) + self.bit_size;

                        let sel_screen_rect = Rect::from_min_max(
                            Pos2::new(x_start, y_start),
                            Pos2::new(x_end, y_end),
                        );
                        let selection_color = Color32::from_rgba_unmultiplied(70, 130, 255, 80);
                        painter.rect_filled(sel_screen_rect, 0.0, selection_color);
                        painter.rect_stroke(
                            sel_screen_rect,
                            0.0,
                            Stroke::new(2.0, Color32::from_rgb(70, 130, 255)),
                            egui::epaint::StrokeKind::Middle,
                        );
                    }
                }
            });

        // Apply selection changes after the closure (avoids borrow conflicts)
        if let Some(bit_idx) = drag_started {
            selection.begin_drag(bit_idx);
        }
        if let Some(bit_idx) = drag_updated {
            selection.update_drag(bit_idx);
        }
        if drag_stopped {
            selection.end_drag();
        }

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
