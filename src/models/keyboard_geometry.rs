//! Keyboard geometry definitions from QMK info.json.

// Allow intentional type casts for terminal coordinate calculations
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use serde::{Deserialize, Serialize};

/// Terminal rendering scale factors.
///
/// These constants control how keyboard units (from QMK info.json) are
/// converted to terminal character cells.
pub mod terminal_scale {
    /// Default horizontal scale: characters per keyboard unit (1U = 7 chars)
    pub const DEFAULT_X_SCALE: f32 = 7.0;
    /// Default vertical scale: lines per keyboard unit (1U = 2.5 lines)  
    pub const DEFAULT_Y_SCALE: f32 = 2.5;
    /// Minimum key width in characters (for visibility)
    pub const MIN_WIDTH: u16 = 3;
    /// Minimum key height in lines (for visibility)
    pub const MIN_HEIGHT: u16 = 3;
}

/// Individual key's physical properties from QMK layout definition.
///
/// # Coordinate Conversion (to terminal)
///
/// - Terminal X = `visual_x` * X_SCALE characters per keyboard unit
/// - Terminal Y = `visual_y` * Y_SCALE lines per keyboard unit
/// - Width chars = width * X_SCALE (minimum MIN_WIDTH)
/// - Height lines = height * Y_SCALE (minimum MIN_HEIGHT)
///
/// Default scale factors are 7.0 for X and 2.5 for Y, but these can be
/// customized via the `with_scale` methods for different terminal sizes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyGeometry {
    /// Electrical matrix position (row, col)
    pub matrix_position: (u8, u8),
    /// Physical LED index from `rgb_matrix.layout` (for RGB colors)
    pub led_index: u8,
    /// Layout array index from info.json (for keymap generation)
    pub layout_index: u8,
    /// Physical X position in keyboard units (1u = key width)
    pub visual_x: f32,
    /// Physical Y position in keyboard units
    pub visual_y: f32,
    /// Key width in keyboard units (default 1.0)
    pub width: f32,
    /// Key height in keyboard units (default 1.0)
    pub height: f32,
    /// Rotation in degrees (default 0.0, future use)
    pub rotation: f32,
}

#[allow(dead_code)]
impl KeyGeometry {
    /// Creates a new `KeyGeometry` with the given parameters.
    #[must_use]
    pub const fn new(
        matrix_position: (u8, u8),
        led_index: u8,
        visual_x: f32,
        visual_y: f32,
    ) -> Self {
        Self {
            matrix_position,
            led_index,
            layout_index: led_index, // Default: layout_index equals led_index
            visual_x,
            visual_y,
            width: 1.0,
            height: 1.0,
            rotation: 0.0,
        }
    }

    /// Sets the key width.
    #[must_use]
    pub const fn with_width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Sets the key height.
    #[must_use]
    pub const fn with_height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Sets the key rotation.
    #[must_use]
    pub const fn with_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    /// Converts visual X position to terminal characters.
    /// Uses the default X scale factor.
    #[must_use]
    pub fn terminal_x(&self) -> u16 {
        self.terminal_x_with_scale(terminal_scale::DEFAULT_X_SCALE)
    }

    /// Converts visual Y position to terminal lines.
    /// Uses the default Y scale factor.
    #[must_use]
    pub fn terminal_y(&self) -> u16 {
        self.terminal_y_with_scale(terminal_scale::DEFAULT_Y_SCALE)
    }

    /// Converts key width to terminal characters.
    /// Uses the default X scale factor.
    #[must_use]
    pub fn terminal_width(&self) -> u16 {
        self.terminal_width_with_scale(terminal_scale::DEFAULT_X_SCALE)
    }

    /// Converts key height to terminal lines.
    /// Uses the default Y scale factor.
    #[must_use]
    pub fn terminal_height(&self) -> u16 {
        self.terminal_height_with_scale(terminal_scale::DEFAULT_Y_SCALE)
    }

    /// Converts visual X position to terminal characters with custom scale.
    #[must_use]
    pub fn terminal_x_with_scale(&self, x_scale: f32) -> u16 {
        (self.visual_x * x_scale) as u16
    }

    /// Converts visual Y position to terminal lines with custom scale.
    #[must_use]
    pub fn terminal_y_with_scale(&self, y_scale: f32) -> u16 {
        (self.visual_y * y_scale) as u16
    }

    /// Converts key width to terminal characters with custom scale.
    #[must_use]
    pub fn terminal_width_with_scale(&self, x_scale: f32) -> u16 {
        ((self.width * x_scale) as u16).max(terminal_scale::MIN_WIDTH)
    }

    /// Converts key height to terminal lines with custom scale.
    #[must_use]
    pub fn terminal_height_with_scale(&self, y_scale: f32) -> u16 {
        ((self.height * y_scale) as u16).max(terminal_scale::MIN_HEIGHT)
    }
}

/// Physical keyboard definition loaded from QMK info.json.
///
/// # Validation
///
/// - `keyboard_name` must match QMK directory structure
/// - `layout_name` must exist in keyboard's info.json layouts section
/// - matrix dimensions must match info.json
/// - keys vec size determines supported key count
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardGeometry {
    /// QMK keyboard identifier (e.g., "crkbd")
    pub keyboard_name: String,
    /// Specific layout variant (e.g., "`LAYOUT_split_3x6_3`")
    pub layout_name: String,
    /// Electrical matrix row count (e.g., 8 for split Corne)
    pub matrix_rows: u8,
    /// Electrical matrix column count (e.g., 7)
    pub matrix_cols: u8,
    /// Physical key definitions (one per key)
    pub keys: Vec<KeyGeometry>,
    /// Number of rotary encoders (0 if no encoders)
    #[serde(default)]
    pub encoder_count: u8,
}

#[allow(dead_code)]
impl KeyboardGeometry {
    /// Creates a new `KeyboardGeometry`.
    pub fn new(
        keyboard_name: impl Into<String>,
        layout_name: impl Into<String>,
        matrix_rows: u8,
        matrix_cols: u8,
    ) -> Self {
        Self {
            keyboard_name: keyboard_name.into(),
            layout_name: layout_name.into(),
            matrix_rows,
            matrix_cols,
            keys: Vec::new(),
            encoder_count: 0,
        }
    }

    /// Adds a key to the geometry.
    pub fn add_key(&mut self, key: KeyGeometry) {
        self.keys.push(key);
    }

    /// Gets the total number of keys.
    #[must_use]
    pub const fn key_count(&self) -> usize {
        self.keys.len()
    }

    /// Gets a key by **LED index** (physical wiring order from `rgb_matrix.layout`).
    ///
    /// LED indices are sequential physical wire numbers (0-based). They do not
    /// correspond to matrix positions or visual positions.
    /// Use [`VisualLayoutMapping::visual_to_led_index`] to convert from visual coordinates.
    #[must_use]
    pub fn get_key_by_led(&self, led_index: u8) -> Option<&KeyGeometry> {
        self.keys.iter().find(|k| k.led_index == led_index)
    }

    /// Gets a key by **matrix position** `(row, col)` (electrical wiring coordinates).
    ///
    /// Matrix coordinates come from QMK `info.json` and correspond to the physical
    /// row/column wiring. They differ from visual positions on split keyboards where
    /// the right half uses rows 4–7 while the visual grid is flat.
    /// Use [`VisualLayoutMapping::visual_to_matrix_pos`] to convert from visual coordinates.
    #[must_use]
    pub fn get_key_by_matrix(&self, position: (u8, u8)) -> Option<&KeyGeometry> {
        self.keys.iter().find(|k| k.matrix_position == position)
    }

    /// Checks if the keyboard has RGB matrix LEDs.
    ///
    /// Returns true if there are any keys defined (RGB LED count matches key count).
    #[must_use]
    pub const fn has_rgb_matrix(&self) -> bool {
        !self.keys.is_empty()
    }
}

#[cfg(test)]
mod tests {
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
}
