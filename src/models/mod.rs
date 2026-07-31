//! Data models for keyboard layouts, layers, and configuration.
//!
//! This module contains all the core data structures used throughout the application.
//! Models are designed to be independent of UI and business logic.

pub mod category;
pub mod color_palette;
pub mod keyboard_geometry;
pub mod layer;
pub mod layout;
pub mod rgb;
pub mod visual_layout_mapping;

// Re-export all model types
pub use category::Category;
pub use color_palette::{ColorPalette, Shade};
pub use keyboard_geometry::{KeyGeometry, KeyboardGeometry};
#[allow(unused_imports)] // bin/lib split: re-exports consumed by lib tests
pub use layer::{
    validate_layer_number, KeyDefinition, Layer, Position, DEFAULT_QMK_LAYER_LIMIT,
    MAX_QMK_LAYER_LIMIT,
};
#[allow(unused_imports)] // bin/lib split: re-exported types consumed by lib tests and web crate
pub use layout::{
    auto_label, compute_diff, revision_filename, sanitize_label, ComboAction, ComboDefinition,
    ComboSettings, DiffSummary, HoldDecisionMode, IdleEffectSettings, KeyChange, LayerDiff,
    Layout, LayoutDiff, LayoutManifest, LayoutMetadata, LayoutRevision, PaletteFxEffect,
    PaletteFxPalette, PaletteFxSettings, RgbBrightness, RgbMatrixEffect, RgbOverlayRippleSettings,
    RgbSaturation, RevisionSummary, RippleColorMode, SettingDiff, TapDanceAction, TapHoldPreset,
    TapHoldSettings, UncoloredKeyBehavior,
};
pub use rgb::RgbColor;
pub use visual_layout_mapping::VisualLayoutMapping;
