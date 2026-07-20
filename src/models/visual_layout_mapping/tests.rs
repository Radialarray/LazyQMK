//! Tests for visual_layout_mapping.
//!
//! Auto-extracted from visual_layout_mapping.rs.

use super::*;

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

    #[test]
    fn test_find_position_right_large_keyboard() {
        // Test navigation on a keyboard with more than 20 columns (e.g., full-size with numpad)
        let mut geometry = KeyboardGeometry::new("fullsize", "LAYOUT", 5, 22);

        // Add keys in a sparse grid (row 0: cols 0, 5, 10, 15, 20, 21)
        geometry.add_key(KeyGeometry::new((0, 0), 0, 0.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 5), 1, 5.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 10), 2, 10.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 15), 3, 15.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 20), 4, 20.0, 0.0));
        geometry.add_key(KeyGeometry::new((0, 21), 5, 21.0, 0.0));

        let mapping = VisualLayoutMapping::build(&geometry);

        // Verify max_col is set correctly
        let (_, max_col) = mapping.get_bounds();
        assert_eq!(max_col, 21);

        // Test navigation from col 0
        let pos = Position::new(0, 0);
        let next = mapping.find_position_right(pos);
        assert_eq!(next, Some(Position::new(0, 5)));

        // Test navigation from col 15
        let pos = Position::new(0, 15);
        let next = mapping.find_position_right(pos);
        assert_eq!(next, Some(Position::new(0, 20)));

        // Test navigation from col 20 (should find 21, which is beyond old hardcoded limit of 20)
        let pos = Position::new(0, 20);
        let next = mapping.find_position_right(pos);
        assert_eq!(next, Some(Position::new(0, 21)));

        // Test navigation from col 21 (should return None, no more keys)
        let pos = Position::new(0, 21);
        let next = mapping.find_position_right(pos);
        assert_eq!(next, None);
    }
