//! Layer and key definition data structures.

use crate::models::RgbColor;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Default QMK layer limit.
///
/// QMK firmware supports different layer limits depending on configuration:
/// - Default (8 layers): Standard QMK without layer state extensions
/// - 16 layers: With `LAYER_STATE_16BIT` configuration option
/// - 32 layers: With `LAYER_STATE_32BIT` configuration option
///
/// This constant represents the default limit of 8 layers. Validation should
/// allow up to 32 layers by default but warn users about the actual QMK configuration.
#[allow(dead_code)]
pub const DEFAULT_QMK_LAYER_LIMIT: u8 = 8;

/// Maximum QMK layer limit (with LAYER_STATE_32BIT configuration).
///
/// Used as the default validation limit in Layer::new() to be permissive while
/// still preventing invalid layer numbers. Users can use Layer::new() to create
/// layers up to index 31 (0-based), but their QMK firmware configuration will
/// determine the actual supported limit (8, 16, or 32 layers).
pub const MAX_QMK_LAYER_LIMIT: u8 = 32;

/// Validates that a layer number is within the QMK layer limit.
///
/// # Arguments
/// * `number` - The layer number to validate (0-based)
/// * `max_layers` - The maximum number of layers supported (8, 16, or 32)
///
/// # Errors
/// Returns an error if the layer number is >= max_layers.
///
/// # Examples
/// ```
/// use lazyqmk::models::validate_layer_number;
///
/// assert!(validate_layer_number(0, 8).is_ok());
/// assert!(validate_layer_number(7, 8).is_ok());
/// assert!(validate_layer_number(8, 8).is_err());
/// assert!(validate_layer_number(31, 32).is_ok());
/// assert!(validate_layer_number(32, 32).is_err());
/// ```
pub fn validate_layer_number(number: u8, max_layers: u8) -> Result<()> {
    if number >= max_layers {
        anyhow::bail!(
            "Layer number {} exceeds maximum of {} layers. QMK firmware supports:\n\
            - 8 layers (default)\n\
            - 16 layers (with LAYER_STATE_16BIT)\n\
            - 32 layers (with LAYER_STATE_32BIT)\n\
            Enable a higher layer state configuration in rules.mk to support more layers.",
            number,
            max_layers
        );
    }
    Ok(())
}

/// Position in visual grid coordinates (user's view).
///
/// This represents the visual position of a key as it appears in
/// Markdown tables and the UI. Position is converted to matrix
/// coordinates via `VisualLayoutMapping`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Position {
    /// Visual row (0-based, typically 0-3 for most keyboards)
    pub row: u8,
    /// Visual column (0-based, 0-13 for 46-key split, 0-11 for 36-key)
    pub col: u8,
}

impl Position {
    /// Creates a new Position with the given row and column.
    #[must_use]
    pub const fn new(row: u8, col: u8) -> Self {
        Self { row, col }
    }
}

/// Individual key assignment with position, keycode, and optional overrides.
///
/// # Validation
///
/// - Position must be within keyboard geometry bounds
/// - Keycode must exist in `KeycodeDatabase`
/// - Keycode "`KC_TRNS`" (transparent) is always valid
/// - Category ID must exist in parent Layout.categories if Some
/// - Position must be unique within parent Layer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyDefinition {
    /// Visual position (row, col) in the grid
    pub position: Position,
    /// QMK keycode (e.g., "`KC_A`", "`KC_TRNS`", "MO(1)")
    pub keycode: String,
    /// Optional display label (currently unused, future feature)
    pub label: Option<String>,
    /// Individual key color override (highest priority)
    pub color_override: Option<RgbColor>,
    /// Category assignment for this key
    pub category_id: Option<String>,
    /// Flag for combo feature (future use)
    pub combo_participant: bool,
    /// Optional user description for this key (e.g., "Primary thumb key")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[allow(dead_code)]
impl KeyDefinition {
    /// Creates a new `KeyDefinition` with the given position and keycode.
    pub fn new(position: Position, keycode: impl Into<String>) -> Self {
        Self {
            position,
            keycode: keycode.into(),
            label: None,
            color_override: None,
            category_id: None,
            combo_participant: false,
            description: None,
        }
    }

    /// Sets the color override for this key.
    #[must_use]
    pub const fn with_color(mut self, color: RgbColor) -> Self {
        self.color_override = Some(color);
        self
    }

    /// Sets the category for this key.
    pub fn with_category(mut self, category_id: impl Into<String>) -> Self {
        self.category_id = Some(category_id.into());
        self
    }

    /// Sets the display label for this key.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the description for this key.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Checks if this key is transparent (passes through to lower layer).
    #[must_use]
    pub fn is_transparent(&self) -> bool {
        self.keycode == "KC_TRNS" || self.keycode == "KC_TRANSPARENT"
    }

    /// Checks if this key is a no-op (no key at this position).
    #[must_use]
    pub fn is_no_op(&self) -> bool {
        self.keycode == "KC_NO"
    }
}

/// A single layer of the keyboard with color and key assignments.
///
/// # Validation
///
/// - Name must be non-empty, max 50 characters
/// - Number must be unique within parent Layout
/// - Keys vec size must match keyboard layout
/// - All positions must be present (no gaps in coordinate space)
/// - Category ID must exist in parent Layout.categories if Some
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    /// Unique identifier for this layer (stable across renames/reorders)
    #[serde(default = "generate_layer_id")]
    pub id: String,
    /// Layer number (0-based, max 255)
    pub number: u8,
    /// Human-readable name (e.g., "Base", "Lower", "Raise")
    pub name: String,
    /// Base color for all keys on this layer (lowest priority)
    pub default_color: RgbColor,
    /// Optional category assignment for entire layer
    pub category_id: Option<String>,
    /// Key assignments for all positions (fixed size per layout)
    pub keys: Vec<KeyDefinition>,
    /// Whether layer-level RGB colors are enabled (default: true)
    /// When false, layer default color and layer category color are disabled,
    /// but individual key colors and key category colors still work.
    #[serde(default = "default_layer_colors_enabled")]
    pub layer_colors_enabled: bool,
}

/// Generates a new unique layer ID
fn generate_layer_id() -> String {
    Uuid::new_v4().to_string()
}

/// Default value for `layer_colors_enabled` (true)
const fn default_layer_colors_enabled() -> bool {
    true
}

#[allow(dead_code)]
impl Layer {
    /// Creates a new Layer with the given number and name.
    ///
    /// # Arguments
    /// * `number` - Layer number (0-based). Validated against MAX_QMK_LAYER_LIMIT (32).
    /// * `name` - Human-readable layer name (must be non-empty, max 50 characters)
    /// * `default_color` - Base color for this layer
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::{Layer, RgbColor};
    ///
    /// let layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The name is empty or exceeds 50 characters
    /// - The layer number exceeds the maximum QMK layer limit (32)
    pub fn new(number: u8, name: impl Into<String>, default_color: RgbColor) -> Result<Self> {
        let name = name.into();
        Self::validate_name(&name)?;
        validate_layer_number(number, MAX_QMK_LAYER_LIMIT)?;

        Ok(Self {
            id: generate_layer_id(),
            number,
            name,
            default_color,
            category_id: None,
            keys: Vec::new(),
            layer_colors_enabled: true,
        })
    }

    /// Validates layer name.
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Layer name cannot be empty");
        }

        if name.len() > 50 {
            anyhow::bail!(
                "Layer name '{}' exceeds maximum length of 50 characters (got {})",
                name,
                name.len()
            );
        }

        Ok(())
    }

    /// Adds a key definition to this layer.
    pub fn add_key(&mut self, key: KeyDefinition) {
        self.keys.push(key);
    }

    /// Gets a reference to the key at the given position.
    #[must_use]
    pub fn get_key(&self, position: Position) -> Option<&KeyDefinition> {
        self.keys.iter().find(|k| k.position == position)
    }

    /// Gets a mutable reference to the key at the given position.
    pub fn get_key_mut(&mut self, position: Position) -> Option<&mut KeyDefinition> {
        self.keys.iter_mut().find(|k| k.position == position)
    }

    /// Sets the category for this layer.
    pub fn set_category(&mut self, category_id: Option<String>) {
        self.category_id = category_id;
    }

    /// Sets the default color for this layer.
    pub const fn set_default_color(&mut self, color: RgbColor) {
        self.default_color = color;
    }

    /// Updates the layer name with validation.
    pub fn set_name(&mut self, name: impl Into<String>) -> Result<()> {
        let name = name.into();
        Self::validate_name(&name)?;
        self.name = name;
        Ok(())
    }

    /// Toggles layer-level RGB colors on/off.
    pub const fn toggle_layer_colors(&mut self) {
        self.layer_colors_enabled = !self.layer_colors_enabled;
    }

    /// Sets whether layer-level RGB colors are enabled.
    pub const fn set_layer_colors_enabled(&mut self, enabled: bool) {
        self.layer_colors_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_definition_new() {
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        assert_eq!(key.position, pos);
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.label, None);
        assert_eq!(key.color_override, None);
        assert_eq!(key.category_id, None);
        assert!(!key.combo_participant);
    }

    #[test]
    fn test_key_definition_builder() {
        let pos = Position::new(0, 0);
        let color = RgbColor::new(255, 0, 0);
        let key = KeyDefinition::new(pos, "KC_A")
            .with_color(color)
            .with_category("navigation")
            .with_label("A");

        assert_eq!(key.color_override, Some(color));
        assert_eq!(key.category_id, Some("navigation".to_string()));
        assert_eq!(key.label, Some("A".to_string()));
    }

    #[test]
    fn test_key_definition_is_transparent() {
        let key = KeyDefinition::new(Position::new(0, 0), "KC_TRNS");
        assert!(key.is_transparent());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_TRANSPARENT");
        assert!(key.is_transparent());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        assert!(!key.is_transparent());
    }

    #[test]
    fn test_key_definition_is_no_op() {
        let key = KeyDefinition::new(Position::new(0, 0), "KC_NO");
        assert!(key.is_no_op());

        let key = KeyDefinition::new(Position::new(0, 0), "KC_A");
        assert!(!key.is_no_op());
    }

    #[test]
    fn test_layer_new_valid() {
        let color = RgbColor::new(255, 0, 0);
        let layer = Layer::new(0, "Base", color).unwrap();

        assert_eq!(layer.number, 0);
        assert_eq!(layer.name, "Base");
        assert_eq!(layer.default_color, color);
        assert_eq!(layer.category_id, None);
        assert!(layer.keys.is_empty());
    }

    #[test]
    fn test_layer_validate_name() {
        let color = RgbColor::new(255, 0, 0);

        assert!(Layer::new(0, "Base", color).is_ok());
        assert!(Layer::new(0, "A", color).is_ok());
        assert!(Layer::new(0, "", color).is_err());
        assert!(Layer::new(0, "a".repeat(51), color).is_err());
    }

    #[test]
    fn test_layer_add_and_get_key() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        layer.add_key(key.clone());
        let retrieved = layer.get_key(pos).unwrap();
        assert_eq!(retrieved, &key);
    }

    #[test]
    fn test_layer_get_key_mut() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let pos = Position::new(0, 0);
        let key = KeyDefinition::new(pos, "KC_A");

        layer.add_key(key);
        {
            let key_mut = layer.get_key_mut(pos).unwrap();
            key_mut.keycode = "KC_B".to_string();
        }

        assert_eq!(layer.get_key(pos).unwrap().keycode, "KC_B");
    }

    #[test]
    fn test_layer_set_category() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer.set_category(Some("navigation".to_string()));
        assert_eq!(layer.category_id, Some("navigation".to_string()));

        layer.set_category(None);
        assert_eq!(layer.category_id, None);
    }

    #[test]
    fn test_layer_set_default_color() {
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let new_color = RgbColor::new(0, 255, 0);
        layer.set_default_color(new_color);
        assert_eq!(layer.default_color, new_color);
    }

    #[test]
    fn test_validate_layer_number_default_limit() {
        // Test with default 8-layer limit
        assert!(validate_layer_number(0, 8).is_ok());
        assert!(validate_layer_number(1, 8).is_ok());
        assert!(validate_layer_number(7, 8).is_ok());
        assert!(validate_layer_number(8, 8).is_err());
        assert!(validate_layer_number(255, 8).is_err());
    }

    #[test]
    fn test_validate_layer_number_16bit_limit() {
        // Test with 16-layer limit (LAYER_STATE_16BIT)
        assert!(validate_layer_number(0, 16).is_ok());
        assert!(validate_layer_number(7, 16).is_ok());
        assert!(validate_layer_number(15, 16).is_ok());
        assert!(validate_layer_number(16, 16).is_err());
        assert!(validate_layer_number(255, 16).is_err());
    }

    #[test]
    fn test_validate_layer_number_32bit_limit() {
        // Test with 32-layer limit (LAYER_STATE_32BIT)
        assert!(validate_layer_number(0, 32).is_ok());
        assert!(validate_layer_number(15, 32).is_ok());
        assert!(validate_layer_number(31, 32).is_ok());
        assert!(validate_layer_number(32, 32).is_err());
        assert!(validate_layer_number(255, 32).is_err());
    }

    #[test]
    fn test_layer_new_respects_max_layer_limit() {
        let color = RgbColor::new(255, 0, 0);

        // These should succeed (within MAX_QMK_LAYER_LIMIT of 32)
        assert!(Layer::new(0, "Base", color).is_ok());
        assert!(Layer::new(7, "Lower", color).is_ok());
        assert!(Layer::new(15, "Layer15", color).is_ok());
        assert!(Layer::new(31, "Layer31", color).is_ok());

        // This should fail (equals MAX_QMK_LAYER_LIMIT)
        assert!(Layer::new(32, "Layer32", color).is_err());
        assert!(Layer::new(255, "Layer255", color).is_err());
    }

    #[test]
    fn test_validate_layer_number_error_message() {
        let result = validate_layer_number(10, 8);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum"));
        assert!(error_msg.contains("LAYER_STATE_16BIT"));
        assert!(error_msg.contains("LAYER_STATE_32BIT"));
    }
}
