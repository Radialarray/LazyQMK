# Lesson 0008: Validate and Generate (Phase 8)

> Final validation + generate QMK source files (`keymap.c`, `config.h`, `rules.mk`, `keymap.json`). End: source files ready for compilation.

## Goal

- `keymap.c`, `config.h`, optionally `rules.mk` and `keymap.json` written to out-dir
- `lazyqmk generate` succeeds (this is the AUTHORITATIVE validation — it uses real QMK geometry)
- Tap dance and layer-refs cross-checks pass

## Important: Authoritative Validation

`lazyqmk validate` (standalone, without QMK path) builds an **estimated square geometry** and may flag valid split-keyboard positions as out-of-bounds. For example, a real Corne layout with 14 visual columns (6 left + 8 right) gets flagged with 138 false matrix errors because the CLI assumes a 7×7 square.

`lazyqmk generate` with the real `--qmk-path` uses the actual keyboard geometry and produces correct firmware. **Generation success is the authoritative validation.** Standalone `lazyqmk validate` is a quick sanity check for keycode syntax and category reference resolution, but is NOT a strict gate for split-keyboard layouts.

When you see "Position (0, 9) maps to matrix (0, 9) which is out of bounds (7×7)" errors from standalone validate, IGNORE them if the layout is a known split-keyboard. Confirm with `lazyqmk generate` which uses real geometry.

## Discovery Gate (silent)

```bash
LAYOUT=~/.../my.json
QMK_PATH=~/qmk_firmware
OUT_DIR=~/Library/Application\ Support/LazyQMK/builds/$(date +%Y%m%d_%H%M%S)_<kb>_<keymap>

mkdir -p "$OUT_DIR"

# Pre-flight checks
which qmk && qmk --version
ls "$QMK_PATH/keyboards/<vendor>/<kb>" | head
```

## User Questions

None — generate is non-destructive (writes to a separate output directory).

## Phase 8: Steps

### Step 1: Validate layout (sanity check only)

```bash
LAYOUT=~/.../my.json

# Basic validation (keycode syntax, category refs — NOT geometry)
lazyqmk validate --layout "$LAYOUT"
```

Expected output:

```
✓ Validation passed

Checks:
  Keycodes:   passed
  Positions:  passed  # may FAIL on split keyboards due to estimated geometry
  Layer refs: passed
  Tap dances: passed
```

**If split-keyboard positions fail with "out of bounds (7×7)" errors**, that's expected — `lazyqmk validate` uses estimated square geometry. `lazyqmk generate` (Step 5) uses real QMK geometry and will succeed. Don't try to "fix" these.

If other categories fail (invalid keycode, dangling category reference, etc.), fix before continuing.

### Step 2: Validate tap dances

```bash
lazyqmk tap-dance validate --layout "$LAYOUT" --json
```

Expected:

```json
{
  "valid": true,
  "orphaned": [],
  "unused": []
}
```

If `orphaned` is non-empty, you have `TD(name)` references without definitions. Fix.

If `unused` is non-empty, you have tap dance definitions not referenced anywhere. Either remove the unused definitions or wire them up.

### Step 3: Validate layer references

```bash
lazyqmk layer-refs --layout "$LAYOUT" --json
```

Expected:

```json
{
  "layers": [
    {
      "number": 0,
      "name": "Base",
      "inbound_refs": [],
      "warnings": []
    }
    // ...
  ]
}
```

`inbound_refs` shows which keys reference each layer (informational). `warnings` should be empty (transparent chains, dangling refs).

### Step 4: Inspect final layout

```bash
# Settings summary
lazyqmk inspect --layout "$LAYOUT" --section settings

# Categories
lazyqmk inspect --layout "$LAYOUT" --section categories

# Layers
lazyqmk inspect --layout "$LAYOUT" --section layers

# Tap dances
lazyqmk inspect --layout "$LAYOUT" --section tap-dances

# Or all in one shot:
bash scripts/inspect-layout.sh "$LAYOUT"
```

### Step 5: Generate QMK source

```bash
LAYOUT=~/.../my.json
QMK_PATH=~/qmk_firmware
OUT_DIR=~/Library/Application\ Support/LazyQMK/builds/$(date +%Y%m%d_%H%M%S)_<kb>_<keymap>

mkdir -p "$OUT_DIR"

lazyqmk generate \
  --layout "$LAYOUT" \
  --qmk-path "$QMK_PATH" \
  --out-dir "$OUT_DIR" \
  --format all
```

Expected output (varies based on which features are enabled):

```
# With combos/tap dance AND PaletteFX (full output):
✓ Generated keymap.c, config.h, rules.mk, and keymap.json

# With neither combos nor tap dance AND no PaletteFX:
✓ Generated keymap.c and config.h

# Other combinations print subsets of the file list.
  Output: <out_dir>
```

Generated files:

- `keymap.c` — main C source with layer arrays, tap dances, combos
- `config.h` — feature flags, RGB settings, idle effect
- `rules.mk` — QMK build flags
- `keymap.json` — community module registration (only if PaletteFX enabled)

### Step 6: Verify generated files

```bash
OUT_DIR=~/...

# Check keymap.c exists and has the expected layers
# Note: there's one `const uint16_t PROGMEM keymaps[]` declaration covering all layers;
# `grep -c` will return 1 regardless of layer count.
grep "PROGMEM keymaps" "$OUT_DIR/keymap.c"

# Check tap dance actions (only if any defined)
[ -s "$OUT_DIR/keymap.c" ] && grep "tap_dance_action_t tap_dance_actions" "$OUT_DIR/keymap.c" || echo "  (no tap dances)"

# Check RGB settings in config.h
grep "RGB_MATRIX\|LQMK_IDLE" "$OUT_DIR/config.h"

# Check rules.mk — only written when combos/tap_dance enabled
[ -f "$OUT_DIR/rules.mk" ] && cat "$OUT_DIR/rules.mk" || echo "  (no rules.mk; not needed without combos or tap dance)"

# Check keymap.json — only written when PaletteFX enabled
[ -f "$OUT_DIR/keymap.json" ] && cat "$OUT_DIR/keymap.json" || echo "  (no keymap.json; PaletteFX not enabled)"
```

`--format all` does NOT guarantee all 4 files are written:
- `keymap.c` and `config.h` always written
- `rules.mk` only when combos OR tap_dances enabled
- `keymap.json` only when PaletteFX enabled

## Validation

```bash
# All validations pass
lazyqmk validate --layout "$LAYOUT" --strict
lazyqmk tap-dance validate --layout "$LAYOUT"
lazyqmk layer-refs --layout "$LAYOUT"

# Generated files exist (keymap.c, config.h always)
test -f "$OUT_DIR/keymap.c"
test -f "$OUT_DIR/config.h"

# Optional files (only when their features enabled)
test -f "$OUT_DIR/rules.mk"      # only if combos or tap_dances enabled
test -f "$OUT_DIR/keymap.json"   # only if PaletteFX enabled

# Generated files are non-empty
test -s "$OUT_DIR/keymap.c"
test -s "$OUT_DIR/config.h"
```

## Workspace Update

Add to NOTES.md:

```markdown
## Generated firmware
- YYYY-MM-DD HH:MM:SS: Generated <kb>_<keymap>_<ts> at $OUT_DIR
  - keymap.c: <lines> lines
  - config.h: <lines> lines
  - rules.mk: <lines> lines
  - keymap.json: present/absent (depends on PaletteFX)
```

## Common Pitfalls

- **Strict mode fails on warnings** — common warnings: orphan tap dances, unused tap dances, dangling layer references. Fix or accept.
- **Generated `keymap.c` has compile errors** — usually means a malformed keycode. Check with `lazyqmk validate` first.
- **`keymap.json` not generated** — only happens when PaletteFX is enabled. Don't worry if missing.
- **Custom QMK fork required** — if `qmk compile` later fails because of missing features (ripple, idle effect, PaletteFX), verify the fork is from `Radialarray`, not `qmk/qmk_firmware`.
- **`--layout-name` mismatch** — if layout metadata says `LAYOUT_split_3x6_3_ex2` but you pass `--layout-name LAYOUT_split_3x6_3`, generates wrong number of keys. Always let it auto-detect from metadata.

## Next Lesson

`lessons/0009-build-uf2.md` (Phase 9) — compile to UF2.