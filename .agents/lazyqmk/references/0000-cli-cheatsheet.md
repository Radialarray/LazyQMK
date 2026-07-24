# Reference 0000: CLI Cheatsheet

> Every `lazyqmk <sub>` subcommand with its arguments, JSON output shape, and one-liner purpose. All subcommands accept `--json` for machine-readable output (where applicable).

## Conventions

- **Paths in examples use macOS defaults** — substitute Linux (`~/.config/LazyQMK/`) or Windows (`%APPDATA%\LazyQMK\`) as needed. Discover the actual paths with `lazyqmk config show --json | jq '{paths, build}'` (works on all OSes).
- **`--layout <path>`** is the path to the user's `*.json` or `*.md` layout file.
- **`--qmk-path <path>`** points to the QMK firmware directory (usually `~/qmk_firmware`).
- **Exit codes** for LazyQMK CLI itself: `0` = success, `1` = validation/user error, `2` = I/O error. Helper scripts define their own exit semantics — see each script's header comment.
- **JSON output** (`--json`) is supported by **read/report commands only**: `doctor`, `validate`, `inspect`, `keycode`, `keycodes`, `tap-dance list/validate`, `layer-refs`, `list-keyboards`, `list-layouts`, `geometry`, `config show`, `category list`. Mutating commands (`generate`, `export`, `tap-dance add/delete`, `category add/delete`, `template save/apply`, `config set`, `web`) do **not** accept `--json`.

## Top-Level

| Command | Purpose |
|---|---|
| `lazyqmk` | Launch TUI (auto-runs onboarding if no config) |
| `lazyqmk <layout.json>` | Open layout in TUI |
| `lazyqmk --init` | Run setup wizard |
| `lazyqmk web [--port 3001]` | Launch web UI (NOT used in agent mode) |
| `lazyqmk --qmk-path <path>` | Override QMK path |
| `lazyqmk --help` | Top-level help |

## `doctor` — environment health

```bash
lazyqmk doctor --json
# Returns:
# {
#   "status": "ready|missing_dependencies|warnings",
#   "passed": <int>,
#   "failed": <int>,
#   "unknown": <int>,
#   "dependencies": [
#     {"name": "QMK CLI"|"ARM GCC"|"AVR GCC"|"QMK Firmware",
#      "status": "available"|"missing"|"unknown",
#      "version": "...",
#      "message": "...",
#      "installation_hint": "..." | null}
#   ],
#   "platform": "macOS"|"Linux"|"Windows"
# }
```

Top-level `status: "ready"` means all 4 dependencies available. Check `.dependencies[]` for per-tool details. Script wrapper: `bash scripts/doctor.sh`.

## `validate` — layout sanity check

```bash
lazyqmk validate --layout <path> [--json] [--strict]
# Returns: { "valid": bool, "checks": { keycodes, positions, layer_refs, tap_dances }, "errors": [...] }
```

- `--strict` makes warnings fail.
- Checks: keycodes valid, no duplicate positions, no matrix out-of-bounds, tap dance references resolve.

## `generate` — produce QMK source

```bash
lazyqmk generate \
  --layout <path> \
  --qmk-path <path> \
  --out-dir <path> \
  [--layout-name <variant>] \
  [--format keymap|config|all] \
  [--deterministic]
```

Writes to `--out-dir`. With `--format all`:
- `keymap.c` always written
- `config.h` always written
- `rules.mk` written ONLY when combos (with at least one non-placeholder combo) OR any tap dance is defined (`COMBO_ENABLE = yes` / `TAP_DANCE_ENABLE = yes`)
- `keymap.json` written ONLY when PaletteFX is enabled (registers the `getreuer/palettefx` community module)

Does NOT accept `--json`. Use `--deterministic` for CI/diff-friendly output (timestamps normalized).

## `export` — shareable markdown documentation

```bash
lazyqmk export \
  --layout <path> \
  --qmk-path <path> \
  [--output <out.md>] \
  [--layout-name <variant>]
```

Generates a human-readable markdown with all layers, colors, tap dances, settings. Auto-named if `--output` omitted: `<layout_name>_export_<date>.md`. Note: `--layout-name` overrides the variant (default: from metadata). Does NOT accept `--json`.

## `web` — web UI server

```bash
lazyqmk web [--port 3001] [--host 127.0.0.1] [--workspace <dir>] [--verbose]
```

NOT used in agent mode (CLI-only by hard rule). Listed here for completeness.

## `inspect` — read layout sections

```bash
lazyqmk inspect --layout <path> --section <name> [--json]
```

| Section | Returns |
|---|---|
| `metadata` | name, author, keyboard, layout_variant, keymap_name, created, modified, tags |
| `layers` | list of {number, name, key_count} |
| `categories` | list of {id, name, color} |
| `tap-dances` | list of {name, single_tap, double_tap, hold, type} |
| `settings` | RGB master + brightness + idle effect summary |

Use `--json` for machine-readable.

## `keycode` — resolve layer-keycode expressions

```bash
lazyqmk keycode --layout <path> --expr "LT(@<uuid>, KC_SPC)" [--json]
# Returns: { input, resolved, layer_name?, valid }
```

Resolves `@<layer-uuid>` references in keycodes like `LT()`, `MO()`, `TG()`, `TO()`, `TT()`, `OSL()`, `DF()`.

## `keycodes` — list available keycodes

```bash
lazyqmk keycodes [--category <id>] [--json]
```

Categories (loaded by LazyQMK from `src/keycode_db/categories/` + `languages/`):

| Core (21 loaded) | Language (10 loaded) |
|---|---|
| `advanced`, `audio`, `backlight`, `basic`, `function`, `international`, `layers`, `magic`, `media`, `mod_combo`, `mod_tap`, `modifiers`, `mouse`, `navigation`, `numpad`, `one_shot`, `rgb`, `shifted`, `symbols`, `system`, `tap_dance` | `lang_danish`, `lang_french`, `lang_french_mac`, `lang_german`, `lang_german_mac`, `lang_italian`, `lang_norwegian`, `lang_spanish`, `lang_swedish`, `lang_uk` |

Total: ~859 keycodes. Note language IDs are `lang_<locale>` (e.g., `lang_german`), not just `german`. The `bluetooth`, `haptic`, and `joystick` files exist in `categories/` but are not currently loaded into the CLI keycode database.

```bash
lazyqmk keycodes --category lang_german --json
lazyqmk keycodes --category basic --json | jq '.count'
```

## `tap-dance` — manage tap dance actions

```bash
lazyqmk tap-dance list   --layout <path> [--json]
lazyqmk tap-dance add    --layout <path> --name <id> --single KC_X [--double KC_Y] [--hold KC_Z]
lazyqmk tap-dance delete --layout <path> --name <id> [--force]
lazyqmk tap-dance validate --layout <path> [--json]
# validate returns: { valid, orphaned: [...TD refs without defs], unused: [...defs not referenced] }
```

`--force` on delete replaces all `TD(name)` references with `KC_TRNS`.

## `layer-refs` — inbound layer references + transparency warnings

```bash
lazyqmk layer-refs --layout <path> [--json]
# Returns: { layers: [{ number, name, inbound_refs: [{from_layer, position, kind, keycode}], warnings: [...] }] }
```

Shows which keys reference each layer (LT, MO, TG, etc.) and warns about transparent chains.

## `list-keyboards` — discover QMK keyboards

```bash
lazyqmk list-keyboards --qmk-path <path> [--filter <regex>] [--json]
# Returns: { keyboards: ["vendor/kb", ...], count }
```

Scans `keyboards/` recursively for `info.json`/`keyboard.json`.

**Test fixture override**: set `LAZYQMK_QMK_FIXTURE=/path/to/fixture` to point these commands (also `list-layouts`, `geometry`) at a test fixture directory instead of the real QMK fork. Useful for testing without the 500MB submodule.

## `list-layouts` — discover layout variants for a keyboard

```bash
lazyqmk list-layouts --qmk-path <path> --keyboard <vendor/kb> [--json]
# Returns: { keyboard, layouts: [{ name, key_count }], count }
```

Use `key_count` to pick a variant matching your key count (e.g., 36/40/42/46).

## `geometry` — coordinate system mapping

```bash
lazyqmk geometry --qmk-path <path> --keyboard <vendor/kb> --layout-name <LAYOUT_xxx> [--json]
```

The flag is `--layout-name`, not `--variant`. Returns matrix, LED, and visual position mappings for the chosen layout. Useful for diagnosing "where is my thumb key" issues.

Shows matrix, LED, and visual position mappings. Useful for diagnosing "where is my thumb key" issues.

## `config` — global configuration

```bash
lazyqmk config show [--json]
lazyqmk config set [--qmk-path <dir>] [--output-dir <dir>] [--theme auto|light|dark]
```

Config file: `~/Library/Application Support/LazyQMK/config.toml`.

## `category` — manage layout categories

```bash
lazyqmk category list   --layout <path> [--json]
lazyqmk category add    --layout <path> --id <kebab-id> --name "Display Name" --color "#RRGGBB"
lazyqmk category delete --layout <path> --id <id> [--force]
```

`--force` clears all references to the category from keys and layers.

## `template` — save/load reusable layouts

```bash
lazyqmk template list [--json]
lazyqmk template save  --layout <path> --name <name> [--tags "tag1,tag2"]
lazyqmk template apply --name <name> --out <path>
```

Templates stored in `~/Library/Application Support/LazyQMK/templates/`.

## `show-help` — in-app help topics

```bash
lazyqmk show-help <topic>     # positional, not --topic
```

Real topics (from `src/data/help.toml`, 28 contexts): `main`, `keycode_picker`, `color_picker_palette`, `color_picker_rgb`, `layer_manager`, `category_manager`, `tap_dance_editor`, `settings_manager`, `metadata_editor`, `modifier_picker`, `layer_picker`, `build_log`, `help`, `selection`, `template_browser`, `template_save`, `export_filename`, `category_picker`, `layout_picker`, `keyboard_picker`, `setup_wizard`, `unsaved_prompt`, `clipboard`, `parameterized_keycodes`, `tap_hold_info`, `color_priority`, `tips`, `cli_commands`. Run `lazyqmk show-help` (no arg) for the full list.

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Validation / user error |
| 2 | I/O / system error |

## Common Patterns

**Read state, mutate, save** — the agent's bread and butter:

```bash
# 1. Inspect
lazyqmk inspect --layout ~/.../my.json --section metadata --json

# 2. Validate
lazyqmk validate --layout ~/.../my.json --json

# 3. Mutate (CLI or direct JSON edit via jq/python)
lazyqmk category add --layout ~/.../my.json --id navigation --name "Navigation" --color "#4ADE80"

# 4. Re-validate
lazyqmk validate --layout ~/.../my.json

# 5. Generate firmware
lazyqmk generate --layout ~/.../my.json --qmk-path ~/qmk_firmware --out-dir ~/.../builds/run-1 --format all
```

**Direct JSON edit** (faster than chaining many CLI calls for bulk changes):

```bash
# Bump all layer default colors to a new scheme
jq '.layers |= map(.default_color = {r: 128, g: 128, b: 128})' \
   ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
lazyqmk validate --layout ~/.../my.json
```

**Parse JSON safely**:

```bash
LAYOUT_KEYMAP=$(lazyqmk inspect --layout <path> --section metadata --json | jq -r '.keymap_name')
LAYOUT_VARIANT=$(lazyqmk inspect --layout <path> --section metadata --json | jq -r '.layout_variant')
LAYOUT_KEYBOARD=$(lazyqmk inspect --layout <path> --section metadata --json | jq -r '.keyboard')
```