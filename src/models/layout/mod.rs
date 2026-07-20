//! Layout module — split across sibling files in this directory.
//!
//! All types are re-exported from this barrel module so existing
//! `use crate::models::layout::Type` paths continue to work.

#![allow(clippy::trivially_copy_pass_by_ref)]

pub mod combo;
pub mod idle_effect_settings;
pub mod layout_core;
pub mod palette_fx;
pub mod rgb_brightness;
pub mod rgb_matrix_effect;
pub mod rgb_saturation;
pub mod ripple;
pub mod tap_dance;
pub mod tap_hold;
pub mod uncolored_key_behavior;

#[cfg(test)]
mod tests;

pub use combo::{ComboAction, ComboDefinition, ComboSettings};
pub use idle_effect_settings::IdleEffectSettings;
pub use layout_core::{Layout, LayoutMetadata};
pub use palette_fx::{PaletteFxEffect, PaletteFxPalette, PaletteFxSettings};
pub use rgb_brightness::RgbBrightness;
pub use rgb_matrix_effect::RgbMatrixEffect;
pub use rgb_saturation::RgbSaturation;
pub use ripple::{RippleColorMode, RgbOverlayRippleSettings};
pub use tap_dance::TapDanceAction;
pub use tap_hold::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};
pub use uncolored_key_behavior::UncoloredKeyBehavior;
