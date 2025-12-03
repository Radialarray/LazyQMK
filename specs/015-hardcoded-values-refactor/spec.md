# Spec 015: Hardcoded Values Refactor

## Overview

Refactor hardcoded keycodes and color codes throughout the codebase to use the centralized keycode database (`keycode_db`) and color palette (`color_palette.json`) instead.

## Problem Statement

Several parts of the codebase contain hardcoded keycode patterns and color values that duplicate information already available in our JSON databases:

1. **Keyboard display logic** (`src/tui/keyboard.rs`) has hardcoded mod-tap prefixes and tap-hold pattern matching
2. **Layer keycode detection** (`src/models/layout.rs`) uses hardcoded regex patterns for layer keycodes
3. **Default colors** are hardcoded as `RgbColor::new(128, 128, 128)` instead of using the color palette

This duplication:
- Makes maintenance harder (changes need to be made in multiple places)
- Risks inconsistency between display and validation logic
- Doesn't leverage the rich metadata in our JSON databases

## Scope

### In Scope

- Replace hardcoded mod-tap prefix array in `keyboard.rs` with keycode_db lookup
- Refactor tap-hold keycode parsing to use keycode_db patterns
- Extract layer keycode prefixes from `layers.json`
- Create named color constants from the color palette

### Out of Scope

- Special keycodes (`KC_NO`, `KC_TRNS`, `_______`, `XXXXXXX`) - these are fundamental QMK conventions
- TUI theme colors in `theme.rs` - these are intentionally separate from RGB matrix colors
- Test file hardcoded values - acceptable for testing

## Implementation Phases

### Phase 1: Keyboard Display (keyboard.rs) - High Priority

**Files:** `src/tui/keyboard.rs`, `src/keycode_db/mod.rs`

1. Add method to `KeycodeDb` to get display abbreviation for mod-tap prefixes
2. Replace `mod_tap_prefixes` array (lines 397-411) with keycode_db lookup
3. Refactor `format_tap_hold()` to use keycode_db pattern matching instead of hardcoded `starts_with`

**Current hardcoded array:**
```rust
let mod_tap_prefixes = [
    ("LCTL_T(", "CTL"),
    ("RCTL_T(", "CTL"),
    ("LSFT_T(", "SFT"),
    // ... 13 entries total
];
```

### Phase 2: Layer Keycode Detection (layout.rs) - Medium Priority

**Files:** `src/models/layout.rs`, `src/keycode_db/mod.rs`

1. Add method to `KeycodeDb` to get layer-related keycode prefixes
2. Build regex patterns dynamically from database
3. Replace hardcoded regex at lines 807-830

**Current hardcoded patterns:**
```rust
r"^(MO|TG|TO|DF|OSL|TT)\("   // Simple layer keycodes
r"^(LT|LM)\("                 // Compound layer keycodes
```

### Phase 3: Default Colors - Medium Priority

**Files:** `src/models/color_palette.rs`, `src/main.rs`, `src/tui/mod.rs`, `src/tui/color_picker.rs`

1. Add helper method to get named colors from palette (e.g., Neutral-400 for default gray)
2. Create constants like `DEFAULT_LAYER_COLOR`
3. Replace hardcoded `RgbColor::new(128, 128, 128)` calls

## Success Criteria

- [ ] No hardcoded mod-tap prefix arrays in `keyboard.rs`
- [ ] Tap-hold parsing uses keycode_db patterns
- [ ] Layer keycode detection uses database-derived prefixes
- [ ] Default colors reference the color palette
- [ ] All existing tests pass
- [ ] Display behavior unchanged (visual regression test)

## Dependencies

- `src/keycode_db/categories/mod_tap.json` - mod-tap keycode definitions
- `src/keycode_db/categories/layers.json` - layer keycode definitions
- `src/data/color_palette.json` - color palette with named colors

## Risks

- **Performance**: Building regex patterns at startup vs compile-time - mitigate with lazy initialization
- **Breaking display**: Need careful testing to ensure display abbreviations match current behavior
