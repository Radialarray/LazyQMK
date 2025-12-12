//! Markdown layout file parsing and generation.
//!
//! This module handles parsing keyboard layouts from human-readable Markdown files
//! and generating them back for saving. The format uses YAML frontmatter for metadata
//! and Markdown tables for key assignments.

// Allow intentional type casts for parsing
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::constants::APP_BINARY_NAME;
use crate::models::{Category, KeyDefinition, Layer, Layout, LayoutMetadata, Position, RgbColor};
use anyhow::{Context, Result};
use regex::Regex;
use std::path::Path;

/// Parsing state machine states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ParseState {
    /// Reading YAML frontmatter (between --- markers)
    InFrontmatter,
    /// Reading main content (after frontmatter)
    InContent,
    /// Reading layer header line (## Layer N: Name)
    InLayerHeader,
    /// Reading layer properties (Color, Category)
    InLayerProperties,
    /// Reading layer table
    InLayerTable,
    /// Reading categories section
    InCategories,
}

/// Parses a Markdown layout file into a Layout structure.
///
/// # File Format
///
/// ```markdown
/// ---
/// name: "Layout Name"
/// description: "Description"
/// author: "Author"
/// created: "2024-01-15T10:30:00Z"
/// modified: "2024-01-20T15:45:00Z"
/// tags: ["tag1", "tag2"]
/// is_template: false
/// version: "1.0"
/// ---
///
/// # Layout Title
///
/// ## Layer 0: Base
/// **Color**: #808080
/// **Category**: optional-category-id
///
/// | KC_TAB | KC_Q | ... |
/// |--------|------|-----|
/// | KC_A   | KC_S | ... |
///
/// ## Categories
///
/// - category-id: Category Name (#RRGGBB)
/// ```
///
/// # Errors
///
/// Returns errors for:
/// - File not found
/// - Invalid YAML frontmatter
/// - Malformed layer headers
/// - Invalid table structure
/// - Invalid keycodes or color syntax
pub fn parse_markdown_layout(path: &Path) -> Result<Layout> {
    // Check if file exists first to provide better error message
    if !path.exists() {
        anyhow::bail!(
            "Layout file not found: {}\n\n\
             Please check the file path and try again.\n\
             If you need help getting started, run: {} --init",
            path.display(),
            APP_BINARY_NAME
        );
    }

    // Check if it's a file (not a directory)
    if !path.is_file() {
        anyhow::bail!(
            "Path is not a file: {}\n\n\
            Please provide a path to a Markdown (.md) file.",
            path.display()
        );
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read layout file: {}", path.display()))?;

    parse_markdown_layout_str(&content)
        .with_context(|| format!("Failed to parse layout file: {}", path.display()))
}

/// Parses a Markdown layout from a string.
pub fn parse_markdown_layout_str(content: &str) -> Result<Layout> {
    let lines: Vec<&str> = content.lines().collect();

    // Parse frontmatter
    let (metadata, content_start) = parse_frontmatter(&lines)?;

    // Create layout
    let mut layout = Layout {
        metadata,
        layers: Vec::new(),
        categories: Vec::new(),
        rgb_enabled: true,
        rgb_brightness: crate::models::RgbBrightness::default(),
        rgb_saturation: crate::models::RgbSaturation::default(),
        rgb_timeout_ms: 0,
        uncolored_key_behavior: crate::models::UncoloredKeyBehavior::default(),
        idle_effect_settings: crate::models::IdleEffectSettings::default(),
        tap_hold_settings: crate::models::TapHoldSettings::default(),
    };

    // Parse content (layers and categories)
    parse_content(&lines[content_start..], &mut layout)?;

    // Validate the parsed layout
    layout.validate()?;

    Ok(layout)
}

/// Parses YAML frontmatter from the beginning of the file.
///
/// Returns the parsed metadata and the line index where content starts.
fn parse_frontmatter(lines: &[&str]) -> Result<(LayoutMetadata, usize)> {
    // Find frontmatter boundaries
    let mut start_idx = None;
    let mut end_idx = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            if start_idx.is_none() {
                start_idx = Some(idx);
            } else if end_idx.is_none() {
                end_idx = Some(idx);
                break;
            }
        }
    }

    let start =
        start_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter start marker (---)"))?;
    let end = end_idx.ok_or_else(|| anyhow::anyhow!("Missing frontmatter end marker (---)"))?;

    // Extract YAML content (between the --- markers)
    let yaml_content = lines[start + 1..end].join("\n");

    // Parse YAML
    let metadata: LayoutMetadata =
        serde_yml::from_str(&yaml_content).context("Failed to parse YAML frontmatter")?;

    // Validate metadata
    validate_metadata(&metadata)?;

    Ok((metadata, end + 1))
}

/// Validates metadata after parsing.
fn validate_metadata(metadata: &LayoutMetadata) -> Result<()> {
    if metadata.name.is_empty() {
        anyhow::bail!("Layout name cannot be empty");
    }

    if metadata.name.len() > 100 {
        anyhow::bail!(
            "Layout name exceeds maximum length of 100 characters (got {})",
            metadata.name.len()
        );
    }

    if metadata.modified < metadata.created {
        anyhow::bail!("Modified timestamp cannot be before created timestamp");
    }

    if metadata.version != "1.0" {
        anyhow::bail!(
            "Unsupported schema version '{}'. Only version '1.0' is supported.",
            metadata.version
        );
    }

    // Validate tags
    let tag_regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
    for tag in &metadata.tags {
        if !tag_regex.is_match(tag) {
            anyhow::bail!(
                "Invalid tag '{tag}'. Tags must be lowercase with hyphens and alphanumeric characters only"
            );
        }
    }

    Ok(())
}

/// Parses the content section (layers and categories).
fn parse_content(lines: &[&str], layout: &mut Layout) -> Result<()> {
    let mut line_num = 0;

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines and main title
        if line.is_empty() || line.starts_with("# ") {
            line_num += 1;
            continue;
        }

        // Check for layer header (## Layer N: Name)
        if line.starts_with("## Layer ") {
            line_num = parse_layer(lines, line_num, layout)
                .with_context(|| format!("Error parsing layer at line {}", line_num + 1))?;
            continue;
        }

        // Check for categories section (## Categories)
        if line == "## Categories" {
            line_num = parse_categories(lines, line_num, layout)
                .with_context(|| format!("Error parsing categories at line {}", line_num + 1))?;
            continue;
        }

        // Check for settings section (## Settings)
        if line == "## Settings" {
            line_num = parse_settings(lines, line_num, layout)
                .with_context(|| format!("Error parsing settings at line {}", line_num + 1))?;
            continue;
        }

        // Check for key descriptions section (## Key Descriptions)
        if line == "## Key Descriptions" {
            line_num = parse_key_descriptions(lines, line_num, layout).with_context(|| {
                format!("Error parsing key descriptions at line {}", line_num + 1)
            })?;
            continue;
        }

        line_num += 1;
    }

    Ok(())
}

/// Parses a single layer section.
fn parse_layer(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    let mut line_num = start_line;
    let header_line = lines[line_num];

    // Parse layer header: ## Layer N: Name
    let layer_regex = Regex::new(r"^##\s+Layer\s+(\d+):\s+(.+)$").unwrap();
    let captures = layer_regex
        .captures(header_line)
        .ok_or_else(|| anyhow::anyhow!("Invalid layer header format: {header_line}"))?;

    let layer_number: u8 = captures[1]
        .parse()
        .context("Failed to parse layer number")?;
    let layer_name = captures[2].trim().to_string();

    line_num += 1;

    // Parse layer properties (Color and optional Category)
    let mut layer_color = None;
    let mut layer_category = None;
    let mut layer_colors_enabled = true; // Default to true
    let mut layer_id = None; // Optional layer ID for persistence

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Parse color: **Color**: #RRGGBB
        if line.starts_with("**Color**:") {
            let color_str = line.strip_prefix("**Color**:").unwrap().trim();
            layer_color =
                Some(RgbColor::from_hex(color_str).context("Failed to parse layer color")?);
            line_num += 1;
            continue;
        }

        // Parse optional ID: **ID**: uuid
        if line.starts_with("**ID**:") {
            layer_id = Some(line.strip_prefix("**ID**:").unwrap().trim().to_string());
            line_num += 1;
            continue;
        }

        // Parse optional category: **Category**: category-id
        if line.starts_with("**Category**:") {
            let category_id = line
                .strip_prefix("**Category**:")
                .unwrap()
                .trim()
                .to_string();
            layer_category = Some(category_id);
            line_num += 1;
            continue;
        }

        // Parse optional layer colors enabled: **Layer Colors**: true/false
        if line.starts_with("**Layer Colors**:") {
            let value = line
                .strip_prefix("**Layer Colors**:")
                .unwrap()
                .trim()
                .to_lowercase();
            layer_colors_enabled = value == "true" || value == "yes" || value == "1";
            line_num += 1;
            continue;
        }

        // Table starts - break out of properties loop
        if line.starts_with('|') {
            break;
        }

        line_num += 1;
    }

    let color = layer_color.ok_or_else(|| {
        anyhow::anyhow!("Layer {layer_number} missing required **Color** property")
    })?;

    // Create layer
    let mut layer = Layer::new(layer_number, layer_name, color)?;
    // Use persisted ID if available, otherwise keep the generated one
    if let Some(id) = layer_id {
        layer.id = id;
    }
    layer.category_id = layer_category;
    layer.layer_colors_enabled = layer_colors_enabled;

    // Parse table
    line_num = parse_layer_table(lines, line_num, &mut layer)?;

    // Add layer to layout
    layout.add_layer(layer)?;

    Ok(line_num)
}

/// Parses a layer's key table.
fn parse_layer_table(lines: &[&str], start_line: usize, layer: &mut Layer) -> Result<usize> {
    let mut line_num = start_line;
    let mut row = 0;

    // Skip table header row
    if line_num < lines.len() && lines[line_num].starts_with('|') {
        line_num += 1;
    }

    // Skip separator row (|---|---|)
    if line_num < lines.len() && lines[line_num].contains("---") {
        line_num += 1;
    }

    // Parse data rows
    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Stop at empty line or next section
        if line.is_empty() || line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Parse table row
        if line.starts_with('|') {
            parse_table_row(line, row, layer).with_context(|| {
                format!("Error parsing table row {} at line {}", row, line_num + 1)
            })?;
            row += 1;
        }

        line_num += 1;
    }

    Ok(line_num)
}

/// Parses a single table row into key definitions.
fn parse_table_row(line: &str, row: u8, layer: &mut Layer) -> Result<()> {
    // Split by pipes and trim, keeping empty cells to preserve column indices
    // This is critical for split keyboards where gaps between halves are empty cells
    let cells: Vec<&str> = line.split('|').map(str::trim).collect();

    // Skip leading empty element from split (line starts with '|')
    // and trailing empty element (line ends with '|')
    let cells = if cells.len() >= 2 {
        &cells[1..cells.len() - 1]
    } else {
        &cells[..]
    };

    for (col, cell) in cells.iter().enumerate() {
        // Skip empty cells (gaps in split keyboards) but preserve column index
        if cell.is_empty() {
            continue;
        }

        // Parse keycode syntax
        let key = parse_keycode_syntax(cell, row, col as u8)
            .with_context(|| format!("Error parsing cell at row {row}, col {col}: {cell}"))?;

        layer.add_key(key);
    }

    Ok(())
}

/// Parses keycode syntax with optional color and category.
///
/// Formats:
/// - `KC_X` - basic keycode
/// - `KC_X{#RRGGBB}` - with color override
/// - `KC_X@category-id` - with category
/// - `KC_X{#RRGGBB}@category-id` - with both
fn parse_keycode_syntax(cell: &str, row: u8, col: u8) -> Result<KeyDefinition> {
    // Updated regex to support:
    // - Basic keycodes: KC_A, KC_LEFT, etc.
    // - Parameterized keycodes: LT(0, KC_A), MT(MOD_LCTL, KC_A)
    // - Layer UUIDs inside params: LT(@f85996a8-8dbd-403d-a804-fac1f2bc751d, KC_R)
    // - With optional color suffix: {#RRGGBB}
    // - With optional category suffix: @category-id
    // Pattern breakdown:
    //   [A-Z_][A-Z_0-9]*  - Keycode prefix (must start with letter or underscore)
    //   (?:\([^)]*\))?    - Optional parentheses with anything inside (for params)
    //   (?:\{...\})?      - Optional color override
    //   (?:@...)?         - Optional category suffix (@ only allowed here, not in keycode)
    let keycode_regex =
        Regex::new(r"^([A-Z_][A-Z_0-9]*(?:\([^)]*\))?)(?:\{(#[0-9A-Fa-f]{6})\})?(?:@([a-z][a-z0-9-]*))?\s*$")
            .unwrap();

    let captures = keycode_regex
        .captures(cell)
        .ok_or_else(|| anyhow::anyhow!("Invalid keycode syntax: {cell}"))?;

    let keycode = captures[1].to_string();
    let color_override = captures
        .get(2)
        .map(|m| RgbColor::from_hex(m.as_str()))
        .transpose()?;
    let category_id = captures.get(3).map(|m| m.as_str().to_string());

    let position = Position::new(row, col);
    let mut key = KeyDefinition::new(position, keycode);

    if let Some(color) = color_override {
        key = key.with_color(color);
    }

    if let Some(cat_id) = category_id {
        key = key.with_category(&cat_id);
    }

    Ok(key)
}

/// Parses the categories section.
fn parse_categories(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Categories" header

    let category_regex =
        Regex::new(r"^-\s+([a-z][a-z0-9-]*):\s+(.+?)\s+\(#([0-9A-Fa-f]{6})\)$").unwrap();

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") {
            break;
        }

        // Parse category line: - id: Name (#RRGGBB)
        if let Some(captures) = category_regex.captures(line) {
            let id = captures[1].to_string();
            let name = captures[2].to_string();
            let color_hex = format!("#{}", &captures[3]);
            let color = RgbColor::from_hex(&color_hex)?;

            let category = Category::new(&id, &name, color)?;
            layout.add_category(category)?;
        }

        line_num += 1;
    }

    Ok(line_num)
}

/// Parses the settings section.
#[allow(clippy::cognitive_complexity, clippy::unnecessary_wraps)]
fn parse_settings(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    use crate::models::{HoldDecisionMode, TapHoldPreset};

    let mut line_num = start_line + 1; // Skip "## Settings" header

    // Track if a preset was explicitly specified in the file
    let mut explicit_preset: Option<TapHoldPreset> = None;

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Parse setting: **Setting Name**: value
        // Support both old and new names for backwards compatibility
        if line.starts_with("**Inactive Key Behavior**:")
            || line.starts_with("**Uncolored Key Behavior**:")
            || line.starts_with("**Uncolored Key Brightness**:")
        {
            let value = line
                .strip_prefix("**Inactive Key Behavior**:")
                .or_else(|| line.strip_prefix("**Uncolored Key Behavior**:"))
                .or_else(|| line.strip_prefix("**Uncolored Key Brightness**:"))
                .unwrap()
                .trim()
                .to_lowercase();

            // Parse as percentage (legacy format support)
            let percent = if value.contains('%') {
                value
                    .trim_end_matches('%')
                    .trim()
                    .parse::<u8>()
                    .unwrap_or(100)
            } else {
                match value.as_str() {
                    "off" | "black" | "off (black)" => 0,
                    "show color" | "full" => 100,
                    _ => value.parse::<u8>().unwrap_or(100),
                }
            }
            .min(100);
            layout.uncolored_key_behavior = crate::models::UncoloredKeyBehavior::from(percent);
        }

        // Parse RGB Master Switch
        if line.starts_with("**RGB Enabled**:") || line.starts_with("**RGB Master Switch**:") {
            let value = line
                .strip_prefix("**RGB Enabled**:")
                .or_else(|| line.strip_prefix("**RGB Master Switch**:"))
                .unwrap()
                .trim()
                .to_lowercase();
            layout.rgb_enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
        }

        // Parse RGB Brightness
        if line.starts_with("**RGB Brightness**:") {
            let value = line
                .strip_prefix("**RGB Brightness**:")
                .unwrap()
                .trim()
                .trim_end_matches('%');
            if let Ok(percent) = value.parse::<u8>() {
                layout.rgb_brightness = crate::models::RgbBrightness::from(percent);
            }
        }

        // Parse RGB Saturation
        if line.starts_with("**RGB Saturation**:") {
            let value = line
                .strip_prefix("**RGB Saturation**:")
                .unwrap()
                .trim()
                .trim_end_matches('%');
            if let Ok(percent) = value.parse::<u8>() {
                layout.rgb_saturation = crate::models::RgbSaturation::from(percent);
            }
        }

        // === Tap-Hold Settings ===

        // Parse Tap-Hold Preset - remember it for later
        if line.starts_with("**Tap-Hold Preset**:") {
            let value = line
                .strip_prefix("**Tap-Hold Preset**:")
                .unwrap()
                .trim()
                .to_lowercase();

            let preset = match value.as_str() {
                "home row mods" | "homerowmods" => TapHoldPreset::HomeRowMods,
                "responsive" => TapHoldPreset::Responsive,
                "deliberate" => TapHoldPreset::Deliberate,
                "custom" => TapHoldPreset::Custom,
                _ => TapHoldPreset::Default,
            };
            explicit_preset = Some(preset);
            layout.tap_hold_settings.preset = preset;
        }

        // Parse Tapping Term
        if line.starts_with("**Tapping Term**:") {
            let value = line
                .strip_prefix("**Tapping Term**:")
                .unwrap()
                .trim()
                .trim_end_matches("ms")
                .trim();
            if let Ok(term) = value.parse::<u16>() {
                layout.tap_hold_settings.tapping_term = term;
            }
        }

        // Parse Quick Tap Term
        if line.starts_with("**Quick Tap Term**:") {
            let value = line
                .strip_prefix("**Quick Tap Term**:")
                .unwrap()
                .trim()
                .to_lowercase();

            if value == "auto" || value == "same as tapping term" || value == "none" {
                layout.tap_hold_settings.quick_tap_term = None;
            } else {
                let term_str = value.trim_end_matches("ms").trim();
                if let Ok(term) = term_str.parse::<u16>() {
                    layout.tap_hold_settings.quick_tap_term = Some(term);
                }
            }
        }

        // Parse Hold Mode
        if line.starts_with("**Hold Mode**:") {
            let value = line
                .strip_prefix("**Hold Mode**:")
                .unwrap()
                .trim()
                .to_lowercase();

            layout.tap_hold_settings.hold_mode = match value.as_str() {
                "permissive" | "permissive hold" => HoldDecisionMode::PermissiveHold,
                "hold on other key" | "hold on other key press" | "aggressive" => {
                    HoldDecisionMode::HoldOnOtherKeyPress
                }
                _ => HoldDecisionMode::Default,
            };
        }

        // Parse Retro Tapping
        if line.starts_with("**Retro Tapping**:") {
            let value = line
                .strip_prefix("**Retro Tapping**:")
                .unwrap()
                .trim()
                .to_lowercase();

            layout.tap_hold_settings.retro_tapping =
                value == "on" || value == "true" || value == "yes" || value == "enabled";
        }

        // Parse Tapping Toggle
        if line.starts_with("**Tapping Toggle**:") {
            let value = line
                .strip_prefix("**Tapping Toggle**:")
                .unwrap()
                .trim()
                .trim_end_matches(" taps")
                .trim();
            if let Ok(count) = value.parse::<u8>() {
                layout.tap_hold_settings.tapping_toggle = count;
            }
        }

        // Parse Flow Tap Term
        if line.starts_with("**Flow Tap Term**:") {
            let value = line
                .strip_prefix("**Flow Tap Term**:")
                .unwrap()
                .trim()
                .to_lowercase();

            if value == "disabled" || value == "off" || value == "none" {
                layout.tap_hold_settings.flow_tap_term = None;
            } else {
                let term_str = value.trim_end_matches("ms").trim();
                if let Ok(term) = term_str.parse::<u16>() {
                    layout.tap_hold_settings.flow_tap_term = Some(term);
                }
            }
        }

        // Parse Chordal Hold
        if line.starts_with("**Chordal Hold**:") {
            let value = line
                .strip_prefix("**Chordal Hold**:")
                .unwrap()
                .trim()
                .to_lowercase();

            layout.tap_hold_settings.chordal_hold =
                value == "on" || value == "true" || value == "yes" || value == "enabled";
        }

        // Parse RGB Timeout
        if line.starts_with("**RGB Timeout**:") {
            let value = line
                .strip_prefix("**RGB Timeout**:")
                .unwrap()
                .trim()
                .to_lowercase();

            // Parse various formats: "5 min", "300 sec", "300000ms", "disabled", "off"
            if value == "disabled" || value == "off" || value == "0" {
                layout.rgb_timeout_ms = 0;
            } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
                if let Ok(m) = mins.trim().parse::<u32>() {
                    layout.rgb_timeout_ms = m * 60000;
                }
            } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
                if let Ok(s) = secs.trim().parse::<u32>() {
                    layout.rgb_timeout_ms = s * 1000;
                }
            } else if let Some(ms) = value.strip_suffix("ms") {
                if let Ok(m) = ms.trim().parse::<u32>() {
                    layout.rgb_timeout_ms = m;
                }
            } else if let Ok(ms) = value.parse::<u32>() {
                // Plain number, assume milliseconds
                layout.rgb_timeout_ms = ms;
            }
        }

        // === Idle Effect Settings ===

        // Parse Idle Effect enabled/disabled
        if line.starts_with("**Idle Effect**:") {
            let value = line
                .strip_prefix("**Idle Effect**:")
                .unwrap()
                .trim()
                .to_lowercase();
            layout.idle_effect_settings.enabled = matches!(value.as_str(), "on" | "true" | "yes" | "enabled");
        }

        // Parse Idle Timeout
        if line.starts_with("**Idle Timeout**:") {
            let value = line
                .strip_prefix("**Idle Timeout**:")
                .unwrap()
                .trim()
                .to_lowercase();

            // Parse various formats: "1 min", "60 sec", "60000ms", "disabled", "off", "0"
            if value == "disabled" || value == "off" || value == "0" {
                layout.idle_effect_settings.idle_timeout_ms = 0;
            } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
                if let Ok(m) = mins.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_timeout_ms = m * 60000;
                }
            } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
                if let Ok(s) = secs.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_timeout_ms = s * 1000;
                }
            } else if let Some(ms) = value.strip_suffix("ms") {
                if let Ok(m) = ms.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_timeout_ms = m;
                }
            } else if let Ok(ms) = value.parse::<u32>() {
                // Plain number, assume milliseconds
                layout.idle_effect_settings.idle_timeout_ms = ms;
            }
        }

        // Parse Idle Effect Duration
        if line.starts_with("**Idle Effect Duration**:") {
            let value = line
                .strip_prefix("**Idle Effect Duration**:")
                .unwrap()
                .trim()
                .to_lowercase();

            // Parse various formats: "5 min", "300 sec", "300000ms", "0"
            if value == "0" || value == "off" {
                layout.idle_effect_settings.idle_effect_duration_ms = 0;
            } else if let Some(mins) = value.strip_suffix(" min").or(value.strip_suffix("min")) {
                if let Ok(m) = mins.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_effect_duration_ms = m * 60000;
                }
            } else if let Some(secs) = value.strip_suffix(" sec").or(value.strip_suffix("sec")) {
                if let Ok(s) = secs.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_effect_duration_ms = s * 1000;
                }
            } else if let Some(ms) = value.strip_suffix("ms") {
                if let Ok(m) = ms.trim().parse::<u32>() {
                    layout.idle_effect_settings.idle_effect_duration_ms = m;
                }
            } else if let Ok(ms) = value.parse::<u32>() {
                // Plain number, assume milliseconds
                layout.idle_effect_settings.idle_effect_duration_ms = ms;
            }
        }

        // Parse Idle Effect Mode
        if line.starts_with("**Idle Effect Mode**:") {
            let value = line
                .strip_prefix("**Idle Effect Mode**:")
                .unwrap()
                .trim();

            if let Some(effect) = crate::models::RgbMatrixEffect::from_name(value) {
                layout.idle_effect_settings.idle_effect_mode = effect;
            }
        }

        line_num += 1;
    }

    // If an explicit preset was specified, ensure it's preserved
    // (individual setting parsing may have changed it via mark_custom in older code)
    if let Some(preset) = explicit_preset {
        layout.tap_hold_settings.preset = preset;
    }

    Ok(line_num)
}

/// Parses the key descriptions section.
///
/// Format: `- layer:row:col: description text`
/// Example: `- 0:1:3: Primary thumb key - hold for symbols, tap for space`
#[allow(clippy::unnecessary_wraps)]
fn parse_key_descriptions(lines: &[&str], start_line: usize, layout: &mut Layout) -> Result<usize> {
    let mut line_num = start_line + 1; // Skip "## Key Descriptions" header

    // Regex to match: - layer:row:col: description
    let desc_regex = Regex::new(r"^-\s+(\d+):(\d+):(\d+):\s+(.+)$").unwrap();

    while line_num < lines.len() {
        let line = lines[line_num].trim();

        // Skip empty lines
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("##") || line.starts_with("---") {
            break;
        }

        // Parse description line: - layer:row:col: description text
        if let Some(captures) = desc_regex.captures(line) {
            let layer_idx: usize = captures[1].parse().unwrap_or(0);
            let row: u8 = captures[2].parse().unwrap_or(0);
            let col: u8 = captures[3].parse().unwrap_or(0);
            let description = captures[4].trim().to_string();

            // Find the key and set its description
            if let Some(layer) = layout.layers.get_mut(layer_idx) {
                let pos = Position::new(row, col);
                if let Some(key) = layer.get_key_mut(pos) {
                    key.description = Some(description);
                }
            }
        }

        line_num += 1;
    }

    Ok(line_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let lines = vec![
            "---",
            "name: \"Test Layout\"",
            "description: \"A test layout\"",
            "author: \"test\"",
            "created: \"2024-01-15T10:30:00Z\"",
            "modified: \"2024-01-20T15:45:00Z\"",
            "tags: [\"test\", \"example\"]",
            "is_template: false",
            "version: \"1.0\"",
            "---",
            "",
            "# Content starts here",
        ];

        let (metadata, content_start) = parse_frontmatter(&lines).unwrap();
        assert_eq!(metadata.name, "Test Layout");
        assert_eq!(metadata.description, "A test layout");
        assert_eq!(metadata.author, "test");
        assert_eq!(metadata.tags, vec!["test", "example"]);
        assert!(!metadata.is_template);
        assert_eq!(metadata.version, "1.0");
        assert_eq!(content_start, 10);
    }

    #[test]
    fn test_parse_keycode_syntax() {
        // Basic keycode
        let key = parse_keycode_syntax("KC_A", 0, 0).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.position, Position::new(0, 0));
        assert_eq!(key.color_override, None);
        assert_eq!(key.category_id, None);

        // With color override
        let key = parse_keycode_syntax("KC_A{#FF0000}", 0, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(255, 0, 0)));

        // With category
        let key = parse_keycode_syntax("KC_LEFT@navigation", 1, 0).unwrap();
        assert_eq!(key.keycode, "KC_LEFT");
        assert_eq!(key.category_id, Some("navigation".to_string()));

        // With both
        let key = parse_keycode_syntax("KC_A{#00FF00}@symbols", 1, 1).unwrap();
        assert_eq!(key.keycode, "KC_A");
        assert_eq!(key.color_override, Some(RgbColor::new(0, 255, 0)));
        assert_eq!(key.category_id, Some("symbols".to_string()));
    }

    #[test]
    fn test_parse_complete_layout() {
        let content = r#"---
name: "Test Layout"
description: "A test"
author: "test"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
tags: ["test"]
is_template: false
version: "1.0"
---

# Test Layout

## Layer 0: Base
**Color**: #808080

| C0   | C1   |
|------|------|
| KC_A | KC_B |
| KC_C | KC_D |

## Categories

- navigation: Navigation (#0000FF)
"#;

        let layout = parse_markdown_layout_str(content).unwrap();
        assert_eq!(layout.metadata.name, "Test Layout");
        assert_eq!(layout.layers.len(), 1);
        assert_eq!(layout.layers[0].keys.len(), 4);
        println!("Parsed {} categories", layout.categories.len());
        for cat in &layout.categories {
            println!("Category: {} - {} - {:?}", cat.id, cat.name, cat.color);
        }
        assert_eq!(layout.categories.len(), 1);
    }

    #[test]
    fn test_parse_split_keyboard_with_gap() {
        use crate::models::Position;

        // Test that split keyboards with empty gap columns preserve correct key positions
        // This is critical: the gap between left and right halves must not shift columns
        let content = r#"---
name: "Split Layout Test"
description: "Tests gap handling"
author: "test"
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
tags: []
is_template: false
version: "1.0"
---

# Split Layout

## Layer 0: Base
**Color**: #808080

| C0   | C1   |      | C3   | C4   |
|------|------|------|------|------|
| KC_A | KC_B |      | KC_C | KC_D |
| KC_E | KC_F |      | KC_G | KC_H |

"#;

        let layout = parse_markdown_layout_str(content).unwrap();
        assert_eq!(layout.layers.len(), 1);

        let layer = &layout.layers[0];
        // Should have 8 keys: 4 on left (cols 0,1), 4 on right (cols 3,4)
        assert_eq!(layer.keys.len(), 8);

        // Verify left side keys are at columns 0 and 1
        let key_a = layer
            .get_key(Position::new(0, 0))
            .expect("KC_A should be at row 0, col 0");
        assert_eq!(key_a.keycode, "KC_A");

        let key_b = layer
            .get_key(Position::new(0, 1))
            .expect("KC_B should be at row 0, col 1");
        assert_eq!(key_b.keycode, "KC_B");

        // Verify right side keys are at columns 3 and 4 (NOT 2 and 3!)
        // This is the critical test - the gap at column 2 must be preserved
        let key_c = layer
            .get_key(Position::new(0, 3))
            .expect("KC_C should be at row 0, col 3");
        assert_eq!(key_c.keycode, "KC_C");

        let key_d = layer
            .get_key(Position::new(0, 4))
            .expect("KC_D should be at row 0, col 4");
        assert_eq!(key_d.keycode, "KC_D");

        // Verify second row maintains the same column structure
        let key_g = layer
            .get_key(Position::new(1, 3))
            .expect("KC_G should be at row 1, col 3");
        assert_eq!(key_g.keycode, "KC_G");

        let key_h = layer
            .get_key(Position::new(1, 4))
            .expect("KC_H should be at row 1, col 4");
        assert_eq!(key_h.keycode, "KC_H");

        // Verify there's no key at the gap column
        assert!(
            layer.get_key(Position::new(0, 2)).is_none(),
            "Column 2 should be empty (gap)"
        );
        assert!(
            layer.get_key(Position::new(1, 2)).is_none(),
            "Column 2 should be empty (gap)"
        );
    }
}

#[cfg(test)]
mod description_tests {
    use super::*;

    #[test]
    fn test_parse_descriptions_from_real_format() {
        let content = r"---
name: test
description: ''
author: ''
created: 2025-12-03T17:29:20.830366Z
modified: 2025-12-03T19:58:22.195899Z
tags: []
is_template: false
version: '1.0'
---
# test

## Layer 0: Base
**ID**: test-id
**Color**: #808080

| C0        | C1   |
|-----------|------|
| LCG(KC_Q) | KC_Q |

---

## Key Descriptions

- 0:0:0: This is a test
- 0:0:1: Another test
";

        let layout = parse_markdown_layout_str(content).expect("Parse failed");

        println!("Layer 0 has {} keys", layout.layers[0].keys.len());
        for key in &layout.layers[0].keys {
            println!(
                "  {:?}: {} -> {:?}",
                key.position, key.keycode, key.description
            );
        }

        // Check that descriptions were parsed
        let key0 = layout.layers[0]
            .keys
            .iter()
            .find(|k| k.position == Position { row: 0, col: 0 });
        let key1 = layout.layers[0]
            .keys
            .iter()
            .find(|k| k.position == Position { row: 0, col: 1 });

        assert!(key0.is_some(), "Key at 0:0 not found");
        assert!(key1.is_some(), "Key at 0:1 not found");

        assert_eq!(
            key0.unwrap().description,
            Some("This is a test".to_string()),
            "Key 0:0:0 description mismatch"
        );
        assert_eq!(
            key1.unwrap().description,
            Some("Another test".to_string()),
            "Key 0:0:1 description mismatch"
        );
    }
}
