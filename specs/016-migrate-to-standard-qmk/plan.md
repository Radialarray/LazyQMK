# Migration Plan: Vial QMK to Standard QMK

## Overview

This document outlines the step-by-step plan to migrate the keyboard-configurator from using `vial-qmk-keebart` (Vial fork) to a personal fork of standard `qmk_firmware`, using the QMK CLI toolchain instead of raw `make` commands.

## Key Changes

1. **QMK Fork**: Create `qmk-keebart` fork of `qmk/qmk_firmware` for custom keyboards
2. **QMK CLI**: Use `qmk compile` instead of `make keyboard:keymap`
3. **Remove Vial**: No more `vial.json` generation or Vial-specific code
4. **Standard Keycodes**: All QMK-documented keycodes work correctly

---

## Phase 0: Setup QMK Fork & Feature Branch

### 0.1 Create Feature Branch

```bash
cd /Users/svenlochner/dev/keyboard-configurator
git checkout -b 016-migrate-to-standard-qmk
```

### 0.2 Create QMK Fork on GitHub

1. Go to https://github.com/qmk/qmk_firmware
2. Click "Fork" → Create `qmk-keebart` (or similar name)
3. This fork will contain custom keyboards not yet in upstream QMK

### 0.3 Add QMK Fork as Submodule

```bash
# Remove old Vial submodule
git submodule deinit -f vial-qmk-keebart
git rm -f vial-qmk-keebart
rm -rf .git/modules/vial-qmk-keebart

# Add new QMK fork
git submodule add https://github.com/YOUR_USERNAME/qmk-keebart.git qmk_firmware
cd qmk_firmware
git remote add upstream https://github.com/qmk/qmk_firmware.git
```

### 0.4 Copy Custom Keyboards to Fork

Copy `keebart/corne_choc_pro` from old Vial repo to new fork:
- Remove Vial-specific files (`vial.json`, `VIAL_*` configs)
- Keep standard QMK keyboard definition

---

## Phase 1: Update Firmware Builder (QMK CLI)

### 1.1 Update Builder to Use QMK CLI

**File**: `src/firmware/builder.rs`

**Current**: Uses `make keyboard:keymap`
**New**: Uses `qmk compile -kb keyboard -km keymap`

Benefits of QMK CLI:
- Better error messages
- Automatic environment setup
- Consistent across platforms
- Parallel compilation support

### 1.2 Update Build Commands

```rust
// Old
Command::new("make")
    .arg(format!("{}:{}", keyboard, keymap))

// New  
Command::new("qmk")
    .args(["compile", "-kb", &keyboard, "-km", &keymap])
```

### 1.3 Add QMK Environment Detection

Add check for `qmk` CLI availability:
```rust
fn check_qmk_cli() -> Result<()> {
    Command::new("qmk").arg("--version").output()?;
    Ok(())
}
```

---

## Phase 2: Update Firmware Generator (Remove Vial)

### 2.1 Remove vial.json Generation

**File**: `src/firmware/generator.rs`

Delete:
- `generate_vial_json()` method
- `generate_vial_layout_array()` method

### 2.2 Update generate() Return Type

```rust
// Old
pub fn generate(&self) -> Result<(String, String, String)>

// New
pub fn generate(&self) -> Result<(String, String)>  // (keymap_path, config_h_path)
```

### 2.3 Remove Vial Unlock Combo Copying

In `generate_merged_config_h()`, remove the section that copies:
- `VIAL_UNLOCK_COMBO_ROWS`
- `VIAL_UNLOCK_COMBO_COLS`

---

## Phase 3: Update Firmware Validator

### 3.1 Remove Deprecated Vial Checks

**File**: `src/firmware/validator.rs`

Delete `check_deprecated_options()` that checks for:
- `VIAL_ENABLE` in rules.mk
- `VIAL_KEYBOARD_UID` in config.h

### 3.2 Add QMK CLI Validation

Add new validation:
```rust
fn validate_qmk_cli(&self) -> Result<()> {
    // Check qmk CLI is installed
    // Check qmk_firmware path is valid
    // Check keyboard exists in QMK
}
```

---

## Phase 4: Update Config & Path Detection

### 4.1 Update Path Auto-Detection

**File**: `src/config.rs`

Change detection from `vial-qmk-keebart` to `qmk_firmware`:
```rust
// Old
if path.ends_with("vial-qmk-keebart") { ... }

// New
if path.ends_with("qmk_firmware") || path.ends_with("qmk-keebart") { ... }
```

### 4.2 Add Migration Warning

```rust
if path.to_string_lossy().contains("vial") {
    warn!("Detected Vial QMK path. Please update to standard QMK.");
}
```

---

## Phase 5: Update TUI

### 5.1 Update Status Messages

**File**: `src/tui/mod.rs`

- Remove vial.json from generated files display
- Update build command display to show `qmk compile`

### 5.2 Update Build Log Display

Show QMK CLI output format instead of make output.

---

## Phase 6: Update Tests

### 6.1 Remove Vial-Specific Tests

**Files to update**:
- `src/firmware/generator.rs` - Remove `test_generate_vial_json`, `test_generate_vial_layout_array`
- `src/firmware/validator.rs` - Remove Vial option tests
- `src/config.rs` - Update path tests
- `tests/qmk_info_json_tests.rs` - Update paths

### 6.2 Add QMK CLI Tests

Add tests for new QMK CLI integration.

---

## Phase 7: Update Documentation

### 7.1 Update QUICKSTART.md

```markdown
## QMK Setup

1. Install QMK CLI:
   ```bash
   python3 -m pip install qmk
   qmk setup
   ```

2. Clone the keyboard-configurator QMK fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/qmk-keebart.git
   cd qmk-keebart
   qmk setup
   ```
```

### 7.2 Update README.md

- Remove Vial references
- Update build instructions to use QMK CLI
- Update feature list

### 7.3 Create MIGRATION.md

Document migration path for existing users.

---

## Implementation Order

| Step | Task | Priority |
|------|------|----------|
| 1 | Create feature branch | P0 |
| 2 | Create QMK fork on GitHub | P0 |
| 3 | Copy keyboards to fork (clean Vial stuff) | P0 |
| 4 | Update submodule reference | P1 |
| 5 | Update `builder.rs` for QMK CLI | P1 |
| 6 | Update `generator.rs` (remove Vial) | P1 |
| 7 | Update `validator.rs` | P1 |
| 8 | Update `config.rs` | P1 |
| 9 | Update `tui/mod.rs` | P2 |
| 10 | Update tests | P2 |
| 11 | Update documentation | P2 |
| 12 | Integration testing | P1 |

---

## Migration Checklist

### Pre-Migration
- [ ] Create feature branch `016-migrate-to-standard-qmk`
- [ ] Create QMK fork on GitHub
- [ ] Verify QMK CLI is installed (`qmk --version`)

### Code Changes
- [ ] Update `src/firmware/builder.rs` - QMK CLI
- [ ] Update `src/firmware/generator.rs` - Remove Vial
- [ ] Update `src/firmware/validator.rs` - Remove Vial checks
- [ ] Update `src/config.rs` - Path detection
- [ ] Update `src/tui/mod.rs` - UI text
- [ ] Update all tests
- [ ] Run `cargo test` - all pass
- [ ] Run `cargo clippy` - clean

### Repository Changes
- [ ] Remove vial-qmk-keebart submodule
- [ ] Add qmk_firmware submodule (fork)
- [ ] Copy corne_choc_pro to fork
- [ ] Update .gitmodules

### Validation
- [ ] `qmk compile` works with generated keymap
- [ ] `LCG(KC_Q)` compiles successfully
- [ ] All modifier combos compile
- [ ] Existing layouts load correctly

### Documentation
- [ ] Update QUICKSTART.md
- [ ] Update README.md  
- [ ] Create MIGRATION.md
- [ ] Update AGENTS.md

---

## Rollback Plan

If issues are discovered:
1. `git checkout main`
2. Delete feature branch if needed
3. Re-add vial-qmk-keebart submodule
4. Document issues for future attempt

---

## Success Criteria

1. **All QMK keycodes work**: `LCG()`, `RCG()`, etc. compile without errors
2. **QMK CLI integration**: `qmk compile` used instead of `make`
3. **Clean codebase**: No Vial references in code
4. **Tests pass**: `cargo test` succeeds
5. **Documentation current**: All docs reference standard QMK
