# Spec 025: CLI Commands & End-to-End Testing

**Status**: Draft  
**Created**: 2025-01-16  
**Updated**: 2025-01-16

## Problem Statement

LazyQMK currently provides a rich TUI for keyboard layout editing, but critical workflows are TUI-only, making them difficult to test in an automated, headless manner. While integration tests exist (firmware generation, layer navigation, tap dance flows), they test internals rather than complete end-to-end user journeys. This leads to:

1. **Limited E2E coverage**: No way to test complete flows (load → validate → generate → verify output) without manual TUI interaction.
2. **Brittle refactoring**: Changes to internal APIs can break without catching regressions in user-facing behavior.
3. **Slow debugging**: Developers must manually reproduce issues via TUI rather than running focused CLI commands.
4. **Poor CI/CD integration**: Cannot script common workflows like validation, generation, or configuration in pipelines.

## Solution Overview

Add a comprehensive CLI surface that exposes all core features in a headless, scriptable manner. This enables:

- **Automation**: Script common workflows for testing and CI/CD.
- **E2E testing**: Test complete user journeys from layout file to firmware artifacts.
- **Golden testing**: Deterministic output enables snapshot/regression testing.
- **Developer productivity**: Quick validation and debugging without TUI startup.

### Design Principles

1. **Service layer reuse**: CLI commands delegate to existing services; no logic duplication.
2. **Deterministic output**: Support `--json` for machine-readable results; `--deterministic` for stable timestamps/UUIDs.
3. **Clear exit codes**: 0 = success; 1 = validation/user error; 2+ = internal errors.
4. **Minimal dependencies**: CLI code paths avoid TUI/Ratatui dependencies.
5. **Feature gating**: QMK-dependent commands gated by `--features qmk` or runtime checks.

## CLI Command Specification

### 1. Validation & Inspection

#### `lazyqmk validate`
Validates a layout file against QMK keycodes, layer structure, and tap dance references.

**Usage:**
```bash
lazyqmk validate --layout <file> [--json] [--strict]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--json`: Output results as JSON for machine parsing
- `--strict`: Treat warnings as errors (exit non-zero)

**Exit codes:**
- `0`: Validation passed
- `1`: Validation errors found
- `2`: File I/O or parsing error

**JSON output schema:**
```json
{
  "valid": true|false,
  "errors": [
    {
      "severity": "error"|"warning",
      "message": "Invalid keycode: KC_INVALID",
      "location": {
        "layer": 0,
        "position": {"row": 1, "col": 2}
      }
    }
  ],
  "checks": {
    "keycodes": "passed"|"failed",
    "positions": "passed"|"failed",
    "layer_refs": "passed"|"failed",
    "tap_dances": "passed"|"failed"
  }
}
```

**Example:**
```bash
# Validate layout
lazyqmk validate --layout my_layout.md

# Get JSON results
lazyqmk validate --layout my_layout.md --json

# Strict mode (warnings fail)
lazyqmk validate --layout my_layout.md --strict
```

**Test coverage:**
- Valid layout → exit 0
- Invalid keycode → exit 1 with error details
- Missing position → exit 1
- Orphaned tap dance → exit 1 (or warning in non-strict)
- Malformed file → exit 2

---

#### `lazyqmk inspect`
Reads and displays specific sections of a layout file.

**Usage:**
```bash
lazyqmk inspect --layout <file> --section <name> [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--section <name>`: Section to inspect (required):
  - `metadata`: Name, author, tags, timestamps, keyboard, layout variant
  - `layers`: Layer count, names, key counts
  - `categories`: Defined categories and colors
  - `tap-dances`: Tap dance definitions
  - `settings`: RGB, idle effect, tap-hold settings
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `1`: Invalid section name
- `2`: File I/O or parsing error

**JSON output schema:**
```json
{
  "section": "metadata",
  "data": {
    "name": "My Corne Layout",
    "author": "user",
    "keyboard": "crkbd/rev1",
    "layout_variant": "LAYOUT_split_3x6_3",
    "created": "2025-01-16T10:00:00Z",
    "modified": "2025-01-16T12:00:00Z",
    "tags": ["programming", "vim"]
  }
}
```

**Example:**
```bash
# View metadata
lazyqmk inspect --layout my_layout.md --section metadata

# View tap dances as JSON
lazyqmk inspect --layout my_layout.md --section tap-dances --json
```

**Test coverage:**
- Each section type readable
- Invalid section name → exit 1
- Empty/missing data handled gracefully

---

### 2. QMK Metadata & Geometry

#### `lazyqmk list-keyboards`
Lists all compilable keyboards found in QMK firmware directory.

**Usage:**
```bash
lazyqmk list-keyboards --qmk-path <dir> [--filter <regex>] [--json]
```

**Arguments:**
- `--qmk-path <dir>`: Path to QMK firmware repository (required)
- `--filter <regex>`: Filter keyboards by regex pattern
- `--json`: Output as JSON array

**Exit codes:**
- `0`: Success
- `1`: Invalid QMK path or no keyboards found
- `2`: QMK CLI not available (if needed)

**JSON output schema:**
```json
{
  "keyboards": [
    "crkbd/rev1",
    "ferris/sweep",
    "splitkb/aurora/corne/rev1"
  ],
  "count": 3
}
```

**Example:**
```bash
# List all keyboards
lazyqmk list-keyboards --qmk-path ~/qmk_firmware

# Filter Corne variants
lazyqmk list-keyboards --qmk-path ~/qmk_firmware --filter "corne"

# JSON output
lazyqmk list-keyboards --qmk-path ~/qmk_firmware --json
```

**Test coverage:**
- Finds known keyboards (crkbd, ferris/sweep)
- Filter narrows results
- Invalid QMK path → exit 1
- Mark `#[ignore]` or gate with `--features qmk`

---

#### `lazyqmk list-layouts`
Lists layout variants for a specific keyboard.

**Usage:**
```bash
lazyqmk list-layouts --qmk-path <dir> --keyboard <path> [--json]
```

**Arguments:**
- `--qmk-path <dir>`: Path to QMK firmware repository (required)
- `--keyboard <path>`: Keyboard path (e.g., `crkbd/rev1`) (required)
- `--json`: Output as JSON array

**Exit codes:**
- `0`: Success
- `1`: Keyboard not found or no layouts defined
- `2`: QMK path invalid

**JSON output schema:**
```json
{
  "keyboard": "crkbd/rev1",
  "layouts": [
    "LAYOUT",
    "LAYOUT_split_3x6_3"
  ],
  "count": 2
}
```

**Example:**
```bash
# List layouts for Corne
lazyqmk list-layouts --qmk-path ~/qmk_firmware --keyboard crkbd/rev1 --json
```

**Test coverage:**
- Known keyboard returns layouts
- Unknown keyboard → exit 1

---

#### `lazyqmk geometry`
Displays matrix, LED, and visual coordinate mappings for a keyboard layout.

**Usage:**
```bash
lazyqmk geometry --qmk-path <dir> --keyboard <path> --layout-name <name> [--json]
```

**Arguments:**
- `--qmk-path <dir>`: Path to QMK firmware repository (required)
- `--keyboard <path>`: Keyboard path (required)
- `--layout-name <name>`: Layout variant (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `1`: Invalid keyboard or layout name
- `2`: QMK path invalid

**JSON output schema:**
```json
{
  "keyboard": "crkbd/rev1",
  "layout_name": "LAYOUT_split_3x6_3",
  "matrix": {
    "rows": 4,
    "cols": 6
  },
  "keys": 42,
  "mappings": {
    "visual_to_matrix": [
      {"visual": {"row": 0, "col": 0}, "matrix": [0, 0], "led": 0}
    ]
  }
}
```

**Example:**
```bash
lazyqmk geometry --qmk-path ~/qmk_firmware --keyboard crkbd/rev1 --layout-name LAYOUT_split_3x6_3 --json
```

**Test coverage:**
- Coordinate mappings correct for known keyboards
- Round-trip transformations

---

### 3. Keycode & Layer Utilities

#### `lazyqmk keycode resolve`
Resolves parameterized keycodes (LT, LM, MO, etc.) with layer UUIDs to indices.

**Usage:**
```bash
lazyqmk keycode resolve --layout <file> --expr "<keycode>" [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required for layer UUID context)
- `--expr "<keycode>"`: Keycode expression to resolve (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: Successfully resolved
- `1`: Cannot resolve (invalid layer UUID/keycode)
- `2`: File I/O error

**JSON output schema:**
```json
{
  "input": "LT(@abc-123-uuid, KC_SPC)",
  "resolved": "LT(1, KC_SPC)",
  "layer_name": "Lower",
  "valid": true
}
```

**Example:**
```bash
# Resolve LT keycode
lazyqmk keycode resolve --layout my_layout.md --expr "LT(@layer-uuid, KC_SPC)"

# Resolve MO keycode
lazyqmk keycode resolve --layout my_layout.md --expr "MO(@layer-uuid)" --json
```

**Test coverage:**
- LT/LM/MO/TG with UUIDs resolve to correct indices
- Invalid UUID → exit 1
- Non-parameterized keycode passes through

---

#### `lazyqmk layer-refs`
Shows layer reference index and transparency conflict warnings.

**Usage:**
```bash
lazyqmk layer-refs --layout <file> [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `2`: File I/O error

**JSON output schema:**
```json
{
  "layers": [
    {
      "number": 1,
      "name": "Lower",
      "inbound_refs": [
        {
          "from_layer": 0,
          "position": {"row": 1, "col": 5},
          "kind": "TapHold",
          "keycode": "LT(1, KC_SPC)"
        }
      ],
      "warnings": [
        {
          "position": {"row": 1, "col": 5},
          "message": "Non-transparent key at position referenced by hold-like keycode"
        }
      ]
    }
  ]
}
```

**Example:**
```bash
lazyqmk layer-refs --layout my_layout.md --json
```

**Test coverage:**
- Detects inbound references
- Reports transparency conflicts

---

### 4. Tap Dance Management

#### `lazyqmk tap-dance list`
Lists all tap dance definitions in a layout.

**Usage:**
```bash
lazyqmk tap-dance list --layout <file> [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `2`: File I/O error

**JSON output schema:**
```json
{
  "tap_dances": [
    {
      "name": "esc_caps",
      "single_tap": "KC_ESC",
      "double_tap": "KC_CAPS",
      "hold": null,
      "type": "two_way"
    }
  ],
  "count": 1
}
```

**Example:**
```bash
lazyqmk tap-dance list --layout my_layout.md --json
```

---

#### `lazyqmk tap-dance add`
Adds a new tap dance definition to a layout.

**Usage:**
```bash
lazyqmk tap-dance add --layout <file> --name <name> --single <kc> [--double <kc>] [--hold <kc>]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--name <name>`: Unique tap dance name (required)
- `--single <kc>`: Single tap keycode (required)
- `--double <kc>`: Double tap keycode (optional)
- `--hold <kc>`: Hold keycode (optional)

**Exit codes:**
- `0`: Success
- `1`: Validation error (duplicate name, invalid keycode)
- `2`: File I/O error

**Example:**
```bash
# Two-way tap dance
lazyqmk tap-dance add --layout my_layout.md --name esc_caps --single KC_ESC --double KC_CAPS

# Three-way tap dance
lazyqmk tap-dance add --layout my_layout.md --name shift_caps --single KC_LSFT --double KC_CAPS --hold KC_RSFT
```

**Test coverage:**
- Add 2-way and 3-way tap dances
- Duplicate name → exit 1
- Invalid keycode → exit 1
- Verify serialization to YAML frontmatter

---

#### `lazyqmk tap-dance delete`
Removes a tap dance definition from a layout.

**Usage:**
```bash
lazyqmk tap-dance delete --layout <file> --name <name> [--force]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--name <name>`: Tap dance name to delete (required)
- `--force`: Delete even if still referenced by keys (removes references)

**Exit codes:**
- `0`: Success
- `1`: Tap dance not found or still referenced (without --force)
- `2`: File I/O error

**Example:**
```bash
# Delete unused tap dance
lazyqmk tap-dance delete --layout my_layout.md --name old_td

# Force delete (removes references)
lazyqmk tap-dance delete --layout my_layout.md --name old_td --force
```

**Test coverage:**
- Delete unused tap dance succeeds
- Delete referenced tap dance without --force → exit 1
- Force delete removes references

---

#### `lazyqmk tap-dance validate`
Validates tap dance definitions and references.

**Usage:**
```bash
lazyqmk tap-dance validate --layout <file> [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: All valid
- `1`: Validation errors found

**JSON output schema:**
```json
{
  "valid": true,
  "errors": [],
  "warnings": [
    {
      "severity": "warning",
      "message": "Tap dance 'unused_td' is defined but not referenced by any keys"
    }
  ]
}
```

**Example:**
```bash
lazyqmk tap-dance validate --layout my_layout.md --json
```

**Test coverage:**
- Detects orphaned tap dances
- Detects missing definitions

---

### 5. Category Management

#### `lazyqmk category list`
Lists all categories defined in a layout.

**Usage:**
```bash
lazyqmk category list --layout <file> [--json]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `2`: File I/O error

**JSON output schema:**
```json
{
  "categories": [
    {
      "id": "navigation",
      "name": "Navigation",
      "color": "#00FF00"
    }
  ],
  "count": 1
}
```

---

#### `lazyqmk category add`
Adds a new category to a layout.

**Usage:**
```bash
lazyqmk category add --layout <file> --id <id> --name <name> --color <hex>
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--id <id>`: Unique category ID (required)
- `--name <name>`: Display name (required)
- `--color <hex>`: Hex color code (e.g., #FF0000) (required)

**Exit codes:**
- `0`: Success
- `1`: Duplicate ID or invalid color
- `2`: File I/O error

**Example:**
```bash
lazyqmk category add --layout my_layout.md --id nav --name "Navigation" --color "#00FF00"
```

---

#### `lazyqmk category delete`
Removes a category from a layout.

**Usage:**
```bash
lazyqmk category delete --layout <file> --id <id> [--force]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--id <id>`: Category ID to delete (required)
- `--force`: Delete even if used by keys/layers

**Exit codes:**
- `0`: Success
- `1`: Category not found or in use (without --force)
- `2`: File I/O error

---

### 6. Firmware Generation

#### `lazyqmk generate`
Generates QMK firmware files (keymap.c, config.h) from a layout.

**Usage:**
```bash
lazyqmk generate --layout <file> --qmk-path <dir> --out-dir <dir> [--layout-name <name>] [--format <type>] [--deterministic]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--qmk-path <dir>`: Path to QMK firmware repository (required)
- `--out-dir <dir>`: Output directory for generated files (required)
- `--layout-name <name>`: QMK layout variant (auto-detected from metadata if omitted)
- `--format <type>`: Output format: `keymap` | `config` | `all` (default: `all`)
- `--deterministic`: Use stable timestamps/UUIDs for golden testing

**Exit codes:**
- `0`: Generation succeeded
- `1`: Validation failed (layout errors)
- `2`: QMK path invalid or file I/O error

**Example:**
```bash
# Generate all files
lazyqmk generate --layout my_layout.md --qmk-path ~/qmk_firmware --out-dir ./build

# Only keymap.c
lazyqmk generate --layout my_layout.md --qmk-path ~/qmk_firmware --out-dir ./build --format keymap

# Deterministic output for testing
lazyqmk generate --layout my_layout.md --qmk-path ~/qmk_firmware --out-dir ./build --deterministic
```

**Test coverage:**
- Golden tests comparing generated keymap.c/config.h
- Idle effect on/off scenarios
- RGB timeout precedence
- Tap dance 2-way and 3-way generation
- Categories (should not affect C output)
- Deterministic mode produces stable output

---

### 7. Configuration

#### `lazyqmk config show`
Displays current configuration.

**Usage:**
```bash
lazyqmk config show [--json]
```

**Arguments:**
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success
- `1`: Config file not found or invalid

**JSON output schema:**
```json
{
  "paths": {
    "qmk_firmware": "/Users/user/qmk_firmware"
  },
  "build": {
    "output_dir": "/tmp/lazyqmk"
  },
  "ui": {
    "theme": "auto"
  }
}
```

**Example:**
```bash
lazyqmk config show --json
```

---

#### `lazyqmk config set`
Updates configuration values.

**Usage:**
```bash
lazyqmk config set --qmk-path <dir> [--output-dir <dir>] [--theme <mode>]
```

**Arguments:**
- `--qmk-path <dir>`: Path to QMK firmware repository
- `--output-dir <dir>`: Default output directory for builds
- `--theme <mode>`: UI theme: `auto` | `light` | `dark`

**Exit codes:**
- `0`: Success
- `1`: Invalid configuration value
- `2`: Unable to write config file

**Example:**
```bash
lazyqmk config set --qmk-path ~/qmk_firmware --output-dir ~/lazyqmk_builds
```

---

### 8. Templates

#### `lazyqmk template list`
Lists available templates.

**Usage:**
```bash
lazyqmk template list [--json]
```

**Arguments:**
- `--json`: Output as JSON

**Exit codes:**
- `0`: Success

**JSON output schema:**
```json
{
  "templates": [
    {
      "name": "Corne Base Layout",
      "file": "corne_base.md",
      "tags": ["corne", "42-key"],
      "author": "user",
      "created": "2025-01-16T10:00:00Z"
    }
  ],
  "count": 1
}
```

---

#### `lazyqmk template save`
Saves current layout as a template.

**Usage:**
```bash
lazyqmk template save --layout <file> --name <name> [--tags <tag1,tag2>]
```

**Arguments:**
- `--layout <file>`: Path to layout markdown file (required)
- `--name <name>`: Template name (required)
- `--tags <list>`: Comma-separated tags

**Exit codes:**
- `0`: Success
- `1`: Template name already exists
- `2`: File I/O error

---

#### `lazyqmk template apply`
Applies a template to create a new layout file.

**Usage:**
```bash
lazyqmk template apply --name <name> --out <file>
```

**Arguments:**
- `--name <name>`: Template name (required)
- `--out <file>`: Output layout file path (required)

**Exit codes:**
- `0`: Success
- `1`: Template not found or output file exists
- `2`: File I/O error

---

### 9. Utilities

#### `lazyqmk keycodes`
Lists available keycodes from the database.

**Usage:**
```bash
lazyqmk keycodes [--category <name>] [--json]
```

**Arguments:**
- `--category <name>`: Filter by category (e.g., `basic`, `navigation`)
- `--json`: Output as JSON

**JSON output schema:**
```json
{
  "keycodes": [
    {
      "code": "KC_A",
      "label": "A",
      "category": "basic"
    }
  ],
  "count": 600
}
```

---

#### `lazyqmk help`
Displays help topics from help.toml (source of truth).

**Usage:**
```bash
lazyqmk help [topic]
```

**Arguments:**
- `topic`: Help topic name (e.g., `navigation`, `editing`)

**Example:**
```bash
# List all topics
lazyqmk help

# Show specific topic
lazyqmk help navigation
```

---

## End-to-End Test Strategy

### Test Fixtures
Create shared fixtures under `tests/fixtures/`:

```rust
// tests/fixtures/mod.rs
pub fn test_layout_basic(rows: usize, cols: usize) -> Layout { ... }
pub fn test_layout_with_tap_dances() -> Layout { ... }
pub fn test_geometry_basic(rows: usize, cols: usize) -> KeyboardGeometry { ... }
pub fn test_mapping_basic(rows: usize, cols: usize) -> VisualLayoutMapping { ... }
pub fn temp_config_with_qmk(qmk_path: Option<PathBuf>) -> Config { ... }
```

### Golden Testing
Store expected outputs under `tests/golden/`:

```
tests/golden/
├── keymap_basic.c
├── keymap_idle_effect_on.c
├── keymap_idle_effect_off.c
├── keymap_tap_dance_2way.c
├── keymap_tap_dance_3way.c
├── config_basic.h
├── config_idle_effect.h
└── config_rgb_timeout.h
```

Golden test helper:
```rust
fn assert_golden(actual: &str, golden_path: &str) {
    if env::var("UPDATE_GOLDEN").is_ok() {
        fs::write(golden_path, actual).unwrap();
    } else {
        let expected = fs::read_to_string(golden_path).unwrap();
        assert_eq!(normalize_output(actual), normalize_output(&expected));
    }
}

fn normalize_output(s: &str) -> String {
    // Strip timestamps, replace UUIDs with stable tokens
    s.lines()
        .map(|line| {
            if line.contains("Generated at:") {
                "// Generated at: <timestamp>"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
```

### Test Matrix

#### Validation Tests
```rust
#[test]
fn test_validate_valid_layout() {
    let layout = create_test_layout_file("valid.md");
    let output = Command::new("lazyqmk")
        .args(&["validate", "--layout", &layout, "--json"])
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(0));
    let result: ValidateResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result.valid);
}

#[test]
fn test_validate_invalid_keycode() {
    let layout = create_test_layout_with_invalid_keycode("invalid.md");
    let output = Command::new("lazyqmk")
        .args(&["validate", "--layout", &layout, "--json"])
        .output()
        .unwrap();
    
    assert_eq!(output.status.code(), Some(1));
    let result: ValidateResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("Invalid keycode")));
}
```

#### Generation Tests (Golden)
```rust
#[test]
fn test_generate_basic_keymap() {
    let layout = create_test_layout_file("basic.md");
    let out_dir = TempDir::new().unwrap();
    
    let status = Command::new("lazyqmk")
        .args(&[
            "generate",
            "--layout", &layout,
            "--qmk-path", QMK_PATH,
            "--out-dir", out_dir.path().to_str().unwrap(),
            "--deterministic"
        ])
        .status()
        .unwrap();
    
    assert!(status.success());
    
    let keymap = fs::read_to_string(out_dir.path().join("keymap.c")).unwrap();
    assert_golden(&keymap, "tests/golden/keymap_basic.c");
}

#[test]
fn test_generate_idle_effect_on() {
    let layout = create_test_layout_with_idle_effect(true);
    let out_dir = TempDir::new().unwrap();
    
    let status = Command::new("lazyqmk")
        .args(&[
            "generate",
            "--layout", &layout,
            "--qmk-path", QMK_PATH,
            "--out-dir", out_dir.path().to_str().unwrap(),
            "--deterministic"
        ])
        .status()
        .unwrap();
    
    assert!(status.success());
    
    let keymap = fs::read_to_string(out_dir.path().join("keymap.c")).unwrap();
    assert!(keymap.contains("idle_state_t"));
    assert!(keymap.contains("matrix_scan_user"));
    
    let config = fs::read_to_string(out_dir.path().join("config.h")).unwrap();
    assert!(config.contains("LQMK_IDLE_TIMEOUT_MS"));
    assert!(!config.contains("RGB_MATRIX_TIMEOUT"));
}
```

#### Tap Dance Tests
```rust
#[test]
fn test_tap_dance_add_validate_generate() {
    let layout = create_test_layout_file("tap_dance_test.md");
    
    // Add tap dance
    let status = Command::new("lazyqmk")
        .args(&[
            "tap-dance", "add",
            "--layout", &layout,
            "--name", "esc_caps",
            "--single", "KC_ESC",
            "--double", "KC_CAPS"
        ])
        .status()
        .unwrap();
    assert!(status.success());
    
    // Validate
    let output = Command::new("lazyqmk")
        .args(&["tap-dance", "validate", "--layout", &layout, "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    
    // Generate and check for TD macros
    let out_dir = TempDir::new().unwrap();
    Command::new("lazyqmk")
        .args(&[
            "generate",
            "--layout", &layout,
            "--qmk-path", QMK_PATH,
            "--out-dir", out_dir.path().to_str().unwrap()
        ])
        .status()
        .unwrap();
    
    let keymap = fs::read_to_string(out_dir.path().join("keymap.c")).unwrap();
    assert!(keymap.contains("enum tap_dance_ids"));
    assert!(keymap.contains("TD_ESC_CAPS"));
    assert!(keymap.contains("ACTION_TAP_DANCE_DOUBLE(KC_ESC, KC_CAPS)"));
}

#[test]
fn test_tap_dance_orphan_detection() {
    let layout = create_test_layout_with_orphaned_tap_dance("orphan.md");
    
    let output = Command::new("lazyqmk")
        .args(&["tap-dance", "validate", "--layout", &layout, "--json"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let result: ValidateResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result.warnings.iter().any(|w| w.message.contains("not referenced")));
}
```

#### Keycode Resolution Tests
```rust
#[test]
fn test_keycode_resolve_lt_with_uuid() {
    let layout = create_test_layout_with_layers("resolve_test.md");
    let layer_uuid = get_layer_uuid(&layout, 1);
    
    let output = Command::new("lazyqmk")
        .args(&[
            "keycode", "resolve",
            "--layout", &layout,
            "--expr", &format!("LT(@{}, KC_SPC)", layer_uuid),
            "--json"
        ])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let result: ResolveResult = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result.resolved, "LT(1, KC_SPC)");
}
```

#### Layer Refs Tests
```rust
#[test]
fn test_layer_refs_transparency_warning() {
    let layout = create_test_layout_with_layer_conflict("layer_refs.md");
    
    let output = Command::new("lazyqmk")
        .args(&["layer-refs", "--layout", &layout, "--json"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let result: LayerRefsResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result.layers.iter().any(|l| !l.warnings.is_empty()));
}
```

#### QMK Parsing Tests (Gated)
```rust
#[test]
#[ignore] // Requires QMK submodule
fn test_list_keyboards() {
    let output = Command::new("lazyqmk")
        .args(&["list-keyboards", "--qmk-path", QMK_PATH, "--json"])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let result: KeyboardsResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(result.keyboards.iter().any(|k| k.contains("crkbd")));
}

#[test]
#[ignore] // Requires QMK submodule
fn test_list_layouts_crkbd() {
    let output = Command::new("lazyqmk")
        .args(&[
            "list-layouts",
            "--qmk-path", QMK_PATH,
            "--keyboard", "crkbd/rev1",
            "--json"
        ])
        .output()
        .unwrap();
    
    assert!(output.status.success());
    let result: LayoutsResult = serde_json::from_slice(&output.stdout).unwrap();
    assert!(!result.layouts.is_empty());
}
```

## Implementation Phases

### Phase 1: Core Commands & Fixtures (Week 1)
**Priority: High**

**Tasks:**
1. Add clap subcommands structure to `src/main.rs`
2. Create `src/cli/` module with command handlers:
   - `validate.rs`
   - `generate.rs`
   - `inspect.rs`
   - `keycode.rs`
3. Create `tests/fixtures/mod.rs` with shared builders
4. Implement `validate` command with JSON output
5. Implement `generate` command with deterministic mode
6. Add `keycode resolve` command
7. Create golden test helper
8. Add initial E2E tests for validate and generate

**Deliverables:**
- Working `validate`, `generate`, `keycode resolve` commands
- Shared test fixtures
- Golden test framework
- 10+ E2E tests passing

---

### Phase 2: Tap Dance & Layer Utilities (Week 2)
**Priority: High**

**Tasks:**
1. Implement tap dance CRUD commands:
   - `tap-dance list`
   - `tap-dance add`
   - `tap-dance delete`
   - `tap-dance validate`
2. Implement `layer-refs` command
3. Add E2E tests for tap dance flows
4. Add golden tests for tap dance generation (2-way, 3-way)
5. Test orphan detection and validation

**Deliverables:**
- Complete tap dance CLI surface
- Layer refs analysis command
- 15+ tap dance E2E tests

---

### Phase 3: QMK Metadata & Categories (Week 3)
**Priority: Medium**

**Tasks:**
1. Implement QMK metadata commands:
   - `list-keyboards`
   - `list-layouts`
   - `geometry`
2. Implement category CRUD commands:
   - `category list/add/delete`
3. Gate QMK commands with feature flag or runtime checks
4. Add contract tests (marked `#[ignore]`)
5. Document running gated tests

**Deliverables:**
- QMK metadata commands
- Category management CLI
- Contract tests for QMK parsing

---

### Phase 4: Templates, Config, Utilities (Week 4)
**Priority: Low**

**Tasks:**
1. Implement template commands
2. Implement config commands (`show`, `set`)
3. Implement utility commands (`keycodes`, `help`)
4. Add E2E tests for template round-trip
5. Add E2E tests for config management
6. Write comprehensive documentation

**Deliverables:**
- Complete CLI surface
- Template and config management
- Full E2E test coverage
- User documentation

---

## Testing Documentation

Add `docs/TESTING.md`:

```markdown
# Testing Guide

## Running Tests

### Fast Unit & Integration Tests
```bash
cargo test --tests
```

### All Tests (including slow QMK-dependent)
```bash
cargo test --features qmk -- --ignored
```

### Updating Golden Files
```bash
UPDATE_GOLDEN=1 cargo test --tests
```

### CLI E2E Tests Only
```bash
cargo test --test cli_e2e_tests
```

## Test Structure

- `tests/fixtures/` - Shared test fixtures and builders
- `tests/golden/` - Expected outputs for golden tests
- `tests/cli_*.rs` - CLI command E2E tests
- `tests/*_tests.rs` - Existing integration tests

## Writing New Tests

### Golden Tests
1. Create fixture layout file
2. Run command with `--deterministic`
3. Compare output to `tests/golden/<file>`
4. Update goldens with `UPDATE_GOLDEN=1`

### CLI E2E Tests
1. Use `Command::new("lazyqmk")` with args
2. Assert exit codes
3. Parse JSON output and assert structure
4. Use temp dirs for file operations

### Fixtures
Use shared builders from `tests/fixtures/`:
```rust
use fixtures::{test_layout_basic, test_geometry_basic};

let layout = test_layout_basic(4, 6);
let geometry = test_geometry_basic(4, 6);
```
```

## CI/CD Integration

Update `.github/workflows/` (if exists) or document CI requirements:

```yaml
# Example CI job
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
      with:
        submodules: true
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run fast tests
      run: cargo test --tests
    - name: Run clippy
      run: cargo clippy --all-features -- -D warnings
    # Optional: QMK tests
    - name: Run QMK integration tests
      run: cargo test --features qmk -- --ignored
      continue-on-error: true
```

## Success Criteria

1. **CLI Surface**: All priority commands implemented with proper exit codes and JSON output
2. **E2E Coverage**: 50+ E2E tests covering major workflows
3. **Golden Tests**: Deterministic firmware generation with snapshot regression testing
4. **CI Integration**: Fast test suite runs in <2 minutes; full suite (with QMK) in <5 minutes
5. **Documentation**: Complete CLI reference and testing guide
6. **Developer Experience**: `lazyqmk --help` provides clear command documentation

## Non-Goals

- **Not replacing TUI**: CLI supplements, doesn't replace interactive editing
- **Not a build system**: Wraps existing QMK tooling, doesn't reimplement
- **Not adding complexity**: Commands map 1:1 to existing service layer functions

## Dependencies

- `clap` 4.5 - CLI argument parsing (already in use)
- `serde_json` - JSON output (already in use)
- `tempfile` - Test temp directories (already in use)

No new external dependencies required.

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Golden tests brittle due to timestamp churn | Medium | Implement normalization helper; document deterministic mode |
| QMK submodule dependency breaks CI | Low | Gate with feature flag; provide clear skip messaging |
| CLI/TUI behavior drift | Medium | Share service layer code; include CLI in integration tests |
| Complex argument parsing | Low | Use clap's derive API; comprehensive `--help` text |

## Future Extensions

- `lazyqmk build` - Full QMK compilation wrapper
- `lazyqmk init` - Non-interactive onboarding for CI
- `lazyqmk export` - Export to VIA JSON or other formats
- `lazyqmk lint` - Static analysis and best practices checker
- `lazyqmk diff` - Compare two layouts
