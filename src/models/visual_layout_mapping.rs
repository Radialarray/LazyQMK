//! Bidirectional coordinate transformation system.

use crate::models::keyboard_geometry::KeyboardGeometry;
use crate::models::layer::Position;
use std::collections::HashMap;

/// Bidirectional coordinate transformation system.
///
/// Manages three coordinate spaces:
/// 1. **Visual** - User's view in Markdown tables (row, col)
/// 2. **Matrix** - Electrical wiring (row, col)
/// 3. **LED** - Sequential LED index
///
/// # Building Process
///
/// 1. Iterate through KeyboardGeometry.keys
/// 2. For each `KeyGeometry`:
///    - `led_to_matrix`[`led_index`] = `matrix_position`
///    - `matrix_to_led`[`matrix_position`] = `led_index`
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
    /// LED index → matrix position (indexed by LED)
    pub led_to_matrix: Vec<(u8, u8)>,
    /// Matrix position → LED index
    pub matrix_to_led: HashMap<(u8, u8), u8>,
    /// Matrix → visual position
    pub matrix_to_visual: HashMap<(u8, u8), Position>,
    /// Visual → matrix position
    pub visual_to_matrix: HashMap<Position, (u8, u8)>,
}

#[allow(dead_code)]
impl VisualLayoutMapping {
    /// Creates a new empty `VisualLayoutMapping`.
    #[must_use] pub fn new() -> Self {
        Self {
            led_to_matrix: Vec::new(),
            matrix_to_led: HashMap::new(),
            matrix_to_visual: HashMap::new(),
            visual_to_matrix: HashMap::new(),
        }
    }

    /// Builds a `VisualLayoutMapping` from `KeyboardGeometry`.
    ///
    /// This computes all coordinate transformations based on the physical
    /// key positions from the QMK info.json layout definition.
    #[must_use] pub fn build(geometry: &KeyboardGeometry) -> Self {
        let mut mapping = Self::new();

        // Pre-allocate led_to_matrix vector
        let max_led = geometry.keys.iter().map(|k| k.led_index).max().unwrap_or(0) as usize;
        mapping.led_to_matrix.resize(max_led + 1, (0, 0));

        for key in &geometry.keys {
            let matrix_pos = key.matrix_position;
            let led_idx = key.led_index;

            // Build LED <-> Matrix mappings
            mapping.led_to_matrix[led_idx as usize] = matrix_pos;
            mapping.matrix_to_led.insert(matrix_pos, led_idx);

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
    #[must_use] pub fn led_to_matrix_pos(&self, led: u8) -> Option<(u8, u8)> {
        self.led_to_matrix.get(led as usize).copied()
    }

    /// Converts matrix position to visual position.
    ///
    /// Used for parsing layouts from files where keys are stored by visual position.
    #[must_use] pub fn matrix_to_visual_pos(&self, row: u8, col: u8) -> Option<Position> {
        self.matrix_to_visual.get(&(row, col)).copied()
    }

    /// Converts visual position to matrix position.
    ///
    /// Used for saving layouts and when user selects a key in the UI.
    #[must_use] pub fn visual_to_matrix_pos(&self, row: u8, col: u8) -> Option<(u8, u8)> {
        self.visual_to_matrix.get(&Position::new(row, col)).copied()
    }

    /// Converts visual position to LED index.
    ///
    /// Used for RGB configuration and LED-based features.
    #[must_use] pub fn visual_to_led_index(&self, row: u8, col: u8) -> Option<u8> {
        let visual_pos = Position::new(row, col);
        let matrix_pos = self.visual_to_matrix.get(&visual_pos)?;
        self.matrix_to_led.get(matrix_pos).copied()
    }

    /// Gets the total number of keys in the mapping.
    #[must_use] pub const fn key_count(&self) -> usize {
        self.led_to_matrix.len()
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
