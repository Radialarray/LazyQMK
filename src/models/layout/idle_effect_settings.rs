//! Configuration for idle effect behavior.

use serde::{Deserialize, Serialize};

use super::RgbMatrixEffect;

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
