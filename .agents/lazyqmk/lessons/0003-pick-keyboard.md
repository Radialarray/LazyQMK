# Lesson 0003: Pick Keyboard (Phase 3)

> Discover QMK keyboards and variants. Pick one. Write MISSION.md. End: `layout_variant`, `keyboard`, `keymap_name` decided.

## Goal

- Concrete `keyboard`, `layout_variant`, `keymap_name` chosen
- `MISSION.md` written
- Empty skeleton `.json` layout file created

## Discovery Gate (silent)

```bash
# 1. List available keyboards (filter by user's known vendor if they've said)
lazyqmk list-keyboards --qmk-path ~/qmk_firmware --json | jq '.keyboards[]' | head -30

# 2. If user mentioned a specific keyboard, get its variants
# e.g., user said "corne" or "keebart/corne_choc_pro"
lazyqmk list-layouts --qmk-path ~/qmk_firmware --keyboard keebart/corne_choc_pro --json
```

## User Questions

| Ask | Why |
|---|---|
| "Which keyboard are you setting up?" | Hardware fact; cannot infer |
| "Which variant?" | If multiple exist (rare); user picks from options |
| "What keymap name do you want?" | Identity of firmware; affects filename |
| "Output format: uf2 / hex / bin?" | Determines build target (uf2 = RP2040 standard) |

If user is unsure, list options you found. **Never** ask "what vendor" or "do you have a qmk_firmware directory" — those are inferrable.

## Phase 3: Steps

### 1. Identify keyboard

Either:

```bash
# User knows exact path
KEYBOARD="keebart/corne_choc_pro"

# Or: list and filter
lazyqmk list-keyboards --qmk-path ~/qmk_firmware --filter "corne" --json
# Pick from output, e.g., "keebart/corne_choc_pro"
```

### 2. List layout variants

```bash
lazyqmk list-layouts --qmk-path ~/qmk_firmware --keyboard "$KEYBOARD" --json
```

Example output for `keebart/corne_choc_pro`:

```json
{
  "keyboard": "keebart/corne_choc_pro",
  "layouts": [
    { "name": "LAYOUT_split_3x5_3", "key_count": 36 },
    { "name": "LAYOUT_split_3x5_3_ex2", "key_count": 40 },
    { "name": "LAYOUT_split_3x6_3", "key_count": 42 },
    { "name": "LAYOUT_split_3x6_3_ex2", "key_count": 46 }
  ]
}
```

Common picks:

- 36 keys → Ferris Sweep, Corne mini, etc.
- 40 keys → 36-key Corne with EX column (one extra key per half)
- 42 keys → standard Corne, Lily58
- 46 keys → 42-key Corne with EX2 column (encoder + extra key per half)

Ask user which key count matches their hardware.

### 3. Get geometry (optional, but recommended)

```bash
lazyqmk geometry --qmk-path ~/qmk_firmware --keyboard "$KEYBOARD" --layout-name "$LAYOUT_VARIANT" --json | head -50
```

Useful to see row/col mapping and LED assignments. Skip if user just wants to proceed.

### 4. Choose keymap name

Ask user. Convention: `<keyboard-slug>_keymap` (e.g., `corne_choc_pro_keymap`, `my_corne`, `default_keymap`). Use generic placeholders; never use personal identifiers.

Avoid spaces and special chars — used in filenames.

### 5. Write MISSION.md

Create workspace folder and MISSION.md (use embedded template from SKILL.md):

```bash
WORKSPACE_DIR=~/Library/Application\ Support/LazyQMK/workspaces/<slug>
mkdir -p "$WORKSPACE_DIR/lessons"

# Write MISSION.md (use template from SKILL.md)
# Replace placeholders:
#   - <keyboard-name>: e.g., "Corne Choc Pro"
#   - <vendor/keyboard/variant>: e.g., "keebart/corne_choc_pro/standard"
#   - <LAYOUT_*>: e.g., "LAYOUT_split_3x6_3_ex2"
#   - N: key count
```

### 6. Write NOTES.md (initial)

Use template from SKILL.md. Fill in detected prefs (theme from `config.toml`, language from user, etc.).

### 7. Write RESOURCES.md

Use template from SKILL.md. Fill in paths to LazyQMK docs in the repo.

### 8. Create empty layout skeleton

**Recommended approach**: copy an existing working layout from `examples/` (e.g., `examples/corne_choc_pro_layout.json`) and adapt it. The example file already has the correct schema, all default settings, and 46 properly-positioned keys for `LAYOUT_split_3x6_3_ex2`.

```bash
# Copy the example for the chosen keyboard/variant
cp ~/dev/LazyQMK/examples/corne_choc_pro_layout.json \
   ~/Library/Application\ Support/LazyQMK/layouts/<keymap_name>.json

# Edit metadata (jq for safe edits)
LAYOUT_FILE=~/Library/Application\ Support/LazyQMK/layouts/<keymap_name>.json
jq --arg name "<keymap_name>" --arg kb "<vendor>/<keyboard>" --arg variant "LAYOUT_*" '
  .metadata.name = $name
  | .metadata.keymap_name = $name
  | .metadata.keyboard = $kb
  | .metadata.layout_variant = $variant
  | .metadata.created = (now | todate)
  | .metadata.modified = (now | todate)
  | .metadata.is_template = false
' "$LAYOUT_FILE" > "$LAYOUT_FILE.new"
mv "$LAYOUT_FILE.new" "$LAYOUT_FILE"
```

If no example exists for the user's keyboard, the alternative is to:

1. Pick any existing user layout as a starting point (`~/.../layouts/*.json`)
2. Modify the `metadata.keyboard`, `metadata.layout_variant`, `metadata.keymap_name`, and `layers[].keys[]` positions to match the new keyboard
3. Add or remove `keys` to match the new key count

For an exact empty skeleton, here is the schema (every field shown; populate `layers[0].keys` with `KEY_COUNT` entries of `"keycode": "KC_TRNS"`):

```json
{
  "metadata": {
    "name": "<keymap_name>",
    "description": "",
    "author": "",
    "created": "<ISO 8601 timestamp>",
    "modified": "<ISO 8601 timestamp>",
    "tags": [],
    "is_template": false,
    "version": "1.0",
    "layout_variant": "LAYOUT_<variant>",
    "keyboard": "<vendor>/<keyboard>",
    "keymap_name": "<keymap_name>",
    "output_format": "uf2"
  },
  "layers": [
    {
      "id": "<uuid-here>",
      "number": 0,
      "name": "Base",
      "default_color": { "r": 212, "g": 212, "b": 212 },
      "category_id": null,
      "keys": [],
      "layer_colors_enabled": true
    }
  ],
  "categories": [],
  "rgb_enabled": true,
  "rgb_brightness": 100,
  "rgb_saturation": 200,
  "rgb_matrix_default_speed": 127,
  "rgb_timeout_ms": 0,
  "uncolored_key_behavior": 100,
  "idle_effect_settings": {
    "enabled": true,
    "idle_timeout_ms": 60000,
    "idle_effect_duration_ms": 300000,
    "idle_effect_mode": "breathing"
  },
  "rgb_overlay_ripple": {
    "enabled": false,
    "max_ripples": 4,
    "duration_ms": 1500,
    "speed": 200,
    "band_width": 30,
    "amplitude_pct": 50,
    "wave_count": 1,
    "wave_delay_ms": 100,
    "color_mode": "fixed",
    "fixed_color": { "r": 0, "g": 255, "b": 255 },
    "hue_shift_deg": 60,
    "trigger_on_press": true,
    "trigger_on_release": false,
    "ignore_transparent": true,
    "ignore_modifiers": false,
    "ignore_layer_switch": false
  },
  "palette_fx": {
    "enabled": false,
    "default_effect": "flow",
    "default_palette": "synthwave",
    "enable_all_effects": true,
    "enable_all_palettes": true
  },
  "tap_hold_settings": {
    "tapping_term": 200,
    "quick_tap_term": null,
    "hold_mode": "Default",
    "retro_tapping": false,
    "tapping_toggle": 5,
    "flow_tap_term": null,
    "chordal_hold": false,
    "preset": "Default"
  },
  "combo_settings": {
    "enabled": false,
    "combos": []
  },
  "tap_dances": []
}
```

Generate the layer's `keys[]` array with correct `(row, col)` for the chosen LAYOUT variant. For 42-key Corne (LAYOUT_split_3x6_3), visual positions are:

```text
row 0: cols 0-5 (left alpha) + cols 9-14 (right alpha)   = 12 keys
row 1: cols 0-5 (left home) + cols 9-14 (right home)      = 12 keys
row 2: cols 0-5 (left lower) + cols 9-14 (right lower)    = 12 keys
row 3: col 6 (left thumb inner) + col 8 (right thumb inner) = 2 keys
row 4: cols 4-5 (left thumb) + cols 9-10 (right thumb)     = 4 keys
                                                        total = 42 keys
```

For other keyboards, use `lazyqmk geometry --keyboard <kb> --layout-name <variant>` to determine positions.

Use jq to generate the keys array (note: split keyboards have a "gap" between cols 5 and 9):

```bash
LAYOUT_FILE=~/Library/Application\ Support/LazyQMK/layouts/<name>.json
KEY_COUNT=42

# Build keys array — split Corne has cols 0-5 left, gap at 6-8, then 9-14 right
jq --argjson kc "$KEY_COUNT" '
  def empty_key: {position: {row: 0, col: 0}, keycode: "KC_TRNS", label: null, color_override: null, category_id: null, combo_participant: false};
  .layers[0].keys = (
    ([range(0; 6)  | . as $c | empty_key | .position = {row: 0, col: $c}]
     + [range(9; 15) | . as $c | empty_key | .position = {row: 0, col: $c}])
    + ([range(0; 6)  | . as $c | empty_key | .position = {row: 1, col: $c}]
       + [range(9; 15) | . as $c | empty_key | .position = {row: 1, col: $c}])
    + ([range(0; 6)  | . as $c | empty_key | .position = {row: 2, col: $c}]
       + [range(9; 15) | . as $c | empty_key | .position = {row: 2, col: $c}])
    + ([empty_key | .position = {row: 3, col: 6}, empty_key | .position = {row: 3, col: 8}])
    + ([range(4; 6)  | . as $c | empty_key | .position = {row: 4, col: $c}]
       + [range(9; 11) | . as $c | empty_key | .position = {row: 4, col: $c}])
  )
' "$LAYOUT_FILE" > "$LAYOUT_FILE.new"
mv "$LAYOUT_FILE.new" "$LAYOUT_FILE"
```

For `LAYOUT_split_3x6_3_ex2` (46 keys), the encoder EX columns add 4 extra positions (verified against `qmk_firmware/keyboards/keebart/corne_choc_pro/info.json`): `(1,6)`, `(1,8)`, `(2,6)`, `(2,8)`. Row 0 has no EX extras.

```bash
KEY_COUNT=46
# Add 4 EX keys at encoder positions (rows 1 and 2, cols 6 and 8):
#   + [empty_key | .position = {row: 1, col: 6}, empty_key | .position = {row: 1, col: 8}]
#   + [empty_key | .position = {row: 2, col: 6}, empty_key | .position = {row: 2, col: 8}]
```

### 9. Save initial layout

```bash
LAYOUT_FILE=~/Library/Application\ Support/LazyQMK/layouts/<name>.json
# (skeleton written above)

# Validate
bash scripts/validate-and-report.sh "$LAYOUT_FILE"
# Expect: "✓ validation passed"
```

**Note**: `lazyqmk validate` (without QMK path) uses an estimated square geometry and may flag valid split-keyboard positions as out-of-bounds. For a fully accurate check, run `lazyqmk generate` with the real QMK path (Phase 8) — the generator-side validator uses real QMK geometry. The CLI `validate` is a quick sanity check only.

## Validation

```bash
# Layout validates
lazyqmk validate --layout "$LAYOUT_FILE"

# MISSION.md exists
test -f ~/Library/Application\ Support/LazyQMK/workspaces/<slug>/MISSION.md

# Has correct metadata
jq '{name: .metadata.name, keyboard: .metadata.keyboard, layout_variant: .metadata.layout_variant, keymap_name: .metadata.keymap_name}' \
  "$LAYOUT_FILE"
```

## Workspace Update

MISSION.md, NOTES.md, RESOURCES.md all written.

Add to NOTES.md:

```markdown
## Key decisions
- YYYY-MM-DD: Selected keyboard <vendor/kb>, variant <LAYOUT_*>, keymap <name>
```

## Next Lesson

`lessons/0004-plan-layers.md` (Phase 4) — decide which layers to create.