//! Core layout types — LayoutMetadata and Layout.

use crate::keycode_db::KeycodeDb;
use crate::models::layer::{KeyDefinition, Layer};
use crate::models::{Category, RgbColor};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::IdleEffectSettings;
use super::RgbBrightness;
use super::RgbOverlayRippleSettings;
use super::RgbSaturation;
use super::{
    ComboSettings, PaletteFxSettings, TapDanceAction, TapHoldSettings, UncoloredKeyBehavior,
};

/// File metadata embedded in YAML frontmatter.
///
/// # Validation
///
/// - name must be non-empty, max 100 characters
/// - created must be <= modified
/// - tags must be lowercase, hyphen/alphanumeric only
/// - version must match supported versions (currently "1.0")
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutMetadata {
    /// Layout name (e.g., "My Corne Layout")
    pub name: String,
    /// Long description
    pub description: String,
    /// Creator name
    pub author: String,
    /// Creation timestamp (ISO 8601)
    pub created: DateTime<Utc>,
    /// Last modification timestamp (ISO 8601)
    pub modified: DateTime<Utc>,
    /// Searchable keywords
    pub tags: Vec<String>,
    /// Template flag (saves to templates/ directory)
    pub is_template: bool,
    /// Schema version (e.g., "1.0")
    pub version: String,
    /// QMK layout variant (e.g., "`LAYOUT_split_3x6_3_ex2`")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_variant: Option<String>,

    // === Keyboard-specific settings ===
    // These were moved from config.toml to be per-layout
    /// QMK keyboard path (e.g., "splitkb/halcyon/corne")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<String>,
    /// QMK keymap name (e.g., "`my_custom_keymap`")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keymap_name: Option<String>,
    /// Firmware output format: "uf2", "hex", or "bin"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

impl LayoutMetadata {
    /// Creates new metadata with default values.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        Self::validate_name(&name)?;

        let now = Utc::now();
        Ok(Self {
            name,
            description: String::new(),
            author: String::new(),
            created: now,
            modified: now,
            tags: Vec::new(),
            is_template: false,
            version: "1.0".to_string(),
            layout_variant: None,
            keyboard: None,
            keymap_name: None,
            output_format: None,
        })
    }

    /// Validates metadata name.
    fn validate_name(name: &str) -> Result<()> {
        if name.is_empty() {
            anyhow::bail!("Layout name cannot be empty");
        }

        if name.len() > 100 {
            anyhow::bail!(
                "Layout name '{}' exceeds maximum length of 100 characters (got {})",
                name,
                name.len()
            );
        }

        Ok(())
    }

    /// Updates the modification timestamp to now.
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Adds a tag with validation.
    #[allow(dead_code)] // Public API; tests are in lib target
    pub fn add_tag(&mut self, tag: impl Into<String>) -> Result<()> {
        let tag = tag.into();
        Self::validate_tag(&tag)?;

        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }

        Ok(())
    }

    /// Validates tag format (lowercase, hyphens, alphanumeric).
    #[allow(dead_code)] // Helper for add_tag
    fn validate_tag(tag: &str) -> Result<()> {
        if tag.is_empty() {
            anyhow::bail!("Tag cannot be empty");
        }

        if !tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            anyhow::bail!(
                "Tag '{tag}' must be lowercase with hyphens and alphanumeric characters only"
            );
        }

        Ok(())
    }
}

impl Default for LayoutMetadata {
    fn default() -> Self {
        Self::new("Untitled Layout".to_string()).unwrap()
    }
}

/// Complete keyboard mapping with metadata and multiple layers.
///
/// # Validation
///
/// - At least one layer required (layer 0)
/// - Layer numbers must be sequential without gaps
/// - All layers must have same number of keys (determined by keyboard layout)
/// - Category IDs must be unique within layout
///
/// # Color Resolution
///
/// The Layout provides a four-level color priority system:
/// 1. `KeyDefinition.color_override` (highest)
/// 2. `KeyDefinition.category_id` → Category.color
/// 3. `Layer.category_id` → Category.color
/// 4. `Layer.default_color` (lowest/fallback)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {
    /// File metadata
    pub metadata: LayoutMetadata,
    /// Ordered list of layers (0-N, typically 0-11)
    pub layers: Vec<Layer>,
    /// User-defined categories for organization
    pub categories: Vec<Category>,

    // === RGB Settings ===
    /// Master switch for all RGB LEDs
    #[serde(default = "default_rgb_enabled")]
    pub rgb_enabled: bool,
    /// Global RGB brightness (0-100%)
    #[serde(default)]
    pub rgb_brightness: RgbBrightness,
    /// Global RGB saturation (0-200%)
    #[serde(default)]
    pub rgb_saturation: RgbSaturation,
    /// RGB Matrix default animation speed (0-255)
    /// Controls the speed of RGB animations. 0 = slowest, 255 = fastest
    /// Default: 127 (mid-speed)
    #[serde(default)]
    pub rgb_matrix_default_speed: u8,
    /// RGB Matrix timeout in milliseconds (0 = disabled)
    /// Automatically turns off RGB after this many ms of inactivity
    #[serde(default)]
    pub rgb_timeout_ms: u32,
    /// Behavior for keys without individual or category colors
    #[serde(default, alias = "inactive_key_behavior")]
    pub uncolored_key_behavior: UncoloredKeyBehavior,

    // === Idle Effect Settings ===
    /// Idle effect configuration (timeout, duration, mode)
    #[serde(default)]
    pub idle_effect_settings: IdleEffectSettings,

    // === RGB Overlay Ripple Settings ===
    /// RGB overlay ripple configuration
    #[serde(default)]
    pub rgb_overlay_ripple: RgbOverlayRippleSettings,

    // === PaletteFX Settings ===
    /// `PaletteFX` community module configuration
    #[serde(default)]
    pub palette_fx: PaletteFxSettings,

    // === Tap-Hold Settings ===
    /// Tap-hold configuration (LT, MT, TT timing and behavior)
    #[serde(default)]
    pub tap_hold_settings: TapHoldSettings,

    // === Combo Settings ===
    /// Two-key hold combo configuration (base layer only)
    #[serde(default)]
    pub combo_settings: ComboSettings,

    // === Tap Dance Actions ===
    /// Tap dance action definitions
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tap_dances: Vec<TapDanceAction>,
}

/// Default for `rgb_enabled` is true
const fn default_rgb_enabled() -> bool {
    true
}

impl Layout {
    /// Creates a new Layout with default metadata.
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let metadata = LayoutMetadata::new(name)?;
        Ok(Self {
            metadata,
            layers: Vec::new(),
            categories: Vec::new(),
            rgb_enabled: true,
            rgb_brightness: RgbBrightness::default(),
            rgb_saturation: RgbSaturation::default(),
            rgb_matrix_default_speed: 127,
            rgb_timeout_ms: 0,
            uncolored_key_behavior: UncoloredKeyBehavior::default(),
            idle_effect_settings: IdleEffectSettings::default(),
            rgb_overlay_ripple: RgbOverlayRippleSettings::default(),
            palette_fx: PaletteFxSettings::default(),
            tap_hold_settings: TapHoldSettings::default(),
            combo_settings: ComboSettings::default(),
            tap_dances: Vec::new(),
        })
    }

    /// Adds a layer to this layout.
    pub fn add_layer(&mut self, layer: Layer) -> Result<()> {
        // Validate sequential layer numbers
        if !self.layers.is_empty() {
            let expected_number = self.layers.len() as u8;
            if layer.number != expected_number {
                anyhow::bail!(
                    "Layer numbers must be sequential. Expected layer {}, got {}",
                    expected_number,
                    layer.number
                );
            }
        } else if layer.number != 0 {
            anyhow::bail!("First layer must have number 0, got {}", layer.number);
        }

        self.layers.push(layer);
        self.metadata.touch();
        Ok(())
    }

    /// Gets a reference to the layer at the given index.
    #[must_use]
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// Gets a mutable reference to the layer at the given index.
    #[allow(dead_code)] // Public API; tests are in lib target
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.metadata.touch();
        self.layers.get_mut(index)
    }

    /// Adds a category to this layout.
    pub fn add_category(&mut self, category: Category) -> Result<()> {
        // Check for duplicate ID
        if self.categories.iter().any(|c| c.id == category.id) {
            anyhow::bail!("Category with ID '{}' already exists", category.id);
        }

        self.categories.push(category);
        self.metadata.touch();
        Ok(())
    }

    /// Gets a category by ID.
    #[must_use]
    pub fn get_category(&self, id: &str) -> Option<&Category> {
        self.categories.iter().find(|c| c.id == id)
    }

    /// Removes a category by ID.
    pub fn remove_category(&mut self, id: &str) -> Option<Category> {
        if let Some(index) = self.categories.iter().position(|c| c.id == id) {
            self.metadata.touch();
            Some(self.categories.remove(index))
        } else {
            None
        }
    }

    /// Toggles layer-level RGB colors for a specific layer.
    pub fn toggle_layer_colors(&mut self, layer_idx: usize) -> Option<bool> {
        if let Some(layer) = self.layers.get_mut(layer_idx) {
            layer.toggle_layer_colors();
            self.metadata.touch();
            Some(layer.layer_colors_enabled)
        } else {
            None
        }
    }

    /// Toggles layer-level RGB colors for all layers at once.
    /// Returns the new state (true if any layer has colors enabled after toggle).
    pub fn toggle_all_layer_colors(&mut self) -> bool {
        // If any layer has colors enabled, disable all. Otherwise, enable all.
        let any_enabled = self.layers.iter().any(|l| l.layer_colors_enabled);
        let new_state = !any_enabled;

        for layer in &mut self.layers {
            layer.set_layer_colors_enabled(new_state);
        }
        self.metadata.touch();
        new_state
    }

    /// Resolves the color for a key using the four-level priority system.
    ///
    /// Priority (highest to lowest):
    /// 1. `KeyDefinition.color_override`
    /// 2. `KeyDefinition.category_id` → Category.color
    /// 3. `Layer.category_id` → Category.color
    /// 4. `Layer.default_color` (fallback)
    ///
    /// # Examples
    ///
    /// ```
    /// use lazyqmk::models::{Layout, Layer, KeyDefinition, Category, Position, RgbColor};
    ///
    /// let mut layout = Layout::new("Test").unwrap();
    /// let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();
    ///
    /// let key = KeyDefinition::new(Position::new(0, 0), "KC_A")
    ///     .with_color(RgbColor::new(255, 0, 0));
    ///
    /// layer.add_key(key.clone());
    /// layout.add_layer(layer).unwrap();
    ///
    /// let color = layout.resolve_key_color(0, &key);
    /// assert_eq!(color, RgbColor::new(255, 0, 0)); // Individual override
    /// ```
    #[must_use]
    pub fn resolve_key_color(&self, layer_idx: usize, key: &KeyDefinition) -> RgbColor {
        // 1. Individual key color override (highest priority)
        if let Some(color) = key.color_override {
            return color;
        }

        // 2. Key category color
        if let Some(cat_id) = &key.category_id {
            if let Some(category) = self.get_category(cat_id) {
                return category.color;
            }
        }

        // 3. Layer category color
        if let Some(layer) = self.get_layer(layer_idx) {
            if let Some(cat_id) = &layer.category_id {
                if let Some(category) = self.get_category(cat_id) {
                    return category.color;
                }
            }

            // 4. Layer default color (fallback)
            return layer.default_color;
        }

        // Fallback to white if layer doesn't exist (shouldn't happen)
        RgbColor::default()
    }

    /// Resolves the color for a key, respecting the layer's `colors_enabled` flag.
    ///
    /// When `colors_enabled = false` for a layer:
    /// - Individual key color overrides still work (priority 1)
    /// - Key category colors still work (priority 2)
    /// - Layer category and layer default colors are disabled (priorities 3-4)
    ///
    /// Returns `None` only if the layer has colors disabled AND the key has no
    /// individual color or key category. This allows showing a neutral color
    /// for keys that would normally inherit from the layer.
    #[allow(dead_code)] // Public API; tests are in lib target
    #[must_use]
    pub fn resolve_key_color_if_enabled(
        &self,
        layer_idx: usize,
        key: &KeyDefinition,
    ) -> Option<RgbColor> {
        // 1. Individual key color override (always works)
        if let Some(color) = key.color_override {
            return Some(color);
        }

        // 2. Key category color (always works)
        if let Some(cat_id) = &key.category_id {
            if let Some(category) = self.get_category(cat_id) {
                return Some(category.color);
            }
        }

        // Check if layer colors are enabled
        if let Some(layer) = self.get_layer(layer_idx) {
            if !layer.layer_colors_enabled {
                // Layer colors disabled - return None for layer-level colors
                return None;
            }

            // 3. Layer category color (only if colors_enabled)
            if let Some(cat_id) = &layer.category_id {
                if let Some(category) = self.get_category(cat_id) {
                    return Some(category.color);
                }
            }

            // 4. Layer default color (only if colors_enabled)
            return Some(layer.default_color);
        }

        // Fallback to white if layer doesn't exist (shouldn't happen)
        Some(RgbColor::default())
    }

    /// Resolves the color for a key for display, respecting `uncolored_key_behavior`.
    ///
    /// This method considers the `uncolored_key_behavior` setting for keys that
    /// don't have an individual color or key category. Keys that would normally
    /// inherit from layer-level colors are considered "uncolored" and their
    /// display is modified based on the setting:
    ///
    /// - `ShowColor`: Show the resolved layer color normally
    /// - `Off`: Show black (RGB 0, 0, 0)
    /// - `Dim`: Show the layer color at 50% brightness
    ///
    /// Returns a tuple of (color, `is_key_specific`) where:
    /// - color: The RGB color to display
    /// - `is_key_specific`: true if color came from individual override or key category
    #[must_use]
    pub fn resolve_display_color(&self, layer_idx: usize, key: &KeyDefinition) -> (RgbColor, bool) {
        // 1. Individual key color override (highest priority, key-specific)
        if let Some(color) = key.color_override {
            return (color, true);
        }

        // 2. Key category color (key-specific)
        if let Some(cat_id) = &key.category_id {
            if let Some(category) = self.get_category(cat_id) {
                return (category.color, true);
            }
        }

        // From here, colors are layer-level (not key-specific)
        // Apply uncolored_key_behavior

        // First, check if layer colors are enabled
        if let Some(layer) = self.get_layer(layer_idx) {
            if !layer.layer_colors_enabled {
                // Layer colors disabled entirely - show gray
                return (RgbColor::new(64, 64, 64), false);
            }

            // Get the layer-level color (layer category or default)
            let layer_color = if let Some(cat_id) = &layer.category_id {
                if let Some(category) = self.get_category(cat_id) {
                    category.color
                } else {
                    layer.default_color
                }
            } else {
                layer.default_color
            };

            // Apply uncolored_key_behavior
            // Apply uncolored key brightness: 0=off, 1-99=dim, 100=full color
            let display_color = match self.uncolored_key_behavior.as_percent() {
                0 => RgbColor::new(0, 0, 0),         // Off
                100 => layer_color,                  // Full color
                percent => layer_color.dim(percent), // Dim to percentage
            };

            return (display_color, false);
        }

        // Fallback to white if layer doesn't exist
        (RgbColor::default(), false)
    }

    /// Applies global RGB settings (master switch, saturation, brightness) to a color.
    ///
    /// This should be called after `resolve_display_color` to apply the global
    /// RGB saturation and brightness multipliers, and respect the master switch.
    ///
    /// The order of operations is:
    /// 1. If RGB is disabled, return black
    /// 2. Apply saturation adjustment (0-200%)
    /// 3. Apply brightness multiplier (0-100%)
    ///
    /// Returns the color with saturation and brightness applied, or black if RGB is disabled.
    #[must_use]
    pub fn apply_rgb_settings(&self, color: RgbColor) -> RgbColor {
        // If RGB master switch is off, return black
        if !self.rgb_enabled {
            return RgbColor::new(0, 0, 0);
        }

        // Apply saturation adjustment first
        let saturation_percent = self.rgb_saturation.as_percent();
        let color = if saturation_percent == 100 {
            color
        } else {
            color.saturate(saturation_percent)
        };

        // Then apply brightness multiplier
        let brightness_percent = self.rgb_brightness.as_percent();
        if brightness_percent == 100 {
            color
        } else {
            color.dim(brightness_percent)
        }
    }

    /// Gets a layer by its unique ID.
    #[allow(dead_code)] // Public API; tests are in lib target
    #[must_use]
    pub fn get_layer_by_id(&self, id: &str) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    /// Gets the index of a layer by its unique ID.
    #[must_use]
    pub fn get_layer_index_by_id(&self, id: &str) -> Option<usize> {
        self.layers.iter().position(|layer| layer.id == id)
    }

    /// Adds a tap dance action to the layout.
    pub fn add_tap_dance(&mut self, tap_dance: TapDanceAction) -> Result<()> {
        // Validate the tap dance
        tap_dance.validate()?;

        // Check for duplicate name
        if self.tap_dances.iter().any(|td| td.name == tap_dance.name) {
            anyhow::bail!("Tap dance with name '{}' already exists", tap_dance.name);
        }

        self.tap_dances.push(tap_dance);
        self.metadata.touch();
        Ok(())
    }

    /// Gets a tap dance by name.
    #[must_use]
    pub fn get_tap_dance(&self, name: &str) -> Option<&TapDanceAction> {
        self.tap_dances.iter().find(|td| td.name == name)
    }

    /// Removes a tap dance by name.
    pub fn remove_tap_dance(&mut self, name: &str) -> Option<TapDanceAction> {
        if let Some(index) = self.tap_dances.iter().position(|td| td.name == name) {
            self.metadata.touch();
            Some(self.tap_dances.remove(index))
        } else {
            None
        }
    }

    /// Auto-creates missing tap dance definitions for all `TD()` references in the layout.
    ///
    /// Scans all keycodes for TD(name) patterns and creates placeholder tap dance
    /// definitions for any referenced names that don't have definitions yet.
    ///
    /// Placeholder tap dances use `KC_NO` (no-op) keycodes that users can edit later.
    pub fn auto_create_tap_dances(&mut self) {
        // Collect all TD() references from keys
        let mut referenced_names = std::collections::HashSet::new();
        let td_pattern = regex::Regex::new(r"TD\(([^)]+)\)").unwrap();

        for layer in &self.layers {
            for key in &layer.keys {
                if let Some(captures) = td_pattern.captures(&key.keycode) {
                    let name = captures[1].to_string();
                    referenced_names.insert(name);
                }
            }
        }

        // Auto-create missing tap dance definitions
        for name in &referenced_names {
            if !self.tap_dances.iter().any(|td| &td.name == name) {
                // Create a placeholder tap dance with KC_NO (no-op) keycodes
                // User can edit these later via the TUI
                let placeholder = TapDanceAction {
                    name: name.clone(),
                    single_tap: "KC_NO".to_string(),
                    double_tap: None,
                    hold: None,
                };
                self.tap_dances.push(placeholder);
            }
        }
    }

    /// Validates all tap dance references in the layout.
    ///
    /// Checks:
    /// - Every TD(name) keycode references a defined tap dance
    /// - No duplicate tap dance names
    /// - Warns about orphaned tap dance definitions (defined but not used)
    pub fn validate_tap_dances(&self) -> Result<()> {
        // Collect all TD() references from keys
        let mut referenced_names = std::collections::HashSet::new();
        let td_pattern = regex::Regex::new(r"TD\(([^)]+)\)").unwrap();

        for layer in &self.layers {
            for key in &layer.keys {
                if let Some(captures) = td_pattern.captures(&key.keycode) {
                    let name = captures[1].to_string();
                    referenced_names.insert(name);
                }
            }
        }

        // Check that all referenced tap dances exist
        for name in &referenced_names {
            if !self.tap_dances.iter().any(|td| &td.name == name) {
                anyhow::bail!("Tap dance '{}' is referenced but not defined", name);
            }
        }

        // Check for duplicate names (should be prevented by add_tap_dance, but double-check)
        let mut seen_names = std::collections::HashSet::new();
        for td in &self.tap_dances {
            if !seen_names.insert(&td.name) {
                anyhow::bail!("Duplicate tap dance name: {}", td.name);
            }
        }

        // Note: We don't error on orphaned definitions, just log them as warnings in the UI
        Ok(())
    }

    /// Returns a list of orphaned tap dance names (defined but not used).
    #[must_use]
    pub fn get_orphaned_tap_dances(&self) -> Vec<String> {
        let td_pattern = regex::Regex::new(r"TD\(([^)]+)\)").unwrap();
        let mut referenced_names = std::collections::HashSet::new();

        for layer in &self.layers {
            for key in &layer.keys {
                if let Some(captures) = td_pattern.captures(&key.keycode) {
                    referenced_names.insert(captures[1].to_string());
                }
            }
        }

        self.tap_dances
            .iter()
            .filter(|td| !referenced_names.contains(&td.name))
            .map(|td| td.name.clone())
            .collect()
    }

    /// Resolves layer references in a keycode to layer indices.
    ///
    /// Uses the keycode database to detect layer keycodes dynamically,
    /// then converts @uuid references to the current layer index.
    /// Returns None if the layer reference is invalid.
    #[must_use]
    pub fn resolve_layer_keycode(&self, keycode: &str, keycode_db: &KeycodeDb) -> Option<String> {
        let (prefix, layer_ref, suffix) = keycode_db.parse_layer_keycode(keycode)?;

        // Check if it's a layer ID reference (starts with @)
        let layer_index = if let Some(layer_id) = layer_ref.strip_prefix('@') {
            // Remove @ prefix
            self.get_layer_index_by_id(layer_id)?
        } else {
            // It's already a number, try to parse it
            layer_ref.parse::<usize>().ok()?
        };

        if suffix.is_empty() {
            Some(format!("{prefix}({layer_index})"))
        } else {
            Some(format!("{prefix}({layer_index}{suffix}"))
        }
    }

    /// Creates a layer keycode with a reference to a layer by ID.
    /// Example: `create_layer_keycode("MO`", "abc-123", None) -> "MO(@abc-123)"
    /// Example: `create_layer_keycode("LT`", "abc-123", `Some("KC_SPC`")) -> "LT(@abc-123, `KC_SPC`)"
    #[allow(dead_code)] // Public API; tests are in lib target
    #[must_use]
    pub fn create_layer_keycode(prefix: &str, layer_id: &str, extra: Option<&str>) -> String {
        match extra {
            Some(e) => format!("{prefix}(@{layer_id}, {e})"),
            None => format!("{prefix}(@{layer_id})"),
        }
    }

    /// Validates the layout structure.
    ///
    /// Checks:
    /// - At least one layer exists
    /// - All layers have the same number of keys
    /// - No duplicate positions within each layer
    /// - All category references exist
    /// - All tap dance references are valid
    pub fn validate(&self) -> Result<()> {
        if self.layers.is_empty() {
            anyhow::bail!("Layout must have at least one layer");
        }

        // Check layer numbers are sequential
        for (idx, layer) in self.layers.iter().enumerate() {
            if layer.number != idx as u8 {
                anyhow::bail!(
                    "Layer numbers must be sequential. Layer at index {} has number {}",
                    idx,
                    layer.number
                );
            }
        }

        // Check all layers have same number of keys
        if let Some(first_layer) = self.layers.first() {
            let expected_key_count = first_layer.keys.len();
            for layer in &self.layers {
                if layer.keys.len() != expected_key_count {
                    anyhow::bail!(
                        "All layers must have the same number of keys. Layer {} has {}, expected {}",
                        layer.number,
                        layer.keys.len(),
                        expected_key_count
                    );
                }
            }
        }

        // Check for duplicate positions within each layer
        for layer in &self.layers {
            let mut positions = std::collections::HashSet::new();
            for key in &layer.keys {
                if !positions.insert(key.position) {
                    anyhow::bail!(
                        "Duplicate position ({}, {}) in layer {}",
                        key.position.row,
                        key.position.col,
                        layer.number
                    );
                }
            }
        }

        // Validate category references
        for layer in &self.layers {
            if let Some(cat_id) = &layer.category_id {
                if !self.categories.iter().any(|c| &c.id == cat_id) {
                    anyhow::bail!(
                        "Layer {} references non-existent category '{}'",
                        layer.number,
                        cat_id
                    );
                }
            }

            for key in &layer.keys {
                if let Some(cat_id) = &key.category_id {
                    if !self.categories.iter().any(|c| &c.id == cat_id) {
                        anyhow::bail!(
                            "Key at ({}, {}) in layer {} references non-existent category '{}'",
                            key.position.row,
                            key.position.col,
                            layer.number,
                            cat_id
                        );
                    }
                }
            }
        }

        // Validate tap dance actions and references
        self.validate_tap_dances()?;

        // Validate ripple settings even when loaded from embedded/frontmatter data
        self.rgb_overlay_ripple.validate()?;

        Ok(())
    }
}
