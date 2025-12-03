# Tasks: Hardcoded Values Refactor

## Phase 1: Keyboard Display (keyboard.rs) ✓ COMPLETE

- [x] 1.1 Add `get_mod_tap_display_name()` method to KeycodeDb
  - Returns short display name (e.g., "CTL", "SFT") for mod-tap prefixes
  - Use data from `mod_tap.json`

- [x] 1.2 Add `is_tap_hold_keycode()` method to KeycodeDb
  - Detect if a keycode is a tap-hold type using database patterns
  - Returns the type (LT, MT, mod-tap shorthand, LM, SH_T)

- [x] 1.3 Refactor `format_tap_hold()` in keyboard.rs
  - Remove hardcoded `mod_tap_prefixes` array
  - Use KeycodeDb methods for pattern detection and display names

- [x] 1.4 Test keyboard display behavior
  - Verify mod-tap keys display correctly
  - Verify LT, MT, LM, SH_T display correctly

## Phase 2: Layer Keycode Detection (layout.rs) ✓ COMPLETE

- [x] 2.1 Add `get_layer_keycode_prefixes()` method to KeycodeDb
  - Return list of simple layer prefixes (MO, TG, TO, DF, OSL, TT)
  - Return list of compound layer prefixes (LT, LM)

- [x] 2.2 Refactor layer detection in layout.rs
  - Build regex patterns from database-provided prefixes
  - Replace hardcoded regex patterns

- [x] 2.3 Test layer reference detection
  - Verify all layer keycode types are detected
  - Run existing layer tests

## Phase 3: Default Colors ✓ COMPLETE

- [x] 3.1 Add `get_color()` and `get_shade()` helpers to ColorPalette
  - Get named colors and shades from palette
  - Added `default_layer_color()` returning Gray-500

- [x] 3.2 Define default color constants
  - `default_layer_color()` - Gray-500 (107, 114, 128)
  - OFF_COLOR (0,0,0) kept as-is (appropriate for "off" state)

- [x] 3.3 Replace hardcoded RgbColor::new() calls
  - `src/main.rs:207` - Updated to use palette.default_layer_color()
  - `src/tui/mod.rs:3632` - Updated to use palette.default_layer_color()
  - `src/parser/template_gen.rs` - Updated test code

## Verification ✓ COMPLETE

- [x] All existing tests pass (238 unit + 16 integration + 5 QMK + 11 doc tests)
- [x] Manual testing of keyboard display
- [x] Manual testing of layer creation
- [x] Build succeeds with no warnings

## Implementation Summary

**Commits:**
- `9ce0b33` - feat: add TapHoldInfo parsing and mod-tap display methods to KeycodeDb
- `7a434c6` - refactor: use KeycodeDb.parse_tap_hold() for tap-hold keycode display
- `fe3b92b` - refactor: use KeycodeDb for layer keycode detection (Phase 2)
- `9fa2a57` - refactor: use ColorPalette for default layer colors (Phase 3)

**Key Changes:**
- Added `TapHoldType` enum and `TapHoldInfo` struct to KeycodeDb
- Added `parse_tap_hold()`, `get_mod_tap_display()` methods
- Added `is_layer_keycode()`, `parse_layer_keycode()`, `get_simple_layer_prefixes()`, `get_compound_layer_prefixes()` methods
- Added `get_color()`, `get_shade()`, `default_layer_color()` to ColorPalette
- Removed hardcoded arrays from keyboard.rs and regex patterns from layout.rs
- Updated FirmwareGenerator and Layout signatures to accept KeycodeDb
