# Reference 0003: Color System

> Four-level color priority (highest → lowest): individual key override > key category > layer category > layer default. Plus categories, the Tailwind-based palette, and `uncolored_key_behavior`.

## The Four-Level Priority

```text
1. KeyDefinition.color_override      (HIGHEST)  — single-key manual override
2. KeyDefinition.category_id         (high)     — key's assigned category color
3. Layer.category_id                 (low)      — whole-layer category color
4. Layer.default_color               (LOWEST)   — fallback layer color
```

When a key has no `color_override` and no `category_id`, and its layer has no `category_id`, the layer's `default_color` is used.

When the layer has `layer_colors_enabled = false`, levels 3 and 4 are skipped (returns dark gray). Levels 1 and 2 still work.

## Resolution Code (Reference)

```rust
fn resolve_key_color(layer_idx, key, layout) -> RgbColor {
    // 1. Individual override
    if let Some(c) = key.color_override { return c; }

    // 2. Key category
    if let Some(cat_id) = &key.category_id {
        if let Some(cat) = layout.get_category(cat_id) {
            return cat.color;
        }
    }

    // 3. Layer category
    if let Some(layer) = layout.get_layer(layer_idx) {
        if let Some(cat_id) = &layer.category_id {
            if let Some(cat) = layout.get_category(cat_id) {
                return cat.color;
            }
        }
        // 4. Layer default
        return layer.default_color;
    }

    RgbColor::default() // white
}
```

## Global RGB Adjustments

After color resolution, `apply_rgb_settings` adjusts:

| Setting | Effect | Range | Default |
|---|---|---|---|
| `rgb_enabled` | Master on/off | bool | true |
| `rgb_brightness` | Dim multiplier | 0–100% | 100% |
| `rgb_saturation` | Saturation | 0–200% | 100% |
| `uncolored_key_behavior` | Brightness for keys with no specific color | 0–100% | 100% |

Order: saturation → brightness.

When `rgb_enabled = false`, all keys display black (RGB off).

## `uncolored_key_behavior` (special)

Affects keys with **no** `color_override` and **no** key `category_id` (only layer-level color inherited):

| Value | Effect |
|---|---|
| 0 | Off (black) |
| 1–99 | Dim the layer color to % |
| 100 | Full layer color |

Use 30–50% to make layer-inheriting keys visibly dimmer than specifically-colored keys — emphasizes your category work.

## Categories (JSON)

```json
{
  "categories": [
    {
      "id": "navigation",
      "name": "Navigation",
      "color": { "r": 74, "g": 222, "b": 128 }
    }
  ]
}
```

Validation:
- `id` must be **kebab-case**: lowercase, digits, hyphens only, no leading/trailing hyphen
- `name` must be non-empty, max 50 chars
- `color` must be valid RGB (0–255 per channel)

## Recommended Semantic Categories

Suggested starter palette (Tailwind-derived; tweak per taste):

| ID | Suggested Color | RGB | Purpose |
|---|---|---|---|
| `delete` | red | `#DC2626` (220, 38, 38) | BSPC, DEL, forward-delete |
| `space` | cyan | `#22D3EE` (34, 211, 238) | SPC, ENT, thumb cluster |
| `modtap` | pink | `#F472B6` (244, 114, 182) | Home-row mods (LSFT_T, LCTL_T, etc.) |
| `navigation` | green | `#4ADE80` (74, 222, 128) | Arrows, HOME, END, PGUP, PGDN |
| `symbols` | lime | `#84CC16` (132, 204, 22) | Punctuation, math operators |
| `numbers` | green-deep | `#16A34A` (22, 163, 74) | Digits 0–9 |
| `function` | teal | `#0891B2` (8, 145, 178) | F-keys |
| `code` | amber | `#CA8A04` (202, 138, 4) | Brackets, braces, parens |
| `media` | pink-deep | `#DB2777` (219, 39, 119) | Vol, mute, play |
| `macos` | orange | `#F97316` (249, 115, 22) | Cmd shortcuts, lock |
| `mouse` | red-orange | `#EA580C` (234, 88, 12) | MS_* keys |
| `lang-german` | orange-light | `#FB923C` (251, 146, 60) | DE_*, Nordic umlauts |
| `backlight` | orange-deep | `#EA580C` (234, 88, 12) | BL_*, brightness |

## CLI Operations

```bash
# List
lazyqmk category list --layout <path> [--json]

# Add
lazyqmk category add --layout <path> \
  --id navigation \
  --name "Navigation" \
  --color "#4ADE80"

# Delete (refuses if category is in use)
lazyqmk category delete --layout <path> --id navigation [--force]

# Assign to a key (direct JSON edit, see recipes below)
# Assign to a layer
# Both done via JSON mutation; no CLI subcommand — use jq/python
```

## Recipes (jq one-liners)

### Add category (idempotent — safe to re-run)

```bash
LAYOUT=~/.../my.json
jq --arg id navigation --arg name "Navigation" --argjson r 74 --argjson g 222 --argjson b 128 '
  .categories |= (if any(.id == $id) then . else . + [{id: $id, name: $name, color: {r: $r, g: $g, b: $b}}] end)
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Assign category to a specific key

```bash
LAYOUT=~/.../my.json
jq --argjson layer 0 --argjson row 0 --argjson col 13 --arg cat navigation '
  .layers[$layer].keys |= map(
    if .position.row == $row and .position.col == $col
    then .category_id = $cat else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Assign category to an entire layer

```bash
LAYOUT=~/.../my.json
jq --argjson layer 2 --arg cat navigation '
  .layers[$layer].category_id = $cat
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Set layer default color

```bash
LAYOUT=~/.../my.json
jq --argjson layer 0 --argjson r 128 --argjson g 128 --argjson b 128 '
  .layers[$layer].default_color = {r: $r, g: $g, b: $b}
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Set per-key color override

```bash
LAYOUT=~/.../my.json
jq --argjson layer 0 --argjson row 1 --argjson col 0 --argjson r 59 --argjson g 130 --argjson b 246 '
  .layers[$layer].keys |= map(
    if .position.row == $row and .position.col == $col
    then .color_override = {r: $r, g: $g, b: $b} else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Validate after any mutation

```bash
lazyqmk validate --layout ~/.../my.json
```

## Color Resolution: Display vs Firmware

The same four-level priority is used in both editor display and firmware generation, but with one important difference: firmware resolution also applies `uncolored_key_behavior`, `rgb_saturation`, `rgb_brightness`, and `rgb_enabled` when computing the per-key color table (`resolve_display_color` + `apply_rgb_settings` are baked into the generated `keymap.c`).

Practical implication:
- Setting `uncolored_key_behavior: 30` in your layout means uncolored keys in the firmware are rendered at 30% brightness — this **does** affect what you see on the physical keyboard.
- Setting `rgb_enabled: false` disables all RGB output regardless of category colors (firmware-side master switch).
- When `layer_colors_enabled: false`, the firmware renders those keys as black (NOT key/category colors).

For display-only tuning, you can still adjust per-key overrides, but the editor faithfully reflects what the firmware will show.

## Tips for Polished Color Schemes

1. **Start with layer defaults** — pick a base gray for layer 0 (e.g., `#D4D4D4`)
2. **Add 3–5 semantic categories** — navigation, symbols, delete, space, modtap
3. **Assign categories to keys** — bulk via JSON edit
4. **Use 30–50% uncolored_key_behavior** — makes category-colored keys pop (firmware-visible)
5. **Tweak individual overrides for special keys** — Esc (red), Space (cyan), Enter (pink)
6. **Layer-specific default colors** — Symbols = gray-purple, Navigation = green-gray
7. **Test by generating + viewing** — `lazyqmk export --output /tmp/preview.md` shows colors in markdown

## Common Pitfalls

- **Category deleted but still referenced** — keys/layers with dangling `category_id` will fail `validate` (`Layout::validate()` rejects dangling references). Use `lazyqmk category delete --force` or fix references first.
- **`uncolored_key_behavior` affects both editor and firmware** — set 30-50% for dim layer-inherited keys; the firmware uses the same value when baking the color table.
- **`layer_colors_enabled: false` makes firmware render keys as black** — including individual `color_override` and key `category_id` (the firmware's `generate_layer_colors_by_led()` short-circuits to all-black). Editor resolution still respects levels 1-2 for display.
- **Colors don't show up in firmware if RGB is disabled** — set `rgb_enabled: true` in layout settings. Note: `rgb_enabled: false` does NOT gate idle/PaletteFX/ripple generation, so disable those too if you want a complete RGB-off state.
- **Manual `color_override` overrides category** — even if key has `category_id: "navigation"`, `color_override: red` makes it red. Use overrides sparingly.