//! Bidirectional coordinate transformation system.

use crate::models::keyboard_geometry::KeyboardGeometry;
use crate::models::layer::Position;
use std::collections::HashMap;

/// Bidirectional coordinate transformation system.
///
/// Manages four coordinate spaces:
/// 1. **Visual** - User's view in Markdown tables (row, col)
/// 2. **Matrix** - Electrical wiring (row, col)
/// 3. **LED** - Physical LED wiring order (for RGB colors)
/// 4. **Layout** - info.json layout array order (for keymap generation)
///
/// # Building Process
///
/// 1. Iterate through KeyboardGeometry.keys
/// 2. For each `KeyGeometry`:
///    - `led_to_matrix`[`led_index`] = `matrix_position`
///    - `matrix_to_led`[`matrix_position`] = `led_index`
///    - `layout_to_matrix`[`layout_index`] = `matrix_position`
///    - `matrix_to_layout`[`matrix_position`] = `layout_index`
///    - Compute `visual_position` from `visual_x/visual_y` (quantize to grid)
///    - `matrix_to_visual`[`matrix_position`] = `visual_position`
///    - `visual_to_matrix`[`visual_position`] = `matrix_position`
///
/// # Special Handling
///
/// - Split keyboards: Right half columns reversed (col 0 = rightmost)
/// - EX keys: Extra column positions (col 6 and 13 for 14-column layouts)
/// - Thumb keys: Span multiple visual columns but single matrix position
#[derive(Debug, Clone)]
pub struct VisualLayoutMapping {
    /// LED index → matrix position (indexed by physical LED order)
    pub led_to_matrix: Vec<(u8, u8)>,
    /// Matrix position → LED index (for RGB colors)
    pub matrix_to_led: HashMap<(u8, u8), u8>,
    /// Layout index → matrix position (indexed by info.json layout order)
    pub layout_to_matrix: Vec<(u8, u8)>,
    /// Matrix position → layout index (for keymap generation)
    pub matrix_to_layout: HashMap<(u8, u8), u8>,
    /// Matrix → visual position
    pub matrix_to_visual: HashMap<(u8, u8), Position>,
    /// Visual → matrix position
    pub visual_to_matrix: HashMap<Position, (u8, u8)>,
}

#[allow(dead_code)]
impl VisualLayoutMapping {
    /// Creates a new empty `VisualLayoutMapping`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            led_to_matrix: Vec::new(),
            matrix_to_led: HashMap::new(),
            layout_to_matrix: Vec::new(),
            matrix_to_layout: HashMap::new(),
            matrix_to_visual: HashMap::new(),
            visual_to_matrix: HashMap::new(),
        }
    }

    /// Builds a `VisualLayoutMapping` from `KeyboardGeometry`.
    ///
    /// This computes all coordinate transformations based on the physical
    /// key positions from the QMK info.json layout definition.
    #[must_use]
    pub fn build(geometry: &KeyboardGeometry) -> Self {
        let mut mapping = Self::new();

        // Pre-allocate vectors
        let max_led = geometry.keys.iter().map(|k| k.led_index).max().unwrap_or(0) as usize;
        let max_layout = geometry.keys.iter().map(|k| k.layout_index).max().unwrap_or(0) as usize;
        mapping.led_to_matrix.resize(max_led + 1, (0, 0));
        mapping.layout_to_matrix.resize(max_layout + 1, (0, 0));

        for key in &geometry.keys {
            let matrix_pos = key.matrix_position;
            let led_idx = key.led_index;
            let layout_idx = key.layout_index;

            // Build LED <-> Matrix mappings (for RGB colors)
            mapping.led_to_matrix[led_idx as usize] = matrix_pos;
            mapping.matrix_to_led.insert(matrix_pos, led_idx);

            // Build Layout <-> Matrix mappings (for keymap generation)
            mapping.layout_to_matrix[layout_idx as usize] = matrix_pos;
            mapping.matrix_to_layout.insert(matrix_pos, layout_idx);

            // Compute visual position from physical coordinates
            // Quantize to grid (round to nearest integer position)
            let visual_row = key.visual_y.round() as u8;
            let visual_col = key.visual_x.round() as u8;
            let visual_pos = Position::new(visual_row, visual_col);

            // Build Matrix <-> Visual mappings
            mapping.matrix_to_visual.insert(matrix_pos, visual_pos);
            mapping.visual_to_matrix.insert(visual_pos, matrix_pos);
        }

        mapping
    }

    /// Converts LED index to matrix position.
    ///
    /// Used for firmware generation to map LED order to electrical matrix.
    #[must_use]
    pub fn led_to_matrix_pos(&self, led: u8) -> Option<(u8, u8)> {
        self.led_to_matrix.get(led as usize).copied()
    }

    /// Converts matrix position to visual position.
    ///
    /// Used for parsing layouts from files where keys are stored by visual position.
    #[must_use]
    pub fn matrix_to_visual_pos(&self, row: u8, col: u8) -> Option<Position> {
        self.matrix_to_visual.get(&(row, col)).copied()
    }

    /// Converts visual position to matrix position.
    ///
    /// Used for saving layouts and when user selects a key in the UI.
    #[must_use]
    pub fn visual_to_matrix_pos(&self, row: u8, col: u8) -> Option<(u8, u8)> {
        self.visual_to_matrix.get(&Position::new(row, col)).copied()
    }

    /// Converts visual position to LED index.
    ///
    /// Used for RGB configuration and LED-based features.
    #[must_use]
    pub fn visual_to_led_index(&self, row: u8, col: u8) -> Option<u8> {
        let visual_pos = Position::new(row, col);
        let matrix_pos = self.visual_to_matrix.get(&visual_pos)?;
        self.matrix_to_led.get(matrix_pos).copied()
    }

    /// Converts visual position to layout index.
    ///
    /// Used for keymap generation - keys must be in info.json layout order.
    #[must_use]
    pub fn visual_to_layout_index(&self, row: u8, col: u8) -> Option<u8> {
        let visual_pos = Position::new(row, col);
        let matrix_pos = self.visual_to_matrix.get(&visual_pos)?;
        self.matrix_to_layout.get(matrix_pos).copied()
    }

    /// Gets the total number of keys in the mapping.
    #[must_use]
    pub const fn key_count(&self) -> usize {
        // Use layout_to_matrix as the canonical key count
        self.layout_to_matrix.len()
    }

    /// Returns all valid visual positions in this mapping.
    ///
    /// Used when adjusting layouts to match a new geometry.
    #[must_use]
    pub fn get_all_visual_positions(&self) -> Vec<Position> {
        self.visual_to_matrix.keys().copied().collect()
    }

    /// Checks if a visual position is valid (has a physical key).
    #[must_use]
    pub fn is_valid_position(&self, pos: Position) -> bool {
        self.visual_to_matrix.contains_key(&pos)
    }

    /// Finds the nearest valid position when moving up from the current position.
    ///
    /// If current row - 1 has a key in same column, returns that.
    /// Otherwise searches for nearest key in same row, preferring left.
    #[must_use]
    pub fn find_position_up(&self, current: Position) -> Option<Position> {
        if current.row == 0 {
            return None;
        }

        // Try same column first
        let target_row = current.row - 1;
        let same_col = Position::new(target_row, current.col);
        if self.is_valid_position(same_col) {
            return Some(same_col);
        }

        // Find nearest valid position in target row
        self.find_nearest_in_row(target_row, current.col)
    }

    /// Finds the nearest valid position when moving down from the current position.
    #[must_use]
    pub fn find_position_down(&self, current: Position) -> Option<Position> {
        let target_row = current.row + 1;

        // Try same column first
        let same_col = Position::new(target_row, current.col);
        if self.is_valid_position(same_col) {
            return Some(same_col);
        }

        // Find nearest valid position in target row
        self.find_nearest_in_row(target_row, current.col)
    }

    /// Finds the nearest valid position when moving left from the current position.
    #[must_use]
    pub fn find_position_left(&self, current: Position) -> Option<Position> {
        if current.col == 0 {
            return None;
        }

        // Search leftward for next valid position in same row
        for col in (0..current.col).rev() {
            let pos = Position::new(current.row, col);
            if self.is_valid_position(pos) {
                return Some(pos);
            }
        }
        None
    }

    /// Finds the nearest valid position when moving right from the current position.
    #[must_use]
    pub fn find_position_right(&self, current: Position) -> Option<Position> {
        // Search rightward for next valid position in same row
        // Use a reasonable max column (split keyboards can have up to 14 columns)
        for col in (current.col + 1)..=20 {
            let pos = Position::new(current.row, col);
            if self.is_valid_position(pos) {
                return Some(pos);
            }
        }
        None
    }

    /// Finds the nearest valid position in a row, closest to target column.
    fn find_nearest_in_row(&self, row: u8, target_col: u8) -> Option<Position> {
        let mut best: Option<Position> = None;
        let mut best_distance = u8::MAX;

        for pos in self.visual_to_matrix.keys() {
            if pos.row == row {
                let distance = pos.col.abs_diff(target_col);

                if distance < best_distance {
                    best_distance = distance;
                    best = Some(*pos);
                }
            }
        }

        best
    }

    /// Returns the bounds of valid positions (max row, max col).
    #[must_use]
    pub fn get_bounds(&self) -> (u8, u8) {
        let mut max_row = 0u8;
        let mut max_col = 0u8;

        for pos in self.visual_to_matrix.keys() {
            max_row = max_row.max(pos.row);
            max_col = max_col.max(pos.col);
        }

        (max_row, max_col)
    }

    /// Returns the first valid position (top-left most key).
    ///
    /// Used to initialize cursor position when loading a layout.
    #[must_use]
    pub fn get_first_position(&self) -> Option<Position> {
        let mut first: Option<Position> = None;

        for pos in self.visual_to_matrix.keys() {
            match first {
                None => first = Some(*pos),
                Some(current) => {
                    // Prefer top-left: lower row first, then lower col
                    if pos.row < current.row || (pos.row == current.row && pos.col < current.col) {
                        first = Some(*pos);
                    }
                }
            }
        }

        first
    }
}

impl Default for VisualLayoutMapping {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::keyboard_geometry::KeyGeometry;

    #[test]
    fn test_visual_layout_mapping_build() {
        let mut geometry = KeyboardGeometry::new("test", "LAYOUT", 4, 12);

        // Add keys in a simple grid
        geometry.add_key(KeyGeometry::new((0, 0), 0, 0.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 1), 1, 1.0, 0.0));
        geometry.add_key(KeyGeometry::new((1, 0), 2, 0.0, 1.0));
        geometry.add_key(KeyGeometry::new((1, 1), 3, 1.0, 1.0));

        let mapping = VisualLayoutMapping::build(&geometry);

        // Verify LED to matrix
        assert_eq!(mapping.led_to_matrix_pos(0), Some((0, 0)));
        assert_eq!(mapping.led_to_matrix_pos(1), Some((0, 1)));
        assert_eq!(mapping.led_to_matrix_pos(2), Some((1, 0)));
        assert_eq!(mapping.led_to_matrix_pos(3), Some((1, 1)));

        // Verify matrix to visual
        assert_eq!(
            mapping.matrix_to_visual_pos(0, 0),
            Some(Position::new(0, 0))
        );
        assert_eq!(
            mapping.matrix_to_visual_pos(0, 1),
            Some(Position::new(0, 1))
        );
        assert_eq!(
            mapping.matrix_to_visual_pos(1, 0),
            Some(Position::new(1, 0))
        );
        assert_eq!(
            mapping.matrix_to_visual_pos(1, 1),
            Some(Position::new(1, 1))
        );

        // Verify visual to matrix
        assert_eq!(mapping.visual_to_matrix_pos(0, 0), Some((0, 0)));
        assert_eq!(mapping.visual_to_matrix_pos(0, 1), Some((0, 1)));
        assert_eq!(mapping.visual_to_matrix_pos(1, 0), Some((1, 0)));
        assert_eq!(mapping.visual_to_matrix_pos(1, 1), Some((1, 1)));

        // Verify visual to LED
        assert_eq!(mapping.visual_to_led_index(0, 0), Some(0));
        assert_eq!(mapping.visual_to_led_index(0, 1), Some(1));
        assert_eq!(mapping.visual_to_led_index(1, 0), Some(2));
        assert_eq!(mapping.visual_to_led_index(1, 1), Some(3));

        assert_eq!(mapping.key_count(), 4);
    }

    #[test]
    fn test_visual_layout_mapping_missing_keys() {
        let mapping = VisualLayoutMapping::new();

        assert_eq!(mapping.led_to_matrix_pos(0), None);
        assert_eq!(mapping.matrix_to_visual_pos(0, 0), None);
        assert_eq!(mapping.visual_to_matrix_pos(0, 0), None);
        assert_eq!(mapping.visual_to_led_index(0, 0), None);
        assert_eq!(mapping.key_count(), 0);
    }

    #[test]
    fn test_visual_layout_mapping_quantization() {
        let mut geometry = KeyboardGeometry::new("test", "LAYOUT", 2, 2);

        // Add keys with fractional positions that should round
        geometry.add_key(KeyGeometry::new((0, 0), 0, 0.3, 0.4)); // Rounds to (0, 0)
        geometry.add_key(KeyGeometry::new((0, 1), 1, 1.6, 0.5)); // Rounds to (1, 2)

        let mapping = VisualLayoutMapping::build(&geometry);

        assert_eq!(
            mapping.matrix_to_visual_pos(0, 0),
            Some(Position::new(0, 0))
        );
        assert_eq!(
            mapping.matrix_to_visual_pos(0, 1),
            Some(Position::new(1, 2))
        );
    }
}
