# Reference 0002: Layer Model

> QMK layers (0–31), named, with stable UUIDs referenced by `LT()`, `MO()`, etc. Up to 32 layers with `LAYER_STATE_32BIT`.

## Layer Limits

| Config | Max layers | Notes |
|---|---|---|
| Default QMK (no flag) | 16 | `LAYER_STATE_16BIT` is the QMK default (`qmk_firmware/quantum/action_layer.h:45-46`) |
| `LAYER_STATE_8BIT` | 8 | Set explicitly in keyboard's `config.h` |
| `LAYER_STATE_32BIT` | 32 | Set explicitly in keyboard's `config.h` (for 17+ layers) |

Validation allows up to 32 layers. **LazyQMK does not automatically emit `LAYER_STATE_*` defines** — the keyboard's own `config.h` controls runtime capacity. Boards like Corne support 16 layers out of the box; for 17+ layers, ensure the keyboard's `config.h` includes `#define LAYER_STATE_32BIT`. Note these are C defines that go in `config.h`, NOT `rules.mk` flags.

## Layer Identity (UUID + Number)

Each layer has both:
- **`number`** (u8, 0-based) — what QMK firmware sees
- **`id`** (UUID string, immutable) — what the JSON references

The UUID is stable across renames and reorders. **Always prefer UUIDs in keycodes** (`LT(@uuid, KC_SPC)`) over numeric indices (`LT(1, KC_SPC)`) — UUIDs survive layer reorder.

## Reading Layers

```bash
lazyqmk inspect --layout <path> --section layers --json
# Returns: { count: N, layers: [{ number, name, key_count }] }

# Detailed (per-layer, per-key) — load JSON directly:
jq '.layers[] | {id, number, name, default_color, category_id, key_count: (.keys | length)}' \
  ~/.../my.json
```

## Layer Structure (JSON)

```json
{
  "id": "2d4174f6-9e41-48a8-9514-fd982fe660b5",
  "number": 0,
  "name": "Base",
  "default_color": { "r": 212, "g": 212, "b": 212 },
  "category_id": null,
  "keys": [
    {
      "position": { "row": 0, "col": 0 },
      "keycode": "KC_Q",
      "label": null,
      "color_override": null,
      "category_id": null,
      "combo_participant": false
    }
  ],
  "layer_colors_enabled": true
}
```

Fields:
- `id` — UUID string
- `number` — 0-based layer index
- `name` — display name (max 50 chars, non-empty)
- `default_color` — base RGB color (lowest color priority)
- `category_id` — optional category for whole layer (3rd color priority)
- `keys` — vector of `KeyDefinition`
- `layer_colors_enabled` — when false, layer-level colors disabled (per-key + key-category still work)

## Key Structure (JSON)

```json
{
  "position": { "row": 0, "col": 0 },
  "keycode": "LT(@5f74eeab-..., KC_SPC)",
  "label": null,
  "color_override": { "r": 244, "g": 114, "b": 182 },
  "category_id": "space",
  "combo_participant": false,
  "description": "MacOS Lock"
}
```

Fields:
- `position` — visual coordinates (row, col) — NOT matrix coordinates
- `keycode` — QMK keycode string (or parameterized expression)
- `label` — optional display label (unused, future feature)
- `color_override` — RGB (highest priority)
- `category_id` — kebab-case category ID
- `combo_participant` — boolean (always present in JSON as `false`; the field is never read by combo logic. Combos use `combo_settings.combos[].key1/key2` positions directly. Reserved for future use.)
- `description` — optional free-text annotation (e.g., "MacOS Lock", "App Switcher"). Useful for documenting non-obvious keys in the export.

## Coordinate Systems

Three coordinate systems; LazyQMK uses **visual** in JSON.

| System | Used for | Source |
|---|---|---|
| Matrix (row, col) | Electrical wiring, firmware | `info.json` matrix field |
| LED index | Sequential RGB LED order | `info.json` led field |
| Visual (row, col) | Markdown tables, JSON, UI | What the user sees |

Conversions handled by `VisualLayoutMapping`. Use `lazyqmk geometry` to inspect all three.

## Layer Naming Conventions (recommend)

Semantic names help when reading exports and discussing with the agent:

| # | Name | Purpose |
|---|---|---|
| 0 | Base | Always present. Main typing layer. |
| 1 | Symbols | Brackets, math, programming symbols |
| 2 | Navigation | Arrows, page up/down, home/end |
| 3 | Numbers | Numpad + math operators |
| 4 | Code | Brackets, quotes, parens optimized for coding |
| 5 | Media | Play/pause, volume, brightness |
| 6 | Mouse | Mouse keys + scroll |
| 7 | Function | F-keys for app shortcuts |
| 8+ | Custom | Gaming, app-specific, macros |

## Layer Switching Keycodes

| Keycode | Behavior |
|---|---|
| `MO(n)` | Momentary: layer n active while held |
| `TG(n)` | Toggle: layer n on/off (sticky) |
| `TO(n)` | Turn on layer n (turn off all others) |
| `TT(n)` | Tap-Toggle: 5 taps = toggle layer n |
| `OSL(n)` | One-Shot Layer: active for next key only |
| `DF(n)` | Default layer: set base layer to n |
| `LT(n, kc)` | Layer-Tap: tap for kc, hold for layer n |
| `LM(n, mod)` | Layer-Mod: layer n + mod active simultaneously |

UUID form: `MO(@<uuid>)`, `LT(@<uuid>, KC_SPC)`, etc.

## Layer Validation Rules

- At least one layer required (layer 0)
- Layer numbers sequential without gaps (0, 1, 2, ...)
- All layers same key count (matrix size of selected LAYOUT variant)
- No duplicate positions within a layer
- All `category_id` references must exist in `layout.categories[]`
- All `TD(name)` references must exist in `layout.tap_dances[]`

## Reading Layer References (Inbound)

```bash
lazyqmk layer-refs --layout <path> --json
# Returns which keys reference each layer + transparency warnings
```

**Caveats**:
- `lazyqmk validate` does NOT check whether `LT(@<uuid>, ...)` references resolve to existing layers. It accepts dangling UUIDs.
- `lazyqmk layer-refs` SILENTLY OMITS dangling UUID references — they don't appear in `inbound_refs` and don't generate warnings.
- Only `lazyqmk keycode --expr "LT(@<uuid>, ...)" --json` reports `valid: false` for unresolved UUIDs (single-key check).
- The firmware GENERATOR accepts dangling UUIDs and emits the raw `@<uuid>` expression in `keymap.c`. QMK's compile step then fails to parse the unresolved reference.

If you're deleting layers, do a manual preflight before generating:

```bash
LAYOUT=~/.../my.json
# Find all LT(@<uuid>, ...), MO(@<uuid>, ...) references and check the uuid exists
jq -r '
  .layers[].keys[]
  | select(.keycode | test("^(@|LT\\(@|MO\\(@|TG\\(@|TO\\(@|TT\\(@|OSL\\(@|DF\\(@|LM\\(@)"))
  | .keycode
' "$LAYOUT" | grep -oE '@[0-9a-f-]+' | sort -u \
  | while read uuid; do
      if ! jq -e --arg u "${uuid#@}" '.layers[] | select(.id == $u)' "$LAYOUT" >/dev/null; then
        echo "DANGLING: ${uuid}"
      fi
    done
```

## Transparency (KC_TRNS) Behavior

`KC_TRNS` is special — it passes through to the underlying layer's keycode:

- Layer 1 key at (0,0) = `KC_TRNS`, Layer 0 key at (0,0) = `KC_Q` → effective key at (0,0) on layer 1 = `KC_Q`

Use `KC_TRNS` to "fall through" to the layer below for keys you don't want to customize on the upper layer.

`KC_NO` is different — it's a true no-op (key does nothing).

## Common Pitfalls

- **Empty layer** — layer must have `keys` matching the keyboard's layout (e.g., 42 keys for a Corne). If you accidentally truncate, `validate` will fail.
- **Wrong key count** — `LAYOUT_split_3x6_3` is 42 keys, `LAYOUT_split_3x5_3` is 36. Mismatch = validate fails.
- **Hardcoded layer numbers in keycodes** — use `@<uuid>` instead, so reorder doesn't break.
- **Layer category without category defined** — `category_id: "foo"` requires `"foo"` in `layout.categories[]`.
- **Renaming a layer doesn't update keycode references** — keycodes use UUIDs (preferred) or numbers, not names. Safe.

## Generating Layer Skeletons (Agent Recipe)

To add a new empty layer, copy positions from layer 0 (so each new layer inherits the same `(row, col)` layout) and start every key as `KC_TRNS` (so it falls through to the layer below until customized):

```bash
LAYOUT_JSON=~/.../my.json
NEW_LAYER_NUM=$(jq '.layers | length' "$LAYOUT_JSON")

jq --argjson num "$NEW_LAYER_NUM" '
  .layers += [{
    id: ("00000000-0000-0000-0000-" + ($num | tostring | ("000000000000" + .)[-12:])),
    number: $num,
    name: "New Layer",
    default_color: { r: 128, g: 128, b: 128 },
    category_id: null,
    keys: [.layers[0].keys | .[] | {
      position: .position,
      keycode: "KC_TRNS",
      label: null,
      color_override: null,
      category_id: null,
      combo_participant: false
    }],
    layer_colors_enabled: true
  }]
' "$LAYOUT_JSON" > "$LAYOUT_JSON.new"
mv "$LAYOUT_JSON.new" "$LAYOUT_JSON"
```

This mirrors layer 0's positions exactly, so duplicates aren't possible. After generating, populate with actual keycodes via `lessons/0006-populate-layers.md`.