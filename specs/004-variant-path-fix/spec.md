# Spec: Variant Path Detection Fix

**Feature ID:** 004-variant-path-fix  
**Status:** Planning  
**Created:** 2025-11-26  
**Updated:** 2025-11-26  
**Priority:** High

## Problem Statement

The TUI fails to compile QMK firmware for keyboards with variant structures because it doesn't target the correct variant path during builds.

### Error Messages

```
quantum/rgb_matrix/rgb_matrix_types.h:74:23: error: 'RGB_MATRIX_LED_COUNT' undeclared
⚠ keebart/corne_choc_pro: Build marker "keyboard.json" not found.
make: *** [keebart/corne_choc_pro:default] Error 1
```

### Root Cause

**Current Behavior:**
- User configures keyboard as: `keebart/corne_choc_pro`
- TUI builds with: `make keebart/corne_choc_pro:default`
- QMK looks for config in base directory
- Keyboard actually has variants: `standard/` and `mini/`
- Each variant has its own `keyboard.json` with RGB config
- QMK can't find variant-specific config → RGB_MATRIX_LED_COUNT undefined

**What Should Happen:**
- Detect that keyboard has variants
- Determine correct variant path: `keebart/corne_choc_pro/standard`
- Build with: `make keebart/corne_choc_pro/standard:default`
- QMK loads variant's `keyboard.json` → RGB config loaded automatically

## Solution Overview

**Fix the build target to include the variant path, letting QMK's native build system handle all configuration.**

Two components:
1. **Variant Detection** - Detect when keyboard uses variant structure
2. **Build Path Resolution** - Use full variant path in make command

### Why This Approach

✅ **Simple** - Minimal code changes  
✅ **Leverages QMK** - Uses QMK's existing variant system  
✅ **Robust** - Won't break when QMK changes  
✅ **Complete** - Handles ALL variant-specific config (RGB, encoders, etc.)  
✅ **Fast** - 3-4 hours vs 11-17 hours for config merger  

## Technical Design

### Component 1: Enhanced Variant Detection

**File:** `src/config.rs`

**Current Implementation:**
```rust
// Config just stores keyboard name as-is
pub struct BuildConfig {
    pub keyboard: String,
    pub layout: String,
    pub keymap: String,
}
```

**New Method:**
```rust
impl Config {
    /// Determines the full keyboard path including variant if present
    /// 
    /// Examples:
    /// - "keebart/corne_choc_pro" → "keebart/corne_choc_pro/standard" (if standard/ has keyboard.json)
    /// - "splitkb/aurora/corne" → "splitkb/aurora/corne/rev1" (if rev1/ has info.json)
    /// - "crkbd" → "crkbd" (no variants)
    pub fn determine_keyboard_variant(&self) -> Result<String> {
        let qmk_path = self.paths.qmk_firmware.as_ref()
            .context("QMK firmware path not configured")?;
        
        let keyboard_base = qmk_path.join("keyboards").join(&self.build.keyboard);
        
        // Check if current path is already a variant (has keyboard.json or info.json)
        if keyboard_base.join("keyboard.json").exists() || 
           keyboard_base.join("info.json").exists() {
            return Ok(self.build.keyboard.clone());
        }
        
        // Check for variant subdirectories
        if let Ok(entries) = std::fs::read_dir(&keyboard_base) {
            let mut variants = Vec::new();
            
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    let has_keyboard_json = entry.path().join("keyboard.json").exists();
                    let has_info_json = entry.path().join("info.json").exists();
                    
                    if has_keyboard_json || has_info_json {
                        variants.push(entry.file_name().to_string_lossy().to_string());
                    }
                }
            }
            
            if !variants.is_empty() {
                // Prefer "standard" if it exists
                if variants.contains(&"standard".to_string()) {
                    return Ok(format!("{}/standard", self.build.keyboard));
                }
                
                // Otherwise use first variant found
                return Ok(format!("{}/{}", self.build.keyboard, variants[0]));
            }
        }
        
        // No variants found, use base keyboard
        Ok(self.build.keyboard.clone())
    }
}
```

**Detection Logic:**
1. Check if configured path already points to variant (has keyboard.json/info.json)
2. If not, scan subdirectories for variant markers
3. Prefer "standard" variant if multiple exist
4. Fall back to base keyboard if no variants

### Component 2: Build Path Resolution

**File:** `src/firmware/builder.rs`

**Current Implementation:**
```rust
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
) -> Result<()> {
    let make_target = format!("{keyboard}:{keymap}");
    // ...
}
```

**Updated Implementation:**
```rust
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
    config: Config,  // NEW: Pass config to access variant detection
) -> Result<()> {
    // Determine full keyboard path with variant
    let full_keyboard_path = config.determine_keyboard_variant()
        .unwrap_or_else(|_| keyboard.clone());
    
    let make_target = format!("{}:{}", full_keyboard_path, keymap);
    
    sender.send(BuildMessage::Log {
        level: LogLevel::Info,
        message: format!("Building: {}", make_target),
    }).ok();
    
    // Rest of build logic unchanged
    // ...
}
```

**Update Call Site in `BuildState`:**
```rust
impl BuildState {
    pub fn start_build(
        &mut self,
        qmk_path: PathBuf,
        keyboard: String,
        keymap: String,
        config: Config,  // NEW: Pass config
    ) -> Result<()> {
        // ...
        thread::spawn(move || {
            if let Err(e) = run_build(sender.clone(), qmk_path, keyboard, keymap, config) {
                // error handling
            }
        });
        Ok(())
    }
}
```

### Component 3: TUI Integration

**File:** `src/tui/mod.rs`

**Update firmware build handler:**
```rust
// In handle_firmware_build or wherever Ctrl+B is handled
KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    if !state.build_state.is_building() {
        let qmk_path = state.config.paths.qmk_firmware.clone()
            .context("QMK path not configured")?;
        
        state.build_state.start_build(
            qmk_path,
            state.config.build.keyboard.clone(),
            state.config.build.keymap.clone(),
            state.config.clone(),  // NEW: Pass full config
        )?;
    }
}
```

## Optional Enhancement: Deprecated Option Warning

**Problem:** Base keyboard files may have deprecated VIAL options that cause build errors:
```c
// config.h
#define VIAL_KEYBOARD_UID {0x89, 0x36, ...}  // Deprecated

// rules.mk
VIAL_ENABLE = yes  // Deprecated
```

**Solution:** Add a simple pre-build validator that warns users.

**Implementation:**
```rust
// src/firmware/validator.rs (add new method)
impl FirmwareValidator {
    /// Checks for deprecated QMK/VIAL options in keyboard files
    pub fn check_deprecated_options(&self, qmk_path: &Path, keyboard: &str) -> Vec<String> {
        let mut warnings = Vec::new();
        
        let keyboard_path = qmk_path.join("keyboards").join(keyboard);
        
        // Check config.h
        if let Ok(config_h) = std::fs::read_to_string(keyboard_path.join("config.h")) {
            if config_h.contains("VIAL_KEYBOARD_UID") {
                warnings.push("config.h contains deprecated VIAL_KEYBOARD_UID".to_string());
            }
        }
        
        // Check rules.mk
        if let Ok(rules_mk) = std::fs::read_to_string(keyboard_path.join("rules.mk")) {
            if rules_mk.contains("VIAL_ENABLE") {
                warnings.push("rules.mk contains deprecated VIAL_ENABLE".to_string());
            }
        }
        
        warnings
    }
}

// In builder.rs before starting build
fn run_build(...) -> Result<()> {
    // Check for deprecated options
    let validator = FirmwareValidator::new(...);
    let warnings = validator.check_deprecated_options(&qmk_path, &keyboard);
    
    if !warnings.is_empty() {
        sender.send(BuildMessage::Log {
            level: LogLevel::Info,
            message: "⚠️  Warning: This keyboard has deprecated options that may cause build failures:".to_string(),
        }).ok();
        
        for warning in warnings {
            sender.send(BuildMessage::Log {
                level: LogLevel::Info,
                message: format!("  - {}", warning),
            }).ok();
        }
        
        sender.send(BuildMessage::Log {
            level: LogLevel::Info,
            message: "Consider updating the keyboard definition in QMK.".to_string(),
        }).ok();
    }
    
    // Continue with build...
}
```

## Implementation Plan

### Phase 1: Variant Detection (2 hours)

**Tasks:**
- [ ] Add `determine_keyboard_variant()` method to `Config` in `src/config.rs`
- [ ] Test variant detection with various keyboard structures:
  - [ ] Keyboards with variants (keebart/corne_choc_pro)
  - [ ] Keyboards without variants (crkbd)
  - [ ] Already-variant paths (keebart/corne_choc_pro/standard)
- [ ] Add unit tests for detection logic

### Phase 2: Build Path Resolution (1 hour)

**Tasks:**
- [ ] Update `run_build()` signature to accept `Config`
- [ ] Use `determine_keyboard_variant()` to get full path
- [ ] Update `BuildState::start_build()` to pass config
- [ ] Update TUI call sites to pass config
- [ ] Test build command generation

### Phase 3: Optional Warning System (1 hour)

**Tasks:**
- [ ] Add `check_deprecated_options()` to validator
- [ ] Integrate warning into build process
- [ ] Test warning display in build log

### Phase 4: Testing (1 hour)

**Tasks:**
- [ ] Test with keebart/corne_choc_pro/standard
- [ ] Verify RGB_MATRIX_LED_COUNT is defined
- [ ] Verify compilation succeeds
- [ ] Test with non-variant keyboards
- [ ] Test firmware on hardware

## Test Cases

### Unit Tests

1. **Variant Detection**
   - Detect "standard" variant when present
   - Detect first variant when no "standard"
   - Return base keyboard when no variants
   - Handle already-variant paths correctly

2. **Build Path Generation**
   - Generate correct make target with variant
   - Fall back gracefully if detection fails
   - Log correct build target

### Integration Tests

1. **Variant Keyboard Build**
   - Configure with base path: `keebart/corne_choc_pro`
   - Detect variant: `keebart/corne_choc_pro/standard`
   - Build succeeds with full RGB config

2. **Non-Variant Keyboard Build**
   - Configure with: `crkbd`
   - No variant detected
   - Build succeeds normally

### Manual Tests

1. **Real Hardware Test**
   - Generate firmware via TUI
   - Build firmware via TUI (Ctrl+B)
   - Verify build log shows: `Building: keebart/corne_choc_pro/standard:default`
   - Verify compilation succeeds
   - Flash and test on keyboard
   - Verify RGB LEDs work correctly

## Success Criteria

- [ ] Variant keyboards compile successfully
- [ ] RGB_MATRIX_LED_COUNT is automatically defined by QMK
- [ ] Non-variant keyboards still work
- [ ] Build log shows correct make target
- [ ] No code changes needed in QMK
- [ ] All tests pass
- [ ] Firmware works on hardware

## Benefits

✅ **Simple**: ~100 lines of code vs 1000+  
✅ **Fast**: 3-4 hours vs 11-17 hours  
✅ **Robust**: Uses QMK's native system  
✅ **Maintainable**: No regex patterns to break  
✅ **Complete**: Handles all variant-specific config  
✅ **Future-proof**: Won't break with QMK updates  

## Risks & Mitigations

**Risk:** User has keyboard path already including variant  
**Mitigation:** Detection checks if path is already variant-complete

**Risk:** Multiple variants with no clear default  
**Mitigation:** Prefer "standard", then first alphabetically

**Risk:** Deprecated options still cause errors  
**Mitigation:** Show helpful warning with suggestion

## Alternative Considered: Config Merger

We considered building a ConfigMerger that reads, parses, filters, and merges QMK config files. **We rejected this approach because:**

1. **Complexity**: 1000+ lines of code vs ~100
2. **Fragility**: Regex patterns break with QMK changes
3. **Reimplementation**: Duplicates QMK's logic
4. **Incomplete**: Can't handle all config variations
5. **Time**: 11-17 hours vs 3-4 hours

The variant path approach solves 95% of the problem with 5% of the effort.

## Future Enhancements

- Auto-detect preferred variant from user's last successful build
- UI to select specific variant when multiple exist
- Automatic fix for deprecated VIAL options (comment them out temporarily)
- Validation before builds to catch more issues early
