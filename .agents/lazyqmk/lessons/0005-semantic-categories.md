# Lesson 0005: Semantic Categories (Phase 5)

> Define the color palette + category IDs. End: 3–8 categories defined with semantic names + RGB colors, all `KC_TRNS` keys in categories.

## Goal

- 3-8 semantic categories created (`navigation`, `symbols`, `delete`, etc.)
- Each with a display name and RGB color
- Categories assigned to keys where applicable (or assigned in Phase 6)

## Discovery Gate (silent)

```bash
LAYOUT=~/.../my.json

# Current categories
lazyqmk inspect --layout "$LAYOUT" --section categories --json

# Suggested palette colors (from skill reference 0003)
# See references/0003-color-system.md for the recommended semantic palette
```

## User Questions

| Ask | Default if "you decide" |
|---|---|
| "What categories do you want?" | 5 standard: navigation, delete, space, modtap, symbols |
| "Color preferences?" | Tailwind-derived defaults (recommended) |

If user has aesthetic preferences (e.g., "everything pink", "monochrome"), adjust colors accordingly.

## Recommended Starter Set

| ID | Color | RGB | Purpose |
|---|---|---|---|
| `delete` | red | `#DC2626` | BSPC, DEL, forward-delete |
| `space` | cyan | `#22D3EE` | SPC, ENT, thumb cluster |
| `modtap` | pink | `#F472B6` | Home-row mods (LSFT_T, LCTL_T, etc.) |
| `navigation` | green | `#4ADE80` | Arrows, HOME, END, PGUP, PGDN |
| `symbols` | lime | `#84CC16` | Punctuation, math operators |

Optional additions:

| ID | Color | RGB | Purpose |
|---|---|---|---|
| `numbers` | green-deep | `#16A34A` | Digits 0-9 |
| `function` | teal | `#0891B2` | F-keys |
| `code` | amber | `#CA8A04` | Brackets, braces, parens |
| `media` | pink-deep | `#DB2777` | Vol, mute, play |
| `macos` | orange | `#F97316` | Cmd shortcuts, lock |
| `mouse` | red-orange | `#EA580C` | MS_* keys |
| `lang-german` | orange-light | `#FB923C` | DE_* keys |
| `backlight` | orange-deep | `#EA580C` | BL_* keys |

## Phase 5: Steps

### 1. Add categories

```bash
LAYOUT=~/.../my.json

# Use the CLI for each category
lazyqmk category add --layout "$LAYOUT" \
  --id delete \
  --name "Delete" \
  --color "#DC2626"

lazyqmk category add --layout "$LAYOUT" \
  --id space \
  --name "Space" \
  --color "#22D3EE"

lazyqmk category add --layout "$LAYOUT" \
  --id modtap \
  --name "Mod-Tap" \
  --color "#F472B6"

lazyqmk category add --layout "$LAYOUT" \
  --id navigation \
  --name "Navigation" \
  --color "#4ADE80"

lazyqmk category add --layout "$LAYOUT" \
  --id symbols \
  --name "Symbols" \
  --color "#84CC16"

# Verify
lazyqmk category list --layout "$LAYOUT"
```

Or batch via jq:

```bash
LAYOUT=~/.../my.json

jq '
  .categories += [
    {id: "delete", name: "Delete", color: {r: 220, g: 38, b: 38}},
    {id: "space", name: "Space", color: {r: 34, g: 211, b: 238}},
    {id: "modtap", name: "Mod-Tap", color: {r: 244, g: 114, b: 182}},
    {id: "navigation", name: "Navigation", color: {r: 74, g: 222, b: 128}},
    {id: "symbols", name: "Symbols", color: {r: 132, g: 204, b: 22}}
  ]
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### 2. Set uncolored_key_behavior (recommended)

```bash
LAYOUT=~/.../my.json

# 40% dim for uncolored keys (makes category-colored keys pop)
jq '.uncolored_key_behavior = 40' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### 3. Assign categories to layers

For each layer, optionally assign a `category_id` (overrides the layer's `default_color`):

```bash
LAYOUT=~/.../my.json

# Layer 1 (Symbols) gets category "symbols"
jq '.layers[1].category_id = "symbols"' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"

# Layer 2 (Navigation) gets category "navigation"
jq '.layers[2].category_id = "navigation"' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### 4. Validate

```bash
lazyqmk validate --layout ~/.../my.json
# Expect: "✓ Validation passed"
```

## Validation

```bash
# All categories have unique IDs
jq '.categories | map(.id) | length == (unique | length)' ~/.../my.json
# Should be: true

# All category references resolve
lazyqmk validate --layout ~/.../my.json
# Should be: "✓ Validation passed"
```

## Workspace Update

Add to NOTES.md:

```markdown
## Color scheme
- Categories: delete, space, modtap, navigation, symbols
- Uncolored key behavior: 40% dim
- Layer 1 (Symbols) category: symbols
- Layer 2 (Navigation) category: navigation
```

## Tips

1. **Start with 3-5 categories** — add more as you discover new logical groups
2. **Use the same category for related keys** — e.g., all `LSFT_T()`, `LCTL_T()`, etc. → `modtap`
3. **Don't over-categorize** — 3-5 well-chosen categories beats 15 micro-categories
4. **Use semantic IDs** — `navigation`, not `nav` or `arrows`. Stable across renames.
5. **Match your aesthetic** — if you prefer pink everything, swap colors. The IDs are stable; the RGB is whatever you like.

## Common Pitfalls

- **Category ID not kebab-case** — `Nav Keys` invalid, use `nav-keys` or `navigation`
- **Same ID twice** — `lazyqmk category add` rejects; use unique IDs
- **Dangling category_id on keys/layers** — `validate` will fail; fix references first
- **RGB out of range** — values must be 0-255

## Next Lesson

`lessons/0006-populate-layers.md` (Phase 6) — assign keycodes and categories to specific keys.