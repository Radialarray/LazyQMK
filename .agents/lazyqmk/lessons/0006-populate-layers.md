# Lesson 0006: Populate Layers (Phase 6)

> Assign keycodes and category IDs to specific keys. End: every layer has meaningful keys, Base is fully populated.

## Goal

- Layer 0 (Base) has full keymap (alpha + home-row mods + layer access)
- Each upper layer has its specialization (symbols, arrows, etc.)
- Each non-Base layer has `LT()` / `MO()` references on Base for access

## Discovery Gate (silent)

```bash
LAYOUT=~/.../my.json

# Get layer IDs (needed for LT() references)
jq '.layers | map({number, name, id})' "$LAYOUT"

# Get current keys for layer 0
jq '.layers[0].keys | map({pos: .position, kc: .keycode})' "$LAYOUT"

# Show categories
lazyqmk category list --layout "$LAYOUT"
```

## User Questions

| Ask | Why |
|---|---|
| "Which layout style do you prefer?" (Colemak, Dvorak, etc.) | Affects Base layer |
| "What mods on home row? (LSFT_T, LCTL_T, LALT_T, LGUI_T)" | Affects thumb accessibility |
| "Where do you want layer access? (LT on thumb, MO elsewhere)" | Affects Base layer |

If user is unsure, suggest the most common pattern (QWERTY + home-row mods + LT thumb access).

## Phase 6: Steps

### Step 1: Get layer UUIDs

```bash
LAYOUT=~/.../my.json
SYMBOLS_ID=$(jq -r '.layers[] | select(.name == "Symbols") | .id' "$LAYOUT")
NAV_ID=$(jq -r '.layers[] | select(.name == "Navigation") | .id' "$LAYOUT")
NUMBERS_ID=$(jq -r '.layers[] | select(.name == "Numbers") | .id' "$LAYOUT")
# (etc.)
```

### Step 2: Populate Base layer (Layer 0)

This is the biggest step. Pattern:

1. Alphas + numbers on top 3 rows
2. Home-row mods (LSFT_T, LCTL_T, etc.) on A, S, D, F (left) and J, K, L, ; (right)
3. Layer access via LT() on thumbs or top-row
4. Space, Enter, Backspace on thumbs

Example for a 42-key Corne (`LAYOUT_split_3x6_3`), QWERTY with German layout, **only if you have these layers and categories** (Symbols, Navigation, Numbers, Code, Globals; macos):

```bash
LAYOUT=~/.../my.json

# Resolve layer UUIDs — empty string if a layer doesn't exist
SYMBOLS_ID=$(jq -r '.layers[] | select(.name == "Symbols") | .id // empty' "$LAYOUT")
NAV_ID=$(jq -r '.layers[] | select(.name == "Navigation") | .id // empty' "$LAYOUT")
NUMBERS_ID=$(jq -r '.layers[] | select(.name == "Numbers") | .id // empty' "$LAYOUT")
CODE_ID=$(jq -r '.layers[] | select(.name == "Code") | .id // empty' "$LAYOUT")
GLOBALS_ID=$(jq -r '.layers[] | select(.name == "Globals") | .id // empty' "$LAYOUT")

# Only include layers you actually have — adapt this for your plan
jq --arg sym "$SYMBOLS_ID" --arg nav "$NAV_ID" --arg num "$NUMBERS_ID" \
   --arg code "$CODE_ID" --arg glb "$GLOBALS_ID" '
  .layers[0].keys |= map(
    if .position.row == 0 and .position.col == 0 then
      .keycode = "LCG(KC_Q)"
      | .color_override = {r: 249, g: 115, b: 22}
      | .category_id = "macos"
    elif .position.row == 0 and .position.col == 1 then
      .keycode = "KC_Q"
    elif .position.row == 0 and .position.col == 2 and $sym != "" then
      .keycode = ("LT(@" + $sym + ", KC_W)")
    elif .position.row == 0 and .position.col == 3 and $nav != "" then
      .keycode = ("LT(@" + $nav + ", KC_E)")
    elif .position.row == 0 and .position.col == 4 and $code != "" then
      .keycode = ("LT(@" + $code + ", KC_R)")
    elif .position.row == 0 and .position.col == 5 and $glb != "" then
      .keycode = ("LT(@" + $glb + ", KC_T)")
    elif .position.row == 1 and .position.col == 0 then
      .keycode = "KC_ESC"
      | .color_override = {r: 59, g: 130, b: 246}
    elif .position.row == 1 and .position.col == 1 then .keycode = "KC_A"
    elif .position.row == 1 and .position.col == 2 then .keycode = "LALT_T(KC_S)"
    elif .position.row == 1 and .position.col == 3 then .keycode = "LCTL_T(KC_D)"
    elif .position.row == 1 and .position.col == 4 then
      .keycode = "LSFT_T(KC_F)"
      | .color_override = {r: 244, g: 114, b: 182}
      | .category_id = "modtap"
    elif .position.row == 1 and .position.col == 5 then .keycode = "LGUI_T(KC_G)"
    # ... continue for all positions ...
    else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

**Critical**: only reference layers and categories that actually exist in the layout. If `Code` and `Globals` are not in your plan, drop those `--arg` flags and corresponding branches.

- A `LT(@<unknown-uuid>, KC_X)` reference causes `lazyqmk validate` to FAIL deterministically — `Layout::validate()` rejects dangling layer references.
- An unknown `category_id` on a key or layer also causes `validate` to FAIL — `Layout::validate()` rejects dangling category references. There is no "renders as gray fallback" path; the layout won't load.

This gets verbose. **For agents**: use a structured approach with helper scripts or jq templates — never hard-code references to layers/categories that may not exist in the user's plan.

### Step 3: Populate upper layers

Each upper layer inherits from Base via `KC_TRNS` and only customizes specific keys:

```bash
LAYOUT=~/.../my.json

# Symbols layer: customize only the symbol positions, keep KC_TRNS elsewhere
jq '
  .layers[1].keys |= map(
    if .position.row == 0 and .position.col == 2 then .keycode = "DE_EXLM" | .color_override = {r: 132, g: 204, b: 22} | .category_id = "symbols"
    elif .position.row == 0 and .position.col == 3 then .keycode = "DE_QUES" | .color_override = {r: 132, g: 204, b: 22} | .category_id = "symbols"
    # ... etc
    else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Step 4: Apply categories to keys (bulk)

E.g., apply `modtap` to all `LSFT_T()` / `LCTL_T()` etc. on Base:

```bash
LAYOUT=~/.../my.json

jq '
  .layers[0].keys |= map(
    if (.keycode | test("^[LR](CTL|SFT|ALT|GUI)_T\\("))
    then .category_id = "modtap"
    else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

(The regex matches both left and right variants of the named mod-taps: `LCTL_T`, `LSFT_T`, `LALT_T`, `LGUI_T`, `RCTL_T`, `RSFT_T`, `RALT_T`, `RGUI_T`.)

Apply `delete` to BSPC/DEL keys:

```bash
LAYOUT=~/.../my.json

jq '
  .layers[].keys |= map(
    if .keycode == "KC_BSPC" or .keycode == "KC_DEL"
    then .category_id = "delete"
    else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

(Note: `KC_FWD_DEL` is not in LazyQMK's keycode database. Use `KC_DEL` (delete forward) instead.)

Apply `navigation` to arrow keys:

```bash
LAYOUT=~/.../my.json

jq '
  .layers[].keys |= map(
    if .keycode | test("^(KC_(UP|DOWN|LEFT|RGHT|HOME|END|PGUP|PGDN))$")
    then .category_id = "navigation"
    else . end
  )
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### Step 5: Validate

```bash
lazyqmk validate --layout ~/.../my.json
# Expect: "✓ Validation passed"
```

## Validation

```bash
# Every layer has meaningful keys (no all-TRNS)
jq '.layers | map({name, trns_count: ([.keys[] | select(.keycode == "KC_TRNS")] | length)})' ~/.../my.json

# Base layer has alphas + mods + layer access
jq '.layers[0].keys | map(.keycode) | join(" ")' ~/.../my.json
# Should have: KC_Q KC_W ... KC_A LALT_T(KC_S) ... LT(@<uuid>, KC_SPC)

# All LT() references resolve to existing layers
lazyqmk layer-refs --layout ~/.../my.json --json | jq '.layers[].warnings'
# Should be empty: []
```

## Workspace Update

Add to NOTES.md:

```markdown
## Populated layers
- Layer 0 (Base): QWERTY with German DE_* keys, home-row mods (LSFT_T, LCTL_T, LALT_T, LGUI_T), LT access on thumbs
- Layer 1 (Symbols): DE_EXLM, DE_QUES, ... on top row
- Layer 2 (Navigation): KC_UP, KC_DOWN, KC_LEFT, KC_RGHT on right home
- (etc.)
```

## Tips

1. **Copy from an existing layout** — start with a working layout (e.g., `examples/corne_choc_pro_layout.json` in the LazyQMK repo) and modify
2. **Use UUIDs in LT()** — `LT(@<uuid>, KC_SPC)` survives layer reorder; numeric form doesn't
3. **Test layer access** — once generated, flash, verify the LT keys work as expected
4. **Iterate** — don't try to get it perfect in one go; flash, type, adjust

## Common Pitfalls

- **Numeric LT() references break on reorder** — always use UUIDs
- **LT() with KC_TRNS or KC_NO** — semantically broken; tap must be a real keycode
- **Duplicate layer access** — same layer accessible from multiple keys is fine; just intentional
- **Empty Base layer** — must have alphas + modifiers + layer access at minimum
- **Validate after every bulk mutation** — `lazyqmk validate --layout <path>`

## Next Lesson

`lessons/0007-configure-features.md` (Phase 7) — RGB, idle, ripple, palettefx, tap-hold, combos, tap dance.