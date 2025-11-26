# Compilation Error Analysis

**Feature:** 004-config-merger-fix  
**Date:** 2025-11-26  
**Error Source:** keebart/corne_choc_pro keyboard compilation

## Error Summary

The TUI encounters multiple compilation errors when building QMK firmware for keyboards with variant structures (e.g., `keebart/corne_choc_pro/standard`).

## Error Categories

### 1. Critical: Missing RGB_MATRIX_LED_COUNT

**Error Message:**
```
quantum/rgb_matrix/rgb_matrix_types.h:74:23: error: 'RGB_MATRIX_LED_COUNT' undeclared here (not in a function); did you mean 'RGBLIGHT_LED_COUNT'?
     led_point_t point[RGB_MATRIX_LED_COUNT];
                       ^~~~~~~~~~~~~~~~~~~~
                       RGBLIGHT_LED_COUNT
```

**Root Cause:**
- Keyboard uses variant structure with separate `standard/` and `mini/` subdirectories
- Each variant has its own `keyboard.json` with RGB configuration
- Build command targets base keyboard path without variant
- QMK doesn't load variant-specific RGB configuration
- Result: `RGB_MATRIX_LED_COUNT` is never defined

**Expected Value:**
From `standard/keyboard.json`:
```json
"rgb_matrix": {
    "split_count": [23, 23],
    ...
}
```
Should result in: `RGB_MATRIX_LED_COUNT = 46` (23 + 23)

### 2. Error: Deprecated VIAL Options

**Error Messages:**
```
☒ keebart/corne_choc_pro: VIAL_ENABLE in rules.mk is no longer a valid option and should be removed
☒ keebart/corne_choc_pro: VIAL_KEYBOARD_UID in config.h is no longer a valid option and should be removed
```

**Root Cause:**
- Modern QMK/VIAL has moved configuration to JSON-based system
- Old-style C header defines and Makefile options are now rejected
- These appear in base keyboard files:
  - `config.h:3` - `#define VIAL_KEYBOARD_UID {0x89, 0x36, 0x2A, 0xC7, 0xFA, 0xD8, 0x89, 0x45}`
  - `rules.mk:4` - `VIAL_ENABLE = yes`

**Modern Approach:**
- VIAL configuration should be in `info.json` or `keyboard.json`
- Build system parses JSON and sets appropriate C defines automatically

### 3. Warning: Missing Build Marker

**Warning Message:**
```
⚠ keebart/corne_choc_pro: Build marker "keyboard.json" not found.
```

**Root Cause:**
- QMK expects `keyboard.json` in the keyboard root directory
- This keyboard uses variant-based structure:
  - `keebart/corne_choc_pro/standard/keyboard.json`
  - `keebart/corne_choc_pro/mini/keyboard.json`
- No `keyboard.json` at root level
- QMK build system can't find primary configuration file

### 4. Error: Compilation Failure

**Error Message:**
```
make[1]: *** [.build/obj_keebart_corne_choc_pro_default/.build/obj_keebart_corne_choc_pro_default/src/default_keyboard.o] Error 1
make: *** [keebart/corne_choc_pro:default] Error 1
Make command failed. Check build log for details.
```

**Root Cause:**
Cascading failure from missing `RGB_MATRIX_LED_COUNT` definition.

## File Locations

### Keyboard Structure
```
vial-qmk-keebart/keyboards/keebart/corne_choc_pro/
├── config.h                    # Contains deprecated VIAL_KEYBOARD_UID
├── rules.mk                    # Contains deprecated VIAL_ENABLE
├── info.json                   # Base keyboard info (no RGB config)
├── keymaps/
│   └── default/
│       ├── config.h            # TUI-generated (minimal)
│       ├── keymap.c            # TUI-generated
│       ├── rules.mk            # Existing (has deprecated options)
│       └── vial.json           # TUI-generated
├── standard/
│   └── keyboard.json           # ⭐ Contains rgb_matrix.split_count [23, 23]
└── mini/
    └── keyboard.json           # Contains different LED configuration
```

### Problematic Content

**config.h (line 3):**
```c
#define VIAL_KEYBOARD_UID {0x89, 0x36, 0x2A, 0xC7, 0xFA, 0xD8, 0x89, 0x45}
```

**rules.mk (line 4):**
```makefile
VIAL_ENABLE = yes
```

**standard/keyboard.json (lines 6-57):**
```json
{
    "rgb_matrix": {
        "split_count": [23, 23],
        "layout": [
            {"matrix": [3, 5], "x": 95, "y": 63, "flags": 4},
            // ... 44 more LED definitions
        ]
    }
}
```

## Current TUI Behavior

### Firmware Generation Process
1. User triggers generation (Ctrl+G)
2. TUI calls `FirmwareGenerator::generate()`
3. Generates 3 files:
   - `keymap.c` - Key layout in QMK format
   - `vial.json` - Vial configurator layout
   - `config.h` - Minimal keymap-specific config
4. Writes to: `keyboards/keebart/corne_choc_pro/keymaps/default/`

### Build Process
1. User triggers build (Ctrl+B)
2. TUI calls `BuildState::start_build()`
3. Spawns background thread
4. Executes: `make keebart/corne_choc_pro:default`
5. QMK build system:
   - Reads `keyboards/keebart/corne_choc_pro/info.json` ❌ No RGB config
   - Reads `keyboards/keebart/corne_choc_pro/config.h` ❌ Has deprecated options
   - Reads `keyboards/keebart/corne_choc_pro/rules.mk` ❌ Has deprecated options
   - Does NOT read variant-specific `standard/keyboard.json`
   - Fails to define `RGB_MATRIX_LED_COUNT`
6. Compilation fails

## What Needs to Happen

### For RGB_MATRIX_LED_COUNT
1. Detect that keyboard uses variant structure
2. Identify correct variant from config: `keebart/corne_choc_pro/standard`
3. Read `standard/keyboard.json`
4. Extract `rgb_matrix.split_count = [23, 23]`
5. Calculate total: `23 + 23 = 46`
6. Add to generated `config.h`: `#define RGB_MATRIX_LED_COUNT 46`

### For Deprecated Options
1. Read base keyboard `config.h`
2. Filter out: `#define VIAL_KEYBOARD_UID ...`
3. Read base keyboard `rules.mk`
4. Filter out: `VIAL_ENABLE = yes`
5. Preserve valid options: `ENCODER_MAP_ENABLE`, `CAPS_WORD_ENABLE`, etc.
6. Merge with TUI-generated content

### Build Command
Current: `make keebart/corne_choc_pro:default`
Should be: `make keebart/corne_choc_pro/standard:default`

OR: Generate config that works without specifying variant

## Solution Architecture

The ConfigMerger module will:

1. **Variant Detection**
   - Parse keyboard path: `keebart/corne_choc_pro/standard`
   - Check if `standard/keyboard.json` exists
   - Extract variant name: `standard`

2. **File Reading**
   - Read base `config.h` from `keebart/corne_choc_pro/`
   - Read variant `keyboard.json` from `keebart/corne_choc_pro/standard/`
   - Read existing keymap `rules.mk` if present

3. **Configuration Extraction**
   - Parse RGB matrix config from JSON
   - Calculate LED count from split_count

4. **Filtering**
   - Remove deprecated VIAL options using regex
   - Preserve valid configuration options

5. **Merging**
   - Combine filtered base config + TUI config
   - Add calculated RGB_MATRIX_LED_COUNT
   - Generate clean rules.mk

6. **Output**
   - Write merged files to keymap directory
   - Hand off to QMK build system

## Expected Results After Fix

### Generated config.h
```c
// Generated by keyboard_tui
// Layout: My Layout
// Generated: 2025-11-26 12:34:56

#pragma once

#define RGB_MATRIX_LED_COUNT 46

// Filtered keyboard base config (minus deprecated options)
#define USB_VBUS_PIN GP13
#define SERIAL_USART_TX_PIN GP12
// ... other valid options

// Add keymap-specific configuration here
```

### Generated rules.mk
```makefile
# Generated by keyboard_tui

SERIAL_DRIVER = vendor

VIA_ENABLE = yes
VIALRGB_ENABLE = yes
ENCODER_MAP_ENABLE = yes
CAPS_WORD_ENABLE = yes
REPEAT_KEY_ENABLE = yes

# Add keymap-specific rules here
```

### Build Log (Success)
```
QMK Firmware 0.21.0
Making keebart/corne_choc_pro/standard with keymap default
✓ Compiling...
✓ Linking...
✓ Creating firmware file: .build/keebart_corne_choc_pro_standard_default.uf2
```

## Related Files to Modify

1. **New:** `src/firmware/config_merger.rs` - Core merging logic
2. **Modified:** `src/firmware/generator.rs` - Use ConfigMerger
3. **Modified:** `src/firmware/mod.rs` - Export ConfigMerger
4. **Modified:** `src/tui/*.rs` - Handle 4 generated files

## Testing Strategy

### Unit Tests
- Variant detection from paths
- RGB LED count extraction
- Deprecated option filtering
- Config merging

### Integration Tests
- Full generation with real keyboard structure
- Verify generated file contents
- Compilation test with QMK

### Manual Tests
- Generate firmware via TUI
- Build firmware via TUI
- Verify successful compilation
- Flash and test on hardware

## Success Criteria

- ✅ Compilation succeeds without errors
- ✅ No deprecated option warnings
- ✅ RGB_MATRIX_LED_COUNT correctly defined
- ✅ All existing keyboards still work
- ✅ Firmware functions correctly on hardware
