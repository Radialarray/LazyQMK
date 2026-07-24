# Reference 0007: Tap Dance

> Tap dance actions fire different keycodes based on tap count and (optionally) hold. Two-way (single/double) or three-way (single/double/hold).

## Tap Dance Structure (JSON)

```json
{
  "tap_dances": [
    {
      "name": "esc_caps",
      "single_tap": "KC_ESC",
      "double_tap": "KC_CAPS",
      "hold": null
    },
    {
      "name": "shift_ctrl",
      "single_tap": "KC_LSFT",
      "double_tap": "KC_CAPS",
      "hold": "KC_LCTL"
    }
  ]
}
```

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string (C identifier) | yes | Unique name. Used in `TD(name)` references. Must be alphanumeric + underscore. |
| `single_tap` | string (keycode) | yes | Keycode sent on single tap |
| `double_tap` | string (keycode) | optional | Keycode sent on double tap. None = repeat single tap |
| `hold` | string (keycode) | optional | Keycode sent on hold. None = no hold action |

## Reference via Keycode

```c
TD(esc_caps)            // In any layer
TD(shift_ctrl)          // In any layer
```

Place `TD(name)` in any key position. The tap dance action is triggered based on how many times the key is tapped within `tapping_term`.

## Validation Rules

- `name` must be non-empty, alphanumeric + underscore only (valid C identifier)
- `single_tap` is required
- `double_tap` (if present) must be non-empty
- `hold` (if present) must be non-empty
- Every `TD(name)` reference must have a matching definition
- Definitions with no references = warning (orphan)

## 2-Way vs 3-Way

| Type | single_tap | double_tap | hold |
|---|---|---|---|
| Single (1-way) | yes | — | — |
| Two-way | yes | yes | no |
| Three-way | yes | yes | yes |

| Type | Generated macro |
|---|---|
| Single | `ACTION_TAP_DANCE_DOUBLE` with single repeated |
| Two-way | `ACTION_TAP_DANCE_DOUBLE` |
| Three-way | `ACTION_TAP_DANCE_FN_ADVANCED` |

LazyQMK chooses the macro automatically based on which fields are set.

## Auto-Creation on Parse

If a keycode references `TD(foo)` but no `foo` definition exists, LazyQMK auto-creates a placeholder on parse:

```json
{ "name": "foo", "single_tap": "KC_NO", "double_tap": null, "hold": null }
```

**Caveat**: `lazyqmk tap-dance validate` will NOT flag the placeholder as an issue — it sees a defined tap dance. The only way to detect a placeholder is to check whether `single_tap == "KC_NO"` for any defined tap dance. Add this check to your validation workflow:

```bash
# Find tap dances that are still placeholders (never edited after auto-create)
jq -r '.tap_dances[] | select(.single_tap == "KC_NO") | .name' ~/.../my.json
```

A non-empty result means those tap dances won't do anything useful in firmware. Edit them before generating.

## Use Cases

| Pattern | Example | Why |
|---|---|---|
| Esc / Caps Lock | single=KC_ESC, double=KC_CAPS | Convenient dual-role |
| Shift / Caps Lock | single=KC_LSFT, double=KC_CAPS | Avoid pinky |
| Slash / Backslash | single=KC_SLSH, double=KC_BSLS | Code editing |
| Bracket pair | single=KC_LBRC, double=KC_RBRC, hold=KC_LSFT | Code editing |
| One-shot modifier + tap | single=KC_SPC, hold=KC_LSFT | Spacebar as Shift when held |

## CLI Operations

```bash
# List all tap dances
lazyqmk tap-dance list --layout <path> [--json]
# Returns: { count, tap_dances: [{name, single_tap, double_tap, hold, type}] }
# type: "single" | "two_way" | "three_way"

# Add (modifies layout file)
lazyqmk tap-dance add --layout <path> \
  --name esc_caps \
  --single KC_ESC \
  --double KC_CAPS

# Add with hold
lazyqmk tap-dance add --layout <path> \
  --name shift_ctrl \
  --single KC_LSFT \
  --double KC_CAPS \
  --hold KC_LCTL

# Delete (refuses if referenced)
lazyqmk tap-dance delete --layout <path> --name esc_caps [--force]
# --force replaces all TD(esc_caps) references with KC_TRNS

# Validate (check for orphan refs + unused defs)
lazyqmk tap-dance validate --layout <path> [--json]
# Returns: { valid, orphaned: [...refs without defs], unused: [...defs not used] }
```

## Recipes (jq)

### Add a tap dance directly

```bash
jq '.tap_dances += [{
  name: "esc_caps",
  single_tap: "KC_ESC",
  double_tap: "KC_CAPS",
  hold: null
}]' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Apply TD to a key

```bash
LAYOUT=~/.../my.json
TD_NAME=esc_caps
jq --argjson layer 0 --argjson row 0 --argjson col 0 --arg td "$TD_NAME" '
  .layers[$layer].keys |= map(
    if .position.row == $row and .position.col == $col
    then .keycode = ("TD(" + $td + ")") else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Find all TD() references in the layout

```bash
jq -r '
  .layers[].keys[]
  | select(.keycode | test("^TD\\("))
  | .keycode
' ~/.../my.json | sort -u
```

### Delete a tap dance (clean removal)

```bash
TD_NAME=esc_caps
LAYOUT=~/.../my.json

# Remove definition
jq --arg td "$TD_NAME" '
  .tap_dances |= map(select(.name != $td))
' "$LAYOUT" > "$LAYOUT.new"

# Replace all TD(name) references with KC_TRNS
jq --arg td "$TD_NAME" '
  .layers[].keys |= map(
    if .keycode == ("TD(" + $td + ")") then .keycode = "KC_TRNS" else . end
  )
' "$LAYOUT.new" > "$LAYOUT.new2"
mv "$LAYOUT.new2" "$LAYOUT"
rm "$LAYOUT.new"
```

## Generated `keymap.c`

```c
// 2-way tap dance (single + double)
tap_dance_action_t tap_dance_actions[] = {
    [TD_ESC_CAPS] = ACTION_TAP_DANCE_DOUBLE(KC_ESC, KC_CAPS),
    [TD_SLASH] = ACTION_TAP_DANCE_DOUBLE(KC_NO, KC_NO),  // placeholder
};

// 3-way tap dance (single + double + hold) requires custom helpers:
void td_shift_ctrl_finished(tap_dance_state_t *state, void *user_data) {
    if (state->count == 1) {
        if (state->interrupted || !state->pressed) {
            register_code16(KC_LSFT);
        } else {
            register_code16(KC_LCTL);
        }
    } else if (state->count == 2) {
        register_code16(KC_CAPS);
    }
}

void td_shift_ctrl_reset(tap_dance_state_t *state, void *user_data) {
    if (state->count == 1) {
        unregister_code16(KC_LSFT);
        unregister_code16(KC_LCTL);
    } else if (state->count == 2) {
        unregister_code16(KC_CAPS);
    }
}

tap_dance_action_t tap_dance_actions[] = {
    [TD_SHIFT_CTRL] = ACTION_TAP_DANCE_FN_ADVANCED(
        NULL, td_shift_ctrl_finished, td_shift_ctrl_reset
    ),
};
```

Uses QMK built-ins `ACTION_TAP_DANCE_DOUBLE` (2-way) and `ACTION_TAP_DANCE_FN_ADVANCED` (3-way). For single-tap-only, generator emits `ACTION_TAP_DANCE_DOUBLE(single, single)`.

## Limitations

- **Built-in patterns only** — no custom C callbacks beyond the finished/reset helpers the generator emits for 3-way.
- **`single + hold` (no `double_tap`) produces BROKEN firmware** — `is_three_way()` returns false (because `double_tap` is None), so the generator's `generate_helpers()` skips emitting the `td_<name>_finished` / `td_<name>_reset` callbacks. But `generate_actions()` selects `ACTION_TAP_DANCE_FN_ADVANCED` because `has_hold()` is true, producing a reference to undefined functions. Compilation fails. **Avoid single+hold without double_tap** until generator supports this pattern.
- **No per-layer tap dance state** — tap count is global; cannot have different behavior on different layers.
- **Tapping term influences behavior** — same `tap_hold_settings.tapping_term` applies to tap dances.
- **TD() replaces the underlying keycode** — assigning `TD(name)` to a position takes over that key position via QMK's tap-dance mechanism. The underlying layer's keycode (including any `KC_TRNS`) is NOT fired.

## Common Pitfalls

- **Auto-created placeholder won't type anything** — single_tap defaults to `KC_NO`. Define it explicitly.
- **References in `keymap.c` require `TAP_DANCE_ENABLE = yes`** in `rules.mk` — generated automatically when `tap_dances[]` is non-empty.
- **TD with `single_tap = KC_NO`** — key does nothing on tap. Almost always wrong; fix the definition.
- **Tap dance on a `KC_TRNS` key** — assigning `TD(name)` to a position takes over that key via QMK's tap-dance mechanism. The underlying layer's keycode (including any `KC_TRNS`) is NOT also fired. Tap dance overrides on the current layer.
- **Multiple TD() with same name** — invalid; `add` will reject.
- **Renaming a tap dance breaks all references** — if you change `name` in the definition, update all `TD(old_name)` keycodes too. Consider UUIDs in future versions.

## Validation Workflow

After any tap dance change:

```bash
# 1. Validate references resolve
lazyqmk tap-dance validate --layout <path>

# 2. Validate overall layout
lazyqmk validate --layout <path>

# 3. Regenerate firmware
lazyqmk generate --layout <path> --qmk-path ~/qmk_firmware --out-dir <out> --format all
```