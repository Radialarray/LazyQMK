//! QMK info.json parser for keyboard geometry and layout information.
//!
//! This module handles parsing QMK's info.json files to extract keyboard metadata,
//! layout definitions, and physical key positions for building coordinate mappings.

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
    /// Available layouts
    pub layouts: HashMap<String, LayoutDefinition>,
    /// Matrix pins configuration
    pub matrix_pins: Option<MatrixPins>,
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

const fn default_key_size() -> f32 {
    1.0
}

/// RGB matrix LED layout entry from keyboard.json.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RgbMatrixLayoutEntry {
    /// Matrix position [row, col] for this LED
    #[serde(default)]
    pub matrix: Option<[u8; 2]>,
    /// Physical X coordinate in arbitrary units (optional)
    #[serde(default)]
    pub x: Option<f32>,
    /// Physical Y coordinate in arbitrary units (optional)
    #[serde(default)]
    pub y: Option<f32>,
    /// Flags bitfield (keyboard-specific semantics, optional)
    #[serde(default)]
    pub flags: Option<u8>,
}

/// RGB matrix configuration from keyboard.json.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RgbMatrixConfig {
    /// Per-LED layout entries in electrical LED order.
    #[serde(default)]
    pub layout: Vec<RgbMatrixLayoutEntry>,
}

/// QMK keyboard.json structure (simplified for our needs).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QmkKeyboardJson {
    /// Optional RGB matrix configuration.
    #[serde(default)]
    pub rgb_matrix: Option<RgbMatrixConfig>,
}

/// Parses a QMK keyboard.json file by keyboard name.
///
/// Returns `Ok(None)` if no keyboard.json is found for the given keyboard
/// (or its base keyboard if the name includes a variant suffix).
pub fn parse_keyboard_json(qmk_path: &Path, keyboard: &str) -> Result<Option<QmkKeyboardJson>> {
    let keyboards_dir = qmk_path.join("keyboards");
    let keyboard_dir = keyboards_dir.join(keyboard);
    let keyboard_json_path = keyboard_dir.join("keyboard.json");

    if keyboard_json_path.exists() {
        let content = fs::read_to_string(&keyboard_json_path).context(format!(
            "Failed to read keyboard.json: {}",
            keyboard_json_path.display()
        ))?;

        let keyboard_json: QmkKeyboardJson = serde_json::from_str(&content).context(format!(
            "Failed to parse keyboard.json: {}",
            keyboard_json_path.display()
        ))?;

        return Ok(Some(keyboard_json));
    }

    // If the keyboard path looks like it includes a variant suffix,
    // fall back to the base keyboard directory for keyboard.json.
    if let Some((base, _variant)) = keyboard.rsplit_once('/') {
        let base_dir = keyboards_dir.join(base);
        let base_keyboard_json_path = base_dir.join("keyboard.json");

        if base_keyboard_json_path.exists() {
            let content = fs::read_to_string(&base_keyboard_json_path).context(format!(
                "Failed to read keyboard.json: {}",
                base_keyboard_json_path.display()
            ))?;

            let keyboard_json: QmkKeyboardJson =
                serde_json::from_str(&content).context(format!(
                    "Failed to parse keyboard.json: {}",
                    base_keyboard_json_path.display()
                ))?;

            return Ok(Some(keyboard_json));
        }
    }

    Ok(None)
}

/// Builds `KeyboardGeometry` from QMK info.json and optional keyboard.json RGB matrix mapping.
///
/// When a keyboard.json with `rgb_matrix.layout` is present for the given keyboard,
/// this function remaps `KeyGeometry.led_index` so that it matches the RGB matrix LED
/// order used by QMK (`g_led_config`). If no keyboard.json (or `rgb_matrix.layout`) is
/// found, it falls back to the sequential LED indices from info.json layout order.
pub fn build_keyboard_geometry_with_led_mapping(
    info: &QmkInfoJson,
    qmk_path: &Path,
    keyboard_name: &str,
    layout_name: &str,
) -> Result<KeyboardGeometry> {
    // First build basic geometry from info.json layout definition.
    let mut geometry = build_keyboard_geometry(info, keyboard_name, layout_name)?;

    // Try to load keyboard.json for RGB matrix information.
    if let Some(kb_json) = parse_keyboard_json(qmk_path, keyboard_name)? {
        if let Some(rgb) = kb_json.rgb_matrix {
            // Build matrix -> LED index map from rgb_matrix.layout.
            let mut matrix_to_led: HashMap<(u8, u8), u8> = HashMap::new();
            for (idx, entry) in rgb.layout.iter().enumerate() {
                if let Some([row, col]) = entry.matrix {
                    // Keep the first mapping we see for a given matrix position.
                    matrix_to_led.entry((row, col)).or_insert(idx as u8);
                }
            }

            // Remap each key's led_index based on its matrix position if present.
            for key in &mut geometry.keys {
                if let Some(&led_idx) = matrix_to_led.get(&key.matrix_position) {
                    key.led_index = led_idx;
                }
            }
        }
    }

    Ok(geometry)
}

/// Scans the QMK keyboards directory and returns a list of available keyboards.
///
/// # Arguments
///
/// * `qmk_path` - Path to QMK firmware root directory
///
/// # Returns
///
/// A vector of keyboard names (relative paths from keyboards/ directory)
#[allow(dead_code)]
pub fn scan_keyboards(qmk_path: &Path) -> Result<Vec<String>> {
    let keyboards_dir = qmk_path.join("keyboards");

    if !keyboards_dir.exists() {
        anyhow::bail!(
            "QMK keyboards directory not found: {}",
            keyboards_dir.display()
        );
    }

    let mut keyboards = Vec::new();
    scan_keyboards_recursive(&keyboards_dir, "", &mut keyboards)?;

    // Sort alphabetically for consistent ordering
    keyboards.sort();

    Ok(keyboards)
}

/// Recursively scans keyboard directories looking for info.json files.
#[allow(dead_code)]
fn scan_keyboards_recursive(dir: &Path, prefix: &str, keyboards: &mut Vec<String>) -> Result<()> {
    let entries =
        fs::read_dir(dir).context(format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden directories and common non-keyboard directories
        if name_str.starts_with('.')
            || name_str == "keymaps"
            || name_str == "lib"
            || name_str == "common"
        {
            continue;
        }

        if path.is_dir() {
            // Check if this directory has an info.json
            let info_json_path = path.join("info.json");
            if info_json_path.exists() {
                let keyboard_name = if prefix.is_empty() {
                    name_str.to_string()
                } else {
                    format!("{prefix}/{name_str}")
                };
                keyboards.push(keyboard_name);
            }

            // Recurse into subdirectories
            let new_prefix = if prefix.is_empty() {
                name_str.to_string()
            } else {
                format!("{prefix}/{name_str}")
            };
            scan_keyboards_recursive(&path, &new_prefix, keyboards)?;
        }
    }

    Ok(())
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
/// variant paths (e.g., "keebart/corne_choc_pro/standard"). If the
/// variant directory does not contain its own `info.json`, it falls back
/// to the base keyboard directory so keyboards that store `info.json`
/// only at the root still work.
///
/// # Arguments
///
/// * `qmk_path` - Path to QMK firmware root directory
/// * `keyboard` - Keyboard name (e.g., "crkbd", "ferris/sweep",
///   "keebart/corne_choc_pro/standard")
///
/// # Returns
///
/// Parsed QMK info.json structure
pub fn parse_keyboard_info_json(qmk_path: &Path, keyboard: &str) -> Result<QmkInfoJson> {
    // First try the path as given (this supports keyboards that keep
    // their info.json inside a variant directory).
    let keyboards_dir = qmk_path.join("keyboards");
    let info_json_path = keyboards_dir.join(keyboard).join("info.json");

    if info_json_path.exists() {
        return parse_info_json(&info_json_path);
    }

    // If that fails and the keyboard path looks like it includes a
    // variant suffix, fall back to the base keyboard directory.
    if let Some((base, _variant)) = keyboard.rsplit_once('/') {
        let base_info_json_path = keyboards_dir.join(base).join("info.json");

        if base_info_json_path.exists() {
            return parse_info_json(&base_info_json_path);
        }
    }

    anyhow::bail!(
        "info.json not found for keyboard '{}' under {}",
        keyboard,
        keyboards_dir.display()
    );
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
pub fn build_keyboard_geometry(
    info: &QmkInfoJson,
    keyboard_name: &str,
    layout_name: &str,
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
    for (led_index, key_pos) in layout_def.layout.iter().enumerate() {
        let matrix_position = key_pos.matrix.unwrap(); // Already validated above

        let key_geometry = KeyGeometry {
            matrix_position: (matrix_position[0], matrix_position[1]),
            led_index: led_index as u8,
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
    fn test_scan_keyboards() {
        let temp_dir = TempDir::new().unwrap();
        let keyboards_dir = temp_dir.path().join("keyboards");
        fs::create_dir(&keyboards_dir).unwrap();

        // Create test keyboard directories with info.json
        let kb1_dir = keyboards_dir.join("keyboard1");
        fs::create_dir(&kb1_dir).unwrap();
        fs::write(kb1_dir.join("info.json"), "{}").unwrap();

        let kb2_dir = keyboards_dir.join("vendor");
        fs::create_dir(&kb2_dir).unwrap();
        let kb2_model_dir = kb2_dir.join("model_a");
        fs::create_dir(&kb2_model_dir).unwrap();
        fs::write(kb2_model_dir.join("info.json"), "{}").unwrap();

        // Scan keyboards
        let keyboards = scan_keyboards(temp_dir.path()).unwrap();

        assert_eq!(keyboards.len(), 2);
        assert!(keyboards.contains(&"keyboard1".to_string()));
        assert!(keyboards.contains(&"vendor/model_a".to_string()));
    }

    #[test]
    fn test_parse_keyboard_json_and_led_mapping() {
        let temp_dir = TempDir::new().unwrap();
        let qmk_root = temp_dir.path();
        let keyboards_dir = qmk_root.join("keyboards");
        fs::create_dir(&keyboards_dir).unwrap();

        // Create a fake keyboard with both info.json and keyboard.json
        let kb_dir = keyboards_dir.join("vendor").join("model");
        fs::create_dir_all(&kb_dir).unwrap();

        // info.json with two keys, sequential layout order
        let info_json = r#"{
            "keyboard_name": "test_keyboard",
            "layouts": {
                "LAYOUT": {
                    "layout": [
                        {"x": 0, "y": 0, "matrix": [0, 0]},
                        {"x": 1, "y": 0, "matrix": [0, 1]}
                    ]
                }
            }
        }"#;
        fs::write(kb_dir.join("info.json"), info_json).unwrap();

        // keyboard.json with rgb_matrix.layout that reverses the LED order
        let keyboard_json = r#"{
            "keyboard_name": "test_keyboard",
            "rgb_matrix": {
                "layout": [
                    {"matrix": [0, 1]},
                    {"matrix": [0, 0]}
                ]
            }
        }"#;
        fs::write(kb_dir.join("keyboard.json"), keyboard_json).unwrap();

        let info = parse_keyboard_info_json(qmk_root, "vendor/model").unwrap();
        let geometry = build_keyboard_geometry_with_led_mapping(
            &info,
            qmk_root,
            "vendor/model",
            "LAYOUT",
        )
        .unwrap();

        assert_eq!(geometry.keys.len(), 2);

        // Matrix positions should remain unchanged
        assert_eq!(geometry.keys[0].matrix_position, (0, 0));
        assert_eq!(geometry.keys[1].matrix_position, (0, 1));

        // LED indices should come from rgb_matrix.layout mapping
        let led_indices: Vec<u8> = geometry.keys.iter().map(|k| k.led_index).collect();
        assert_eq!(led_indices, vec![1, 0]);
    }
}
