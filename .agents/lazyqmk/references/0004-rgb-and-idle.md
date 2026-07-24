# Reference 0004: RGB and Idle Effect

> Master RGB switch + brightness + saturation + speed + timeout + idle screensaver. The idle effect is the centerpiece feature — keys sit idle → effect plays → LEDs off (3-state).

## Master RGB Settings (JSON)

```json
{
  "rgb_enabled": true,
  "rgb_brightness": 100,
  "rgb_saturation": 200,
  "rgb_matrix_default_speed": 127,
  "rgb_timeout_ms": 60000,
  "uncolored_key_behavior": 40
}
```

| Setting | Type | Range | Default | Description |
|---|---|---|---|---|
| `rgb_enabled` | bool | — | `true` | Master switch. False = all LEDs off (firmware + display). |
| `rgb_brightness` | u8 (percent) | 0–100 | 100 | Global brightness multiplier (applied after saturation). |
| `rgb_saturation` | u8 (percent) | 0–200 | 100 | Saturation. 100 = original, 200 = max saturation, 0 = grayscale. |
| `rgb_matrix_default_speed` | u8 | 0–255 | 127 | Animation speed for effects. 0 = slowest, 255 = fastest. **127 (default) is NOT written to `config.h`** — only emitted when changed. |
| `rgb_timeout_ms` | u32 (ms) | 0 = disabled | 0 | Auto-off after inactivity. 60000 = 1 min. Conflicts with idle effect (see below). |
| `uncolored_key_behavior` | u8 (percent) | 0–100 | 100 | Display brightness for keys with no specific color. See `references/0003-color-system.md`. |

## Conflicting Settings: `rgb_timeout_ms` vs `idle_effect_settings`

These overlap:

- `rgb_timeout_ms` is QMK's built-in "timeout" — turn off LEDs after N ms idle.
- `idle_effect_settings` is LazyQMK's enhanced 3-state system: Active → Idle Effect → Off.

**When `idle_effect_settings.enabled = true`** AND the keyboard has RGB, `rgb_timeout_ms` is NOT emitted in `config.h` (mutually exclusive with `LQMK_IDLE_*` defines). Set `rgb_timeout_ms` is ignored at firmware compile time.

**When `idle_effect_settings.enabled = false`** (and keyboard has RGB), `rgb_timeout_ms > 0` emits `#define RGB_MATRIX_TIMEOUT <ms>` and provides simple auto-off without the 3-state system.

**`idle_timeout_ms: 0` does NOT disable the idle effect** — the generated state machine checks `elapsed >= 0`, which triggers immediately on first scan. To actually disable, set `enabled: false`.

## Idle Effect (3-State System)

```json
{
  "idle_effect_settings": {
    "enabled": true,
    "idle_timeout_ms": 60000,
    "idle_effect_duration_ms": 300000,
    "idle_effect_mode": "breathing"
  }
}
```

State machine:

```
Active ──(idle_timeout_ms)──> Idle Effect ──(idle_effect_duration_ms)──> Off
   ▲                              │
   └────(any keypress)────────────┘
```

| Setting | Type | Default | Description |
|---|---|---|---|
| `enabled` | bool | `true` | Master switch for the 3-state system. |
| `idle_timeout_ms` | u32 (ms) | 60000 (1 min) | Time before transitioning from Active → Idle Effect. **0 = triggers immediately** (not disabled). |
| `idle_effect_duration_ms` | u32 (ms) | 300000 (5 min) | How long the Idle Effect plays before transitioning to Off. 0 = immediate off. |
| `idle_effect_mode` | enum | `breathing` | Which RGB effect plays during Idle (see below). |

## Idle Effect Modes (9 options, from `RgbMatrixEffect` enum)

The `idle_effect_mode` accepts the following string values (snake_case; must match `serde` rename exactly):

| Value | QMK macro |
|---|---|
| `solid_color` | `RGB_MATRIX_SOLID_COLOR` |
| `breathing` | `RGB_MATRIX_BREATHING` (default) |
| `rainbow_moving_chevron` | `RGB_MATRIX_RAINBOW_MOVING_CHEVRON` |
| `cycle_all` | `RGB_MATRIX_CYCLE_ALL` |
| `cycle_left_right` | `RGB_MATRIX_CYCLE_LEFT_RIGHT` |
| `cycle_up_down` | `RGB_MATRIX_CYCLE_UP_DOWN` |
| `rainbow_beacon` | `RGB_MATRIX_RAINBOW_BEACON` |
| `rainbow_pinwheels` | `RGB_MATRIX_RAINBOW_PINWHEELS` |
| `jellybean_raindrops` | `RGB_MATRIX_JELLYBEAN_RAINDROPS` |

When `idle_effect_mode = "solid_color"`, the generator selects `RGB_MATRIX_SOLID_COLOR` (uses current RGB Matrix HSV state — not layer 0's `default_color`).

## PaletteFX Integration

If `palette_fx.enabled = true`, LazyQMK automatically registers the `getreuer/palettefx` QMK community module and overrides the `idle_effect_mode` with the chosen PaletteFX effect (see `references/0005-ripple-and-palettefx.md`).

The PaletteFX idle mode is **additive** — your layer colors still show during Active, PaletteFX only plays during Idle.

## Generated `config.h` (what actually gets emitted)

```c
// RGB Matrix Maximum Brightness (0-255, derived from 0-100% input — only if < 255)
#define RGB_MATRIX_MAXIMUM_BRIGHTNESS 200

// RGB Matrix Default Animation Speed (only if != 127)
#define RGB_MATRIX_DEFAULT_SPD 200

// Idle Effect (when enabled)
#define LQMK_IDLE_TIMEOUT_MS 120000          // always emitted when enabled
#define LQMK_IDLE_EFFECT_DURATION_MS 300000  // always emitted when enabled
#define LQMK_IDLE_EFFECT_MODE RGB_MATRIX_BREATHING

// RGB Matrix Timeout (only when idle effect disabled and rgb_timeout_ms > 0)
#define RGB_MATRIX_TIMEOUT 60000

// Default mode (only when keyboard has RGB + custom colors)
#define RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_TUI_LAYER_COLORS
#define LAYER_BASE_COLORS_LAYER_COUNT 6
```

Notes on actual emission:
- **Brightness** uses `RGB_MATRIX_MAXIMUM_BRIGHTNESS` (0–255), NOT `RGB_MATRIX_DEFAULT_VAL`.
- **Saturation** and **uncolored_key_behavior** are NOT emitted as `#define`s — they're baked into the generated color table in `keymap.c` via `apply_rgb_settings`.
- **Idle timeout/duration** are always emitted (when enabled), regardless of whether they match defaults.
- When `rgb_enabled = false`, base layer colors are emitted as black — but `idle_effect_settings`, `palette_fx`, and `rgb_overlay_ripple` generation are NOT gated on `rgb_enabled`. They are gated on `geometry.has_rgb_matrix()`. So an RGB keyboard with `rgb_enabled: false` AND `idle_effect_settings.enabled: true` will compile, with black base colors but a working idle animation. To truly disable all RGB behavior, disable all four features explicitly (see below).

## CLI Operations

```bash
# Read RGB settings
jq '.rgb_enabled, .rgb_brightness, .rgb_saturation, .rgb_matrix_default_speed, .rgb_timeout_ms, .uncolored_key_behavior' \
  ~/.../my.json

# Read idle effect
jq '.idle_effect_settings' ~/.../my.json
```

## Recipes (jq one-liners)

### Disable RGB entirely (safer)

`rgb_enabled = false` only blacks base layer colors in firmware. Idle effect, PaletteFX, and ripple generation are gated on `geometry.has_rgb_matrix()` and `palette_fx.enabled`, NOT on `rgb_enabled`. To truly disable all RGB behavior, set ALL of:

```bash
jq '
  .rgb_enabled = false
  | .rgb_timeout_ms = 0
  | .idle_effect_settings.enabled = false
  | .palette_fx.enabled = false
  | .rgb_overlay_ripple.enabled = false
' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Set 200% saturation, 80% brightness, 200 speed

```bash
jq '.rgb_saturation = 200 | .rgb_brightness = 80 | .rgb_matrix_default_speed = 200' \
  ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Enable idle effect with breathing, 2 min timeout, 10 min duration

```bash
jq '.idle_effect_settings = {
  enabled: true,
  idle_timeout_ms: 120000,
  idle_effect_duration_ms: 600000,
  idle_effect_mode: "breathing"
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Disable idle effect (fall back to plain rgb_timeout)

```bash
jq '.idle_effect_settings.enabled = false' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

Note: setting `idle_timeout_ms: 0` does NOT disable the idle effect — the generator uses `elapsed >= 0` which triggers immediately. Use `enabled: false` to actually disable.

### Dim uncolored keys to 30%

```bash
jq '.uncolored_key_behavior = 30' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

## When to Use RGB Timeout vs Idle Effect

| Use case | Setting |
|---|---|
| Simple auto-off after N minutes | `rgb_timeout_ms: 60000`, `idle_effect_settings.enabled: false` |
| Nice screensaver then off | `idle_effect_settings.enabled: true` with `mode: breathing` |
| PaletteFX screensaver | `idle_effect_settings.enabled: true` + `palette_fx.enabled: true` |
| LEDs always on, never auto-off | `rgb_timeout_ms: 0`, `idle_effect_settings.enabled: false` |

## Tips for Polished RGB Setup

1. **`rgb_saturation: 200`** — oversaturated colors look better on most RGB matrices
2. **`uncolored_key_behavior: 30–50`** — dim layer-default keys so category-colored keys pop
3. **Idle effect `breathing`** — gentle, default, looks good on any keyboard
4. **Idle timeout 60–120s** — long enough to not interrupt short pauses
5. **Duration 5–10 min** — long enough to enjoy, short enough to actually turn off
6. **Speed 127 (default)** — usually fine; increase to 200+ for faster animation

## Common Pitfalls

- **`rgb_matrix_default_speed = 127` not emitted to `config.h`** — intentional; only non-default values written. To customize, use 128 or 126.
- **Idle effect gated on RGB matrix presence, NOT on `rgb_enabled`** — the generator (`src/firmware/generator/idle.rs:20`) emits idle code only if `geometry.has_rgb_matrix()`. Setting `rgb_enabled = false` does NOT prevent idle/PaletteFX generation; you must also set `idle_effect_settings.enabled = false` and `palette_fx.enabled = false`.
- **Idle-disabled compile bug**: if the keyboard has RGB but `idle_effect_settings.enabled = false`, generated `keymap.c` may fail to compile because `housekeeping_task_user()` references `bootloader_combo_active/timer` globals that are only emitted by the idle code. Workaround: enable idle, OR use a non-RGB keyboard.
- **`idle_timeout_ms: 0` does NOT disable the effect** — the state machine checks `elapsed >= 0` which triggers immediately. Use `enabled: false` to actually disable.
- **RGB timeout and idle effect are exclusive** — see "Conflicting Settings" above.
- **Custom QMK fork required** — official QMK won't compile the idle effect state machine. Use the Radialarray fork.
- **`inspect --section settings` reports `idle_effect_mode` in PascalCase** (e.g., `Breathing`, `CycleAll`) due to `Debug` formatting, while the layout JSON uses snake_case (`breathing`, `cycle_all`). When scripting, compare by snake_case value, not display name.