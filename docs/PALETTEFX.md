# PaletteFX — Palette-Based Idle Screensaver

> **Status:** Implemented v0.21.0
> **Module:** [getreuer/palettefx](https://github.com/getreuer/qmk-modules/tree/main/palettefx) (Apache 2.0)
> **Requires:** QMK 0.28+, community modules support

> **Vendored:** This feature depends on `getreuer/palettefx`, which lazyqmk
> vendors in-tree at `qmk_firmware/modules/getreuer/palettefx/` (Apache 2.0).
> See the root `NOTICE` file for attribution and pinned version.
> No internet access or external `qmk module update` is required to build.

## Concept

PaletteFX provides 6 animated RGB effects driven by 16 curated color palettes. LazyQMK integrates it as an **idle screensaver** — it plays a PaletteFX animation when the keyboard sits idle, then restores your normal per-key layer colors when you press a key.

This is NOT a replacement for the default RGB mode or the reactive key-action burst. Your carefully configured TUI layer colors remain the primary display at all times.

## What Uses PaletteFX vs. Custom

| Feature | Implementation |
|---------|----------------|
| **Idle screensaver** | PaletteFX community module (the Flow/Vortex/Sparkle/Gradient animations) |
| **Reactive key-action burst** | **Custom implementation** (expanding wavefront ring, additive overlay, per-key originating color) |

The reactive key-action burst started as a port of PaletteFX's Reactive algorithm but diverged to better fit the overlay use case:

- **PaletteFX Reactive** is a full RGB matrix mode that *replaces* all LED colors. We needed an *additive overlay* that preserves the TUI base colors.
- **PaletteFX Reactive** has a fixed-radius static bump. We needed an *expanding wavefront ring* that travels outward from the key.
- **PaletteFX Reactive** uses one global palette color. We needed *per-LED contributions* from multiple in-range ripples, each colored by its own originating key's TUI color.

The custom implementation uses the same mathematical shape (radial falloff, amplitude envelope) but with these overlay-specific behaviors.

## Architecture

### Three Operating Modes

| State | What you see | Trigger |
|-------|-------------|---------|
| **Active** | TUI layer colors (per-key colors from your layout) | Normal typing |
| **Idle** | PaletteFX effect (e.g., Flow with Synthwave palette) | No keypress for N seconds |
| **Off** | LEDs off | Idle effect duration expires |

### State Machine

```
Active ──(idle_timeout)──→ Idle (PaletteFX) ──(duration)──→ Off
  ↑                            │
  └────(any keypress)──────────┘
```

The state machine is the same one already used for the standard idle effect. When PaletteFX is enabled at compile time, the idle effect mode is automatically set to the PaletteFX default effect instead of a standard `RGB_MATRIX_*` effect.

### What PaletteFX Does NOT Do

- ❌ Does NOT replace `RGB_MATRIX_DEFAULT_MODE` (always `RGB_MATRIX_TUI_LAYER_COLORS` when custom colors are configured)
- ❌ Does NOT interfere with ripple overlay (ripple still provides keypress feedback independently)
- ❌ Does NOT change behavior while keyboard is active (only activates during idle)

### Generated Code

**`config.h`** — Compilation flags:
```c
// PaletteFX Community Module Configuration
#define PALETTEFX_ENABLE_ALL_EFFECTS    // Compile in all 6 effects
#define PALETTEFX_ENABLE_ALL_PALETTES   // Compile in all 16 palettes

// Idle Effect Configuration (uses PaletteFX as screensaver)
#define LQMK_IDLE_TIMEOUT_MS 120000
#define LQMK_IDLE_EFFECT_DURATION_MS 300000
#define LQMK_IDLE_EFFECT_MODE RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_FLOW

// Default mode is always TUI layer colors
#define RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_TUI_LAYER_COLORS
```

**`keymap.json`** — Module registration:
```json
{
  "modules": ["getreuer/palettefx"]
}
```

**`keymap.c`** — State machine logic:
```c
// On keypress during idle → restore TUI_LAYER_COLORS
rgb_matrix_mode_noeeprom(RGB_MATRIX_TUI_LAYER_COLORS);
idle_state = IDLE_STATE_ACTIVE;
```

## Configuration

### In Markdown Layout (Settings section)

```yaml
## Settings

**PaletteFX**: On
**PaletteFX Default Effect**: Flow        # Gradient | Flow | Ripple | Sparkle | Vortex | Reactive
**PaletteFX Default Palette**: Synthwave  # Afterburn | Amber | Bad Wolf | Carnival | Classic | Dracula | Groovy | Not Pink | Phosphor | Polarized | Rose Gold | Sport | Synthwave | Thermal | Viridis | Watermelon
**PaletteFX All Effects**: On             # On = compile all 6 effects, Off = compile only default
**PaletteFX All Palettes**: On            # On = compile all 16 palettes, Off = compile only default
```

### In TUI (Shift+S → Lighting Behavior → PaletteFX effects)

- **PaletteFX Effects** — On/Off master switch
- **PaletteFX Default Effect** — Select from 6 effects
- **PaletteFX Default Palette** — Select from 16 palettes
- **PaletteFX All Effects** — Compile all effects (vs. only default)
- **PaletteFX All Palettes** — Compile all palettes (vs. only default)

### In WebUI (Layout → PaletteFX tab)

Same set of controls in a dedicated tab.

## Effects

| Effect | Description |
|--------|-------------|
| Gradient | Smooth color transitions across the keyboard |
| Flow | Flowing wave-like animation |
| Ripple | Rippling rings from center |
| Sparkle | Twinkling LED sparkles |
| Vortex | Rotating color vortex |
| Reactive | Responds to key presses |

## Palettes

Afterburn, Amber, Bad Wolf, Carnival, Classic, Dracula, Groovy, Not Pink, Phosphor, Polarized, Rose Gold, Sport, Synthwave, Thermal, Viridis, Watermelon

## Setup

PaletteFX requires the getreuer/qmk-modules repository to be available:

```bash
cd qmk_firmware
mkdir -p modules
git submodule add https://github.com/getreuer/qmk-modules.git modules/getreuer
git submodule update --init --recursive
```

## Implementation Details

### Files Changed

| File | Purpose |
|------|---------|
| `src/models/layout.rs` | `PaletteFxEffect`, `PaletteFxPalette`, `PaletteFxSettings` types |
| `src/parser/layout.rs` | Markdown parsing of PaletteFX settings |
| `src/firmware/generator.rs` | Config.h defines, idle effect mode override, keymap.json |
| `src/tui/settings_manager.rs` | Settings UI in TUI |
| `src/tui/handlers/settings.rs` | Settings input handling |
| `src/web/mod.rs` | REST API DTOs and conversion |
| `web/src/lib/api/types.ts` | TypeScript types |
| `web/src/routes/layouts/[name]/+page.svelte` | WebUI tab |

### Key Design Decision

PaletteFX is **exclusively an idle screensaver** — it never replaces the default RGB mode. This preserves the core LazyQMK value proposition: you configure per-key layer colors in the TUI, and those colors are what you see during normal use. PaletteFX adds an animated interlude when you step away.

This is in contrast to a "replace ripple" approach where PaletteFX would be the primary mode. Such an approach would defeat the purpose of a visual keymap editor.
