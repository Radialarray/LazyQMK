# QMK Keyboard Structure Analysis

## Problem Statement

The current "create new layout" picker fails with errors like:
```
Failed to parse keyboard info: info.json not found for keyboard 'lupkeyboards/lupsuper16v3' 
under /Users/user/dev/keyboard-configurator/qmk_firmware/keyboards
```

The error reveals two critical issues:
1. **Typo**: `lupkeyboards/lupsuper16v3` instead of `1upkeyboards/1upsuper16v3` (likely OCR/copy error)
2. **Fragile Path Logic**: The system doesn't validate or discover actual keyboard structures

## QMK Firmware Structure Overview

### Key Statistics
- **Total Keyboards**: ~3,699 compilable keyboard targets (as of 2025-12)
- **Structure Depth**: From 0 levels (single folder) to 5+ levels of nesting
- **Configuration Files**: `info.json` (parent), `keyboard.json` (variant), or both

### Keyboard Organization Patterns

QMK keyboards follow several organizational patterns:

#### Pattern 1: Single Directory with info.json
```
keyboards/crkbd/
  ├── info.json         # Contains all layouts and config
  ├── crkbd.c
  ├── keymaps/
  └── [subdirs for variants]
```
Example: `keyboards/crkbd/info.json` defines the base keyboard.

#### Pattern 2: Variant Subdirectories with keyboard.json
```
keyboards/1upkeyboards/pi50/
  ├── info.json         # Parent config (encoder, matrix pins)
  ├── grid/
  │   └── keyboard.json # Variant-specific layouts
  └── mit/
      └── keyboard.json # Different layout variant
```
Compilable targets: `1upkeyboards/pi50/grid`, `1upkeyboards/pi50/mit`

#### Pattern 3: Deep Nesting (5 levels)
```
keyboards/mechlovin/adelais/standard_led/arm/rev4/stm32f303/
```
Each level may have configuration files that inherit/override parent settings.

#### Pattern 4: Mixed Configuration
```
keyboards/1upkeyboards/1upsuper16v3/
  └── keyboard.json     # Has layouts but no info.json
```
Some keyboards only have `keyboard.json` with complete configuration.

### Critical Observations

1. **QMK CLI is the Source of Truth**: `qmk list-keyboards` returns ONLY compilable targets
   - Returns: `1upkeyboards/1upsuper16v3` (correct, compilable)
   - Does NOT return: `1upkeyboards/super16v3` (doesn't exist)
   - Does NOT return: parent paths that aren't compilable

2. **info.json vs keyboard.json**:
   - `info.json`: Parent/base keyboard configuration
   - `keyboard.json`: Variant-specific configuration
   - Layouts can be in either file
   - Some keyboards merge both (parent for encoder config, variant for layouts)

3. **File Location Variations**:
   - Base keyboard: `keyboards/crkbd/info.json`
   - Variant only: `keyboards/1upkeyboards/pi50/grid/keyboard.json`
   - Parent + Variant: Both files, with variant overriding/extending parent
   - No info.json: `keyboards/1upkeyboards/1upsuper16v3/keyboard.json` only

4. **Path Complexity**:
   - Simple: `crkbd` (0 slashes)
   - Moderate: `1upkeyboards/1upsuper16v3` (1 slash)
   - Complex: `splitkb/aurora/corne/rev1` (3 slashes)
   - Extreme: `mechlovin/adelais/standard_led/arm/rev4/stm32f303` (5 slashes)

## Current Implementation Analysis

### scan_keyboards() Function
Location: `src/parser/keyboard_json.rs:152-188`

**Current Approach**:
```rust
pub fn scan_keyboards(qmk_path: &Path) -> Result<Vec<String>> {
    let output = Command::new("qmk")
        .arg("list-keyboards")
        .current_dir(qmk_path)
        .output()?;
    
    let keyboards: Vec<String> = stdout
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    
    keyboards.sort();
    Ok(keyboards)
}
```

**✅ Strengths**:
- Uses QMK CLI as source of truth
- Returns only compilable targets
- Simple, reliable implementation

**❌ Issues**:
- None! This function is actually correct.

### parse_keyboard_info_json() Function
Location: `src/parser/keyboard_json.rs:234-315`

**Current Approach**:
```rust
pub fn parse_keyboard_info_json(qmk_path: &Path, keyboard: &str) -> Result<QmkInfoJson> {
    let keyboard_dir = keyboards_dir.join(keyboard);
    let info_json_path = keyboard_dir.join("info.json");
    let keyboard_json_path = keyboard_dir.join("keyboard.json");
    
    // 1. Try info.json directly
    if info_json_path.exists() { ... }
    
    // 2. Try keyboard.json
    if keyboard_json_path.exists() { ... }
    
    // 3. Fall back to parent directory
    if let Some((base, _variant)) = keyboard.rsplit_once('/') { ... }
    
    // 4. Error: info.json not found
    anyhow::bail!("info.json not found for keyboard '{}' ...", keyboard);
}
```

**✅ Strengths**:
- Handles multiple configuration patterns
- Merges parent + variant configs
- Falls back to parent directory

**⚠️ Weaknesses**:
1. **Error message assumes info.json must exist**: But some keyboards only have `keyboard.json`
2. **No validation of keyboard name**: Accepts any string, even typos like `lupkeyboards/lupsuper16v3`
3. **Fallback logic is fragile**: Assumes parent always has `info.json`
4. **No helpful debugging**: Error doesn't suggest valid alternatives or corrections

### Onboarding Wizard
Location: `src/tui/onboarding_wizard.rs`

**Current Approach**:
```rust
// Step 1: Scan keyboards
let keyboards = scan_keyboards(&qmk_path)?;  // ✅ Correct

// Step 2: User selects keyboard
let keyboard = filtered_keyboards[self.keyboard_selected_index].clone();

// Step 3: Parse keyboard info
match parse_keyboard_info_json(&qmk_path, &keyboard) {
    Ok(info) => { ... }
    Err(e) => {
        self.error_message = Some(format!("Failed to parse keyboard info: {e}"));
    }
}
```

**✅ Strengths**:
- Keyboard list comes from QMK CLI (source of truth)
- User selects from valid options
- Filter function for searching

**❌ Issues**:
- Error handling shows cryptic parser errors to user
- No validation that selected keyboard actually exists in filesystem
- No recovery mechanism if info.json parsing fails

## Root Cause Analysis

The error `lupkeyboards/lupsuper16v3` suggests:

1. **User Input Error**: Typo or clipboard issue (OCR error: `1` → `l`)
2. **Passed Validation**: System didn't validate keyboard name against QMK CLI list
3. **Parser Failed**: `parse_keyboard_info_json()` couldn't find config files

The actual failure flow:
```
User enters: "lupkeyboards/lupsuper16v3"
              ↓
wizard.next_step() saves to state.inputs["keyboard"]
              ↓ 
parse_keyboard_info_json() called with invalid name
              ↓
Looks for: qmk_firmware/keyboards/lupkeyboards/lupsuper16v3/info.json
              ↓
File doesn't exist → Error: "info.json not found"
```

**Why validation failed**:
- Wizard keyboard selection loads from `qmk list-keyboards` ✅
- BUT: If keyboard name comes from elsewhere (config file, command line, direct input), no validation occurs
- The filter only narrows the displayed list; it doesn't validate against the full list

## Edge Cases to Handle

### 1. Keyboard Name Variations
- Correct: `1upkeyboards/1upsuper16v3`
- Typo: `lupkeyboards/lupsuper16v3` (OCR error)
- Typo: `1upkeyboards/super16v3` (name mismatch)
- Parent: `1upkeyboards` (not compilable)

### 2. Configuration File Combinations
- ✅ info.json only: `crkbd/info.json`
- ✅ keyboard.json only: `1upkeyboards/1upsuper16v3/keyboard.json`
- ✅ Both files: `1upkeyboards/pi50/info.json` + `grid/keyboard.json`
- ✅ Parent + variant: `splitkb/aurora/info.json` + `corne/rev1/keyboard.json`
- ❌ Neither file: Error case

### 3. Layout Definition Locations
- In `info.json` at keyboard root
- In `keyboard.json` in variant directory
- Inherited from parent `info.json`
- Split between parent (encoder config) and variant (layouts)

### 4. Nesting Depth
- 0 levels: `planck` → `keyboards/planck/info.json`
- 1 level: `crkbd` → `keyboards/crkbd/info.json`
- 2 levels: `1upkeyboards/pi50/grid` → `keyboards/1upkeyboards/pi50/grid/keyboard.json`
- 5+ levels: `mechlovin/adelais/standard_led/arm/rev4/stm32f303`

### 5. User Error Scenarios
- Keyboard name with typo
- Keyboard name from old QMK version (renamed/moved)
- Keyboard name with wrong casing (case sensitivity varies by OS)
- Copy-paste error (extra spaces, wrong characters)

## Recommendations

### Immediate Fixes

1. **Validation Layer**:
   - Create `validate_keyboard_name()` function
   - Check against `qmk list-keyboards` output
   - Suggest corrections for typos (fuzzy matching)

2. **Better Error Messages**:
   - Instead of: `"info.json not found for keyboard '...'"`
   - Show: `"Keyboard 'lupkeyboards/lupsuper16v3' not found. Did you mean '1upkeyboards/1upsuper16v3'?"`

3. **Keyboard Discovery**:
   - Create `discover_keyboard_config()` function
   - Try multiple paths systematically
   - Return detailed result (which files found, which paths tried)

4. **Parser Robustness**:
   - Accept keyboards with ONLY `keyboard.json`
   - Gracefully handle missing encoder config
   - Don't require `info.json` to exist

### Long-term Improvements

1. **Caching Layer**:
   - Cache `qmk list-keyboards` output
   - Build keyboard → config file mapping
   - Invalidate cache when QMK firmware updates

2. **Better UX**:
   - Show keyboard structure preview before selection
   - Display available layouts count in keyboard list
   - Warn if keyboard has no layouts defined

3. **Fuzzy Matching**:
   - Levenshtein distance for typo detection
   - Suggest similar keyboard names
   - Handle OCR-like errors (`1` ↔ `l`, `0` ↔ `O`)

4. **Configuration File Analysis**:
   - Build complete keyboard metadata on demand
   - Show which config files exist for each keyboard
   - Detect inheritance patterns

## Testing Strategy

### Unit Tests
1. Test `validate_keyboard_name()` with valid/invalid names
2. Test `discover_keyboard_config()` with all file patterns
3. Test fuzzy matching with common typos
4. Test parser with all configuration file combinations

### Integration Tests
1. Test against real QMK firmware submodule
2. Sample keyboards from each pattern (simple, nested, variant)
3. Test error recovery flow
4. Test keyboard selection wizard end-to-end

### Edge Case Tests
1. Non-existent keyboard names
2. Keyboards with only `keyboard.json`
3. Deep nesting (5+ levels)
4. Parent keyboards without compilable target

## Impact Assessment

### Current Bug Severity
- **Severity**: High (blocks layout creation)
- **Frequency**: Medium (affects users with typos or specific keyboards)
- **Workaround**: None (requires code fix)

### Proposed Changes Risk
- **Risk Level**: Low-Medium
- **Risk Factors**:
  - Parser changes could affect layout loading
  - Validation might be too strict
  - Error messages might confuse users
- **Mitigation**:
  - Comprehensive tests with real QMK firmware
  - Backward compatibility with existing layouts
  - Graceful fallbacks

## Conclusion

The root cause is **insufficient validation** combined with **fragile error handling**. The fix requires:

1. ✅ Keep `scan_keyboards()` as-is (already correct)
2. ✅ Add validation layer before calling parser
3. ✅ Improve `parse_keyboard_info_json()` error messages
4. ✅ Add keyboard config discovery function
5. ✅ Handle keyboards with only `keyboard.json`
6. ✅ Add fuzzy matching for typo suggestions

This will make the system robust against user errors, OCR issues, and QMK firmware structure variations.
