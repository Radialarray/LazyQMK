# Lesson 0007: Configure Features (Phase 7)

> Enable and tune 8 feature groups. Each is opt-in. End: MISSION.md "must-have features" all checked.

## Goal

- All MISSION.md "must-have features" enabled
- Settings match user preferences
- Each feature validated independently

## 6 Feature Branches (in this lesson)

| # | Feature | Reference |
|---|---|---|
| 1 | RGB master (always) | `references/0004-rgb-and-idle.md` |
| 2 | Idle effect / PaletteFX screensaver | `references/0004-rgb-and-idle.md`, `references/0005-ripple-and-palettefx.md` |
| 3 | Ripple overlay (keypress feedback) | `references/0005-ripple-and-palettefx.md` |
| 4 | Tap-hold (LT/MT/TT behavior) | `references/0006-tap-hold-and-combos.md` |
| 5 | Combos (up to 32, 3 hardcoded actions) | `references/0006-tap-hold-and-combos.md` |
| 6 | Tap dance (TD(name) actions) | `references/0007-tap-dance.md` |

Categories + semantic palette are covered in `lessons/0005-semantic-categories.md` (Phase 5, already done by this phase).
Output format / keymap identity is handled in Phase 3 (`lessons/0003-pick-keyboard.md`).

## User Questions

For each feature branch the user wants, ask only what matters (defaults otherwise):

| Feature | Ask | Default |
|---|---|---|
| Idle effect | "Breathing / Rainbow / Solid / other?" | breathing, 60s timeout, 5min duration |
| PaletteFX | "Want PaletteFX community module for idle?" | disabled |
| Ripple | "Want ripple on keypress? Fixed color or key-based?" | disabled |
| Tap-hold | "Home-row mods? (use HomeRowMods preset)" | Default preset |
| Combos | "Want bootloader combo? Which two keys?" | disabled |
| Tap dance | "Any specific tap dance? (esc_caps, shift_ctrl, slash variant)" | none |

## Phase 7: Steps

### Feature 1: RGB Master (always)

Already set during lesson 0003 (skeleton). Verify:

```bash
LAYOUT=~/.../my.json

jq '{rgb_enabled, rgb_brightness, rgb_saturation, rgb_matrix_default_speed, rgb_timeout_ms, uncolored_key_behavior}' "$LAYOUT"
```

Recommended defaults:

```bash
jq '
  .rgb_enabled = true |
  .rgb_brightness = 100 |
  .rgb_saturation = 200 |
  .rgb_matrix_default_speed = 127 |
  .rgb_timeout_ms = 0 |
  .uncolored_key_behavior = 40
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Feature 2: Idle Effect / PaletteFX

#### Option A: Standard idle effect (no PaletteFX)

```bash
LAYOUT=~/.../my.json

jq '
  .idle_effect_settings = {
    enabled: true,
    idle_timeout_ms: 60000,
    idle_effect_duration_ms: 300000,
    idle_effect_mode: "breathing"
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Other 8 modes (total 9 incl. breathing): `solid_color`, `rainbow_moving_chevron`, `cycle_all`, `cycle_left_right`, `cycle_up_down`, `rainbow_beacon`, `rainbow_pinwheels`, `jellybean_raindrops`.

#### Option B: PaletteFX community module

```bash
LAYOUT=~/.../my.json

jq '
  .palette_fx = {
    enabled: true,
    default_effect: "flow",
    default_palette: "synthwave",
    enable_all_effects: true,
    enable_all_palettes: true
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Effects: `gradient`, `flow`, `ripple`, `sparkle`, `vortex`, `reactive`.
Palettes: `afterburn`, `amber`, `bad_wolf`, `carnival`, `classic`, `dracula`, `groovy`, `not_pink`, `phosphor`, `polarized`, `rose_gold`, `sport`, `synthwave`, `thermal`, `viridis`, `watermelon`.

When `palette_fx.enabled = true`, `idle_effect_settings` is automatically overridden with the PaletteFX effect during firmware generation.

### Feature 3: Ripple Overlay

```bash
LAYOUT=~/.../my.json

jq '
  .rgb_overlay_ripple = {
    enabled: true,
    max_ripples: 4,
    duration_ms: 800,
    speed: 200,
    band_width: 25,
    amplitude_pct: 60,
    wave_count: 1,
    wave_delay_ms: 100,
    color_mode: "key_based",
    fixed_color: {r: 0, g: 255, b: 255},
    hue_shift_deg: 60,
    trigger_on_press: true,
    trigger_on_release: false,
    ignore_transparent: true,
    ignore_modifiers: true,
    ignore_layer_switch: false
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Color modes: `fixed` (use `fixed_color`), `key_based` (use key's base color), `hue_shift` (rotate by `hue_shift_deg`).

### Feature 4: Tap-Hold (5 presets)

```bash
LAYOUT=~/.../my.json

# HomeRowMods preset (recommended for productivity)
jq '
  .tap_hold_settings = {
    tapping_term: 175,
    quick_tap_term: 120,
    hold_mode: "PermissiveHold",
    retro_tapping: true,
    tapping_toggle: 5,
    flow_tap_term: 150,
    chordal_hold: true,
    preset: "HomeRowMods"
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Other presets: `Default`, `Responsive`, `Deliberate`, `Custom`.

### Feature 5: Combos (up to 32, per-action)

Up to 32 combos per layout. Each combo picks its own action independently
(`disable_effects`, `disable_lighting`, `bootloader`) — there is no longer
a slot-bound mapping. Both keys (key1, key2) and `hold_duration_ms` are
configurable per combo. Combo keys are rendered on the TUI and WebUI
keyboard previews with per-action color and 1-char badge.

Find visual coordinates of two keys the user wants to use:

```bash
LAYOUT=~/.../my.json

# Show all layer 0 keys
jq '.layers[0].keys | map({pos: .position, kc: .keycode})' "$LAYOUT"

# Find specific keycode's position
jq --arg kc "KC_W" '.layers[0].keys | map(select(.keycode == $kc)) | .[0].position' "$LAYOUT"
# Returns: {row: 0, col: 2} for a Corne
```

Add a single bootloader combo (W+E held 800ms):

```bash
LAYOUT=~/.../my.json

jq '
  .combo_settings = {
    enabled: true,
    combos: [
      {
        key1: {row: 0, col: 2},
        key2: {row: 0, col: 3},
        action: {type: "bootloader"},
        hold_duration_ms: 800
      }
    ]
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Add multiple combos of mixed actions (any combo action can appear in any
slot; up to 32 total):

```bash
LAYOUT=~/.../my.json

jq '
  .combo_settings = {
    enabled: true,
    combos: [
      { key1: {row:0,col:0}, key2: {row:0,col:1}, action: {type:"disable_effects"},  hold_duration_ms: 500 },
      { key1: {row:1,col:0}, key2: {row:1,col:1}, action: {type:"disable_lighting"}, hold_duration_ms: 500 },
      { key1: {row:0,col:2}, key2: {row:0,col:3}, action: {type:"bootloader"},        hold_duration_ms: 1000 }
    ]
  }
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

`action` is internally tagged (snake_case): `{"type": "disable_effects"}`,
`{"type": "disable_lighting"}`, or `{"type": "bootloader"}`. Don't include
`placeholder` — it's `#[serde(skip)]` and never appears in JSON. Slot numbers
on disk (`**Combo N**`) are 1-indexed; parser gaps are tolerated and don't
emit phantom combos.

Note: the action fires on key RELEASE after the elapsed duration (not while
keys remain held). User instruction: hold both keys for `hold_duration_ms`
then release.

**Bootloader fallback removed**: there is no longer a hidden `Q+R`/`U+P`
1500ms bootloader combo tied to idle effect. If the user wants a bootloader
shortcut, add it as a normal combo with action `bootloader`.

### Feature 6: Tap Dance

```bash
# Add tap dance definition
lazyqmk tap-dance add --layout ~/.../my.json \
  --name esc_caps \
  --single KC_ESC \
  --double KC_CAPS

# Apply TD(esc_caps) to a key
LAYOUT=~/.../my.json
jq --argjson layer 0 --argjson row 0 --argjson col 0 '
  .layers[$layer].keys |= map(
    if .position.row == $row and .position.col == $col
    then .keycode = "TD(esc_caps)" else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"

# Validate
lazyqmk tap-dance validate --layout ~/.../my.json
```

Common patterns:

| Name | Single | Double | Hold | Use |
|---|---|---|---|---|
| esc_caps | KC_ESC | KC_CAPS | — | Dual-role Esc/Caps |
| shift_caps | KC_LSFT | KC_CAPS | — | Avoid pinky shift |
| slash_bsls | KC_SLSH | KC_BSLS | — | Code editing |
| bracket_pair | KC_LBRC | KC_RBRC | KC_LSFT | Code editing |

### Feature 8: Output Format / Keymap Identity

Set during Phase 3 (lesson 0003). Verify:

```bash
jq '{keymap_name: .metadata.keymap_name, output_format: .metadata.output_format}' ~/.../my.json
```

If user wants to change (NOT recommended — changes firmware identity):

```bash
jq '.metadata.keymap_name = "new_name" | .metadata.output_format = "hex"' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

## Validation

After every feature enable:

```bash
# Layout still validates
lazyqmk validate --layout ~/.../my.json

# Tap dance references resolve (if any)
lazyqmk tap-dance validate --layout ~/.../my.json

# Layer references resolve
lazyqmk layer-refs --layout ~/.../my.json

# Settings summary
lazyqmk inspect --layout ~/.../my.json --section settings
```

## Workspace Update

Add to NOTES.md:

```markdown
## Features enabled
- [x] RGB master: brightness 100%, saturation 200%, speed 127
- [x] Idle effect: breathing, 60s timeout, 5min duration
- [ ] PaletteFX: disabled
- [ ] Ripple overlay: disabled
- [x] Tap-hold preset: HomeRowMods
- [x] Combo: Bootloader on W+E (800ms)
- [x] Tap dance: esc_caps on (0,0)
```

## Common Pitfalls

- **PaletteFX requires community modules** — uses `getreuer/palettefx` via `keymap.json` `modules[]`. Custom QMK fork must be used.
- **Custom ripple requires custom QMK fork** — official QMK doesn't have the ripple code.
- **Combo keys must differ** — `key1 == key2` rejected
- **Tap dance name must be valid C identifier** — alphanumeric + underscore only
- **Tap dance `single_tap = KC_NO`** — auto-created placeholder; define explicitly before generating
- **RGB master switch doesn't gate idle/PaletteFX/ripple** — setting `rgb_enabled: false` only blacks base layer colors. Idle/PaletteFX/ripple generation is gated on `geometry.has_rgb_matrix()`, NOT on `rgb_enabled`. To fully disable RGB behavior, set all four (`rgb_enabled`, `idle_effect_settings.enabled`, `palette_fx.enabled`, `rgb_overlay_ripple.enabled`) to false.

## Next Lesson

`lessons/0008-validate-and-generate.md` (Phase 8) — final validation + generate QMK source.