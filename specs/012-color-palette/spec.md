# Spec 012: Enhanced Color Picker with Palette

## Overview

Enhance the color picker dialog to include a curated color palette with shades, making it faster and easier to select consistent, visually pleasing colors for keyboard LEDs.

## Current State

The existing color picker (`src/tui/color_picker.rs`) provides:
- RGB channel sliders (0-255)
- Color preview
- Hex code display

**Pain Points:**
- Selecting colors requires manual RGB adjustment
- No preset colors for quick selection
- Hard to pick consistent, harmonious colors

## Goal

Add a two-step color selection flow:
1. **Palette Mode**: Select from curated base colors with shade variations
2. **Custom RGB Mode**: Fine-tune with existing RGB sliders (optional)

## Design (Concept 3: Sequential Flow)

```
┌─────────────────── Color Picker ────────────────────┐
│                                                     │
│  Step 1: Choose Base Color                          │
│  ┌─────────────────────────────────────────────────┐│
│  │ ● Red      ● Orange   ● Yellow   ● Lime        ││
│  │ ● Green    ● Teal     ● Cyan     ● Blue        ││
│  │ ● Purple   ● Pink     ● White    ● Gray        ││
│  └─────────────────────────────────────────────────┘│
│                                                     │
│  Step 2: Choose Shade                               │
│  ┌─────────────────────────────────────────────────┐│
│  │  ██   ██   ██  [██]  ██   ██   ██   ██   ██    ││
│  │  50  100  200  300  400  500  600  700  800    ││
│  └─────────────────────────────────────────────────┘│
│                                                     │
│  Preview: ████████████████  #EF4444                 │
│                                                     │
│  ←→↑↓ Navigate  c Custom RGB  Enter Apply  Esc    │
└─────────────────────────────────────────────────────┘
```

## Color Palette

Based on Tailwind CSS colors, stored in `src/data/color_palette.json`:

**Base Colors (12):**
- Red, Orange, Yellow, Lime
- Green, Teal, Cyan, Blue  
- Purple, Pink, White, Gray

**Shades per color (9):**
- 50, 100, 200, 300, 400, 500, 600, 700, 800

## Data Model

### ColorPalette (new)
```rust
pub struct ColorPalette {
    pub colors: Vec<PaletteColor>,
}

pub struct PaletteColor {
    pub name: String,        // "Red", "Blue", etc.
    pub shades: Vec<Shade>,  // 9 shades from light to dark
}

pub struct Shade {
    pub level: u16,          // 50, 100, 200, etc.
    pub hex: String,         // "#FEE2E2"
    pub rgb: RgbColor,       // (254, 226, 226)
}
```

### ColorPickerState (updated)
```rust
pub struct ColorPickerState {
    // Existing RGB fields
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub active_channel: RgbChannel,
    
    // New palette fields
    pub mode: ColorPickerMode,
    pub selected_color_idx: usize,  // 0-11 (base color)
    pub selected_shade_idx: usize,  // 0-8 (shade level)
}

pub enum ColorPickerMode {
    Palette,   // Selecting from palette
    CustomRgb, // Fine-tuning with RGB sliders
}
```

## Keyboard Controls

### Palette Mode
| Key | Action |
|-----|--------|
| `←` `→` | Navigate colors horizontally |
| `↑` `↓` | Navigate color rows / shades |
| `Enter` | Apply selected color |
| `c` | Switch to Custom RGB mode |
| `Esc` | Cancel |

### Custom RGB Mode
| Key | Action |
|-----|--------|
| `←` `→` | Adjust value ±1 |
| `↑` `↓` | Adjust value ±10 |
| `Tab` | Next channel |
| `Shift+Tab` | Previous channel |
| `p` | Switch back to Palette mode |
| `Enter` | Apply color |
| `Esc` | Cancel |

## Implementation Phases

### Phase 1: Color Palette Data
- [ ] Create `src/data/color_palette.json` with Tailwind colors
- [ ] Create `src/models/color_palette.rs` with data structures
- [ ] Load palette at startup
- [ ] Add unit tests for palette loading

### Phase 2: Update ColorPickerState
- [ ] Add `ColorPickerMode` enum
- [ ] Add palette selection fields
- [ ] Update `ColorPickerState` with mode switching
- [ ] Sync RGB values when selecting from palette

### Phase 3: Render Palette UI
- [ ] Render base color grid (4x3) with colored dots and names
- [ ] Render shade bar for selected color
- [ ] Highlight selected color and shade
- [ ] Show preview and hex code

### Phase 4: Handle Input
- [ ] Arrow key navigation in palette
- [ ] Mode switching (`c` for custom, `p` for palette)
- [ ] Apply color on Enter

### Phase 5: Update Help Overlay
- [ ] Document new color picker controls

## File Changes

| File | Change |
|------|--------|
| `src/data/color_palette.json` | New - color palette data |
| `src/models/color_palette.rs` | New - palette data structures |
| `src/models/mod.rs` | Export ColorPalette |
| `src/tui/color_picker.rs` | Update - add palette mode |
| `src/tui/mod.rs` | Load palette, pass to state |
| `src/tui/help_overlay.rs` | Update help text |

## Testing

- Unit tests for palette loading
- Unit tests for palette navigation
- Manual testing of color selection flow
