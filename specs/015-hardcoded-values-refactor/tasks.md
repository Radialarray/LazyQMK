# Tasks: Hardcoded Values Refactor

## Phase 1: Keyboard Display (keyboard.rs)

- [ ] 1.1 Add `get_mod_tap_display_name()` method to KeycodeDb
  - Returns short display name (e.g., "CTL", "SFT") for mod-tap prefixes
  - Use data from `mod_tap.json`

- [ ] 1.2 Add `is_tap_hold_keycode()` method to KeycodeDb
  - Detect if a keycode is a tap-hold type using database patterns
  - Returns the type (LT, MT, mod-tap shorthand, LM, SH_T)

- [ ] 1.3 Refactor `format_tap_hold()` in keyboard.rs
  - Remove hardcoded `mod_tap_prefixes` array
  - Use KeycodeDb methods for pattern detection and display names

- [ ] 1.4 Test keyboard display behavior
  - Verify mod-tap keys display correctly
  - Verify LT, MT, LM, SH_T display correctly

## Phase 2: Layer Keycode Detection (layout.rs)

- [ ] 2.1 Add `get_layer_keycode_prefixes()` method to KeycodeDb
  - Return list of simple layer prefixes (MO, TG, TO, DF, OSL, TT)
  - Return list of compound layer prefixes (LT, LM)

- [ ] 2.2 Refactor layer detection in layout.rs
  - Build regex patterns from database-provided prefixes
  - Replace hardcoded regex patterns

- [ ] 2.3 Test layer reference detection
  - Verify all layer keycode types are detected
  - Run existing layer tests

## Phase 3: Default Colors

- [ ] 3.1 Add `get_neutral_color()` helper to ColorPalette
  - Get named neutral colors (gray shades) from palette
  - E.g., `get_neutral_color(400)` returns Neutral-400

- [ ] 3.2 Define default color constants
  - `DEFAULT_LAYER_COLOR` - Neutral-400 (gray)
  - `DISABLED_LAYER_COLOR` - Neutral-600 (dark gray)
  - `OFF_COLOR` - Black (0,0,0) - may keep as-is

- [ ] 3.3 Replace hardcoded RgbColor::new() calls
  - `src/main.rs:207`
  - `src/tui/mod.rs:3632`
  - `src/tui/color_picker.rs:864`
  - `src/models/layout.rs:762`

## Verification

- [ ] All existing tests pass
- [ ] Manual testing of keyboard display
- [ ] Manual testing of layer creation
- [ ] Manual testing of color picker reset
