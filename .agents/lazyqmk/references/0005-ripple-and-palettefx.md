# Reference 0005: Ripple Overlay and PaletteFX

> Two RGB feature systems: ripple (custom, keypress-reactive) and PaletteFX (community module, idle screensaver). They can coexist — ripple for press feedback, PaletteFX for idle.

## Ripple Overlay (custom)

An expanding ring of light radiates from each keypress, layered on top of base layer colors.

### JSON Structure

```json
{
  "rgb_overlay_ripple": {
    "enabled": true,
    "max_ripples": 4,
    "duration_ms": 500,
    "speed": 200,
    "band_width": 20,
    "amplitude_pct": 60,
    "wave_count": 1,
    "wave_delay_ms": 400,
    "color_mode": "key_based",
    "fixed_color": { "r": 0, "g": 255, "b": 255 },
    "hue_shift_deg": 60,
    "trigger_on_press": true,
    "trigger_on_release": false,
    "ignore_transparent": true,
    "ignore_modifiers": true,
    "ignore_layer_switch": false
  }
}
```

### Ripple Settings

| Setting | Type | Range | Default | Description |
|---|---|---|---|---|
| `enabled` | bool | — | `false` | Master switch |
| `max_ripples` | u8 | 1–8 | 4 | Concurrent ripples allowed |
| `duration_ms` | u16 (ms) | 1–5000 (any nonzero) | 1500 | How long each ripple lasts |
| `speed` | u8 | 1–255 | 200 | Expansion speed in 2D pixel space |
| `band_width` | u8 | 1+ physical distance | 30 | Ring thickness (declared but not directly referenced in generator — see notes below) |
| `amplitude_pct` | u8 (%) | 0–100 | 50 | Controls the maximum radius in the generated wavefront (NOT a direct brightness multiplier) |
| `wave_count` | u8 | 1–5 | 1 | Concentric waves per press |
| `wave_delay_ms` | u16 (ms) | 50–500 | 100 | Delay between waves |
| `trigger_on_press` | bool | — | `true` | Fire on keydown |
| `trigger_on_release` | bool | — | `false` | Fire on keyup |
| `ignore_transparent` | bool | — | `true` | Skip `KC_TRNS` keys |
| `ignore_modifiers` | bool | — | `false` | Skip Shift/Ctrl/Alt/GUI |
| `ignore_layer_switch` | bool | — | `false` | Skip MO/LT/TG keys |

Caveats from inspecting the generator:
- `LQMK_RIPPLE_BAND_WIDTH` is emitted but not directly used in the wavefront calculation.
- `amplitude_pct` controls generated maximum radius, NOT a brightness boost. To make ripples visually brighter, increase `speed` or `duration_ms` instead.

### Ripple Color Mode (3 options)

| Mode | JSON value | Description | Extra setting |
|---|---|---|---|
| Fixed Color | `fixed` | All ripples use one fixed color | `fixed_color` (RGB) |
| Key Color | `key_based` | Ripple matches the key's base layer color | — |
| Hue Shift | `hue_shift` | Hue-shifted from key's base color | `hue_shift_deg` (-180 to 180) |

### Recommended Ripple Settings

```json
{
  "enabled": true,
  "max_ripples": 4,
  "duration_ms": 800,
  "speed": 200,
  "band_width": 25,
  "amplitude_pct": 60,
  "wave_count": 1,
  "color_mode": "key_based",
  "trigger_on_press": true,
  "ignore_transparent": true,
  "ignore_modifiers": true
}
```

Tunable: increase `amplitude_pct` (60 → 80) for more visible ripples on dim backgrounds; increase `duration_ms` (800 → 1500) for slower, more dramatic rings.

## PaletteFX (community module)

Drives the **idle screensaver** with 6 animated effects and 16 color palettes. Separate from ripple — PaletteFX handles idle, ripple handles keypress feedback.

### JSON Structure

```json
{
  "palette_fx": {
    "enabled": true,
    "default_effect": "flow",
    "default_palette": "synthwave",
    "enable_all_effects": true,
    "enable_all_palettes": true
  }
}
```

| Setting | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `false` | Master switch. Enables `getreuer/palettefx` community module. |
| `default_effect` | enum | `flow` | Which effect plays during idle (see table below) |
| `default_palette` | enum | `synthwave` | Which palette is active (see table below) |
| `enable_all_effects` | bool | `true` | Compile in all 6 effects (vs individual selection) |
| `enable_all_palettes` | bool | `true` | Compile in all 16 palettes (vs individual selection) |

### PaletteFX Effects (6)

| Value | Name | Description |
|---|---|---|
| `gradient` | Gradient | Vertical color gradient |
| `flow` | Flow | Animated wave patterns (default) |
| `ripple` | Ripple | Circular rings emanating from random points |
| `sparkle` | Sparkle | LEDs sparkle with pseudorandom phase |
| `vortex` | Vortex | Spinning vortex centered on keyboard |
| `reactive` | Reactive | Responds to key presses |

### PaletteFX Palettes (16)

| Value | Name | Aesthetic |
|---|---|---|
| `afterburn` | Afterburn | Fiery oranges/reds |
| `amber` | Amber | Warm amber tones |
| `bad_wolf` | Bad Wolf | Dark theme-inspired |
| `carnival` | Carnival | Vibrant rainbow |
| `classic` | Classic | Traditional color wheel |
| `dracula` | Dracula | Dark purple/pink theme |
| `groovy` | Groovy | Retro warm tones |
| `not_pink` | Not Pink | Pink/magenta tones |
| `phosphor` | Phosphor | Green/teal glow (terminal vibes) |
| `polarized` | Polarized | Blue/cyan tones |
| `rose_gold` | Rose Gold | Warm rose tones |
| `sport` | Sport | Athletic team colors |
| `synthwave` | Synthwave | Retrowave purple/pink/cyan (default) |
| `thermal` | Thermal | Heat map colors |
| `viridis` | Viridis | Perceptually uniform green/blue/purple |
| `watermelon` | Watermelon | Pink/green tones |

### Generated `config.h`

When `palette_fx.enabled = true`:

```c
// PaletteFX Community Module Configuration
#define PALETTEFX_ENABLE_ALL_EFFECTS    // or individual effect enables
#define PALETTEFX_ENABLE_ALL_PALETTES   // or individual palette enables

// Idle Effect Configuration (uses PaletteFX as screensaver)
#define LQMK_IDLE_TIMEOUT_MS 120000
#define LQMK_IDLE_EFFECT_DURATION_MS 300000
#define LQMK_IDLE_EFFECT_MODE RGB_MATRIX_COMMUNITY_MODULE_PALETTEFX_FLOW

// Default mode always TUI layer colors
#define RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_TUI_LAYER_COLORS
```

When PaletteFX is enabled, `idle_effect_mode` is **automatically** overridden with the PaletteFX default effect — you don't need to set `idle_effect_settings.idle_effect_mode` separately.

### Generated `keymap.json`

When `palette_fx.enabled = true`:

```json
{
  "modules": ["getreuer/palettefx"]
}
```

This registers the community module with QMK's module system.

## Ripple + PaletteFX Together

They serve different purposes and coexist:

| State | What plays |
|---|---|
| Active (typing) | Layer colors + ripple on keypress |
| Idle (no typing) | PaletteFX effect (flow / vortex / sparkle / etc.) |
| Off (long idle) | LEDs off |

Both can be enabled simultaneously. Ripple affects Active, PaletteFX affects Idle. No conflict.

## PaletteFX `default_palette` vs Runtime Palette

`default_palette` only matters when `enable_all_palettes = false` — then it's the only palette compiled into the firmware. When `enable_all_palettes = true`, all 16 palettes are compiled and the active palette is chosen at runtime by PaletteFX's own state machine (not by `default_palette`).

**`default_effect` is different**: regardless of `enable_all_effects`, `default_effect.qmk_mode_name()` is what gets written to `LQMK_IDLE_EFFECT_MODE` in `config.h` (when PaletteFX is enabled and idle effect is also enabled). So changing `default_effect` always changes which effect plays during idle.

## Optional `key_action_palette`

The ripple settings (NOT PaletteFX settings) include an optional `key_action_palette: Option<PaletteFxPalette>` field. When `Some(palette)`, it overrides the active PaletteFX palette during ripple bursts. Only effective when `palette_fx.enabled = true`. Default: `null` (use current palette).

## CLI Operations

```bash
# Read ripple settings
jq '.rgb_overlay_ripple' ~/.../my.json

# Read palettefx settings
jq '.palette_fx' ~/.../my.json
```

## Recipes

### Enable ripple with key-color mode

```bash
jq '.rgb_overlay_ripple = {
  enabled: true,
  max_ripples: 4,
  duration_ms: 800,
  speed: 200,
  band_width: 25,
  amplitude_pct: 60,
  wave_count: 1,
  wave_delay_ms: 100,
  color_mode: "key_based",
  fixed_color: { r: 0, g: 255, b: 255 },
  hue_shift_deg: 60,
  trigger_on_press: true,
  trigger_on_release: false,
  ignore_transparent: true,
  ignore_modifiers: true,
  ignore_layer_switch: false
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Enable PaletteFX with flow + synthwave

```bash
jq '.palette_fx = {
  enabled: true,
  default_effect: "flow",
  default_palette: "synthwave",
  enable_all_effects: true,
  enable_all_palettes: true
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Disable ripple, keep PaletteFX

```bash
jq '.rgb_overlay_ripple.enabled = false' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Disable PaletteFX, keep ripple

```bash
jq '.palette_fx.enabled = false' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

## When to Use What

| Aesthetic | Setup |
|---|---|
| Minimal | `palette_fx.enabled: false`, `rgb_overlay_ripple.enabled: false` |
| Subtle press feedback | `ripple: enabled, key_based, amplitude 40` |
| Dramatic press feedback | `ripple: enabled, fixed color cyan, amplitude 80, duration 1500` |
| Idle screensaver (no press feedback) | `palette_fx: enabled, flow, synthwave`; `ripple: disabled` |
| Full experience | Both enabled (recommended) |

## Common Pitfalls

- **PaletteFX requires community modules** — the custom QMK fork must be used (Radialarray's). The official QMK repo won't compile.
- **`palettefx/reactive` is NOT the same as custom ripple** — PaletteFX's reactive replaces all LED colors with a static bump; our custom ripple is an expanding wavefront overlay that preserves base colors. Use custom ripple for press feedback.
- **Custom QMK fork required** for ripple — official QMK won't compile the ripple code.
- **Ripple + idle effect** — ripple fires on press during Active; doesn't fire during Idle/Off. This is correct behavior.
- **`amplitude_pct: 0` = invisible ripple** — keep at 30+ for visible feedback.
- **`duration_ms: 0` rejected by validate** — minimum 1.
- **`wave_delay_ms < 50` rejected** — minimum 50ms to prevent flicker.