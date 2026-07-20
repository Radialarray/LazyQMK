//! Tests for keyboard_geometry.
//!
//! Auto-extracted from keyboard_geometry.rs.

use super::*;

use super::*;

#[test]
#[allow(clippy::float_cmp)]
fn test_key_geometry_new() {
    let key = KeyGeometry::new((0, 0), 0, 0.0, 0.0);
    assert_eq!(key.matrix_position, (0, 0));
    assert_eq!(key.led_index, 0);
    assert_eq!(key.visual_x, 0.0);
    assert_eq!(key.visual_y, 0.0);
    assert_eq!(key.width, 1.0);
    assert_eq!(key.height, 1.0);
    assert_eq!(key.rotation, 0.0);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_key_geometry_builder() {
    let key = KeyGeometry::new((0, 0), 0, 0.0, 0.0)
        .with_width(1.5)
        .with_height(2.0)
        .with_rotation(15.0);

    assert_eq!(key.width, 1.5);
    assert_eq!(key.height, 2.0);
    assert_eq!(key.rotation, 15.0);
}

#[test]
fn test_key_geometry_terminal_conversion() {
    let key = KeyGeometry::new((0, 0), 0, 2.0, 1.0);
    assert_eq!(key.terminal_x(), 14); // 2.0 * 7
    assert_eq!(key.terminal_y(), 2); // 1.0 * 2.5 = 2.5 -> 2

    assert_eq!(key.terminal_width(), 7); // 1.0 * 7
    assert_eq!(key.terminal_height(), 3); // 1.0 * 2.5 = 2.5 -> 3 (min 3)
}

#[test]
fn test_keyboard_geometry_new() {
    let geom = KeyboardGeometry::new("crkbd", "LAYOUT_split_3x6_3", 8, 7);
    assert_eq!(geom.keyboard_name, "crkbd");
    assert_eq!(geom.layout_name, "LAYOUT_split_3x6_3");
    assert_eq!(geom.matrix_rows, 8);
    assert_eq!(geom.matrix_cols, 7);
    assert_eq!(geom.key_count(), 0);
}

#[test]
fn test_keyboard_geometry_add_key() {
    let mut geom = KeyboardGeometry::new("test", "LAYOUT", 4, 12);
    geom.add_key(KeyGeometry::new((0, 0), 0, 0.0, 0.0));
    geom.add_key(KeyGeometry::new((0, 1), 1, 1.0, 0.0));

    assert_eq!(geom.key_count(), 2);
    assert!(geom.get_key_by_led(0).is_some());
    assert!(geom.get_key_by_led(1).is_some());
    assert!(geom.get_key_by_matrix((0, 0)).is_some());
    assert!(geom.get_key_by_matrix((0, 1)).is_some());
}
