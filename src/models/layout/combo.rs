//! Combo settings — two-key hold combos on the base layer.

use crate::models::layer::Position;
use serde::{Deserialize, Serialize};

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
    #[allow(dead_code)] // Public API; used by tests/lib consumers (bin doesn't link)
    #[must_use]
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
    /// First key trigger position in **visual** coordinates.
    ///
    /// Stored as a visual-grid `Position` (matching `KeyDefinition.position`).
    /// Set from the currently selected key in the UI.
    pub key1: Position,
    /// Second key trigger position in **visual** coordinates.
    ///
    /// See `key1` for coordinate-system details.
    pub key2: Position,
    /// Action to perform when combo is held
    pub action: ComboAction,
    /// Duration in milliseconds both keys must be held to activate
    /// Default: 500ms
    #[serde(default = "default_combo_hold_duration")]
    pub hold_duration_ms: u16,
    /// Whether this is a gap-filler placeholder created when non-contiguous
    /// combo indices are parsed (e.g. only Combo 2 and Combo 3 are defined).
    /// Placeholders are never emitted to generated C code.
    #[serde(skip)]
    pub placeholder: bool,
}

const fn default_combo_hold_duration() -> u16 {
    500 // 500ms default
}

impl ComboDefinition {
    /// Creates a new combo with default hold duration.
    ///
    /// Both `key1` and `key2` are **visual** grid positions (see [`Position`]).
    #[must_use]
    pub fn new(key1: Position, key2: Position, action: ComboAction) -> Self {
        Self {
            key1,
            key2,
            action,
            hold_duration_ms: default_combo_hold_duration(),
            placeholder: false,
        }
    }

    /// Creates a gap-filler placeholder used when non-contiguous combo indices
    /// are parsed (e.g. only Combo 2 and Combo 3 defined, leaving index 0 empty).
    /// Placeholders are **never** emitted to generated C code.
    #[must_use]
    pub fn new_placeholder() -> Self {
        Self {
            key1: Position::new(0, 0),
            key2: Position::new(0, 0),
            action: ComboAction::DisableEffects,
            hold_duration_ms: default_combo_hold_duration(),
            placeholder: true,
        }
    }

    /// Creates a new combo with custom hold duration.
    ///
    /// Both `key1` and `key2` are **visual** grid positions (see [`Position`]).
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
            placeholder: false,
        }
    }

    /// Validates the combo definition.
    /// Part of public API for future validation in UI/settings.
    ///
    /// Checks:
    /// - Key positions are different
    /// - Hold duration is reasonable (50-2000ms)
    #[allow(dead_code)] // Public API; tests are in lib target
    pub fn validate(&self) -> Result<(), anyhow::Error> {
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
    #[allow(dead_code)] // Public API; tests are in lib target
    pub const fn new(enabled: bool) -> Self {
        Self {
            enabled,
            combos: Vec::new(),
        }
    }

    /// Adds a combo definition.
    #[allow(dead_code)] // Public API; tests are in lib target
    pub fn add_combo(&mut self, combo: ComboDefinition) -> Result<(), anyhow::Error> {
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

    /// Checks if any settings differ from defaults.
    #[must_use]
    pub fn has_custom_settings(&self) -> bool {
        self.enabled || !self.combos.is_empty()
    }
}
