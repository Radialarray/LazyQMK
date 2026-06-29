# RGB Overlay Ripple — Feature Documentation

## What it does

A reactive keypress effect: pressing a key emits a soft pulse of light that expands outward from the key's LED position. Multiple concentric waves can be emitted per keypress with staggered timing, creating a water-ripple cascading effect.

Ripple renders as an **additive overlay** on top of your TUI layer colors — it does not replace them. Each wave uses the originating key's base color.

## Algorithm

### Soft pulse (not hard ring)

The core rendering uses a **soft radial pulse** rather than a discrete ring:

- **Distance**: Manhattan (`|drow| + |dcol|`) in matrix space — avoids `g_led_config.point` initialization issues on some keyboards
- **Wavefront**: expands from center (radius 0) to `LQMK_RIPPLE_MAX_RADIUS` over `LQMK_RIPPLE_DURATION_MS`
- **Gradient**: brightness peaks at the current wavefront, then fades linearly in **both directions** — inward (toward center) and outward (toward edge)
- **Fade width**: proportional to max radius (`max_radius / 2`), ensuring the pulse band is narrower than the total span so movement is visible

```
Visual representation of a single pulse at different radii:

  Radius 0:          Radius 2 (peak):      Radius 4:
  center → edge      center → edge         center → edge
  ████ ██ ░░ · ·     ░░ ██ ████ ██ ░░      · · ░░ ██ ████
```

### Brightness envelope

Each ripple has a triangular amplitude envelope (`0→255→0`) over its duration, peaking at the midpoint. This creates a natural fade-in/fade-out:

```
amp
255 ┤     ╱╲
    ┤    ╱  ╲
    ┤   ╱    ╲
  0 ┤──╱      ╲──
    0ms  half   duration
```

### Speed-aware expansion

The expansion rate formula:

```
radius = (elapsed × max_radius × speed) / (duration × 200)
```

At the default speed of 200, this simplifies to `(elapsed × max_radius) / duration` — linear expansion. Higher speed values accelerate the wavefront, lower values decelerate it.

### Multi-wave cascading

When `wave_count > 1`, each keypress spawns multiple ripples at staggered intervals. Each wave gets a `trigger_delay_ms = wave_index × wave_delay_ms`, enforced by skipping the ripple in the render loop until enough time has passed.

```
Wave 0: ████░░░░····  (0→500ms)
Wave 1: ····████░░░░  (400→900ms)
Wave 2: ········████  (800→1300ms)
```

## Settings

All settings are available in both **TUI** (Settings Manager → RGB → Ripple subgroup) and **WebUI** (Settings panel).

| Setting | Range | Default | Description |
|---------|-------|---------|-------------|
| **Ripple Overlay** | On/Off | Off | Master switch |
| **Max Concurrent Ripples** | 1–8 | 4 | How many ripples can run at once |
| **Ripple Duration** | 1–5000ms | 1500ms | How long each ripple lasts |
| **Ripple Speed** | 1–255 | 200 | Expansion rate multiplier |
| **Ripple Band Width** | 1–255 | 30 | Band width (legacy, now computed from amplitude) |
| **Ripple Amplitude** | 0–100% | 50% | Controls max expansion radius (2–6 matrix units) |
| **Ripple Waves per Key** | 1–5 | 1 | How many concentric waves per keypress |
| **Delay Between Waves** | 50–500ms | 100ms | Stagger between consecutive waves |
| **Ripple Color Mode** | Fixed / Key Color / Hue Shift | Fixed | How ripple color is chosen |
| **Ripple Fixed Color** | RGB | Cyan (#00FFFF) | Color when mode = Fixed |
| **Ripple Hue Shift** | -180° to 180° | 60° | Hue offset when mode = Hue Shift |
| **Trigger on Press** | On/Off | On | Emit ripple on key down |
| **Trigger on Release** | On/Off | Off | Emit ripple on key up |
| **Ignore Transparent** | On/Off | On | Skip KC_TRNS keys |
| **Ignore Modifiers** | On/Off | Off | Skip modifier keys |
| **Ignore Layer Switch** | On/Off | Off | Skip layer-switch keys |

## Development Journey

### Bug 1: Amp always zero (critical)
**Symptom**: Ripple completely invisible.
**Root cause**: `255 / (LQMK_RIPPLE_DURATION_MS / 2 + 1)` → integer division collapsed to 0 for any duration > 508ms.
**Fix**: Pre-compute `LQMK_RIPPLE_AMP_SCALE = (255 × 256) / half_duration` for `scale16by8`.

### Bug 2: Speed unused
**Symptom**: `LQMK_RIPPLE_SPEED` defined in config.h but never referenced in ripple code.
**Fix**: Pre-compute `LQMK_RIPPLE_SCALED_DURATION = duration × 200` and use `(elapsed × max_radius × speed) / scaled_duration`.

### Bug 3: brightness undefined
**Symptom**: Would cause compilation error for layouts without custom layer colors.
**Fix**: Added `uint8_t brightness = rgb_matrix_get_val();` in non-background code paths.

### Iteration: Hard ring → soft pulse
**Symptom**: Ripple looked jagged and flickered.
- **Attempt 1**: Switched from Manhattan to Chebyshev distance → still jagged
- **Attempt 2**: Widened ring thickness to 5 → ring wider than max radius, entire area lit, looked like blinking
- **Solution**: Replaced the hard ring (`if dist < radius: skip`) with a soft pulse — brightness peaks at the wavefront and fades smoothly in both directions. Fade width is `max_radius / 2` so the pulse band is always narrower than the total span.

### Iteration: Multi-wave timing
**Symptom**: Waves overlapped too much, looked like flickering not cascading.
**Fix**: Used visual timing diagrams to find the right stagger — delay ≈ 80% of duration for clean separation between waves.

### Communication pattern
ASCII timing diagrams proved effective for discussing wave timing:

```
Wave 0: ████░░░░····  (0→500ms, fades by 400ms)
Wave 1: ····████░░░░  (400→900ms, starts cleanly as wave 0 disappears)
```

## Generated C Code

### Key structures
- `ripple_t` — per-ripple state (origin LED, pixel position, start time, trigger delay, active flag)
- `ripples[]` — array of up to `LQMK_RIPPLE_MAX_RIPPLES` active ripples
- `lazyqmk_reactive_apply()` — called per-LED per-frame from `rgb_matrix_indicators_advanced_user`
- `lazyqmk_ripple_trigger()` — called from `process_record_user` to spawn ripples on key events

### Config.h defines
```c
#define LQMK_RIPPLE_OVERLAY_ENABLED
#define LQMK_RIPPLE_MAX_RIPPLES 4
#define LQMK_RIPPLE_DURATION_MS 500
#define LQMK_RIPPLE_SPEED 200
#define LQMK_RIPPLE_BAND_WIDTH 20
#define LQMK_RIPPLE_AMPLITUDE_PCT 60
#define LQMK_RIPPLE_MAX_RADIUS 4
#define LQMK_RIPPLE_FADE_WIDTH 2
#define LQMK_RIPPLE_SCALED_DURATION 100000UL
#define LQMK_RIPPLE_AMP_SCALE 255
#define LQMK_RIPPLE_WAVE_COUNT 1
#define LQMK_RIPPLE_WAVE_DELAY_MS 400
```

## Files

| File | Purpose |
|------|---------|
| `src/models/layout.rs` | `RgbOverlayRippleSettings` struct + validation |
| `src/firmware/generator.rs` | C code generation (keymap.c + config.h) |
| `src/parser/template_gen.rs` | Markdown round-trip writer |
| `src/parser/layout.rs` | Markdown round-trip parser |
| `src/tui/settings_manager.rs` | TUI settings enum, display, help text |
| `src/tui/handlers/settings.rs` | TUI numeric/toggle handlers |
| `src/web/mod.rs` | Web API DTO |
| `docs/RIPPLE_VALIDATION.md` | Manual validation checklist |
