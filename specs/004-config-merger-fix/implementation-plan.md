# Implementation Plan: Config Merger

**Feature:** 004-config-merger-fix  
**Created:** 2025-11-26

## Overview

Step-by-step implementation checklist for the config merger feature that fixes QMK compilation issues by automatically merging keyboard configurations and filtering deprecated options.

## Phase 1: Create ConfigMerger Module

### Step 1.1: Create File Structure

- [ ] Create `src/firmware/config_merger.rs`
- [ ] Add module to `src/firmware/mod.rs`
- [ ] Add dependencies to `Cargo.toml` if needed (regex, serde_json)

### Step 1.2: Define Core Struct

```rust
pub struct ConfigMerger {
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
}

impl ConfigMerger {
    pub fn new(qmk_path: PathBuf, keyboard: String, keymap: String) -> Self {
        Self {
            qmk_path,
            keyboard,
            keymap,
        }
    }
}
```

**Tasks:**
- [ ] Implement ConfigMerger struct
- [ ] Implement `new()` constructor
- [ ] Add basic struct tests

### Step 1.3: Implement Variant Detection

```rust
impl ConfigMerger {
    /// Detects if keyboard uses variant subdirectories
    pub fn detect_variants(&self) -> Result<Vec<String>>;
    
    /// Extracts variant from keyboard path
    /// e.g., "keebart/corne_choc_pro/standard" -> Some("standard")
    pub fn extract_variant(&self) -> Option<String>;
    
    /// Gets base keyboard path without variant
    /// e.g., "keebart/corne_choc_pro/standard" -> "keebart/corne_choc_pro"
    pub fn get_base_keyboard_path(&self) -> String;
}
```

**Implementation Details:**
- Split keyboard path by `/`
- Check if last component is a subdirectory with keyboard.json
- Verify variant directory exists before treating as variant

**Tasks:**
- [ ] Implement `detect_variants()`
- [ ] Implement `extract_variant()`
- [ ] Implement `get_base_keyboard_path()`
- [ ] Add unit tests for variant detection
  - [ ] Test with variant path: "keebart/corne_choc_pro/standard"
  - [ ] Test without variant: "keebart/corne_choc_pro"
  - [ ] Test with deep nesting: "vendor/keyboard/variant1/variant2"

### Step 1.4: Implement File Reading

```rust
impl ConfigMerger {
    /// Reads keyboard's base config.h
    pub fn read_keyboard_config_h(&self) -> Result<String>;
    
    /// Reads variant-specific keyboard.json
    pub fn read_variant_keyboard_json(&self) -> Result<serde_json::Value>;
    
    /// Reads existing keymap rules.mk if present
    pub fn read_keymap_rules_mk(&self) -> Result<Option<String>>;
}
```

**File Paths:**
- config.h: `{qmk_path}/keyboards/{base_keyboard}/config.h`
- keyboard.json: `{qmk_path}/keyboards/{keyboard}/keyboard.json` (includes variant)
- rules.mk: `{qmk_path}/keyboards/{base_keyboard}/keymaps/{keymap}/rules.mk`

**Tasks:**
- [ ] Implement `read_keyboard_config_h()`
- [ ] Implement `read_variant_keyboard_json()`
- [ ] Implement `read_keymap_rules_mk()`
- [ ] Add error handling for missing files
- [ ] Add unit tests with mock file system

### Step 1.5: Implement RGB LED Count Extraction

```rust
impl ConfigMerger {
    /// Extracts RGB LED count from keyboard.json
    /// Returns total LED count from split_count array
    pub fn extract_rgb_led_count(&self, json: &serde_json::Value) -> Option<usize>;
}
```

**Logic:**
```rust
// Parse: {"rgb_matrix": {"split_count": [23, 23]}}
if let Some(split_count) = json["rgb_matrix"]["split_count"].as_array() {
    let left = split_count[0].as_u64().unwrap_or(0);
    let right = split_count[1].as_u64().unwrap_or(0);
    Some((left + right) as usize)
} else if let Some(led_count) = json["rgb_matrix"]["led_count"].as_u64() {
    Some(led_count as usize)
} else {
    None
}
```

**Tasks:**
- [ ] Implement `extract_rgb_led_count()`
- [ ] Handle split_count array format
- [ ] Handle led_count single value format
- [ ] Add unit tests
  - [ ] Test split_count [23, 23] → 46
  - [ ] Test split_count [21, 21] → 42
  - [ ] Test led_count 46 → 46
  - [ ] Test missing config → None

### Step 1.6: Implement Deprecated Option Filtering

```rust
impl ConfigMerger {
    /// Filters deprecated VIAL options from config.h
    pub fn filter_deprecated_options(&self, config_content: String) -> String;
    
    /// Sanitizes rules.mk by removing deprecated options
    pub fn sanitize_rules_mk(&self, rules_content: String) -> String;
}
```

**Deprecated Patterns (config.h):**
```rust
const DEPRECATED_CONFIG_PATTERNS: &[&str] = &[
    r"#define\s+VIAL_KEYBOARD_UID\s+\{[^}]+\}",
    r"#define\s+VIAL_UNLOCK_COMBO_ROWS\s+\{[^}]+\}",
    r"#define\s+VIAL_UNLOCK_COMBO_COLS\s+\{[^}]+\}",
];
```

**Deprecated Patterns (rules.mk):**
```rust
const DEPRECATED_RULES_PATTERNS: &[&str] = &[
    r"^\s*VIAL_ENABLE\s*=\s*yes\s*$",
];
```

**Implementation:**
- Use regex crate for pattern matching
- Process line-by-line to preserve formatting
- Keep comments and blank lines
- Log filtered options for debugging

**Tasks:**
- [ ] Implement `filter_deprecated_options()`
- [ ] Implement `sanitize_rules_mk()`
- [ ] Add regex dependency to Cargo.toml
- [ ] Add unit tests
  - [ ] Test VIAL_KEYBOARD_UID removal
  - [ ] Test VIAL_ENABLE removal
  - [ ] Test VIA_ENABLE preservation
  - [ ] Test ENCODER_MAP_ENABLE preservation
  - [ ] Test comment preservation

### Step 1.7: Implement Config Merging

```rust
impl ConfigMerger {
    /// Merges filtered base config with TUI-generated config
    pub fn merge_config_h(&self, base_config: String, tui_config: String) -> Result<String>;
    
    /// Generates clean rules.mk from base rules
    pub fn generate_rules_mk(&self, base_rules: Option<String>) -> Result<String>;
}
```

**merge_config_h() Structure:**
```
// TUI header
// Generated by keyboard_tui
// ...

#pragma once

// TUI additions (RGB LED count, etc.)

// Filtered keyboard base config
```

**generate_rules_mk() Structure:**
```
# Generated by keyboard_tui
# ...

# Sanitized keyboard rules
ENCODER_MAP_ENABLE = yes
CAPS_WORD_ENABLE = yes
...

# Add keymap-specific rules here
```

**Tasks:**
- [ ] Implement `merge_config_h()`
- [ ] Implement `generate_rules_mk()`
- [ ] Ensure proper section ordering
- [ ] Avoid duplicate definitions
- [ ] Add unit tests
  - [ ] Test merge without duplicates
  - [ ] Test pragma once preservation
  - [ ] Test section ordering

### Step 1.8: Add Module Tests

Create comprehensive test suite in `config_merger.rs`:

**Tasks:**
- [ ] Create test fixtures directory with sample files
- [ ] Test variant detection edge cases
- [ ] Test RGB LED count calculation
- [ ] Test deprecated option filtering
- [ ] Test config merging
- [ ] Test error handling for missing files
- [ ] Add integration test with real QMK structure

---

## Phase 2: Update FirmwareGenerator

### Step 2.1: Import ConfigMerger

**File:** `src/firmware/generator.rs`

**Tasks:**
- [ ] Add `use crate::firmware::config_merger::ConfigMerger;` at top
- [ ] Verify Config import for qmk_path access

### Step 2.2: Update generate_merged_config_h()

**Location:** Line 380-393 in `generator.rs`

**New Implementation:**
```rust
fn generate_merged_config_h(&self) -> Result<String> {
    let qmk_path = self.config.paths.qmk_firmware
        .as_ref()
        .context("QMK firmware path not configured")?;
    
    let merger = ConfigMerger::new(
        qmk_path.clone(),
        self.config.build.keyboard.clone(),
        self.config.build.keymap.clone(),
    );
    
    // Read and filter keyboard base config
    let base_config = merger.read_keyboard_config_h()
        .unwrap_or_default();
    let clean_base = merger.filter_deprecated_options(base_config);
    
    // Extract RGB LED count from variant
    let rgb_led_count = if let Ok(variant_json) = merger.read_variant_keyboard_json() {
        merger.extract_rgb_led_count(&variant_json)
    } else {
        None
    };
    
    // Generate TUI-specific config
    let mut tui_config = String::new();
    tui_config.push_str("// Generated by keyboard_tui\n");
    tui_config.push_str(&format!("// Layout: {}\n", self.layout.metadata.name));
    tui_config.push_str(&format!("// Generated: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
    tui_config.push_str("\n");
    
    // Add RGB_MATRIX_LED_COUNT if detected
    if let Some(led_count) = rgb_led_count {
        tui_config.push_str(&format!("#define RGB_MATRIX_LED_COUNT {}\n", led_count));
        tui_config.push_str("\n");
    }
    
    tui_config.push_str("// Add keymap-specific configuration here\n");
    
    // Merge configs
    merger.merge_config_h(clean_base, tui_config)
}
```

**Tasks:**
- [ ] Replace existing `generate_merged_config_h()` implementation
- [ ] Test with and without RGB matrix keyboards
- [ ] Test with and without variants
- [ ] Verify backward compatibility

### Step 2.3: Add generate_sanitized_rules_mk()

**New Method:**
```rust
fn generate_sanitized_rules_mk(&self) -> Result<String> {
    let qmk_path = self.config.paths.qmk_firmware
        .as_ref()
        .context("QMK firmware path not configured")?;
    
    let merger = ConfigMerger::new(
        qmk_path.clone(),
        self.config.build.keyboard.clone(),
        self.config.build.keymap.clone(),
    );
    
    // Read existing keymap rules if any
    let existing_rules = merger.read_keymap_rules_mk()
        .ok()
        .flatten();
    
    // Generate clean rules.mk
    merger.generate_rules_mk(existing_rules)
}
```

**Tasks:**
- [ ] Add method to FirmwareGenerator impl block
- [ ] Handle missing rules.mk gracefully
- [ ] Add unit test

### Step 2.4: Update generate() Method

**Location:** Around line 50-67 in `generator.rs`

**Changes:**
- Change return type from `(String, String, String)` to `(String, String, String, String)`
- Add rules.mk generation step
- Update write_file_to_both calls

**New Implementation:**
```rust
pub fn generate(&self) -> Result<(String, String, String, String)> {
    let timestamp_dir = self.create_timestamped_output_dir()?;
    
    // Generate config.h (with merging)
    let config_h = self.generate_merged_config_h()?;
    let config_h_path = self.write_file_to_both(&timestamp_dir, "config.h", &config_h)?;
    
    // Generate rules.mk (NEW - with sanitization)
    let rules_mk = self.generate_sanitized_rules_mk()?;
    let rules_mk_path = self.write_file_to_both(&timestamp_dir, "rules.mk", &rules_mk)?;
    
    // Generate keymap.c
    let keymap_c = self.generate_keymap_c()?;
    let keymap_path = self.write_file_to_both(&timestamp_dir, "keymap.c", &keymap_c)?;
    
    // Generate vial.json
    let vial_json = self.generate_vial_json()?;
    let vial_path = self.write_file_to_both(&timestamp_dir, "vial.json", &vial_json)?;
    
    Ok((keymap_path, vial_path, config_h_path, rules_mk_path))
}
```

**Tasks:**
- [ ] Update return type signature
- [ ] Add rules_mk generation
- [ ] Update return statement with 4 paths
- [ ] Update all callers of generate()
- [ ] Update tests

---

## Phase 3: Module Integration

### Step 3.1: Update Firmware Module

**File:** `src/firmware/mod.rs`

**Tasks:**
- [ ] Add `pub mod config_merger;`
- [ ] Add `pub use config_merger::ConfigMerger;`

### Step 3.2: Update TUI Call Sites

Find all places that call `generator.generate()`:

**Likely Locations:**
- `src/tui/mod.rs` (firmware generation handler)
- Any firmware generation UI components

**Tasks:**
- [ ] Find all callers of `generate()`
- [ ] Update tuple destructuring from 3 to 4 elements
- [ ] Update display messages to show rules.mk generation
- [ ] Update success messages to include all 4 files

**Example Update:**
```rust
// OLD:
let (keymap_path, vial_path, config_h_path) = generator.generate()?;

// NEW:
let (keymap_path, vial_path, config_h_path, rules_mk_path) = generator.generate()?;
```

### Step 3.3: Verify Builder Integration

**File:** `src/firmware/builder.rs`

**Verification:**
- [ ] Check that `keyboard` parameter includes variant
- [ ] Verify build command: `make {keyboard}:{keymap}`
- [ ] Ensure variant is in path (e.g., "keebart/corne_choc_pro/standard")
- [ ] No code changes needed if variant already in config

---

## Phase 4: Testing

### Step 4.1: Unit Tests

**ConfigMerger Tests:**
- [ ] Test variant detection with various path formats
- [ ] Test RGB LED count extraction from JSON
- [ ] Test deprecated option filtering (config.h)
- [ ] Test deprecated option filtering (rules.mk)
- [ ] Test config merging without duplicates
- [ ] Test error handling for missing files

**FirmwareGenerator Tests:**
- [ ] Test generate() returns 4 paths
- [ ] Test config.h includes RGB_MATRIX_LED_COUNT
- [ ] Test config.h excludes VIAL_KEYBOARD_UID
- [ ] Test rules.mk excludes VIAL_ENABLE
- [ ] Test rules.mk includes valid options

### Step 4.2: Integration Tests

**Test with Real Keyboard:**
- [ ] Use keebart/corne_choc_pro/standard
- [ ] Generate all files via TUI
- [ ] Inspect generated config.h
  - [ ] Has `#define RGB_MATRIX_LED_COUNT 46`
  - [ ] Lacks `#define VIAL_KEYBOARD_UID`
- [ ] Inspect generated rules.mk
  - [ ] Lacks `VIAL_ENABLE = yes`
  - [ ] Has `ENCODER_MAP_ENABLE = yes`

### Step 4.3: Compilation Tests

**Manual Testing:**
- [ ] Generate firmware via TUI (Ctrl+G)
- [ ] Build firmware via TUI (Ctrl+B)
- [ ] Verify build log shows no deprecated option warnings
- [ ] Verify compilation succeeds
- [ ] Verify firmware file is created
- [ ] Test firmware on physical keyboard (if available)

**Automated Testing:**
- [ ] Add CI test that compiles generated firmware
- [ ] Mock QMK environment in tests
- [ ] Verify no compilation errors

---

## Phase 5: Documentation & Cleanup

### Step 5.1: Code Documentation

- [ ] Add comprehensive rustdoc to ConfigMerger
- [ ] Document each public method
- [ ] Add usage examples in module docs
- [ ] Document deprecated option patterns

### Step 5.2: Update Project Docs

- [ ] Update README.md if needed
- [ ] Add to CHANGELOG.md
- [ ] Update architecture docs if they exist

### Step 5.3: Clean Up Test Artifacts

- [ ] Remove debug print statements
- [ ] Clean up test fixture files
- [ ] Ensure all tests pass cleanly

---

## Rollout Plan

### Pre-Merge Checklist

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Manual compilation test succeeds
- [ ] Code reviewed
- [ ] Documentation complete
- [ ] No breaking changes to public API

### Merge Strategy

1. Merge feature branch to main
2. Tag release if appropriate
3. Monitor for issues
4. Update users via changelog

---

## Rollback Plan

If issues arise after merge:

1. **Minor Issues:** Hot-fix on main branch
2. **Major Issues:** Revert merge commit
3. **Critical Issues:** Release patch with ConfigMerger disabled

**Disable Mechanism:**
Add feature flag to fall back to old behavior:
```rust
if config.experimental.use_config_merger {
    // New behavior
} else {
    // Old behavior
}
```

---

## Success Metrics

- [ ] Zero compilation errors for variant keyboards
- [ ] No deprecated option warnings in build logs
- [ ] All existing keyboards still work
- [ ] User feedback positive
- [ ] No new bug reports related to config generation

---

## Timeline Estimate

- **Phase 1:** 4-6 hours (ConfigMerger module)
- **Phase 2:** 2-3 hours (FirmwareGenerator updates)
- **Phase 3:** 1-2 hours (Integration)
- **Phase 4:** 3-4 hours (Testing)
- **Phase 5:** 1-2 hours (Documentation)

**Total:** ~11-17 hours

---

## Notes

- Keep ConfigMerger module independent for reusability
- Maintain backward compatibility with non-variant keyboards
- Log filtered options for user transparency
- Consider adding user notification when deprecated options are found
