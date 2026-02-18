//! Layout and metadata data structures.

// Allow small types passed by reference for API consistency
#![allow(clippy::trivially_copy_pass_by_ref)]
// Allow intentional type casts
#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]

use crate::keycode_db::KeycodeDb;
use crate::models::layer::{KeyDefinition, Layer, Position};
use crate::models::{Category, RgbColor};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Brightness level for keys without an individual or category color assignment.
///
/// This controls how keys are displayed when they only have layer-level colors
/// (layer category or layer default) but no individual override or key category.
///
/// - 0 = Off (black LEDs)
/// - 1-99 = Dim to that percentage
/// - 100 = Show full color
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncoloredKeyBehavior(u8);

impl UncoloredKeyBehavior {
    /// Creates a new brightness value (0-100).
    ///
    /// # Panics
    /// Panics if value > 100
    #[must_use]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 100, "Brightness must be 0-100");
        Self(value)
    }

    /// Returns the brightness as a percentage (0-100).
    #[must_use]
    pub const fn as_percent(&self) -> u8 {
        self.0
    }

    /// Full brightness - show resolved color (100%).
    pub const FULL: Self = Self(100);
}

impl Default for UncoloredKeyBehavior {
    fn default() -> Self {
        Self::FULL // Show full color by default
    }
}

impl From<u8> for UncoloredKeyBehavior {
    fn from(value: u8) -> Self {
        Self::new(value.min(100))
    }
}

// ============================================================================
// RGB Settings
// ============================================================================

/// RGB brightness level (0-100%).
///
/// Controls the global brightness multiplier for all RGB LEDs.
/// 0 = LEDs off, 100 = full brightness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbBrightness(u8);

impl RgbBrightness {
    /// Creates a new brightness value (0-100).
    ///
    /// # Panics
    /// Panics if value > 100
    #[must_use]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 100, "Brightness must be 0-100");
        Self(value)
    }

    /// Returns the brightness as a percentage (0-100).
    #[must_use]
    pub const fn as_percent(&self) -> u8 {
        self.0
    }

    /// Full brightness (100%).
    pub const FULL: Self = Self(100);
}

impl Default for RgbBrightness {
    fn default() -> Self {
        Self::FULL
    }
}

impl From<u8> for RgbBrightness {
    fn from(value: u8) -> Self {
        Self::new(value.min(100))
    }
}

/// RGB saturation level (0-200%).
///
/// Controls the global saturation multiplier for all RGB LEDs.
/// 0 = fully desaturated (grayscale), 100 = original colors, 200 = maximum saturation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbSaturation(u8);

impl RgbSaturation {
    /// Creates a new saturation value (0-200).
    ///
    /// # Panics
    /// Panics if value > 200
    #[must_use]
    pub const fn new(value: u8) -> Self {
        assert!(value <= 200, "Saturation must be 0-200");
        Self(value)
    }

    /// Returns the saturation as a percentage (0-200).
    #[must_use]
    pub const fn as_percent(&self) -> u8 {
        self.0
    }

    /// Neutral saturation (100%) - no change to colors.
    pub const NEUTRAL: Self = Self(100);
}

impl Default for RgbSaturation {
    fn default() -> Self {
        Self::NEUTRAL
    }
}

impl From<u8> for RgbSaturation {
    fn from(value: u8) -> Self {
        Self::new(value.min(200))
    }
}

// ============================================================================
// RGB Matrix Effects
// ============================================================================

/// Standard RGB Matrix effect modes available in QMK.
///
/// These correspond to QMK's RGB_MATRIX_* modes and are used for idle effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RgbMatrixEffect {
    /// Solid color (no animation)
    #[default]
    #[serde(rename = "solid_color")]
    SolidColor,

    /// Breathing animation
    #[serde(rename = "breathing")]
    Breathing,

    /// Rainbow moving chevron
    #[serde(rename = "rainbow_moving_chevron")]
    RainbowMovingChevron,

    /// Cycle all LEDs through hue
    #[serde(rename = "cycle_all")]
    CycleAll,

    /// Cycle left to right
    #[serde(rename = "cycle_left_right")]
    CycleLeftRight,

    /// Cycle up and down
    #[serde(rename = "cycle_up_down")]
    CycleUpDown,

    /// Rainbow beacon animation
    #[serde(rename = "rainbow_beacon")]
    RainbowBeacon,

    /// Rainbow pinwheels
    #[serde(rename = "rainbow_pinwheels")]
    RainbowPinwheels,

    /// Jellybean raindrops
    #[serde(rename = "jellybean_raindrops")]
    JellybeanRaindrops,
}

impl RgbMatrixEffect {
    /// Returns all available effects.
    #[must_use]
    #[allow(dead_code)]
    pub const fn all() -> &'static [Self] {
        &[
            Self::SolidColor,
            Self::Breathing,
            Self::RainbowMovingChevron,
            Self::CycleAll,
            Self::CycleLeftRight,
            Self::CycleUpDown,
            Self::RainbowBeacon,
            Self::RainbowPinwheels,
            Self::JellybeanRaindrops,
        ]
    }

    /// Returns a human-readable name for this effect.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::SolidColor => "Solid Color",
            Self::Breathing => "Breathing",
            Self::RainbowMovingChevron => "Rainbow Moving Chevron",
            Self::CycleAll => "Cycle All",
            Self::CycleLeftRight => "Cycle Left/Right",
            Self::CycleUpDown => "Cycle Up/Down",
            Self::RainbowBeacon => "Rainbow Beacon",
            Self::RainbowPinwheels => "Rainbow Pinwheels",
            Self::JellybeanRaindrops => "Jellybean Raindrops",
        }
    }

    /// Returns the QMK RGB_MATRIX_* mode identifier for code generation.
    ///
    /// These map to the RGB_MATRIX_* enum values defined in QMK's rgb_matrix_types.h.
    /// The mode IDs are used in firmware to set the RGB effect mode.
    #[must_use]
    pub const fn qmk_mode_name(&self) -> &'static str {
        match self {
            Self::SolidColor => "RGB_MATRIX_SOLID_COLOR",
            Self::Breathing => "RGB_MATRIX_BREATHING",
            Self::RainbowMovingChevron => "RGB_MATRIX_RAINBOW_MOVING_CHEVRON",
            Self::CycleAll => "RGB_MATRIX_CYCLE_ALL",
            Self::CycleLeftRight => "RGB_MATRIX_CYCLE_LEFT_RIGHT",
            Self::CycleUpDown => "RGB_MATRIX_CYCLE_UP_DOWN",
            Self::RainbowBeacon => "RGB_MATRIX_RAINBOW_BEACON",
            Self::RainbowPinwheels => "RGB_MATRIX_RAINBOW_PINWHEELS",
            Self::JellybeanRaindrops => "RGB_MATRIX_JELLYBEAN_RAINDROPS",
        }
    }

    /// Parses an effect from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase().replace([' ', '_', '-'], "");
        match name_lower.as_str() {
            "solidcolor" | "solid" => Some(Self::SolidColor),
            "breathing" | "breath" => Some(Self::Breathing),
            "rainbowmovingchevron" | "chevron" => Some(Self::RainbowMovingChevron),
            "cycleall" | "cycle" => Some(Self::CycleAll),
            "cycleleftright" | "leftright" => Some(Self::CycleLeftRight),
            "cycleupdown" | "updown" => Some(Self::CycleUpDown),
            "rainbowbeacon" | "beacon" => Some(Self::RainbowBeacon),
            "rainbowpinwheels" | "pinwheels" => Some(Self::RainbowPinwheels),
            "jellybeanraindrops" | "raindrops" | "jellybean" => Some(Self::JellybeanRaindrops),
            _ => None,
        }
    }
}

// ============================================================================
// Idle Effect Settings
// ============================================================================

/// Configuration for idle effect behavior.
///
/// When the keyboard is idle (no key presses for `idle_timeout_ms`), it can
/// trigger a special RGB effect. After `idle_effect_duration_ms`, the effect
/// stops and RGB turns off or returns to normal state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdleEffectSettings {
    /// Whether idle effect is enabled
    #[serde(default = "default_idle_effect_enabled")]
    pub enabled: bool,

    /// Time in milliseconds before entering idle effect (0 = disabled)
    /// Default: 60000ms (1 minute)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_ms: u32,

    /// Duration in milliseconds to run the idle effect (0 = immediate off)
    /// Default: 300000ms (5 minutes)
    #[serde(default = "default_idle_duration")]
    pub idle_effect_duration_ms: u32,

    /// Which RGB matrix effect to use during idle
    #[serde(default)]
    pub idle_effect_mode: RgbMatrixEffect,
}

const fn default_idle_effect_enabled() -> bool {
    true
}

const fn default_idle_timeout() -> u32 {
    60_000 // 1 minute
}

const fn default_idle_duration() -> u32 {
    300_000 // 5 minutes
}

impl Default for IdleEffectSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            idle_timeout_ms: 60_000,
            idle_effect_duration_ms: 300_000,
            idle_effect_mode: RgbMatrixEffect::Breathing,
        }
    }
}

impl IdleEffectSettings {
    /// Checks if any settings differ from defaults.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        let defaults = Self::default();
        self.enabled != defaults.enabled
            || self.idle_timeout_ms != defaults.idle_timeout_ms
            || self.idle_effect_duration_ms != defaults.idle_effect_duration_ms
            || self.idle_effect_mode != defaults.idle_effect_mode
    }
}

// ============================================================================
// RGB Overlay Ripple Settings
// ============================================================================

/// Color mode for ripple overlay effects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RippleColorMode {
    /// Use a fixed color for all ripples
    #[default]
    #[serde(rename = "fixed")]
    Fixed,
    /// Use the key's base color (from layer colors)
    #[serde(rename = "key_based")]
    KeyBased,
    /// Shift the hue by a fixed amount from the key's base color
    #[serde(rename = "hue_shift")]
    HueShift,
}

impl RippleColorMode {
    /// Returns all available color modes
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[Self::Fixed, Self::KeyBased, Self::HueShift]
    }

    /// Returns a human-readable display name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Fixed => "Fixed Color",
            Self::KeyBased => "Key Color",
            Self::HueShift => "Hue Shift",
        }
    }

    /// Returns a description of the color mode
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Fixed => "Use the same color for all ripples",
            Self::KeyBased => "Use each key's base color from layer settings",
            Self::HueShift => "Shift hue from key's base color by fixed degrees",
        }
    }
}

/// Configuration for RGB overlay ripple effects.
///
/// Ripples are triggered on keypress and rendered as an additive overlay
/// on top of the base TUI layer colors using `rgb_matrix_indicators_advanced_user`.
#[allow(clippy::struct_excessive_bools)] // Configuration flags are naturally bools
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RgbOverlayRippleSettings {
    /// Whether ripple overlay is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Maximum number of concurrent ripples (1-8)
    /// Default: 4
    #[serde(default = "default_max_ripples")]
    pub max_ripples: u8,

    /// Duration of each ripple in milliseconds
    /// Default: 500ms
    #[serde(default = "default_ripple_duration_ms")]
    pub duration_ms: u16,

    /// Speed multiplier (0-255, higher = faster expansion)
    /// Default: 128
    #[serde(default = "default_ripple_speed")]
    pub speed: u8,

    /// Band width in LED units
    /// Default: 3
    #[serde(default = "default_ripple_band_width")]
    pub band_width: u8,

    /// Amplitude as percentage of base brightness (0-100)
    /// Default: 50
    #[serde(default = "default_ripple_amplitude_pct")]
    pub amplitude_pct: u8,

    /// Color mode for ripples
    #[serde(default)]
    pub color_mode: RippleColorMode,

    /// Fixed color (used when color_mode = Fixed)
    #[serde(default = "default_ripple_fixed_color")]
    pub fixed_color: RgbColor,

    /// Hue shift in degrees (used when color_mode = HueShift)
    /// Default: 60 (complementary color)
    #[serde(default = "default_ripple_hue_shift")]
    pub hue_shift_deg: i16,

    /// Trigger on key press
    #[serde(default = "default_true")]
    pub trigger_on_press: bool,

    /// Trigger on key release
    #[serde(default)]
    pub trigger_on_release: bool,

    /// Ignore transparent keys (KC_TRNS)
    #[serde(default = "default_true")]
    pub ignore_transparent: bool,

    /// Ignore modifier keys
    #[serde(default)]
    pub ignore_modifiers: bool,

    /// Ignore layer switch keys
    #[serde(default)]
    pub ignore_layer_switch: bool,
}

const fn default_max_ripples() -> u8 {
    4
}

const fn default_ripple_duration_ms() -> u16 {
    500
}

const fn default_ripple_speed() -> u8 {
    128
}

const fn default_ripple_band_width() -> u8 {
    3
}

const fn default_ripple_amplitude_pct() -> u8 {
    50
}

fn default_ripple_fixed_color() -> RgbColor {
    RgbColor::new(0, 255, 255) // Cyan
}

const fn default_ripple_hue_shift() -> i16 {
    60
}

const fn default_true() -> bool {
    true
}

impl Default for RgbOverlayRippleSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            max_ripples: 4,
            duration_ms: 500,
            speed: 128,
            band_width: 3,
            amplitude_pct: 50,
            color_mode: RippleColorMode::Fixed,
            fixed_color: RgbColor::new(0, 255, 255),
            hue_shift_deg: 60,
            trigger_on_press: true,
            trigger_on_release: false,
            ignore_transparent: true,
            ignore_modifiers: false,
            ignore_layer_switch: false,
        }
    }
}

impl RgbOverlayRippleSettings {
    /// Validates the settings.
    pub fn validate(&self) -> Result<()> {
        if self.max_ripples == 0 || self.max_ripples > 8 {
            anyhow::bail!("max_ripples must be between 1 and 8");
        }
        if self.amplitude_pct > 100 {
            anyhow::bail!("amplitude_pct must be between 0 and 100");
        }
        if self.hue_shift_deg < -180 || self.hue_shift_deg > 180 {
            anyhow::bail!("hue_shift_deg must be between -180 and 180");
        }
        Ok(())
    }

    /// Checks if any settings differ from defaults.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        let defaults = Self::default();
        self.enabled != defaults.enabled
            || self.max_ripples != defaults.max_ripples
            || self.duration_ms != defaults.duration_ms
            || self.speed != defaults.speed
            || self.band_width != defaults.band_width
            || self.amplitude_pct != defaults.amplitude_pct
            || self.color_mode != defaults.color_mode
            || self.fixed_color != defaults.fixed_color
            || self.hue_shift_deg != defaults.hue_shift_deg
            || self.trigger_on_press != defaults.trigger_on_press
            || self.trigger_on_release != defaults.trigger_on_release
            || self.ignore_transparent != defaults.ignore_transparent
            || self.ignore_modifiers != defaults.ignore_modifiers
            || self.ignore_layer_switch != defaults.ignore_layer_switch
    }
}

// ============================================================================
// Combo Settings
// ============================================================================

/// Action to perform when a combo is activated.
///
/// Combos are two-key combinations that trigger special actions when held together.
/// All combos are restricted to the base layer (layer 0) only.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ComboAction {
    /// Disable RGB effects and revert to static TUI layer colors
    DisableEffects,
    /// Turn off all RGB lighting completely
    DisableLighting,
    /// Enter bootloader mode for firmware flashing
    Bootloader,
}

impl ComboAction {
    /// Returns all available combo actions.
    /// Part of public API for future UI/settings integration.
    #[must_use]
    #[allow(dead_code)]
    pub const fn all() -> &'static [Self] {
        &[
            Self::DisableEffects,
            Self::DisableLighting,
            Self::Bootloader,
        ]
    }

    /// Returns a human-readable name for this action.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::DisableEffects => "Disable Effects",
            Self::DisableLighting => "Disable Lighting",
            Self::Bootloader => "Bootloader",
        }
    }

    /// Returns a description of this action.
    /// Part of public API for future UI/settings integration.
    #[must_use]
    #[allow(dead_code)]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::DisableEffects => "Disable RGB effects and revert to TUI layer colors",
            Self::DisableLighting => "Turn off all RGB lighting completely",
            Self::Bootloader => "Enter bootloader mode for firmware flashing",
        }
    }

    /// Parses an action from a string name (case-insensitive).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase().replace([' ', '_', '-'], "");
        match name_lower.as_str() {
            "disableeffects" | "effects" => Some(Self::DisableEffects),
            "disablelighting" | "lighting" | "off" => Some(Self::DisableLighting),
            "bootloader" | "boot" | "flash" => Some(Self::Bootloader),
            _ => None,
        }
    }
}

/// A two-key hold combo configuration.
///
/// Combos detect when two specific keys are held together on the base layer
/// and trigger a special action after a configurable hold duration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComboDefinition {
    /// First key position (matrix coordinates)
    pub key1: Position,
    /// Second key position (matrix coordinates)
    pub key2: Position,
    /// Action to perform when combo is held
    pub action: ComboAction,
    /// Duration in milliseconds both keys must be held to activate
    /// Default: 500ms
    #[serde(default = "default_combo_hold_duration")]
    pub hold_duration_ms: u16,
}

const fn default_combo_hold_duration() -> u16 {
    500 // 500ms default
}

impl ComboDefinition {
    /// Creates a new combo with default hold duration.
    #[must_use]
    pub fn new(key1: Position, key2: Position, action: ComboAction) -> Self {
        Self {
            key1,
            key2,
            action,
            hold_duration_ms: default_combo_hold_duration(),
        }
    }

    /// Creates a new combo with custom hold duration.
    #[must_use]
    pub fn with_duration(
        key1: Position,
        key2: Position,
        action: ComboAction,
        hold_duration_ms: u16,
    ) -> Self {
        Self {
            key1,
            key2,
            action,
            hold_duration_ms,
        }
    }

    /// Validates the combo definition.
    /// Part of public API for future validation in UI/settings.
    ///
    /// Checks:
    /// - Key positions are different
    /// - Hold duration is reasonable (50-2000ms)
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        if self.key1 == self.key2 {
            anyhow::bail!(
                "Combo keys must be different (both are at row {}, col {})",
                self.key1.row,
                self.key1.col
            );
        }

        if self.hold_duration_ms < 50 || self.hold_duration_ms > 2000 {
            anyhow::bail!(
                "Combo hold duration must be between 50 and 2000ms (got {}ms)",
                self.hold_duration_ms
            );
        }

        Ok(())
    }
}

/// Configuration for two-key hold combos.
///
/// Supports up to three custom combos that are active only on the base layer (layer 0).
/// Each combo triggers when two specific keys are held together for a minimum duration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ComboSettings {
    /// Whether combo feature is enabled
    #[serde(default)]
    pub enabled: bool,

    /// List of combo definitions (max 3)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub combos: Vec<ComboDefinition>,
}

impl ComboSettings {
    /// Creates new combo settings with enabled flag.
    /// Part of public API for future UI/settings integration.
    #[must_use]
    #[allow(dead_code)]
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            combos: Vec::new(),
        }
    }

    /// Adds a combo definition.
    /// Part of public API for future UI/settings integration.
    #[allow(dead_code)]
    pub fn add_combo(&mut self, combo: ComboDefinition) -> Result<()> {
        if self.combos.len() >= 3 {
            anyhow::bail!("Maximum of 3 combos allowed");
        }

        combo.validate()?;

        // Check for duplicate key pairs (order-independent)
        for existing in &self.combos {
            if (existing.key1 == combo.key1 && existing.key2 == combo.key2)
                || (existing.key1 == combo.key2 && existing.key2 == combo.key1)
            {
                anyhow::bail!(
                    "Combo with keys ({},{}) and ({},{}) already exists",
                    combo.key1.row,
                    combo.key1.col,
                    combo.key2.row,
                    combo.key2.col
                );
            }
        }

        self.combos.push(combo);
        Ok(())
    }

    /// Removes a combo by index.
    /// Part of public API for future UI/settings integration.
    #[allow(dead_code)]
    pub fn remove_combo(&mut self, index: usize) -> Option<ComboDefinition> {
        if index < self.combos.len() {
            Some(self.combos.remove(index))
        } else {
            None
        }
    }

    /// Validates all combo definitions.
    /// Part of public API for future validation in UI/settings.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<()> {
        for combo in &self.combos {
            combo.validate()?;
        }
        Ok(())
    }

    /// Checks if any settings differ from defaults.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        self.enabled || !self.combos.is_empty()
    }
}

// ============================================================================
// Tap Dance Settings
// ============================================================================

/// A tap dance action that performs different keycodes based on tap count.
///
/// Tap dances support 2-way (single/double tap) and 3-way (single/double/hold) patterns.
/// The hold action activates when the key is held past the tapping term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TapDanceAction {
    /// Unique name for this tap dance (used in TD() references)
    /// Must be a valid C identifier (alphanumeric + underscore)
    pub name: String,
    /// Keycode sent on single tap
    pub single_tap: String,
    /// Optional keycode sent on double tap (None = 2-way disabled, single tap repeats)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_tap: Option<String>,
    /// Optional keycode sent on hold (None = no hold action)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hold: Option<String>,
}

// Keep public API methods for future TUI editor (Phase 3) and firmware generation (Phase 4)
#[allow(dead_code)]
impl TapDanceAction {
    /// Creates a new tap dance with only single tap defined.
    ///
    /// Use the builder methods `with_double_tap()` and `with_hold()` to add more actions.
    pub fn new(name: impl Into<String>, single_tap: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            single_tap: single_tap.into(),
            double_tap: None,
            hold: None,
        }
    }

    /// Adds a double-tap action (builder pattern).
    pub fn with_double_tap(mut self, keycode: impl Into<String>) -> Self {
        self.double_tap = Some(keycode.into());
        self
    }

    /// Adds a hold action (builder pattern).
    pub fn with_hold(mut self, keycode: impl Into<String>) -> Self {
        self.hold = Some(keycode.into());
        self
    }

    /// Validates the tap dance action.
    ///
    /// Checks:
    /// - Name is non-empty and valid C identifier
    /// - Single tap keycode is non-empty
    /// - Double tap keycode (if present) is non-empty
    /// - Hold keycode (if present) is non-empty
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Tap dance name cannot be empty");
        }

        // Validate name is a valid C identifier (alphanumeric + underscore)
        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            anyhow::bail!(
                "Tap dance name '{}' must be alphanumeric with underscores only",
                self.name
            );
        }

        if self.single_tap.is_empty() {
            anyhow::bail!("Tap dance '{}' must have a single_tap keycode", self.name);
        }

        if let Some(ref double_tap) = self.double_tap {
            if double_tap.is_empty() {
                anyhow::bail!("Tap dance '{}': double_tap cannot be empty", self.name);
            }
        }

        if let Some(ref hold) = self.hold {
            if hold.is_empty() {
                anyhow::bail!("Tap dance '{}': hold cannot be empty", self.name);
            }
        }

        Ok(())
    }

    /// Returns true if this is a 2-way tap dance (single/double).
    /// Used in firmware generation to determine QMK macro type.
    #[must_use]
    pub const fn is_two_way(&self) -> bool {
        self.double_tap.is_some() && self.hold.is_none()
    }

    /// Returns true if this is a 3-way tap dance (single/double/hold).
    /// Used in firmware generation to determine QMK macro type.
    #[must_use]
    pub const fn is_three_way(&self) -> bool {
        self.double_tap.is_some() && self.hold.is_some()
    }

    /// Returns true if this tap dance has a hold action (2-way or 3-way).
    /// Used in firmware generation for conditional logic.
    #[must_use]
    pub const fn has_hold(&self) -> bool {
        self.hold.is_some()
    }
}

// ============================================================================
// Tap-Hold Settings
// ============================================================================

/// How QMK decides between tap and hold when other keys are involved.
///
/// These modes affect what happens when you press a tap-hold key and then
/// press another key before releasing the first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HoldDecisionMode {
    /// Only timing-based: hold if key held longer than `tapping_term`.
    /// Most conservative, requires deliberate holds.
    #[default]
    Default,

    /// Hold when another key is tapped (pressed and released) during hold.
    /// Pattern: A↓ B↓ B↑ A↑ (nested) triggers hold.
    /// Good for home-row mods with fast typing.
    PermissiveHold,

    /// Hold when another key is pressed during hold.
    /// Pattern: A↓ B↓ triggers hold immediately.
    /// Most aggressive, fastest hold detection.
    HoldOnOtherKeyPress,
}

impl HoldDecisionMode {
    /// Returns all available modes.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::PermissiveHold,
            Self::HoldOnOtherKeyPress,
        ]
    }

    /// Returns a human-readable name for this mode.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default (Timing Only)",
            Self::PermissiveHold => "Permissive Hold",
            Self::HoldOnOtherKeyPress => "Hold On Other Key",
        }
    }

    /// Returns a description of this mode.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Default => "Hold only when held past tapping term",
            Self::PermissiveHold => "Hold when another key is tapped during hold",
            Self::HoldOnOtherKeyPress => "Hold when another key is pressed during hold",
        }
    }

    /// Returns the QMK #define name for this mode (if not default).
    #[must_use]
    pub const fn config_define(&self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::PermissiveHold => Some("PERMISSIVE_HOLD"),
            Self::HoldOnOtherKeyPress => Some("HOLD_ON_OTHER_KEY_PRESS"),
        }
    }
}

/// Preset configurations for tap-hold behavior.
///
/// These provide sensible defaults for common use cases while allowing
/// full customization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TapHoldPreset {
    /// QMK defaults - conservative timing, suitable for beginners
    #[default]
    Default,

    /// Optimized for home row mods - lower tapping term, flow tap, chordal hold
    HomeRowMods,

    /// Very responsive - quick hold detection for gaming or fast modifier access
    Responsive,

    /// Deliberate - higher tapping term, requires intentional holds
    Deliberate,

    /// User has customized values
    Custom,
}

impl TapHoldPreset {
    /// Returns all available presets.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        &[
            Self::Default,
            Self::HomeRowMods,
            Self::Responsive,
            Self::Deliberate,
            Self::Custom,
        ]
    }

    /// Returns a human-readable name for this preset.
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Default => "Default",
            Self::HomeRowMods => "Home Row Mods",
            Self::Responsive => "Responsive",
            Self::Deliberate => "Deliberate",
            Self::Custom => "Custom",
        }
    }

    /// Returns a description of this preset.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Default => "QMK defaults, conservative timing",
            Self::HomeRowMods => "Optimized for home row modifiers",
            Self::Responsive => "Fast hold detection for gaming",
            Self::Deliberate => "Requires intentional holds",
            Self::Custom => "User-defined settings",
        }
    }

    /// Returns the settings for this preset.
    #[must_use]
    pub fn settings(&self) -> TapHoldSettings {
        match self {
            Self::Default | Self::Custom => TapHoldSettings::default(),
            Self::HomeRowMods => TapHoldSettings {
                tapping_term: 175,
                quick_tap_term: Some(120),
                hold_mode: HoldDecisionMode::PermissiveHold,
                retro_tapping: true,
                tapping_toggle: 5,
                flow_tap_term: Some(150),
                chordal_hold: true,
                preset: Self::HomeRowMods,
            },
            Self::Responsive => TapHoldSettings {
                tapping_term: 150,
                quick_tap_term: Some(100),
                hold_mode: HoldDecisionMode::HoldOnOtherKeyPress,
                retro_tapping: false,
                tapping_toggle: 5,
                flow_tap_term: None,
                chordal_hold: false,
                preset: Self::Responsive,
            },
            Self::Deliberate => TapHoldSettings {
                tapping_term: 250,
                quick_tap_term: None,
                hold_mode: HoldDecisionMode::Default,
                retro_tapping: false,
                tapping_toggle: 5,
                flow_tap_term: None,
                chordal_hold: false,
                preset: Self::Deliberate,
            },
        }
    }
}

/// Configuration for tap-hold behavior (LT, MT, TT, etc.).
///
/// These settings control how QMK distinguishes between tap and hold actions
/// for dual-function keys like Layer-Tap and Mod-Tap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TapHoldSettings {
    // === Core Timing ===
    /// Time in milliseconds to distinguish tap from hold.
    /// If a key is held longer than this, it's a hold; otherwise it's a tap.
    /// Range: 100-500ms, Default: 200ms
    pub tapping_term: u16,

    /// Time window for tap-then-hold to trigger auto-repeat instead of hold.
    /// If you tap and then hold within this time, it repeats the tap action.
    /// None = same as `tapping_term` (QMK default behavior).
    /// Range: 0-500ms
    pub quick_tap_term: Option<u16>,

    // === Decision Mode ===
    /// How to decide between tap and hold when other keys are involved.
    pub hold_mode: HoldDecisionMode,

    // === Special Behaviors ===
    /// Send tap keycode even if held past tapping term (when no other key pressed).
    /// Useful for home-row mods to avoid accidental modifiers.
    pub retro_tapping: bool,

    /// Number of taps required to toggle layer with `TT()` keys.
    /// Range: 1-10, Default: 5
    pub tapping_toggle: u8,

    // === Home Row Mod Optimizations ===
    /// Flow Tap: time window where rapid typing triggers tap, not hold.
    /// Helps prevent accidental modifiers during fast typing.
    /// None = disabled, Some(ms) = enabled with specified term.
    /// Range: 0-300ms
    pub flow_tap_term: Option<u16>,

    /// Chordal Hold: use opposite-hand rule for tap-hold decision.
    /// Same-hand key presses favor tap, opposite-hand presses favor hold.
    /// Excellent for home-row mods.
    pub chordal_hold: bool,

    // === Preset Tracking ===
    /// Which preset these settings are based on (Custom if modified).
    #[serde(default)]
    pub preset: TapHoldPreset,
}

impl Default for TapHoldSettings {
    fn default() -> Self {
        Self {
            tapping_term: 200,
            quick_tap_term: None,
            hold_mode: HoldDecisionMode::Default,
            retro_tapping: false,
            tapping_toggle: 5,
            flow_tap_term: None,
            chordal_hold: false,
            preset: TapHoldPreset::Default,
        }
    }
}

impl TapHoldSettings {
    /// Creates settings from a preset.
    #[must_use]
    pub fn from_preset(preset: TapHoldPreset) -> Self {
        preset.settings()
    }

    /// Applies a preset, updating all values.
    pub fn apply_preset(&mut self, preset: TapHoldPreset) {
        *self = Self::from_preset(preset);
    }

    /// Marks settings as custom (called when any value is manually changed).
    pub const fn mark_custom(&mut self) {
        self.preset = TapHoldPreset::Custom;
    }

    /// Checks if `tapping_term` differs from QMK default.
    #[must_use]
    pub const fn has_custom_tapping_term(&self) -> bool {
        self.tapping_term != 200
    }

    /// Checks if `quick_tap_term` is explicitly set.
    #[must_use]
    pub const fn has_custom_quick_tap_term(&self) -> bool {
        self.quick_tap_term.is_some()
    }

    /// Checks if any non-default settings are configured.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        self.has_custom_tapping_term()
            || self.has_custom_quick_tap_term()
            || self.hold_mode != HoldDecisionMode::Default
            || self.retro_tapping
            || self.tapping_toggle != 5
            || self.flow_tap_term.is_some()
            || self.chordal_hold
    }

    /// Validates settings are within acceptable ranges.
    pub fn validate(&self) -> Result<()> {
        if self.tapping_term < 50 || self.tapping_term > 1000 {
            anyhow::bail!("Tapping term must be between 50 and 1000ms");
        }
        if let Some(qt) = self.quick_tap_term {
            if qt > 1000 {
                anyhow::bail!("Quick tap term must be at most 1000ms");
            }
        }
        if self.tapping_toggle < 1 || self.tapping_toggle > 10 {
            anyhow::bail!("Tapping toggle must be between 1 and 10");
        }
        if let Some(ft) = self.flow_tap_term {
            if ft > 500 {
                anyhow::bail!("Flow tap term must be at most 500ms");
            }
        }
        Ok(())
    }
}

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
    /// QMK keymap name (e.g., "my_custom_keymap")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keymap_name: Option<String>,
    /// Firmware output format: "uf2", "hex", or "bin"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_format: Option<String>,
}

#[allow(dead_code)]
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

    /// Sets the description.
    pub fn set_description(&mut self, description: impl Into<String>) {
        self.description = description.into();
        self.touch();
    }

    /// Sets the author.
    pub fn set_author(&mut self, author: impl Into<String>) {
        self.author = author.into();
        self.touch();
    }

    /// Adds a tag with validation.
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

/// Default for rgb_enabled is true
const fn default_rgb_enabled() -> bool {
    true
}

#[allow(dead_code)]
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

    /// Gets a mutable reference to a category by ID.
    pub fn get_category_mut(&mut self, id: &str) -> Option<&mut Category> {
        self.metadata.touch();
        self.categories.iter_mut().find(|c| c.id == id)
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

    /// Checks if any layer has layer-level colors enabled.
    #[must_use]
    pub fn any_layer_colors_enabled(&self) -> bool {
        self.layers.iter().any(|l| l.layer_colors_enabled)
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
    /// This should be called after resolve_display_color to apply the global
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

    /// Gets a mutable reference to a tap dance by name.
    pub fn get_tap_dance_mut(&mut self, name: &str) -> Option<&mut TapDanceAction> {
        self.metadata.touch();
        self.tap_dances.iter_mut().find(|td| td.name == name)
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

    /// Auto-creates missing tap dance definitions for all TD() references in the layout.
    ///
    /// Scans all keycodes for TD(name) patterns and creates placeholder tap dance
    /// definitions for any referenced names that don't have definitions yet.
    ///
    /// Placeholder tap dances use KC_NO (no-op) keycodes that users can edit later.
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_metadata_new() {
        let metadata = LayoutMetadata::new("Test Layout").unwrap();
        assert_eq!(metadata.name, "Test Layout");
        assert!(metadata.description.is_empty());
        assert!(metadata.author.is_empty());
        assert!(metadata.tags.is_empty());
        assert!(!metadata.is_template);
        assert_eq!(metadata.version, "1.0");
    }

    #[test]
    fn test_layout_metadata_validate_name() {
        assert!(LayoutMetadata::new("Valid Name").is_ok());
        assert!(LayoutMetadata::new("").is_err());
        assert!(LayoutMetadata::new("a".repeat(101)).is_err());
    }

    #[test]
    fn test_layout_metadata_add_tag() {
        let mut metadata = LayoutMetadata::new("Test").unwrap();
        metadata.add_tag("programming").unwrap();
        metadata.add_tag("vim").unwrap();

        assert_eq!(metadata.tags, vec!["programming", "vim"]);

        // Duplicate tag should not be added
        metadata.add_tag("programming").unwrap();
        assert_eq!(metadata.tags, vec!["programming", "vim"]);
    }

    #[test]
    fn test_layout_new() {
        let layout = Layout::new("Test Layout").unwrap();
        assert_eq!(layout.metadata.name, "Test Layout");
        assert!(layout.layers.is_empty());
        assert!(layout.categories.is_empty());
    }

    #[test]
    fn test_layout_add_layer() {
        let mut layout = Layout::new("Test").unwrap();
        let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let layer1 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_layer(layer0).is_ok());
        assert!(layout.add_layer(layer1).is_ok());
        assert_eq!(layout.layers.len(), 2);
    }

    #[test]
    fn test_layout_add_layer_sequential_validation() {
        let mut layout = Layout::new("Test").unwrap();
        let layer0 = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        let layer2 = Layer::new(2, "Skip", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_layer(layer0).is_ok());
        assert!(layout.add_layer(layer2).is_err()); // Should fail - not sequential
    }

    #[test]
    fn test_layout_add_category() {
        let mut layout = Layout::new("Test").unwrap();
        let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();

        assert!(layout.add_category(category).is_ok());
        assert_eq!(layout.categories.len(), 1);
    }

    #[test]
    fn test_layout_add_category_duplicate() {
        let mut layout = Layout::new("Test").unwrap();
        let category1 =
            Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
        let category2 = Category::new("navigation", "Nav Keys", RgbColor::new(255, 0, 0)).unwrap();

        assert!(layout.add_category(category1).is_ok());
        assert!(layout.add_category(category2).is_err()); // Duplicate ID
    }

    #[test]
    fn test_layout_resolve_key_color() {
        let mut layout = Layout::new("Test").unwrap();
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 255, 255)).unwrap();

        // Test 1: Individual override (highest priority)
        let key_with_override =
            KeyDefinition::new(Position::new(0, 0), "KC_A").with_color(RgbColor::new(255, 0, 0));
        layer.add_key(key_with_override.clone());

        layout.add_layer(layer).unwrap();

        let color = layout.resolve_key_color(0, &key_with_override);
        assert_eq!(color, RgbColor::new(255, 0, 0));

        // Test 2: Key category (second priority)
        let category = Category::new("navigation", "Navigation", RgbColor::new(0, 255, 0)).unwrap();
        layout.add_category(category).unwrap();

        let key_with_category =
            KeyDefinition::new(Position::new(0, 1), "KC_B").with_category("navigation");
        layout
            .get_layer_mut(0)
            .unwrap()
            .add_key(key_with_category.clone());

        let color = layout.resolve_key_color(0, &key_with_category);
        assert_eq!(color, RgbColor::new(0, 255, 0));

        // Test 3: Layer default (fallback)
        let key_default = KeyDefinition::new(Position::new(0, 2), "KC_C");
        layout
            .get_layer_mut(0)
            .unwrap()
            .add_key(key_default.clone());

        let color = layout.resolve_key_color(0, &key_default);
        assert_eq!(color, RgbColor::new(255, 255, 255));
    }

    #[test]
    fn test_layout_validate() {
        let mut layout = Layout::new("Test").unwrap();

        // Empty layout should fail
        assert!(layout.validate().is_err());

        // Add a layer with keys
        let mut layer = Layer::new(0, "Base", RgbColor::new(255, 0, 0)).unwrap();
        layer.add_key(KeyDefinition::new(Position::new(0, 0), "KC_A"));
        layer.add_key(KeyDefinition::new(Position::new(0, 1), "KC_B"));
        layout.add_layer(layer).unwrap();

        // Should pass now
        assert!(layout.validate().is_ok());

        // Add another layer with different key count
        let mut layer2 = Layer::new(1, "Lower", RgbColor::new(0, 255, 0)).unwrap();
        layer2.add_key(KeyDefinition::new(Position::new(0, 0), "KC_1"));
        layout.add_layer(layer2).unwrap();

        // Should fail - mismatched key counts
        assert!(layout.validate().is_err());
    }

    // === Tap-Hold Settings Tests ===

    #[test]
    fn test_tap_hold_settings_default() {
        let settings = TapHoldSettings::default();
        assert_eq!(settings.tapping_term, 200);
        assert_eq!(settings.quick_tap_term, None);
        assert_eq!(settings.hold_mode, HoldDecisionMode::Default);
        assert!(!settings.retro_tapping);
        assert_eq!(settings.tapping_toggle, 5);
        assert_eq!(settings.flow_tap_term, None);
        assert!(!settings.chordal_hold);
        assert_eq!(settings.preset, TapHoldPreset::Default);
    }

    #[test]
    fn test_tap_hold_preset_home_row_mods() {
        let settings = TapHoldPreset::HomeRowMods.settings();
        assert_eq!(settings.tapping_term, 175);
        assert_eq!(settings.quick_tap_term, Some(120));
        assert_eq!(settings.hold_mode, HoldDecisionMode::PermissiveHold);
        assert!(settings.retro_tapping);
        assert_eq!(settings.flow_tap_term, Some(150));
        assert!(settings.chordal_hold);
        assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
    }

    #[test]
    fn test_tap_hold_preset_responsive() {
        let settings = TapHoldPreset::Responsive.settings();
        assert_eq!(settings.tapping_term, 150);
        assert_eq!(settings.quick_tap_term, Some(100));
        assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
        assert!(!settings.retro_tapping);
        assert_eq!(settings.flow_tap_term, None);
        assert!(!settings.chordal_hold);
    }

    #[test]
    fn test_tap_hold_preset_deliberate() {
        let settings = TapHoldPreset::Deliberate.settings();
        assert_eq!(settings.tapping_term, 250);
        assert_eq!(settings.quick_tap_term, None);
        assert_eq!(settings.hold_mode, HoldDecisionMode::Default);
    }

    #[test]
    fn test_tap_hold_settings_apply_preset() {
        let mut settings = TapHoldSettings::default();
        settings.apply_preset(TapHoldPreset::HomeRowMods);

        assert_eq!(settings.tapping_term, 175);
        assert!(settings.chordal_hold);
        assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
    }

    #[test]
    fn test_tap_hold_settings_mark_custom() {
        let mut settings = TapHoldPreset::HomeRowMods.settings();
        assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);

        settings.mark_custom();
        assert_eq!(settings.preset, TapHoldPreset::Custom);
    }

    #[test]
    fn test_tap_hold_settings_has_custom_settings() {
        let default_settings = TapHoldSettings::default();
        assert!(!default_settings.has_custom_settings());

        let custom = TapHoldSettings {
            tapping_term: 180,
            ..TapHoldSettings::default()
        };
        assert!(custom.has_custom_settings());

        let with_retro = TapHoldSettings {
            retro_tapping: true,
            ..TapHoldSettings::default()
        };
        assert!(with_retro.has_custom_settings());
    }

    #[test]
    fn test_tap_hold_settings_validation() {
        let mut settings = TapHoldSettings::default();
        assert!(settings.validate().is_ok());

        // Invalid tapping term (too low)
        settings.tapping_term = 10;
        assert!(settings.validate().is_err());

        // Invalid tapping term (too high)
        settings.tapping_term = 2000;
        assert!(settings.validate().is_err());

        // Valid tapping term
        settings.tapping_term = 200;
        assert!(settings.validate().is_ok());

        // Invalid tapping toggle
        settings.tapping_toggle = 0;
        assert!(settings.validate().is_err());

        settings.tapping_toggle = 5;
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_hold_decision_mode_config_define() {
        assert_eq!(HoldDecisionMode::Default.config_define(), None);
        assert_eq!(
            HoldDecisionMode::PermissiveHold.config_define(),
            Some("PERMISSIVE_HOLD")
        );
        assert_eq!(
            HoldDecisionMode::HoldOnOtherKeyPress.config_define(),
            Some("HOLD_ON_OTHER_KEY_PRESS")
        );
    }

    // === RGB Saturation Tests ===

    #[test]
    fn test_rgb_saturation_new() {
        let sat = RgbSaturation::new(0);
        assert_eq!(sat.as_percent(), 0);

        let sat = RgbSaturation::new(100);
        assert_eq!(sat.as_percent(), 100);

        let sat = RgbSaturation::new(200);
        assert_eq!(sat.as_percent(), 200);
    }

    #[test]
    #[should_panic(expected = "Saturation must be 0-200")]
    fn test_rgb_saturation_new_too_high() {
        let _ = RgbSaturation::new(201);
    }

    #[test]
    fn test_rgb_saturation_default() {
        let sat = RgbSaturation::default();
        assert_eq!(sat.as_percent(), 100);
        assert_eq!(sat, RgbSaturation::NEUTRAL);
    }

    #[test]
    fn test_rgb_saturation_from_u8() {
        let sat = RgbSaturation::from(50);
        assert_eq!(sat.as_percent(), 50);

        // Should clamp to 200
        let sat = RgbSaturation::from(255);
        assert_eq!(sat.as_percent(), 200);
    }

    #[test]
    fn test_apply_rgb_settings_with_saturation() {
        let mut layout = Layout::new("Test").unwrap();

        // Test with saturation at neutral (100%)
        layout.rgb_saturation = RgbSaturation::new(100);
        layout.rgb_brightness = RgbBrightness::new(100);
        let color = RgbColor::new(200, 100, 50);
        let result = layout.apply_rgb_settings(color);
        assert_eq!(result, color); // Unchanged at 100% saturation and brightness

        // Test with reduced saturation (0% = grayscale)
        layout.rgb_saturation = RgbSaturation::new(0);
        let result = layout.apply_rgb_settings(color);
        // Should be grayscale (all channels equal)
        assert_eq!(result.r, result.g);
        assert_eq!(result.g, result.b);

        // Test with increased saturation (150%)
        layout.rgb_saturation = RgbSaturation::new(150);
        let result = layout.apply_rgb_settings(color);
        // Should be more saturated (more difference between channels)
        // The exact values depend on HSV conversion, just verify it's not the original
        // and channels are still different
        assert_ne!(result, color);
        assert_ne!(result.r, result.g);

        // Test order: saturation then brightness
        layout.rgb_saturation = RgbSaturation::new(50); // Half saturation
        layout.rgb_brightness = RgbBrightness::new(50); // Half brightness
        let result = layout.apply_rgb_settings(color);
        // Should be dimmed AND desaturated
        // With 50% saturation, color moves toward grayscale
        // Then 50% brightness dims everything
        // The total brightness should be reduced
        let original_brightness = u16::from(color.r) + u16::from(color.g) + u16::from(color.b);
        let result_brightness = u16::from(result.r) + u16::from(result.g) + u16::from(result.b);
        assert!(result_brightness < original_brightness);
    }

    #[test]
    fn test_apply_rgb_settings_disabled() {
        let mut layout = Layout::new("Test").unwrap();
        layout.rgb_enabled = false;

        let color = RgbColor::new(200, 100, 50);
        let result = layout.apply_rgb_settings(color);

        // Should be black when RGB is disabled
        assert_eq!(result, RgbColor::new(0, 0, 0));
    }

    #[test]
    fn test_layout_new_has_default_saturation() {
        let layout = Layout::new("Test").unwrap();
        assert_eq!(layout.rgb_saturation, RgbSaturation::NEUTRAL);
    }

    // === RGB Matrix Effect Tests ===

    #[test]
    fn test_rgb_matrix_effect_display_names() {
        assert_eq!(RgbMatrixEffect::Breathing.display_name(), "Breathing");
        assert_eq!(
            RgbMatrixEffect::RainbowMovingChevron.display_name(),
            "Rainbow Moving Chevron"
        );
        assert_eq!(RgbMatrixEffect::CycleAll.display_name(), "Cycle All");
    }

    #[test]
    fn test_rgb_matrix_effect_from_name() {
        // Exact matches
        assert_eq!(
            RgbMatrixEffect::from_name("Breathing"),
            Some(RgbMatrixEffect::Breathing)
        );
        assert_eq!(
            RgbMatrixEffect::from_name("breathing"),
            Some(RgbMatrixEffect::Breathing)
        );

        // With spaces and underscores
        assert_eq!(
            RgbMatrixEffect::from_name("Rainbow Moving Chevron"),
            Some(RgbMatrixEffect::RainbowMovingChevron)
        );
        assert_eq!(
            RgbMatrixEffect::from_name("rainbow_moving_chevron"),
            Some(RgbMatrixEffect::RainbowMovingChevron)
        );

        // Short aliases
        assert_eq!(
            RgbMatrixEffect::from_name("breath"),
            Some(RgbMatrixEffect::Breathing)
        );
        assert_eq!(
            RgbMatrixEffect::from_name("chevron"),
            Some(RgbMatrixEffect::RainbowMovingChevron)
        );
        assert_eq!(
            RgbMatrixEffect::from_name("cycle"),
            Some(RgbMatrixEffect::CycleAll)
        );

        // Invalid name
        assert_eq!(RgbMatrixEffect::from_name("invalid_effect"), None);
    }

    #[test]
    fn test_rgb_matrix_effect_default() {
        assert_eq!(RgbMatrixEffect::default(), RgbMatrixEffect::SolidColor);
    }

    // === Idle Effect Settings Tests ===

    #[test]
    fn test_idle_effect_settings_default() {
        let settings = IdleEffectSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.idle_timeout_ms, 60_000);
        assert_eq!(settings.idle_effect_duration_ms, 300_000);
        assert_eq!(settings.idle_effect_mode, RgbMatrixEffect::Breathing);
    }

    #[test]
    fn test_idle_effect_settings_has_custom_settings() {
        let default_settings = IdleEffectSettings::default();
        assert!(!default_settings.has_custom_settings());

        // Test enabled change
        let custom = IdleEffectSettings {
            enabled: false,
            ..IdleEffectSettings::default()
        };
        assert!(custom.has_custom_settings());

        // Test timeout change
        let custom = IdleEffectSettings {
            idle_timeout_ms: 30_000,
            ..IdleEffectSettings::default()
        };
        assert!(custom.has_custom_settings());

        // Test duration change
        let custom = IdleEffectSettings {
            idle_effect_duration_ms: 600_000,
            ..IdleEffectSettings::default()
        };
        assert!(custom.has_custom_settings());

        // Test mode change
        let custom = IdleEffectSettings {
            idle_effect_mode: RgbMatrixEffect::RainbowBeacon,
            ..IdleEffectSettings::default()
        };
        assert!(custom.has_custom_settings());
    }

    #[test]
    fn test_layout_new_has_default_idle_settings() {
        let layout = Layout::new("Test").unwrap();
        assert_eq!(layout.idle_effect_settings, IdleEffectSettings::default());
    }

    // === Combo Settings Tests ===

    #[test]
    fn test_combo_action_all() {
        let actions = ComboAction::all();
        assert_eq!(actions.len(), 3);
        assert!(actions.contains(&ComboAction::DisableEffects));
        assert!(actions.contains(&ComboAction::DisableLighting));
        assert!(actions.contains(&ComboAction::Bootloader));
    }

    #[test]
    fn test_combo_action_display_name() {
        assert_eq!(
            ComboAction::DisableEffects.display_name(),
            "Disable Effects"
        );
        assert_eq!(
            ComboAction::DisableLighting.display_name(),
            "Disable Lighting"
        );
        assert_eq!(ComboAction::Bootloader.display_name(), "Bootloader");
    }

    #[test]
    fn test_combo_action_from_name() {
        assert_eq!(
            ComboAction::from_name("disable effects"),
            Some(ComboAction::DisableEffects)
        );
        assert_eq!(
            ComboAction::from_name("DisableEffects"),
            Some(ComboAction::DisableEffects)
        );
        assert_eq!(
            ComboAction::from_name("lighting"),
            Some(ComboAction::DisableLighting)
        );
        assert_eq!(
            ComboAction::from_name("bootloader"),
            Some(ComboAction::Bootloader)
        );
        assert_eq!(ComboAction::from_name("invalid"), None);
    }

    #[test]
    fn test_combo_definition_new() {
        let combo = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
        );
        assert_eq!(combo.key1, Position::new(0, 0));
        assert_eq!(combo.key2, Position::new(0, 1));
        assert_eq!(combo.action, ComboAction::DisableEffects);
        assert_eq!(combo.hold_duration_ms, 500);
    }

    #[test]
    fn test_combo_definition_with_duration() {
        let combo = ComboDefinition::with_duration(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::Bootloader,
            1000,
        );
        assert_eq!(combo.hold_duration_ms, 1000);
    }

    #[test]
    fn test_combo_definition_validate_same_keys() {
        let combo = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 0),
            ComboAction::DisableEffects,
        );
        assert!(combo.validate().is_err());
    }

    #[test]
    fn test_combo_definition_validate_duration() {
        let combo = ComboDefinition::with_duration(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
            30, // Too short
        );
        assert!(combo.validate().is_err());

        let combo = ComboDefinition::with_duration(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
            3000, // Too long
        );
        assert!(combo.validate().is_err());

        let combo = ComboDefinition::with_duration(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
            500, // Valid
        );
        assert!(combo.validate().is_ok());
    }

    #[test]
    fn test_combo_settings_default() {
        let settings = ComboSettings::default();
        assert!(!settings.enabled);
        assert!(settings.combos.is_empty());
        assert!(!settings.has_custom_settings());
    }

    #[test]
    fn test_combo_settings_add_combo() {
        let mut settings = ComboSettings::new(true);

        let combo1 = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
        );
        assert!(settings.add_combo(combo1).is_ok());
        assert_eq!(settings.combos.len(), 1);

        let combo2 = ComboDefinition::new(
            Position::new(1, 0),
            Position::new(1, 1),
            ComboAction::DisableLighting,
        );
        assert!(settings.add_combo(combo2).is_ok());
        assert_eq!(settings.combos.len(), 2);

        let combo3 = ComboDefinition::new(
            Position::new(2, 0),
            Position::new(2, 1),
            ComboAction::Bootloader,
        );
        assert!(settings.add_combo(combo3).is_ok());
        assert_eq!(settings.combos.len(), 3);

        // Fourth combo should fail (max 3)
        let combo4 = ComboDefinition::new(
            Position::new(3, 0),
            Position::new(3, 1),
            ComboAction::DisableEffects,
        );
        assert!(settings.add_combo(combo4).is_err());
    }

    #[test]
    fn test_combo_settings_duplicate_detection() {
        let mut settings = ComboSettings::new(true);

        let combo1 = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
        );
        assert!(settings.add_combo(combo1).is_ok());

        // Same key pair in same order
        let combo2 = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableLighting,
        );
        assert!(settings.add_combo(combo2).is_err());

        // Same key pair in reverse order
        let combo3 = ComboDefinition::new(
            Position::new(0, 1),
            Position::new(0, 0),
            ComboAction::Bootloader,
        );
        assert!(settings.add_combo(combo3).is_err());
    }

    #[test]
    fn test_combo_settings_has_custom_settings() {
        let settings = ComboSettings::default();
        assert!(!settings.has_custom_settings());

        let settings = ComboSettings::new(true);
        assert!(settings.has_custom_settings());

        let mut settings = ComboSettings::default();
        let combo = ComboDefinition::new(
            Position::new(0, 0),
            Position::new(0, 1),
            ComboAction::DisableEffects,
        );
        settings.add_combo(combo).unwrap();
        assert!(settings.has_custom_settings());
    }
}
