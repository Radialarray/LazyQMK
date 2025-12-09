//! Centralized geometry building service.
//!
//! This module provides functions for building keyboard geometry from QMK firmware files.
//! It handles parsing QMK JSON files, resolving keyboard variants, and creating
//! visual layout mappings with RGB matrix support.

use anyhow::{Context, Result};

use crate::{
    config::Config,
    models::{KeyboardGeometry, LayoutMetadata, VisualLayoutMapping},
    parser::keyboard_json::{
        build_keyboard_geometry_with_rgb, build_matrix_to_led_map, parse_keyboard_info_json,
        parse_variant_keyboard_json,
    },
};

/// Context required for building keyboard geometry.
///
/// This struct encapsulates all the information needed to build geometry
/// from QMK firmware files.
#[derive(Debug, Clone)]
pub struct GeometryContext<'a> {
    /// Application configuration
    pub config: &'a Config,
    /// Layout metadata containing keyboard and layout variant
    pub metadata: &'a LayoutMetadata,
}

/// Result of building keyboard geometry.
///
/// Contains the built geometry, visual mapping, and potentially updated metadata
/// with the resolved keyboard variant path.
#[derive(Debug)]
pub struct GeometryResult {
    /// Built keyboard geometry
    pub geometry: KeyboardGeometry,
    /// Visual layout mapping for position resolution
    pub mapping: VisualLayoutMapping,
    /// Updated keyboard variant path (may differ from input if variant was resolved)
    pub variant_path: String,
}

/// Extracts the base keyboard name from a keyboard path that may include a variant.
///
/// # Examples
///
/// - `"keebart/corne_choc_pro/standard"` → `"keebart/corne_choc_pro"`
/// - `"keebart/corne_choc_pro"` → `"keebart/corne_choc_pro"`
/// - `"crkbd"` → `"crkbd"`
#[must_use]
pub fn extract_base_keyboard(keyboard_path: &str) -> String {
    let keyboard_parts: Vec<&str> = keyboard_path.split('/').collect();
    
    // If path has 3+ components, check if last component looks like a variant
    if keyboard_parts.len() >= 3 {
        let last_part = keyboard_parts[keyboard_parts.len() - 1];
        
        // Check if the last component matches common variant patterns
        let looks_like_variant = 
            // Common variant names
            matches!(last_part, "standard" | "mini" | "normal" | "full" | "compact" | 
                               "rgb" | "wireless" | "ansi" | "iso" | "hotswap")
            // Revision patterns: rev1, rev2, etc.
            || last_part.starts_with("rev")
            // Version patterns: v1, v2, v3, etc.
            || (last_part.starts_with('v') && last_part.len() <= 3 && last_part[1..].chars().all(|c| c.is_ascii_digit()));
        
        if looks_like_variant {
            // Has variant subdirectory - use parent path
            return keyboard_parts[..keyboard_parts.len() - 1].join("/");
        }
    }
    
    // No variant detected or not enough path components
    keyboard_path.to_string()
}

/// Builds keyboard geometry and mapping for the given layout.
///
/// This function centralizes all the QMK → geometry building logic:
/// 1. Extracts base keyboard name (without variant subdirectory)
/// 2. Parses keyboard info.json from QMK firmware
/// 3. Determines the correct keyboard variant based on key count
/// 4. Loads RGB matrix mapping from variant's keyboard.json (if available)
/// 5. Builds geometry with RGB support
/// 6. Creates visual layout mapping
///
/// # Arguments
///
/// * `context` - Geometry context with config and metadata
/// * `layout_name` - Name of the layout variant (e.g., "LAYOUT_split_3x6_3")
///
/// # Returns
///
/// Returns a `GeometryResult` containing the geometry, mapping, and resolved variant path.
///
/// # Errors
///
/// Returns error if:
/// - QMK firmware path is not configured
/// - Keyboard is not specified in metadata
/// - Layout variant is not specified in metadata
/// - Failed to parse keyboard info.json
/// - Layout not found in keyboard info.json
/// - Failed to build geometry
///
/// # Fallback Behavior
///
/// If QMK path is missing or JSON parsing fails, this function returns a minimal
/// empty geometry instead of failing completely. This allows the application to
/// continue running even without valid QMK configuration.
pub fn build_geometry_for_layout(
    context: GeometryContext<'_>,
    layout_name: &str,
) -> Result<GeometryResult> {
    // Get QMK path from config
    let qmk_path = context
        .config
        .paths
        .qmk_firmware
        .as_ref()
        .context("QMK firmware path not configured")?;

    // Get keyboard from metadata
    let keyboard = context
        .metadata
        .keyboard
        .as_ref()
        .context("Keyboard not specified in layout metadata")?;

    // Extract base keyboard name (without any variant subdirectory)
    let base_keyboard = extract_base_keyboard(keyboard);

    // Parse keyboard info.json using the base keyboard path
    let keyboard_info = parse_keyboard_info_json(qmk_path, &base_keyboard)
        .context("Failed to parse keyboard info.json")?;

    // Get the key count for the selected layout to determine the correct variant
    let layout_def = keyboard_info.layouts.get(layout_name).context(format!(
        "Layout '{}' not found in keyboard info.json",
        layout_name
    ))?;
    let key_count = layout_def.layout.len();

    // Determine the correct keyboard variant based on key count
    // This is critical for split keyboards where different variants have different key counts
    let variant_path = context
        .config
        .build
        .determine_keyboard_variant(qmk_path, &base_keyboard, key_count)
        .unwrap_or_else(|_| base_keyboard.clone());

    // Try to get RGB matrix mapping from the variant's keyboard.json
    let matrix_to_led = parse_variant_keyboard_json(qmk_path, &variant_path)
        .and_then(|variant| variant.rgb_matrix)
        .map(|rgb_config| build_matrix_to_led_map(&rgb_config));

    // Build geometry from the selected layout with RGB matrix mapping if available
    let mut geometry = build_keyboard_geometry_with_rgb(
        &keyboard_info,
        &base_keyboard,
        layout_name,
        matrix_to_led.as_ref(),
    )
    .context("Failed to build keyboard geometry")?;

    // Extract encoder count from keyboard info (capped at u8::MAX)
    geometry.encoder_count = keyboard_info
        .encoder
        .as_ref()
        .and_then(|enc| enc.rotary.as_ref())
        .map(|rotary| u8::try_from(rotary.len()).unwrap_or(u8::MAX))
        .unwrap_or(0);

    // Build visual mapping
    let mapping = VisualLayoutMapping::build(&geometry);

    Ok(GeometryResult {
        geometry,
        mapping,
        variant_path,
    })
}

/// Builds minimal fallback geometry when QMK configuration is unavailable.
///
/// Creates an empty geometry with no keys. This allows the application to
/// continue running even when QMK firmware is not configured or accessible.
///
/// # Returns
///
/// Returns a `GeometryResult` with empty geometry and mapping.
#[must_use]
pub fn build_minimal_geometry() -> GeometryResult {
    let geometry = KeyboardGeometry::new("unknown", "LAYOUT", 0, 0);
    let mapping = VisualLayoutMapping::build(&geometry);

    GeometryResult {
        geometry,
        mapping,
        variant_path: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_base_keyboard() {
        // With standard variant names
        assert_eq!(
            extract_base_keyboard("keebart/corne_choc_pro/standard"),
            "keebart/corne_choc_pro"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/mini"),
            "manufacturer/keyboard"
        );

        // With revision variants
        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/rev1"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/rev2"),
            "manufacturer/keyboard"
        );

        // With version variants
        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/v1"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/v2"),
            "manufacturer/keyboard"
        );

        // With other common variant names
        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/rgb"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/wireless"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/ansi"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/iso"),
            "manufacturer/keyboard"
        );

        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/hotswap"),
            "manufacturer/keyboard"
        );

        // Without variant subdirectory
        assert_eq!(
            extract_base_keyboard("keebart/corne_choc_pro"),
            "keebart/corne_choc_pro"
        );

        assert_eq!(extract_base_keyboard("crkbd"), "crkbd");

        // Edge case: single directory with variant name (not enough path components)
        assert_eq!(extract_base_keyboard("standard"), "standard");
        assert_eq!(extract_base_keyboard("rev1"), "rev1");

        // Edge case: non-variant subdirectory (should not be stripped)
        assert_eq!(
            extract_base_keyboard("manufacturer/keyboard/custom"),
            "manufacturer/keyboard/custom"
        );
    }

    #[test]
    fn test_build_minimal_geometry() {
        let result = build_minimal_geometry();
        assert!(result.geometry.keys.is_empty());
        assert_eq!(result.geometry.keyboard_name, "unknown");
        assert_eq!(result.geometry.layout_name, "LAYOUT");
        assert_eq!(result.geometry.matrix_rows, 0);
        assert_eq!(result.geometry.matrix_cols, 0);
        assert_eq!(result.variant_path, "");
    }
}
