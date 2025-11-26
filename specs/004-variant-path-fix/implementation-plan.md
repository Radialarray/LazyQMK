# Implementation Plan: Variant Path Detection Fix

**Feature:** 004-variant-path-fix  
**Created:** 2025-11-26

## Overview

Simple, focused implementation to fix QMK variant path detection, allowing QMK's native build system to handle all configuration automatically.

**Estimated Time:** 3-4 hours total

## Phase 1: Variant Detection (2 hours)

### Step 1.1: Add determine_keyboard_variant() Method

**File:** `src/config.rs`

**Add this method to the `Config` impl block:**

```rust
impl Config {
    /// Determines the full keyboard path including variant if present
    /// 
    /// Checks if the keyboard uses a variant structure and returns the
    /// full path including the variant subdirectory.
    /// 
    /// # Examples
    /// 
    /// ```
    /// // Keyboard with variants
    /// "keebart/corne_choc_pro" → "keebart/corne_choc_pro/standard"
    /// 
    /// // Keyboard without variants
    /// "crkbd" → "crkbd"
    /// 
    /// // Path already includes variant
    /// "keebart/corne_choc_pro/standard" → "keebart/corne_choc_pro/standard"
    /// ```
    pub fn determine_keyboard_variant(&self) -> Result<String> {
        let qmk_path = self.paths.qmk_firmware
            .as_ref()
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
            
            // Sort variants for consistent behavior
            variants.sort();
            
            if !variants.is_empty() {
                // Prefer "standard" if it exists
                if variants.contains(&"standard".to_string()) {
                    return Ok(format!("{}/standard", self.build.keyboard));
                }
                
                // Otherwise use first variant alphabetically
                return Ok(format!("{}/{}", self.build.keyboard, variants[0]));
            }
        }
        
        // No variants found, use base keyboard
        Ok(self.build.keyboard.clone())
    }
}
```

**Tasks:**
- [ ] Add method to Config impl block
- [ ] Add necessary imports at top of file: `use anyhow::{Context, Result};`
- [ ] Build and verify no compilation errors

### Step 1.2: Add Unit Tests

**Add to `src/config.rs` at bottom:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_determine_keyboard_variant_with_standard() {
        let temp = TempDir::new().unwrap();
        let qmk_path = temp.path().join("qmk");
        let keyboard_path = qmk_path.join("keyboards/test/keyboard");
        let standard_path = keyboard_path.join("standard");
        
        fs::create_dir_all(&standard_path).unwrap();
        fs::write(standard_path.join("keyboard.json"), "{}").unwrap();
        
        let mut config = Config::default();
        config.paths.qmk_firmware = Some(qmk_path);
        config.build.keyboard = "test/keyboard".to_string();
        
        let result = config.determine_keyboard_variant().unwrap();
        assert_eq!(result, "test/keyboard/standard");
    }

    #[test]
    fn test_determine_keyboard_variant_no_variants() {
        let temp = TempDir::new().unwrap();
        let qmk_path = temp.path().join("qmk");
        let keyboard_path = qmk_path.join("keyboards/crkbd");
        
        fs::create_dir_all(&keyboard_path).unwrap();
        fs::write(keyboard_path.join("info.json"), "{}").unwrap();
        
        let mut config = Config::default();
        config.paths.qmk_firmware = Some(qmk_path);
        config.build.keyboard = "crkbd".to_string();
        
        let result = config.determine_keyboard_variant().unwrap();
        assert_eq!(result, "crkbd");
    }

    #[test]
    fn test_determine_keyboard_variant_already_variant() {
        let temp = TempDir::new().unwrap();
        let qmk_path = temp.path().join("qmk");
        let variant_path = qmk_path.join("keyboards/test/keyboard/mini");
        
        fs::create_dir_all(&variant_path).unwrap();
        fs::write(variant_path.join("keyboard.json"), "{}").unwrap();
        
        let mut config = Config::default();
        config.paths.qmk_firmware = Some(qmk_path);
        config.build.keyboard = "test/keyboard/mini".to_string();
        
        let result = config.determine_keyboard_variant().unwrap();
        assert_eq!(result, "test/keyboard/mini");
    }
}
```

**Tasks:**
- [ ] Add tests to config.rs
- [ ] Add `tempfile` dependency to Cargo.toml if not present: `tempfile = "3.8"`
- [ ] Run tests: `cargo test config::tests`
- [ ] Verify all tests pass

---

## Phase 2: Build Path Resolution (1 hour)

### Step 2.1: Update run_build() Function

**File:** `src/firmware/builder.rs`

**Find the `run_build` function (around line 230) and update signature:**

```rust
// OLD:
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
) -> Result<()> {

// NEW:
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
    config: Config,  // ADD THIS PARAMETER
) -> Result<()> {
```

**Update the function body (around line 236-255):**

```rust
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
    config: Config,
) -> Result<()> {
    // Determine full keyboard path with variant
    let full_keyboard_path = config.determine_keyboard_variant()
        .unwrap_or_else(|e| {
            // Log warning and fall back to original keyboard path
            sender.send(BuildMessage::Log {
                level: LogLevel::Info,
                message: format!("⚠️  Could not detect variant: {}. Using base keyboard.", e),
            }).ok();
            keyboard.clone()
        });

    // Send progress: Compiling
    sender
        .send(BuildMessage::Progress {
            status: BuildStatus::Compiling,
            message: format!("Compiling {keymap} keymap for {full_keyboard_path}..."),
        })
        .context("Failed to send progress message")?;

    sender
        .send(BuildMessage::Log {
            level: LogLevel::Info,
            message: format!("Running: make {}:{}", full_keyboard_path, keymap),
        })
        .ok();

    // Build make command with full keyboard path (including variant if present)
    let make_target = format!("{full_keyboard_path}:{keymap}");

    // Rest of function unchanged...
    let mut cmd = Command::new("make");
    cmd.arg(&make_target)
        .current_dir(&qmk_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // ... continue with existing code
}
```

**Tasks:**
- [ ] Update `run_build` signature to accept `config: Config`
- [ ] Add variant detection at start of function
- [ ] Update make target to use `full_keyboard_path`
- [ ] Update log messages to show full path

### Step 2.2: Update BuildState::start_build()

**File:** `src/firmware/builder.rs`

**Find the `start_build` method (around line 189-220):**

```rust
// OLD signature:
pub fn start_build(
    &mut self,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
) -> Result<()> {

// NEW signature:
pub fn start_build(
    &mut self,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
    config: Config,  // ADD THIS PARAMETER
) -> Result<()> {
```

**Update the thread spawn (around line 209-216):**

```rust
// Spawn background thread
thread::spawn(move || {
    if let Err(e) = run_build(sender.clone(), qmk_path, keyboard, keymap, config) {
        //                                                                  ^^^^^^ ADD config here
        let _ = sender.send(BuildMessage::Complete {
            success: false,
            firmware_path: None,
            error: Some(format!("Build failed: {e}")),
        });
    }
});
```

**Tasks:**
- [ ] Update `start_build` signature
- [ ] Pass `config` to `run_build` in thread spawn
- [ ] Build and verify no compilation errors

### Step 2.3: Update TUI Call Sites

**File:** `src/tui/mod.rs`

**Find the firmware build handler (search for "Ctrl+B" or "start_build"):**

```rust
// OLD (approximate location):
state.build_state.start_build(
    qmk_path,
    state.config.build.keyboard.clone(),
    state.config.build.keymap.clone(),
)?;

// NEW:
state.build_state.start_build(
    qmk_path,
    state.config.build.keyboard.clone(),
    state.config.build.keymap.clone(),
    state.config.clone(),  // ADD THIS LINE
)?;
```

**Tasks:**
- [ ] Find all call sites to `start_build` in tui/mod.rs
- [ ] Add `state.config.clone()` parameter
- [ ] Build and verify no compilation errors
- [ ] Run TUI and verify Ctrl+B still works

---

## Phase 3: Optional Deprecation Warning (1 hour)

### Step 3.1: Add Deprecation Check to Validator

**File:** `src/firmware/validator.rs`

**Add new method to `FirmwareValidator` impl:**

```rust
impl FirmwareValidator<'_> {
    /// Checks for deprecated QMK/VIAL options in keyboard files
    /// 
    /// Returns a list of warning messages for deprecated options found.
    pub fn check_deprecated_options(
        qmk_path: &Path,
        keyboard: &str,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Get base keyboard path (remove variant if present)
        let base_keyboard = keyboard
            .split('/')
            .take(keyboard.split('/').count().saturating_sub(1).max(1))
            .collect::<Vec<_>>()
            .join("/");
        
        let keyboard_path = qmk_path.join("keyboards").join(&base_keyboard);
        
        // Check config.h for deprecated defines
        if let Ok(config_h) = std::fs::read_to_string(keyboard_path.join("config.h")) {
            if config_h.contains("VIAL_KEYBOARD_UID") {
                warnings.push(
                    "config.h contains deprecated VIAL_KEYBOARD_UID (should be in info.json)".to_string()
                );
            }
            if config_h.contains("VIAL_UNLOCK_COMBO_ROWS") {
                warnings.push(
                    "config.h contains deprecated VIAL_UNLOCK_COMBO_ROWS".to_string()
                );
            }
            if config_h.contains("VIAL_UNLOCK_COMBO_COLS") {
                warnings.push(
                    "config.h contains deprecated VIAL_UNLOCK_COMBO_COLS".to_string()
                );
            }
        }
        
        // Check rules.mk for deprecated options
        if let Ok(rules_mk) = std::fs::read_to_string(keyboard_path.join("rules.mk")) {
            if rules_mk.contains("VIAL_ENABLE") {
                warnings.push(
                    "rules.mk contains deprecated VIAL_ENABLE (should be in info.json)".to_string()
                );
            }
        }
        
        warnings
    }
}
```

**Tasks:**
- [ ] Add method to validator
- [ ] Build and verify no compilation errors

### Step 3.2: Integrate Warning into Build

**File:** `src/firmware/builder.rs`

**Add warning check at start of `run_build` (after variant detection):**

```rust
fn run_build(
    sender: Sender<BuildMessage>,
    qmk_path: PathBuf,
    keyboard: String,
    keymap: String,
    config: Config,
) -> Result<()> {
    // Determine full keyboard path with variant
    let full_keyboard_path = config.determine_keyboard_variant()
        .unwrap_or_else(|e| {
            sender.send(BuildMessage::Log {
                level: LogLevel::Info,
                message: format!("⚠️  Could not detect variant: {}. Using base keyboard.", e),
            }).ok();
            keyboard.clone()
        });

    // Check for deprecated options (NEW)
    let warnings = crate::firmware::validator::FirmwareValidator::check_deprecated_options(
        &qmk_path,
        &full_keyboard_path,
    );
    
    if !warnings.is_empty() {
        sender.send(BuildMessage::Log {
            level: LogLevel::Info,
            message: "⚠️  Warning: This keyboard has deprecated configuration:".to_string(),
        }).ok();
        
        for warning in warnings {
            sender.send(BuildMessage::Log {
                level: LogLevel::Info,
                message: format!("  • {}", warning),
            }).ok();
        }
        
        sender.send(BuildMessage::Log {
            level: LogLevel::Info,
            message: "These may cause build failures. Consider updating the keyboard in QMK.".to_string(),
        }).ok();
        sender.send(BuildMessage::Log {
            level: LogLevel::Info,
            message: "".to_string(),
        }).ok();
    }

    // Continue with build...
    sender
        .send(BuildMessage::Progress {
            status: BuildStatus::Compiling,
            message: format!("Compiling {keymap} keymap for {full_keyboard_path}..."),
        })
        .context("Failed to send progress message")?;

    // ... rest of function
}
```

**Tasks:**
- [ ] Add deprecation check after variant detection
- [ ] Send warning messages to build log
- [ ] Build and verify warnings appear in TUI

---

## Phase 4: Testing (1 hour)

### Step 4.1: Manual Testing with Real Keyboard

**Test Scenario 1: Variant Keyboard**

1. [ ] Configure TUI with: `keebart/corne_choc_pro`
2. [ ] Trigger build (Ctrl+B)
3. [ ] Check build log shows: `Running: make keebart/corne_choc_pro/standard:default`
4. [ ] Verify compilation succeeds
5. [ ] Check for RGB_MATRIX_LED_COUNT errors → Should be NONE
6. [ ] Verify firmware file is created

**Test Scenario 2: Non-Variant Keyboard**

1. [ ] Configure TUI with a non-variant keyboard (e.g., `crkbd`)
2. [ ] Trigger build (Ctrl+B)
3. [ ] Check build log shows: `Running: make crkbd:default`
4. [ ] Verify compilation succeeds

**Test Scenario 3: Deprecated Options Warning**

1. [ ] Build keyboard with deprecated options (e.g., keebart/corne_choc_pro)
2. [ ] Check build log for deprecation warnings
3. [ ] Verify warnings are helpful and clear

### Step 4.2: Automated Testing

**Run existing tests:**
```bash
cargo test
cargo clippy
```

**Tasks:**
- [ ] All existing tests pass
- [ ] No new clippy warnings
- [ ] New config tests pass

### Step 4.3: Integration Test

**Test on real hardware (if available):**

1. [ ] Generate firmware via TUI (Ctrl+G)
2. [ ] Build firmware via TUI (Ctrl+B)
3. [ ] Verify successful compilation
4. [ ] Flash firmware to keyboard
5. [ ] Test RGB LEDs work
6. [ ] Test all keys work
7. [ ] Test encoders work

---

## Rollout Checklist

- [ ] All phases complete
- [ ] All tests pass
- [ ] Manual testing complete
- [ ] Code compiles without warnings
- [ ] Build succeeds for variant keyboards
- [ ] Build succeeds for non-variant keyboards
- [ ] Deprecation warnings display correctly
- [ ] Ready to commit

---

## Expected Results

### Before Fix
```
Running: make keebart/corne_choc_pro:default
⚠ keebart/corne_choc_pro: Build marker "keyboard.json" not found
error: 'RGB_MATRIX_LED_COUNT' undeclared
make: *** [keebart/corne_choc_pro:default] Error 1
```

### After Fix
```
Running: make keebart/corne_choc_pro/standard:default
⚠️  Warning: This keyboard has deprecated configuration:
  • config.h contains deprecated VIAL_KEYBOARD_UID
  • rules.mk contains deprecated VIAL_ENABLE
These may cause build failures. Consider updating the keyboard in QMK.

Compiling default keymap for keebart/corne_choc_pro/standard...
✓ Success: Firmware written to .build/keebart_corne_choc_pro_standard_default.uf2
```

---

## Time Estimate

- **Phase 1:** 2 hours (variant detection + tests)
- **Phase 2:** 1 hour (build path resolution)
- **Phase 3:** 1 hour (optional warnings)
- **Phase 4:** 1 hour (testing)

**Total:** 4-5 hours

**Compared to Config Merger:** 11-17 hours saved ✅

---

## Notes

- Keep changes minimal and focused
- Prefer QMK's native systems over custom solutions
- Maintain backward compatibility
- Test thoroughly with real hardware if possible
