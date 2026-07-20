//! Tap dance action definitions.

use serde::{Deserialize, Serialize};

/// A tap dance action that performs different keycodes based on tap count.
///
/// Tap dances support 2-way (single/double tap) and 3-way (single/double/hold) patterns.
/// The hold action activates when the key is held past the tapping term.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TapDanceAction {
    /// Unique name for this tap dance (used in `TD()` references)
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
    pub fn validate(&self) -> Result<(), anyhow::Error> {
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
