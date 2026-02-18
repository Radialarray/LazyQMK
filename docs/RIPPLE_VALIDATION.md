# RGB Overlay Ripple - Manual Validation Checklist

This document provides a comprehensive manual validation checklist for the RGB Overlay Ripple feature. The ripple effect creates visual feedback by displaying animated ripples on your RGB keyboard when keys are pressed.

## Prerequisites

Before testing, ensure:

- [ ] QMK firmware submodule is initialized: `git submodule update --init --recursive qmk_firmware`
- [ ] QMK CLI tools are installed: `pip3 install qmk && qmk setup`
- [ ] You have a keyboard with RGB matrix support configured in QMK
- [ ] LazyQMK is built with latest changes: `cargo build --release`
- [ ] You have a test layout file ready (or create one during testing)

## Feature Overview

**What is Overlay Ripple?**
Ripple effects are animated overlays rendered on top of base layer colors. When you press a key, a ripple expands outward from that LED position, creating a visual wave effect.

**How it works:**
- Triggered on keypress (and/or key release, configurable)
- Rendered as additive overlay using `rgb_matrix_indicators_advanced_user`
- Up to 8 concurrent ripples supported
- Three color modes: Fixed Color, Key-Based Color, Hue Shift

## Settings Validation

### Access Settings Manager

1. **Launch LazyQMK TUI** with a layout file
   ```bash
   ./target/release/lazyqmk <your-layout.md>
   ```

2. **Open Settings Manager**
   - Press `Shift+S`
   - Navigate to RGB section (if not already there)

3. **Verify all ripple settings are present:**
   - [ ] Overlay Ripple Enabled (boolean)
   - [ ] Max Concurrent Ripples (1-8)
   - [ ] Ripple Duration (milliseconds)
   - [ ] Ripple Speed (0-255)
   - [ ] Ripple Band Width (LED units)
   - [ ] Ripple Amplitude (0-100%)
   - [ ] Ripple Color Mode (Fixed/Key Color/Hue Shift)
   - [ ] Ripple Fixed Color (hex color)
   - [ ] Ripple Hue Shift (degrees, -180 to 180)
   - [ ] Trigger on Press (boolean)
   - [ ] Trigger on Release (boolean)
   - [ ] Ignore Transparent Keys (boolean)
   - [ ] Ignore Modifier Keys (boolean)
   - [ ] Ignore Layer Switch Keys (boolean)

### Test Setting Modifications

#### Enable/Disable Ripple

1. **Select "Overlay Ripple Enabled"**
   - Press `Enter` to toggle
   - [ ] Verify status changes between On/Off
   - [ ] Verify dirty flag appears in title (asterisk)

#### Numeric Settings

2. **Test Max Concurrent Ripples**
   - Select setting, press `Enter`
   - [ ] Try values: 1, 4, 8
   - [ ] Verify values outside range (0, 9) are rejected or clamped
   - [ ] Confirm value updates in display

3. **Test Ripple Duration**
   - Select setting, press `Enter`
   - [ ] Try values: 200, 500, 1000, 2000
   - [ ] Verify shorter durations = faster ripples
   - [ ] Confirm millisecond unit in description

4. **Test Ripple Speed**
   - Select setting, press `Enter`
   - [ ] Try values: 50, 128, 200, 255
   - [ ] Verify higher values = faster expansion
   - [ ] Confirm 0-255 range

5. **Test Ripple Band Width**
   - Select setting, press `Enter`
   - [ ] Try values: 1, 3, 5, 10
   - [ ] Confirm LED unit description

6. **Test Ripple Amplitude**
   - Select setting, press `Enter`
   - [ ] Try values: 0, 25, 50, 75, 100
   - [ ] Verify 0-100% range
   - [ ] Confirm percentage description

#### Color Mode Selection

7. **Test Color Mode Picker**
   - Select "Ripple Color Mode", press `Enter`
   - [ ] Verify three options appear: Fixed Color, Key Color, Hue Shift
   - [ ] Navigate with arrow keys or j/k
   - [ ] Select each mode with Enter
   - [ ] Verify mode name updates in settings list

8. **Test Fixed Color Mode**
   - Select "Ripple Fixed Color", press `Enter`
   - [ ] Verify RGB color picker opens
   - [ ] Test hex input: #FF0000 (red), #00FF00 (green), #0000FF (blue), #FF00FF (magenta)
   - [ ] Verify color preview updates
   - [ ] Confirm color value updates in settings

9. **Test Hue Shift**
   - Select "Ripple Hue Shift", press `Enter`
   - [ ] Try values: -180, -90, 0, 60, 90, 180
   - [ ] Verify degree symbol (°) in display
   - [ ] Confirm negative values accepted

#### Trigger and Filter Settings

10. **Test Trigger Options**
    - [ ] Toggle "Trigger on Press" - verify boolean state changes
    - [ ] Toggle "Trigger on Release" - verify boolean state changes
    - [ ] Enable both - should be valid
    - [ ] Disable both - should be valid (no ripples will trigger)

11. **Test Filter Options**
    - [ ] Toggle "Ignore Transparent Keys" - verify boolean state
    - [ ] Toggle "Ignore Modifier Keys" - verify boolean state
    - [ ] Toggle "Ignore Layer Switch Keys" - verify boolean state

### Settings Persistence

12. **Save and Reload Settings**
    - Modify several ripple settings
    - [ ] Press `Ctrl+S` to save layout
    - [ ] Exit LazyQMK
    - [ ] Reload layout: `./target/release/lazyqmk <your-layout.md>`
    - [ ] Open Settings Manager (Shift+S)
    - [ ] Verify all ripple settings persisted correctly

13. **Verify Markdown Format**
    - Open layout file in text editor
    - [ ] Verify YAML frontmatter contains ripple settings
    - [ ] Check for keys like `rgb_overlay_ripple:`, `enabled:`, `color_mode:`, etc.
    - [ ] Verify hex color format (e.g., `fixed_color: "#FF00FF"`)
    - [ ] Confirm integer values are unquoted
    - [ ] Verify boolean values are `true`/`false`

## Firmware Generation Validation

### Test Basic Generation

14. **Generate Firmware with Default Settings**
    - Enable ripple with default settings
    - Save layout
    - Generate firmware:
      ```bash
      ./target/release/lazyqmk generate --layout <your-layout.md> --output ./test_output
      ```
    - [ ] Verify generation succeeds (exit code 0)
    - [ ] Check output directory contains `keymap.c` and `config.h`

15. **Verify Generated Code Structure**
    - Open `test_output/keymap.c`
    - [ ] Search for `RIPPLE_MAX_CONCURRENT` - should be defined
    - [ ] Search for `ripple_state` struct array
    - [ ] Search for `rgb_matrix_indicators_advanced_user` function
    - [ ] Verify ripple logic is present (distance calculation, brightness modulation)
    - [ ] Check for trigger conditions (press/release checks)

### Test Color Mode Generation

16. **Fixed Color Mode**
    - Set color mode to "Fixed Color"
    - Set fixed color to `#FF0000` (red)
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for color assignment code in ripple logic
    - [ ] Verify RGB values match: `r = 255, g = 0, b = 0`
    - [ ] Confirm comment indicates "Fixed color mode"

17. **Key-Based Color Mode**
    - Set color mode to "Key Color"
    - Ensure layout has per-key or layer colors defined
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for "Key-based color mode" comment
    - [ ] Verify code preserves base LED color: `led_color->r`, `led_color->g`, `led_color->b`
    - [ ] Confirm brightness is added to base color: `MIN(led_color->r + brightness, 255)`

18. **Hue Shift Mode**
    - Set color mode to "Hue Shift"
    - Set hue shift to `60` degrees
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for "Hue shift mode" comment
    - [ ] Verify hue shift value in code: `shift by 60 degrees`
    - [ ] Confirm HSV → RGB conversion code is present
    - [ ] Check for modulo arithmetic: `new_hue = (old_hue + 60) % 360`

### Test Filter Generation

19. **Test Ignore Transparent Keys**
    - Enable "Ignore Transparent Keys"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for `KC_TRNS` check in trigger logic
    - [ ] Verify early return or skip condition: `if (keycode == KC_TRNS) { return; }`

20. **Test Ignore Modifiers**
    - Enable "Ignore Modifier Keys"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for modifier check: `QK_MODS`, `QK_MOD_TAP`, or specific `KC_LCTL`, `KC_LSFT`, etc.
    - [ ] Verify early return for modifiers

21. **Test Ignore Layer Switch**
    - Enable "Ignore Layer Switch Keys"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Search for layer switch check: `QK_LAYER_TAP`, `QK_MOMENTARY`, `QK_TO`, etc.
    - [ ] Verify early return for layer keys

### Test Trigger Conditions

22. **Press Only**
    - Enable "Trigger on Press", disable "Trigger on Release"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Find `process_record_user` or similar hook
    - [ ] Verify check: `if (record->event.pressed) { /* trigger ripple */ }`
    - [ ] Confirm no trigger on `!record->event.pressed`

23. **Release Only**
    - Disable "Trigger on Press", enable "Trigger on Release"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Verify check: `if (!record->event.pressed) { /* trigger ripple */ }`
    - [ ] Confirm no trigger on `record->event.pressed`

24. **Both Press and Release**
    - Enable both "Trigger on Press" and "Trigger on Release"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Verify ripple triggers on both conditions
    - [ ] No early return based on press state

25. **Neither (No Triggers)**
    - Disable both "Trigger on Press" and "Trigger on Release"
    - Generate firmware
    - Open `keymap.c`
    - [ ] Verify ripple logic is still present but never triggered
    - [ ] OR verify ripple code is optimized out (acceptable behavior)

## Integration with Idle Effect

The idle effect and overlay ripple are separate features that should work together harmoniously.

26. **Test Idle Effect + Ripple Enabled**
    - Enable idle effect (Shift+S → Idle Effect Enabled)
    - Enable overlay ripple
    - Generate firmware
    - Open `keymap.c`
    - [ ] Verify both idle effect state machine and ripple logic are present
    - [ ] Confirm no naming conflicts (separate variable namespaces)
    - [ ] Check that `rgb_matrix_indicators_advanced_user` handles both features

27. **Test Idle Effect Priority**
    - Enable both features
    - Generate firmware
    - Open `keymap.c`
    - [ ] Verify idle effect runs during idle state
    - [ ] Verify ripples render on top of idle animation (or after idle exit)
    - [ ] Confirm ripple resets idle timer (user activity)

28. **Test Conflict Resolution**
    - Enable both features with aggressive settings (short idle timeout, many ripples)
    - Generate firmware
    - [ ] Verify compilation succeeds with no warnings
    - [ ] Check that both features have independent configuration guards
    - [ ] Confirm no buffer overflows or array size conflicts

## Visual Appearance Validation (On-Hardware)

**Note:** These tests require flashing firmware to actual hardware with RGB matrix support.

29. **Flash and Test Basic Ripple**
    - Flash generated firmware to keyboard
    - Press keys
    - [ ] Verify ripple appears on keypress (if trigger on press enabled)
    - [ ] Verify ripple expands outward from pressed key
    - [ ] Confirm ripple fades after duration expires

30. **Test Color Modes Visually**
    - **Fixed Color Mode:**
      - Set fixed color to bright red (#FF0000)
      - Flash firmware
      - [ ] Verify all ripples are red regardless of key pressed
    - **Key-Based Color Mode:**
      - Configure different layer colors (e.g., blue layer, red layer)
      - Flash firmware
      - [ ] Verify ripples match the base color of each key
    - **Hue Shift Mode:**
      - Set hue shift to 180° (complementary color)
      - Flash firmware
      - [ ] Verify ripples have shifted hue from base color

31. **Test Multiple Concurrent Ripples**
    - Set max concurrent ripples to 4
    - Flash firmware
    - Rapidly press 4+ keys
    - [ ] Verify up to 4 ripples animate simultaneously
    - [ ] Confirm oldest ripple disappears when limit exceeded
    - [ ] Check for smooth animation (no flickering)

32. **Test Ripple Parameters**
    - **Short Duration (200ms):**
      - [ ] Ripples fade quickly
    - **Long Duration (2000ms):**
      - [ ] Ripples persist and expand slowly
    - **High Speed (255):**
      - [ ] Ripples expand rapidly
    - **Low Speed (50):**
      - [ ] Ripples expand slowly
    - **Wide Band (10):**
      - [ ] Ripple band is thick/wide
    - **Narrow Band (1):**
      - [ ] Ripple band is thin/narrow
    - **High Amplitude (100%):**
      - [ ] Ripples are very bright
    - **Low Amplitude (25%):**
      - [ ] Ripples are subtle

33. **Test Filters**
    - **Ignore Transparent Keys:**
      - Press transparent keys (KC_TRNS)
      - [ ] Verify no ripples appear
    - **Ignore Modifier Keys:**
      - Press Shift, Ctrl, Alt, GUI keys
      - [ ] Verify no ripples appear
    - **Ignore Layer Switch:**
      - Press MO(), LT(), TG() keys
      - [ ] Verify no ripples appear

## Performance Validation

34. **Test Firmware Size**
    - Generate firmware with ripple disabled
    - Note firmware size
    - Generate firmware with ripple enabled
    - [ ] Verify size increase is reasonable (<5KB for ripple logic)
    - [ ] Confirm firmware still fits on target MCU

35. **Test CPU Usage**
    - Flash firmware with ripple enabled
    - Monitor keyboard responsiveness
    - [ ] Rapid typing does not cause input lag
    - [ ] Ripple rendering does not block key input
    - [ ] No noticeable performance degradation

36. **Test Memory Usage**
    - Review generated code for array sizes
    - [ ] Verify `ripple_state` array matches max_ripples setting
    - [ ] Confirm no dynamic allocation (embedded constraint)
    - [ ] Check SRAM usage is within MCU limits

## Edge Cases and Error Handling

37. **Test Invalid Settings**
    - Try to set max ripples to 0
    - [ ] Verify rejected or clamped to minimum (1)
    - Try to set max ripples to 20
    - [ ] Verify rejected or clamped to maximum (8)
    - Try to set duration to 0
    - [ ] Verify behavior (instant ripple or minimum value)

38. **Test Conflicting Settings**
    - Disable both "Trigger on Press" and "Trigger on Release"
    - [ ] Verify firmware generates without errors
    - [ ] Confirm no ripples appear on hardware (expected behavior)

39. **Test Missing RGB Matrix Support**
    - Attempt to enable ripple on keyboard without RGB matrix in QMK
    - Generate firmware
    - [ ] Verify generation warns or gracefully handles missing RGB support
    - [ ] Confirm compilation does not fail due to missing RGB functions

## Documentation and Help Validation

40. **Verify In-App Help**
    - Open LazyQMK TUI
    - Press `?` for help overlay
    - [ ] Search for ripple or overlay documentation
    - [ ] Verify descriptions match actual behavior
    - [ ] Confirm keybindings for Settings Manager are documented

41. **Verify CLI Help**
    - Run `./target/release/lazyqmk --help`
    - [ ] Check for any ripple-related flags or options (if applicable)
    - Run `./target/release/lazyqmk generate --help`
    - [ ] Verify no conflicting options with ripple feature

## Export and Inspect Validation

42. **Test Layout Export**
    - Enable ripple with custom settings
    - Export layout:
      ```bash
      ./target/release/lazyqmk export --layout <your-layout.md> --output exported.md
      ```
    - Open exported file
    - [ ] Verify ripple settings are included in settings summary
    - [ ] Confirm human-readable format
    - [ ] Check for correct color mode and values

43. **Test Layout Inspect**
    - Run:
      ```bash
      ./target/release/lazyqmk inspect --layout <your-layout.md> --json
      ```
    - Parse JSON output
    - [ ] Verify `rgb_overlay_ripple` object is present
    - [ ] Confirm all settings are included with correct types
    - [ ] Validate color format in JSON (hex string)

## Regression Testing

44. **Test Backward Compatibility**
    - Create layout WITHOUT ripple settings
    - Load in LazyQMK
    - [ ] Verify ripple settings default to disabled
    - [ ] Confirm defaults match expected values (from `RgbOverlayRippleSettings::default()`)

45. **Test Forward Compatibility**
    - Manually add future ripple setting to markdown (e.g., `ripple_shape: circle`)
    - Load in LazyQMK
    - [ ] Verify unknown settings are ignored (not error)
    - [ ] Confirm other settings still load correctly

46. **Test Migration**
    - If updating from previous version, load old layout files
    - [ ] Verify existing settings are preserved
    - [ ] Confirm new ripple settings have sensible defaults

## Summary Checklist

### Critical Tests (Must Pass)

- [ ] Settings Manager displays all 14 ripple settings
- [ ] Enable/disable toggle works
- [ ] Settings persist to markdown YAML
- [ ] Firmware generates without errors
- [ ] All three color modes generate correct code
- [ ] Ripple logic is present in `rgb_matrix_indicators_advanced_user`
- [ ] Filters generate correct trigger conditions
- [ ] No conflicts with idle effect feature

### Important Tests (Should Pass)

- [ ] Numeric settings validate ranges
- [ ] Color picker accepts hex input
- [ ] Trigger on press/release generates correct logic
- [ ] Export includes ripple settings
- [ ] Inspect JSON includes ripple settings
- [ ] Backward compatibility with layouts without ripple
- [ ] Firmware size increase is reasonable

### Optional Tests (Nice to Have)

- [ ] Visual validation on real hardware
- [ ] Performance testing (no input lag)
- [ ] Multiple concurrent ripples work smoothly
- [ ] All parameter variations produce expected visual effects

## Reporting Issues

If any test fails, report:

1. **Test number and description**
2. **Expected behavior**
3. **Actual behavior**
4. **Steps to reproduce**
5. **Environment:** OS, terminal, LazyQMK version, QMK version
6. **Generated code snippet** (if firmware generation related)
7. **Layout file excerpt** (if settings related)

---

**Last Updated:** 2025-12-12 (matching FEATURES.md)
