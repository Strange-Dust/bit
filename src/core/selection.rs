/// 2D rectangular selection state for the frame grid.
///
/// Selection stores two absolute bit positions (anchor and current) and derives
/// a 2D rectangle using the current frame_length. This allows the selection to
/// persist across view mode switches since it uses absolute bit indices.

/// Describes a 2D rectangle in the frame grid (columns x rows).
#[derive(Debug, Clone, Copy)]
pub struct SelectionRect {
    pub col_start: usize,
    pub col_end: usize,   // inclusive
    pub row_start: usize,
    pub row_end: usize,   // inclusive
}

impl SelectionRect {
    pub fn width(&self) -> usize {
        self.col_end - self.col_start + 1
    }

    pub fn height(&self) -> usize {
        self.row_end - self.row_start + 1
    }

    pub fn contains(&self, col: usize, row: usize) -> bool {
        col >= self.col_start && col <= self.col_end
            && row >= self.row_start && row <= self.row_end
    }

    pub fn total_bits(&self) -> usize {
        self.width() * self.height()
    }
}

/// Tracks a click-drag selection as two absolute bit indices.
pub struct BitSelection {
    pub is_active: bool,
    pub is_dragging: bool,
    anchor_bit: usize,
    current_bit: usize,
}

impl Default for BitSelection {
    fn default() -> Self {
        Self {
            is_active: false,
            is_dragging: false,
            anchor_bit: 0,
            current_bit: 0,
        }
    }
}

impl BitSelection {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new drag at the given absolute bit index.
    pub fn begin_drag(&mut self, bit_index: usize) {
        self.anchor_bit = bit_index;
        self.current_bit = bit_index;
        self.is_dragging = true;
        self.is_active = true;
    }

    /// Update the drag endpoint (anchor stays fixed).
    pub fn update_drag(&mut self, bit_index: usize) {
        self.current_bit = bit_index;
    }

    /// Finish the drag. If anchor == current (single click), clear the selection.
    pub fn end_drag(&mut self) {
        self.is_dragging = false;
        if self.anchor_bit == self.current_bit {
            self.clear();
        }
    }

    /// Clear the selection entirely.
    pub fn clear(&mut self) {
        self.is_active = false;
        self.is_dragging = false;
        self.anchor_bit = 0;
        self.current_bit = 0;
    }

    /// Derive the 2D rectangle from anchor + current using the given frame_length.
    pub fn rect(&self, frame_length: usize) -> SelectionRect {
        if frame_length == 0 {
            return SelectionRect {
                col_start: 0,
                col_end: 0,
                row_start: 0,
                row_end: 0,
            };
        }
        let anchor_col = self.anchor_bit % frame_length;
        let anchor_row = self.anchor_bit / frame_length;
        let current_col = self.current_bit % frame_length;
        let current_row = self.current_bit / frame_length;
        SelectionRect {
            col_start: anchor_col.min(current_col),
            col_end: anchor_col.max(current_col),
            row_start: anchor_row.min(current_row),
            row_end: anchor_row.max(current_row),
        }
    }

    /// Check if a given absolute bit index falls within the selection rectangle.
    pub fn contains_bit(&self, bit_index: usize, frame_length: usize) -> bool {
        if !self.is_active || frame_length == 0 {
            return false;
        }
        let col = bit_index % frame_length;
        let row = bit_index / frame_length;
        self.rect(frame_length).contains(col, row)
    }

    /// Return all selected absolute bit indices (for Save to file).
    pub fn selected_bits(&self, frame_length: usize, total_bits: usize) -> Vec<usize> {
        if !self.is_active || frame_length == 0 {
            return Vec::new();
        }
        let rect = self.rect(frame_length);
        let mut bits = Vec::with_capacity(rect.total_bits());
        let total_rows = (total_bits + frame_length - 1) / frame_length;
        let actual_end_row = rect.row_end.min(total_rows.saturating_sub(1));
        for row in rect.row_start..=actual_end_row {
            for col in rect.col_start..=rect.col_end {
                let bit_idx = row * frame_length + col;
                if bit_idx < total_bits && col < frame_length {
                    bits.push(bit_idx);
                }
            }
        }
        bits
    }

    /// First selected bit: row_start * frame_length + col_start.
    pub fn start_bit(&self, frame_length: usize) -> usize {
        let rect = self.rect(frame_length);
        rect.row_start * frame_length + rect.col_start
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_rect_basic() {
        let mut sel = BitSelection::new();
        sel.begin_drag(5);
        sel.update_drag(27);
        sel.end_drag();

        assert!(sel.is_active);
        let rect = sel.rect(10);
        assert_eq!(rect.col_start, 5);
        assert_eq!(rect.col_end, 7);
        assert_eq!(rect.row_start, 0);
        assert_eq!(rect.row_end, 2);
        assert_eq!(rect.width(), 3);
        assert_eq!(rect.height(), 3);
    }

    #[test]
    fn test_single_click_clears() {
        let mut sel = BitSelection::new();
        sel.begin_drag(5);
        sel.end_drag();
        assert!(!sel.is_active);
    }

    #[test]
    fn test_contains_bit() {
        let mut sel = BitSelection::new();
        sel.begin_drag(5);
        sel.update_drag(27);
        sel.end_drag();

        // frame_length=10, selection is cols 5-7, rows 0-2
        assert!(sel.contains_bit(5, 10));   // row 0, col 5
        assert!(sel.contains_bit(7, 10));   // row 0, col 7
        assert!(sel.contains_bit(15, 10));  // row 1, col 5
        assert!(sel.contains_bit(27, 10));  // row 2, col 7
        assert!(!sel.contains_bit(4, 10));  // row 0, col 4 - outside
        assert!(!sel.contains_bit(8, 10));  // row 0, col 8 - outside
        assert!(!sel.contains_bit(35, 10)); // row 3, col 5 - outside row
    }

    #[test]
    fn test_selected_bits() {
        let mut sel = BitSelection::new();
        sel.begin_drag(5);
        sel.update_drag(27);
        sel.end_drag();

        let bits = sel.selected_bits(10, 100);
        // cols 5,6,7 across rows 0,1,2
        assert_eq!(bits, vec![5, 6, 7, 15, 16, 17, 25, 26, 27]);
    }

    #[test]
    fn test_clear() {
        let mut sel = BitSelection::new();
        sel.begin_drag(5);
        sel.update_drag(27);
        sel.end_drag();
        assert!(sel.is_active);

        sel.clear();
        assert!(!sel.is_active);
        assert!(!sel.is_dragging);
    }
}
