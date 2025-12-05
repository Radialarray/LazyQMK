# Tasks: Migrate to Standard QMK

## Task 1: Update Firmware Generator (Priority: P1)

**File**: `src/firmware/generator.rs`

### 1.1 Remove vial.json Generation

**Delete these methods:**
- `generate_vial_json()` (lines ~418-443)
- `generate_vial_layout_array()` (lines ~445-483)

### 1.2 Update generate() Method

**Current signature:**
```rust
pub fn generate(&self) -> Result<(String, String, String)>
```

**New signature:**
```rust
pub fn generate(&self) -> Result<(String, String)>  // (keymap_path, config_h_path)
```

**Changes:**
- Remove vial.json generation call
- Update return tuple to only include keymap.c and config.h paths

### 1.3 Remove Vial Unlock Combo Copying

**In `generate_merged_config_h()`**, remove the section that copies `VIAL_UNLOCK_COMBO_ROWS` and `VIAL_UNLOCK_COMBO_COLS` (lines ~649-692).

### 1.4 Update Tests

Remove or update these tests:
- `test_generate_vial_json`
- `test_generate_vial_layout_array`

---

## Task 2: Update Firmware Validator (Priority: P1)

**File**: `src/firmware/validator.rs`

### 2.1 Remove Deprecated Vial Options Check

**Delete:**
- `check_deprecated_options()` method (lines ~446-502)
- Any calls to this method in the validation flow

### 2.2 Update Tests

Remove Vial-specific test cases (lines ~624-680).

---

## Task 3: Update TUI Display (Priority: P1)

**File**: `src/tui/mod.rs`

### 3.1 Update Generation Success Message

Find the code that displays generated file paths and remove the vial.json reference.

**Current** (approximate):
```rust
let (keymap_path, vial_path, config_path) = generator.generate()?;
// Shows all three paths
```

**Updated:**
```rust
let (keymap_path, config_path) = generator.generate()?;
// Shows only keymap.c and config.h paths
```

---

## Task 4: Update Config Path Detection (Priority: P2)

**File**: `src/config.rs`

### 4.1 Update Auto-Detection Logic

**Find and replace:**
- `"vial-qmk-keebart"` → `"qmk_firmware"`

**In these locations:**
- Path auto-fix logic (~lines 267-283)
- Any default path suggestions

### 4.2 Add Migration Warning

Add a warning when detecting old `vial-qmk-keebart` path:
```rust
if path.ends_with("vial-qmk-keebart") {
    warn!("Detected Vial QMK path. Consider switching to standard qmk_firmware.");
}
```

### 4.3 Update Tests

Update test paths from `vial-qmk-keebart` to `qmk_firmware` (~lines 702, 708, 724).

---

## Task 5: Remove Git Submodule (Priority: P2)

### 5.1 Remove vial-qmk-keebart Submodule

```bash
# From repository root
git submodule deinit -f vial-qmk-keebart
git rm -f vial-qmk-keebart
rm -rf .git/modules/vial-qmk-keebart
```

### 5.2 Update .gitmodules

Remove the vial-qmk-keebart entry.

### 5.3 Update .gitignore (if needed)

Add `qmk_firmware/` if users will clone it inside the project.

---

## Task 6: Update Documentation (Priority: P2)

### 6.1 Update QUICKSTART.md

Replace Vial QMK setup instructions with:
```markdown
## QMK Firmware Setup

1. Clone QMK firmware:
   ```bash
   git clone https://github.com/qmk/qmk_firmware.git
   cd qmk_firmware
   qmk setup
   ```

2. Configure the path in keyboard-configurator:
   - Set `qmk_firmware` path in settings
```

### 6.2 Update README.md

- Remove "Vial support" from features
- Update prerequisites section
- Update any setup instructions

### 6.3 Create MIGRATION.md

Create a migration guide for existing users:
```markdown
# Migrating from Vial QMK to Standard QMK

## What Changed
- The configurator now uses standard QMK firmware instead of the Vial fork
- vial.json is no longer generated

## Migration Steps
1. Clone standard QMK: `git clone https://github.com/qmk/qmk_firmware.git`
2. Update your config.toml to point to the new path
3. Ensure your keyboard exists in standard QMK (or add it)

## What You Lose
- Vial app live editing compatibility
- VIA compatibility (unless you enable it in QMK)
```

### 6.4 Update AGENTS.md

Remove any Vial-specific development guidelines.

---

## Task 7: Integration Testing (Priority: P1)

### 7.1 Test Modifier Combo Keycodes

Create a test layout with these keycodes and verify compilation:
- `LCG(KC_Q)` - Left Ctrl + Left GUI + Q
- `RCG(KC_A)` - Right Ctrl + Right GUI + A  
- `LCS(KC_S)` - Left Ctrl + Left Shift + S
- `LSA(KC_D)` - Left Shift + Left Alt + D
- `LSG(KC_F)` - Left Shift + Left GUI + F
- `LAG(KC_G)` - Left Alt + Left GUI + G
- All other modifier combos from QMK docs

### 7.2 Test Full Build Cycle

1. Load existing layout
2. Generate firmware
3. Compile with `make keyboard:keymap`
4. Verify no errors

### 7.3 Run Full Test Suite

```bash
cargo test
cargo clippy
```

---

## Task 8: Keyboard Availability Check (Priority: P3)

### 8.1 Verify keebart Keyboards in Standard QMK

Check if `keebart/corne_choc_pro` exists in standard QMK.

**If not:**
- Option A: Submit keyboard to QMK upstream
- Option B: Document how to add custom keyboards
- Option C: Keep keyboard definitions in a separate repo

---

## Completion Checklist

### Code Changes
- [ ] `src/firmware/generator.rs` - Remove Vial generation
- [ ] `src/firmware/validator.rs` - Remove Vial checks
- [ ] `src/tui/mod.rs` - Update status display
- [ ] `src/config.rs` - Update path detection

### Tests
- [ ] All unit tests pass
- [ ] `LCG(KC_Q)` compiles successfully
- [ ] All modifier combos compile
- [ ] Existing layouts load correctly

### Repository
- [ ] vial-qmk-keebart submodule removed
- [ ] .gitmodules updated
- [ ] No broken references

### Documentation  
- [ ] QUICKSTART.md updated
- [ ] README.md updated
- [ ] MIGRATION.md created
- [ ] AGENTS.md updated

### Final Validation
- [ ] `cargo test` passes
- [ ] `cargo clippy` clean
- [ ] Full build cycle works
- [ ] Fresh clone + setup works
