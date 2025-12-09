//! QMK info.json parser for keyboard geometry and layout information.
//!
//! This module handles parsing QMK's info.json files to extract keyboard metadata,
//! layout definitions, and physical key positions for building coordinate mappings.

// Allow intentional type casts for QMK coordinate parsing
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::doc_link_with_quotes)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::models::{KeyGeometry, KeyboardGeometry};

/// QMK info.json structure (simplified for our needs)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QmkInfoJson {
    /// Keyboard name/identifier
    pub keyboard_name: Option<String>,
    /// Manufacturer name
    pub manufacturer: Option<String>,
    /// Maintainer name
    pub maintainer: Option<String>,
    /// URL to keyboard information
    pub url: Option<String>,
    /// Available layouts (may be empty if layouts are in keyboard.json or inherited)
    #[serde(default)]
    pub layouts: HashMap<String, LayoutDefinition>,
    /// Matrix pins configuration
    pub matrix_pins: Option<MatrixPins>,
    /// Encoder configuration
    pub encoder: Option<EncoderConfig>,
}

/// Encoder configuration from info.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EncoderConfig {
    /// Rotary encoder definitions
    pub rotary: Option<Vec<RotaryEncoder>>,
}

/// Rotary encoder definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RotaryEncoder {
    /// Pin A
    pub pin_a: Option<String>,
    /// Pin B  
    pub pin_b: Option<String>,
}

/// Layout definition from info.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LayoutDefinition {
    /// Physical key positions and matrix assignments
    pub layout: Vec<KeyPosition>,
}

/// Key position information from info.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyPosition {
    /// Physical X position in keyboard units
    pub x: f32,
    /// Physical Y position in keyboard units
    pub y: f32,
    /// Matrix position [row, col]
    pub matrix: Option<[u8; 2]>,
    /// Key width in keyboard units (default 1.0)
    #[serde(default = "default_key_size")]
    pub w: f32,
    /// Key height in keyboard units (default 1.0)
    #[serde(default = "default_key_size")]
    pub h: f32,
    /// Rotation in degrees (default 0.0)
    #[serde(default)]
    pub r: f32,
}

/// Matrix pins configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MatrixPins {
    /// Row pins
    pub rows: Option<Vec<String>>,
    /// Column pins
    pub cols: Option<Vec<String>>,
}

/// Variant-specific keyboard.json structure (for RGB matrix info and layouts)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VariantKeyboardJson {
    /// Keyboard name
    pub keyboard_name: Option<String>,
    /// RGB matrix configuration
    pub rgb_matrix: Option<RgbMatrixConfig>,
    /// Layout definitions (some keyboards define layouts here instead of info.json)
    #[serde(default)]
    pub layouts: HashMap<String, LayoutDefinition>,
    /// Encoder configuration (can also be in keyboard.json)
    pub encoder: Option<EncoderConfig>,
}

/// RGB matrix configuration from keyboard.json
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RgbMatrixConfig {
    /// Split keyboard LED counts [left, right]
    pub split_count: Option<[u8; 2]>,
    /// LED layout - array defines physical wiring order
    pub layout: Vec<RgbLedEntry>,
}

/// RGB LED entry from `rgb_matrix.layout` array
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RgbLedEntry {
    /// Matrix position [row, col] for this LED
    pub matrix: [u8; 2],
    /// Physical X position
    pub x: u8,
    /// Physical Y position
    pub y: u8,
    /// LED flags
    pub flags: u8,
}

const fn default_key_size() -> f32 {
    1.0
}

/// Scans the QMK keyboards directory and returns a list of available keyboards.
///
/// This function uses the `qmk list-keyboards` command to get all compilable
/// keyboard paths directly from QMK. This ensures only valid, buildable keyboards
/// are returned (e.g., `splitkb/halcyon/ferris/rev1` instead of the non-compilable
/// parent `splitkb/halcyon/ferris`).
///
/// # Arguments
///
/// * `qmk_path` - Path to QMK firmware root directory
///
/// # Returns
///
/// A vector of keyboard names (relative paths from keyboards/ directory)
///
/// # Errors
///
/// Returns an error if:
/// - The QMK keyboards directory doesn't exist
/// - The `qmk list-keyboards` command fails
/// - The command output cannot be parsed
#[allow(dead_code)]
pub fn scan_keyboards(qmk_path: &Path) -> Result<Vec<String>> {
    use std::process::Command;

    let keyboards_dir = qmk_path.join("keyboards");

    if !keyboards_dir.exists() {
        anyhow::bail!(
            "QMK keyboards directory not found: {}",
            keyboards_dir.display()
        );
    }

    // Run `qmk list-keyboards` command to get all compilable keyboards
    let output = Command::new("qmk")
        .arg("list-keyboards")
        .current_dir(qmk_path)
        .output()
        .context("Failed to execute 'qmk list-keyboards'. Is QMK CLI installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("qmk list-keyboards failed: {}", stderr);
    }

    // Parse output - one keyboard per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut keyboards: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    // Sort alphabetically for consistent ordering
    keyboards.sort();

    Ok(keyboards)
}

/// Parses a QMK info.json file.
///
/// # Arguments
///
/// * `path` - Path to the info.json file
///
/// # Returns
///
/// Parsed QMK info.json structure
pub fn parse_info_json(path: &Path) -> Result<QmkInfoJson> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read info.json: {}", path.display()))?;

    let info: QmkInfoJson = serde_json::from_str(&content)
        .context(format!("Failed to parse info.json: {}", path.display()))?;

    Ok(info)
}

/// Parses a QMK info.json file by keyboard name.
///
/// This helper supports both base keyboard paths (e.g., "crkbd") and
/// variant paths (e.g., "`keebart/corne_choc_pro/standard`"). It handles
/// multiple QMK keyboard configuration patterns:
///
/// 1. **Single info.json**: Keyboard with `info.json` in variant directory
/// 2. **Parent info.json only**: Variant inherits from parent's `info.json`
/// 3. **Split configuration**: Parent `info.json` + variant `keyboard.json`
///    (e.g., `1upkeyboards/pi50/grid` where parent has encoder config but
///    layouts are in variant's `keyboard.json`)
///
/// When a split configuration is detected, this function merges data from both
/// files: layouts come from `keyboard.json`, while other config (encoder, etc.)
/// comes from the parent `info.json`.
///
/// # Arguments
///
/// * `qmk_path` - Path to QMK firmware root directory
/// * `keyboard` - Keyboard name (e.g., "crkbd", "ferris/sweep",
///   "`keebart/corne_choc_pro/standard`")
///
/// # Returns
///
/// Parsed QMK info.json structure (potentially merged from multiple files)
pub fn parse_keyboard_info_json(qmk_path: &Path, keyboard: &str) -> Result<QmkInfoJson> {
    let keyboards_dir = qmk_path.join("keyboards");
    let keyboard_dir = keyboards_dir.join(keyboard);
    let info_json_path = keyboard_dir.join("info.json");
    let keyboard_json_path = keyboard_dir.join("keyboard.json");

    // First try the path as given (this supports keyboards that keep
    // their info.json inside a variant directory).
    if info_json_path.exists() {
        let mut info = parse_info_json(&info_json_path)?;
        
        // If info.json has no layouts but keyboard.json exists, merge layouts from it
        if info.layouts.is_empty() && keyboard_json_path.exists() {
            if let Ok(variant) = parse_variant_json(&keyboard_json_path) {
                if !variant.layouts.is_empty() {
                    info.layouts = variant.layouts;
                }
            }
        }
        
        return Ok(info);
    }

    // Check if variant has keyboard.json with layouts (even without info.json)
    if keyboard_json_path.exists() {
        if let Ok(variant) = parse_variant_json(&keyboard_json_path) {
            if !variant.layouts.is_empty() {
                // Try to get parent info.json for additional config (encoder, etc.)
                if let Some((base, _)) = keyboard.rsplit_once('/') {
                    let base_info_json_path = keyboards_dir.join(base).join("info.json");
                    if base_info_json_path.exists() {
                        let mut parent_info = parse_info_json(&base_info_json_path)?;
                        // Merge: layouts from variant, other config from parent
                        parent_info.layouts = variant.layouts;
                        // Also use encoder from variant if parent doesn't have it
                        if parent_info.encoder.is_none() {
                            parent_info.encoder = variant.encoder;
                        }
                        return Ok(parent_info);
                    }
                }
                
                // No parent info.json, create QmkInfoJson from variant keyboard.json
                return Ok(QmkInfoJson {
                    keyboard_name: variant.keyboard_name,
                    manufacturer: None,
                    maintainer: None,
                    url: None,
                    layouts: variant.layouts,
                    matrix_pins: None,
                    encoder: variant.encoder,
                });
            }
        }
    }

    // Fall back to base keyboard directory if keyboard path includes a variant suffix
    if let Some((base, _variant)) = keyboard.rsplit_once('/') {
        let base_info_json_path = keyboards_dir.join(base).join("info.json");

        if base_info_json_path.exists() {
            let mut info = parse_info_json(&base_info_json_path)?;
            
            // Check if variant's keyboard.json has layouts
            if info.layouts.is_empty() && keyboard_json_path.exists() {
                if let Ok(variant) = parse_variant_json(&keyboard_json_path) {
                    if !variant.layouts.is_empty() {
                        info.layouts = variant.layouts;
                    }
                }
            }
            
            return Ok(info);
        }
    }

    anyhow::bail!(
        "info.json not found for keyboard '{}' under {}",
        keyboard,
        keyboards_dir.display()
    );
}

/// Internal helper to parse a keyboard.json file.
fn parse_variant_json(path: &Path) -> Result<VariantKeyboardJson> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read keyboard.json: {}", path.display()))?;

    let variant: VariantKeyboardJson = serde_json::from_str(&content)
        .context(format!("Failed to parse keyboard.json: {}", path.display()))?;

    Ok(variant)
}

/// Parses a variant-specific keyboard.json file for RGB matrix configuration.
///
/// This looks for a keyboard.json file in the variant directory that contains
/// the `rgb_matrix.layout` array, which defines the physical LED wiring order.
///
/// # Arguments
///
/// * `qmk_path` - Path to QMK firmware root directory
/// * `keyboard` - Full keyboard path including variant (e.g., "`keebart/corne_choc_pro/standard`")
///
/// # Returns
///
/// Parsed variant keyboard.json structure, or None if not found
#[must_use]
pub fn parse_variant_keyboard_json(qmk_path: &Path, keyboard: &str) -> Option<VariantKeyboardJson> {
    let keyboards_dir = qmk_path.join("keyboards");
    let keyboard_json_path = keyboards_dir.join(keyboard).join("keyboard.json");

    if !keyboard_json_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&keyboard_json_path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Builds a mapping from matrix position (row, col) to physical LED index.
///
/// The RGB matrix layout in keyboard.json defines LEDs in their physical wiring order.
/// This function creates a reverse lookup to find the LED index for any matrix position.
///
/// # Arguments
///
/// * `rgb_config` - RGB matrix configuration from keyboard.json
///
/// # Returns
///
/// `HashMap` from (row, col) to LED index
#[must_use]
pub fn build_matrix_to_led_map(rgb_config: &RgbMatrixConfig) -> HashMap<(u8, u8), u8> {
    let mut map = HashMap::new();
    for (led_index, led_entry) in rgb_config.layout.iter().enumerate() {
        let row = led_entry.matrix[0];
        let col = led_entry.matrix[1];
        map.insert((row, col), led_index as u8);
    }
    map
}

/// Layout variant information including name and key count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutVariant {
    /// Layout name (e.g., "`LAYOUT_split_3x6_3`")
    pub name: String,
    /// Number of keys in this layout
    pub key_count: usize,
}

/// Extracts available layout names from info.json.
///
/// # Arguments
///
/// * `info` - Parsed QMK info.json structure
///
/// # Returns
///
/// Vector of layout names (e.g., ["LAYOUT", "`LAYOUT_split_3x6_3`"])
#[must_use]
pub fn extract_layout_names(info: &QmkInfoJson) -> Vec<String> {
    let mut names: Vec<String> = info.layouts.keys().cloned().collect();
    names.sort();
    names
}

/// Extracts available layout variants with key counts from info.json.
///
/// # Arguments
///
/// * `info` - Parsed QMK info.json structure
///
/// # Returns
///
/// Vector of layout variants with names and key counts
#[must_use]
pub fn extract_layout_variants(info: &QmkInfoJson) -> Vec<LayoutVariant> {
    let mut variants: Vec<LayoutVariant> = info
        .layouts
        .iter()
        .map(|(name, def)| LayoutVariant {
            name: name.clone(),
            key_count: def.layout.len(),
        })
        .collect();

    // Sort by name for consistent ordering
    variants.sort_by(|a, b| a.name.cmp(&b.name));
    variants
}

/// Extracts a specific layout definition from info.json.
///
/// # Arguments
///
/// * `info` - Parsed QMK info.json structure
/// * `layout_name` - Name of the layout to extract
///
/// # Returns
///
/// Layout definition if found
pub fn extract_layout_definition<'a>(
    info: &'a QmkInfoJson,
    layout_name: &str,
) -> Result<&'a LayoutDefinition> {
    info.layouts.get(layout_name).context(format!(
        "Layout '{}' not found in info.json. Available layouts: {:?}",
        layout_name,
        extract_layout_names(info)
    ))
}

/// Builds `KeyboardGeometry` from QMK info.json layout definition.
///
/// # Arguments
///
/// * `info` - Parsed QMK info.json structure
/// * `keyboard_name` - Keyboard identifier
/// * `layout_name` - Layout variant name
///
/// # Returns
///
/// `KeyboardGeometry` with physical key positions and matrix mappings
#[allow(dead_code)]
pub fn build_keyboard_geometry(
    info: &QmkInfoJson,
    keyboard_name: &str,
    layout_name: &str,
) -> Result<KeyboardGeometry> {
    build_keyboard_geometry_with_rgb(info, keyboard_name, layout_name, None)
}

/// Builds `KeyboardGeometry` from QMK info.json layout definition with optional RGB matrix mapping.
///
/// When `matrix_to_led` is provided (from parsing the variant's keyboard.json `rgb_matrix` section),
/// it uses the physical LED wiring order instead of the layout array order. This is critical for
/// keyboards with serpentine LED wiring where the physical LED order doesn't match the logical
/// key order.
///
/// # Arguments
///
/// * `info` - Parsed QMK info.json structure
/// * `keyboard_name` - Keyboard identifier
/// * `layout_name` - Layout variant name
/// * `matrix_to_led` - Optional map from matrix position (row, col) to physical LED index
///
/// # Returns
///
/// `KeyboardGeometry` with physical key positions and correct LED mappings
pub fn build_keyboard_geometry_with_rgb(
    info: &QmkInfoJson,
    keyboard_name: &str,
    layout_name: &str,
    matrix_to_led: Option<&HashMap<(u8, u8), u8>>,
) -> Result<KeyboardGeometry> {
    let layout_def = extract_layout_definition(info, layout_name)?;

    // Determine matrix dimensions
    let mut max_row = 0u8;
    let mut max_col = 0u8;

    for (idx, key_pos) in layout_def.layout.iter().enumerate() {
        if let Some([row, col]) = key_pos.matrix {
            max_row = max_row.max(row);
            max_col = max_col.max(col);
        } else {
            anyhow::bail!("Key at index {idx} in layout '{layout_name}' has no matrix position");
        }
    }

    let matrix_rows = max_row + 1;
    let matrix_cols = max_col + 1;

    // Build KeyGeometry for each key
    let mut keys = Vec::new();
    for (layout_index, key_pos) in layout_def.layout.iter().enumerate() {
        let matrix_position = key_pos.matrix.unwrap(); // Already validated above
        let matrix_pos_tuple = (matrix_position[0], matrix_position[1]);

        // Use physical LED index from RGB matrix mapping if available,
        // otherwise fall back to layout array index
        let led_index = matrix_to_led
            .and_then(|map| map.get(&matrix_pos_tuple).copied())
            .unwrap_or(layout_index as u8);

        let key_geometry = KeyGeometry {
            matrix_position: matrix_pos_tuple,
            led_index,
            layout_index: layout_index as u8,
            visual_x: key_pos.x,
            visual_y: key_pos.y,
            width: key_pos.w,
            height: key_pos.h,
            rotation: key_pos.r,
        };

        keys.push(key_geometry);
    }

    Ok(KeyboardGeometry {
        keyboard_name: keyboard_name.to_string(),
        layout_name: layout_name.to_string(),
        matrix_rows,
        matrix_cols,
        keys,
        encoder_count: 0, // Will be set by caller if encoder info is available
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_info_json() -> String {
        r#"{
            "keyboard_name": "test_keyboard",
            "manufacturer": "Test Manufacturer",
            "layouts": {
                "LAYOUT": {
                    "layout": [
                        {"x": 0, "y": 0, "matrix": [0, 0]},
                        {"x": 1, "y": 0, "matrix": [0, 1]},
                        {"x": 2, "y": 0, "matrix": [0, 2]},
                        {"x": 0, "y": 1, "matrix": [1, 0]},
                        {"x": 1, "y": 1, "matrix": [1, 1]},
                        {"x": 2, "y": 1, "matrix": [1, 2]}
                    ]
                },
                "LAYOUT_split": {
                    "layout": [
                        {"x": 0, "y": 0, "matrix": [0, 0], "w": 1.5},
                        {"x": 1.5, "y": 0, "matrix": [0, 1]},
                        {"x": 5, "y": 0, "matrix": [4, 0]},
                        {"x": 6, "y": 0, "matrix": [4, 1]}
                    ]
                }
            }
        }"#
        .to_string()
    }

    #[test]
    fn test_parse_info_json() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();
        assert_eq!(info.keyboard_name, Some("test_keyboard".to_string()));
        assert_eq!(info.layouts.len(), 2);
        assert!(info.layouts.contains_key("LAYOUT"));
        assert!(info.layouts.contains_key("LAYOUT_split"));
    }

    #[test]
    fn test_extract_layout_names() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();
        let names = extract_layout_names(&info);

        assert_eq!(names.len(), 2);
        assert!(names.contains(&"LAYOUT".to_string()));
        assert!(names.contains(&"LAYOUT_split".to_string()));
    }

    #[test]
    fn test_extract_layout_variants() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();
        let variants = extract_layout_variants(&info);

        assert_eq!(variants.len(), 2);

        // Check LAYOUT variant
        let layout = variants.iter().find(|v| v.name == "LAYOUT").unwrap();
        assert_eq!(layout.key_count, 6);

        // Check LAYOUT_split variant
        let split = variants.iter().find(|v| v.name == "LAYOUT_split").unwrap();
        assert_eq!(split.key_count, 4);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_extract_layout_definition() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();

        let layout = extract_layout_definition(&info, "LAYOUT").unwrap();
        assert_eq!(layout.layout.len(), 6);

        let split_layout = extract_layout_definition(&info, "LAYOUT_split").unwrap();
        assert_eq!(split_layout.layout.len(), 4);
        assert_eq!(split_layout.layout[0].w, 1.5);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_build_keyboard_geometry() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();
        let geometry = build_keyboard_geometry(&info, "test_keyboard", "LAYOUT").unwrap();

        assert_eq!(geometry.keyboard_name, "test_keyboard");
        assert_eq!(geometry.layout_name, "LAYOUT");
        assert_eq!(geometry.matrix_rows, 2);
        assert_eq!(geometry.matrix_cols, 3);
        assert_eq!(geometry.keys.len(), 6);

        // Check first key
        assert_eq!(geometry.keys[0].matrix_position, (0, 0));
        assert_eq!(geometry.keys[0].led_index, 0);
        assert_eq!(geometry.keys[0].visual_x, 0.0);
        assert_eq!(geometry.keys[0].visual_y, 0.0);
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn test_build_keyboard_geometry_with_split_layout() {
        let temp_dir = TempDir::new().unwrap();
        let info_path = temp_dir.path().join("info.json");
        fs::write(&info_path, create_test_info_json()).unwrap();

        let info = parse_info_json(&info_path).unwrap();
        let geometry = build_keyboard_geometry(&info, "test_keyboard", "LAYOUT_split").unwrap();

        assert_eq!(geometry.matrix_rows, 5); // max row is 4, so rows = 5
        assert_eq!(geometry.matrix_cols, 2);
        assert_eq!(geometry.keys.len(), 4);

        // Check key with custom width
        assert_eq!(geometry.keys[0].width, 1.5);
    }

    #[test]
    fn test_scan_keyboards_invalid_path() {
        let temp_dir = TempDir::new().unwrap();

        // Test with a path that has no keyboards directory
        let result = scan_keyboards(temp_dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("QMK keyboards directory not found"));
    }

    // Note: Testing scan_keyboards with actual QMK requires the QMK CLI to be installed
    // and a valid QMK repository. See tests/qmk_info_json_tests.rs for integration tests
    // that test against the actual QMK firmware submodule.
}
