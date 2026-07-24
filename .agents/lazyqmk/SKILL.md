---
name: lazyqmk
description: Guide a user from a blank keyboard to a polished UF2 firmware file using the LazyQMK CLI toolchain. Use when the user wants to set up a new keyboard, modify an existing layout, add QMK features (RGB, idle effect, ripple overlay, PaletteFX, tap dance, combos, tap-hold), validate a layout, build firmware, or flash it to a device. Covers all LazyQMK CLI subcommands, the JSON/Markdown layout schema, the four-level color priority system, semantic categories, the layer system, and the full feature set. Pipeline-driven (10 phases), CLI-only, stateful across sessions via a per-keyboard workspace.
argument-hint: "<task> e.g. 'set up new Corne Choc Pro', 'add idle effect', 'modify base layer'"
license: MIT
---

# LazyQMK Agent Skill

> **Mission:** Walk a user from blank slate (no keyboard, no firmware) to a polished `*.uf2` file in `~/Library/Application Support/LazyQMK/builds/<keyboard>_<keymap>_<timestamp>/`. All work is CLI-driven (`lazyqmk ... --json | jq`); never touch the TUI or Web UI in agent mode.

## Discovery Gate (run silently every session)

Before saying anything to the user, run these checks. They work on **all platforms** (macOS, Linux, Windows):

```bash
# 1. Is lazyqmk installed?
which lazyqmk || echo "MISSING: lazyqmk"

# 2. Discover QMK path + output dir (works on every OS)
lazyqmk config show --json | jq '{qmk: .paths.qmk_firmware, output: .build.output_dir, theme: .ui.theme}'

# 3. Environment healthy?
bash scripts/doctor.sh

# 4. Discover config + layouts + workspaces dirs cross-platform
#    (`Config::config_dir()` returns the platform-specific dir)
case "$(uname)" in
  Darwin)  CONFIG_DIR=~/Library/Application\ Support/LazyQMK ;;
  Linux)   CONFIG_DIR=~/.config/LazyQMK ;;
  MINGW*|MSYS*|CYGWIN*) CONFIG_DIR="${APPDATA}/LazyQMK" ;;
  *)       CONFIG_DIR=~/.config/LazyQMK ;;
esac
ls "$CONFIG_DIR/layouts/"
ls "$CONFIG_DIR/workspaces/" 2>/dev/null
```

If `scripts/doctor.sh` fails or `lazyqmk config show` returns empty, the user has no setup yet — jump to **Phase 1** (install) and **Phase 2** (config) before anything else.

Report findings in one sentence, then ask only what's missing.

### Workspace Slug Convention

The workspace folder name for a keyboard (`<slug>` in paths like `workspaces/<slug>/MISSION.md`) is the layout keymap_name converted to a slug, e.g. `corne_choc_pro` → `corne-choc-pro`. Derive consistently:

```bash
# From metadata.keymap_name (preferred) — kebab-case the layout name
LAYOUT_FILE=~/.../my.json
SLUG=$(jq -r '.metadata.keymap_name // "untitled"' "$LAYOUT_FILE" | tr '[:upper:]' '[:lower:]' | tr '_ ' '-')

# Or from keyboard path
SLUG=$(jq -r '.metadata.keyboard // "untitled"' "$LAYOUT_FILE" | tr '/' '-' | tr '[:upper:]' '[:lower:]')
```

## Routing Table

| User wants | Go to |
|---|---|
| Start a new keyboard from scratch | `lessons/0001-pipeline-overview.md` then phase ladder |
| Modify an existing layout | `references/0000-cli-cheatsheet.md` + relevant `references/000N-*.md` |
| Add RGB / idle effect | `references/0004-rgb-and-idle.md` + `lessons/0007-configure-features.md` |
| Add ripple overlay | `references/0005-ripple-and-palettefx.md` + `lessons/0007-configure-features.md` |
| Add PaletteFX | `references/0005-ripple-and-palettefx.md` + `lessons/0007-configure-features.md` |
| Add tap dance | `references/0007-tap-dance.md` + `lessons/0007-configure-features.md` |
| Add combo | `references/0006-tap-hold-and-combos.md` + `lessons/0007-configure-features.md` |
| Add multiple combos (any actions) | `references/0006-tap-hold-and-combos.md` "Multi-combo layouts" + `lessons/0007-configure-features.md` |
| Visualize combo keys on keyboard preview | `references/0006-tap-hold-and-combos.md` "Visualization on the keyboard" |
| Configure tap-hold (HRM etc.) | `references/0006-tap-hold-and-combos.md` + `lessons/0007-configure-features.md` |
| Generate keymap only (don't compile) | `lessons/0008-validate-and-generate.md` with `--format keymap` |
| Build UF2 / flash | `lessons/0009-build-uf2.md` then `lessons/0010-flash-and-verify.md` |
| Debug a post-flash issue (keys wrong, mods trigger, etc.) | `lessons/0010-flash-and-verify.md` Common Issues + relevant feature reference |
| Export / share layout as markdown | `lessons/0010-flash-and-verify.md` Optional: Export Layout Documentation |
| List keyboards / layouts / templates | `references/0000-cli-cheatsheet.md` (cli cheatsheet) |
| Doctor failing / build error | `lazyqmk doctor --verbose` (real `--verbose`) + `references/0000-cli-cheatsheet.md` |
| "What can lazyqmk do?" | `references/0003-color-system.md`, `0004-rgb-and-idle.md`, `0005-ripple-and-palettefx.md`, `0006-tap-hold-and-combos.md`, `0007-tap-dance.md` |

## Phase Ladder (the 10-phase pipeline)

```
Phase 1: Install              (lazyqmk + qmk fork)         — lesson 0002
Phase 2: Doctor               (env healthy)                — lesson 0002
Phase 3: Pick keyboard        (vendor/kb/variant)          — lesson 0003
Phase 4: Plan layers          (count + names)              — lesson 0004
Phase 5: Semantic categories  (palette + category IDs)     — lesson 0005
Phase 6: Populate layers      (CLI mutations per key)      — lesson 0006
Phase 7: Configure features   (8 groups, opt-in)           — lesson 0007
Phase 8: Validate + generate  (keymap.c, config.h)         — lesson 0008
Phase 9: Build UF2            (qmk compile, locate output) — lesson 0009
Phase 10: Flash + verify      (bootloader, qmk flash)      — lesson 0010
```

Full details: `lessons/0001-pipeline-overview.md`.

## CLI Cheatsheet (summary; full version in `references/0000-cli-cheatsheet.md`)

Read-state commands accept `--json` for machine parsing. Mutating commands (`generate`, `export`, `tap-dance add`, `category add`, `template save`, `config set`, `web`) do NOT.

```bash
# Read state (all accept --json)
lazyqmk doctor --json
lazyqmk config show --json | jq '{paths, build}'
lazyqmk list-keyboards --qmk-path <path> --json | jq '.keyboards'
lazyqmk list-layouts --qmk-path <path> --keyboard <vendor/kb> --json | jq '.layouts'
lazyqmk inspect --layout <path> --section <name> --json
lazyqmk validate --layout <path> --json
lazyqmk tap-dance validate --layout <path> --json
lazyqmk layer-refs --layout <path> --json
lazyqmk keycode --layout <path> --expr "LT(@<uuid>, KC_SPC)" --json

# Mutate (no --json)
lazyqmk tap-dance add --layout <path> --name <id> --single KC_X [--double KC_Y] [--hold KC_Z]
lazyqmk category add --layout <path> --id <kebab-id> --name "Display Name" --color "#RRGGBB"
lazyqmk generate --layout <path> --qmk-path <path> --out-dir <out> --format all
lazyqmk export --layout <path> --qmk-path <path> --output <out.md>
```

## Conversational Philosophy

1. **Act silently first** — run discovery gate, inspect state, read files
2. **Ask only what matters** — hardware facts (which keyboard, RGB), user preferences (language, theme), irreversible confirmations (flash)
3. **Never ask** — paths, config locations, QMK fork URL, info.json contents, default settings
4. **Ground every decision in MISSION.md** — re-read it before any feature branch
5. **Persist decisions in NOTES.md** — one source of truth across sessions
6. **Prefer declarative over procedural** — for category/feature changes, edit the JSON directly via `python -c` or `jq` then re-validate (faster than chaining many CLI calls)

## Hard Rules (don't-asks, non-negotiables)

- Never ask which directory layouts live in — discover via `lazyqmk config show --json | jq '{paths, build}'`
- Never ask the user to look up `info.json` — use `lazyqmk list-layouts --json`
- Never manually edit `keymap.c` / `config.h` / `rules.mk` — always run `lazyqmk generate`
- Never use the TUI or Web UI in agent mode — only CLI (user can drive the TUI/Web UI themselves)
- Never change `keymap_name` without asking (changes firmware identity)
- Never flash without explicit user confirmation — use a `read -p "Confirm flash? (type 'flash') "` gate before copying to `/Volumes/RPI-RP2/` or running `qmk flash`
- Never use the official QMK repo — must be the Radialarray fork for ripple/idle/PaletteFX to work
- Always run `lazyqmk validate` after mutating the layout (use `bash scripts/validate-and-report.sh` to combine with tap-dance + layer-refs validation)
- Always run `lazyqmk generate --format all` before `qmk compile`
- Always re-deploy the generated `keymap.c` to `<qmk-path>/keyboards/<metadata.keyboard>/keymaps/<keymap_name>/` before `qmk compile` (otherwise `qmk compile` builds a stale keymap)
- Remember: the idle effect code is gated on `geometry.has_rgb_matrix()` but NOT on `rgb_enabled` — disable idle/PaletteFX/ripple explicitly when disabling RGB

## Mission Gate (first session on a keyboard)

If `~/Library/Application Support/LazyQMK/workspaces/<slug>/MISSION.md` doesn't exist, gather mission before doing anything else:

1. **Which keyboard?** (vendor/keyboard/variant) — or "I don't have one yet" → Phase 1
2. **Primary use?** (code / prose / both / gaming / mixed)
3. **Non-English layout?** (DE / FR / Nordic / none)
4. **RGB LEDs present?** (yes / no / don't know)

Then write MISSION.md, NOTES.md, RESOURCES.md using the templates below.

If MISSION.md exists, read it and proceed directly to the phase ladder.

## End State (when you're done)

- A working `*.uf2` (or `.hex` / `.bin`) in `~/Library/Application Support/LazyQMK/builds/<kb>_<keymap>_<ts>/`
- Validates with `lazyqmk validate` (zero errors)
- All MISSION.md "must-have features" checked off
- NOTES.md updated with decisions made
- Optional: `lazyqmk export` produced a shareable `*.md`

## Reference Index (one-stop cheat sheets)

- `references/0000-cli-cheatsheet.md` — every CLI subcommand with `--json` shape
- `references/0001-keycode-categories.md` — 22 categories + language keycodes (DE/FR/...)
- `references/0002-layer-model.md` — layers, LT/MO/TG/TO/TT/OSL/DF, UUIDs
- `references/0003-color-system.md` — 4-level priority + categories + semantic palette
- `references/0004-rgb-and-idle.md` — RGB master + 9 idle modes (incl. PaletteFX)
- `references/0005-ripple-and-palettefx.md` — ripple knobs + PaletteFX 6×16 matrix
- `references/0006-tap-hold-and-combos.md` — 5 tap-hold presets + combos (up to 32 per layout, 3 hardcoded actions: DisableEffects / DisableLighting / Bootloader). Per-combo action choice (not slot-bound). Combo keys are visualized on both TUI and WebUI keyboard previews with red/yellow/gray borders and B/E/L badges.
- `references/0007-tap-dance.md` — 2-way vs 3-way TD(name), auto-create, validation

## Lesson Index (numbered CLI flows)

- `lessons/0001-pipeline-overview.md` — read first
- `lessons/0002-install-and-doctor.md` — Phase 1-2
- `lessons/0003-pick-keyboard.md` — Phase 3
- `lessons/0004-plan-layers.md` — Phase 4
- `lessons/0005-semantic-categories.md` — Phase 5
- `lessons/0006-populate-layers.md` — Phase 6
- `lessons/0007-configure-features.md` — Phase 7 (all 8 feature groups)
- `lessons/0008-validate-and-generate.md` — Phase 8
- `lessons/0009-build-uf2.md` — Phase 9
- `lessons/0010-flash-and-verify.md` — Phase 10

## Helper Scripts

- `scripts/doctor.sh` — `lazyqmk doctor --json` with summary, exits non-zero if any tool missing
- `scripts/inspect-layout.sh <path>` — every section as JSON
- `scripts/validate-and-report.sh <path>` — validate + tap-dance validate + layer-refs in one shot

## Embedded Templates (copy into workspace)

### `MISSION.md`

```markdown
# Mission: <keyboard-name>

## Hardware
- Keyboard: <vendor/keyboard/variant>
- Layout variant: LAYOUT_*
- Key count: N
- RGB: yes/no
- Output format: uf2/hex/bin
- Keymap name: <keymap-name>

## Use
- Primary: code/prose/gaming/mixed
- Languages: en/DE/FR/...
- Special: macOS shortcuts? gaming layer? etc.

## Must-have features
- [ ] Tap dance TD(<name>)
- [ ] Home-row mods
- [ ] Idle effect: <mode>
- [ ] Ripple overlay: <mode/color>
- [ ] PaletteFX: <effect>/<palette>
- [ ] Bootloader combo
- [ ] (etc)

## Success looks like
- A working `*.uf2` in `~/.../builds/<kb>_<keymap>_<ts>/` that flashes and types my keymap correctly.
```

### `NOTES.md`

```markdown
# Notes: <keyboard-slug>

## User preferences
- Theme: auto/dark/light
- Language layout: en/DE/...
- Output format: uf2
- Tap-hold preset: Default/HomeRowMods/Responsive/Deliberate/Custom
- Uncolored key behavior: <0-100>% (0=off, 100=full color)

## Key decisions
- YYYY-MM-DD: <decision 1>
- YYYY-MM-DD: <decision 2>

## Known issues
- <issue 1 with workaround>
```

### `RESOURCES.md`

```markdown
# Resources: <keyboard-slug>

## LazyQMK docs (in repo)
- docs/AGENT_GUIDE.md — agent-friendly phase ladder (closest to this skill)
- docs/FEATURES.md — full feature catalog
- docs/EXPORT_FORMAT.md — export output spec
- docs/PALETTEFX.md — PaletteFX community module
- docs/RIPPLE_FEATURE.md — ripple overlay design
- docs/ARCHITECTURE.md — internal design

## QMK fork
- URL: https://github.com/Radialarray/qmk_firmware.git
- Local: ~/qmk_firmware
- Why fork: contains ripple overlay, idle effect, PaletteFX integration
- Common modules: getreuer/palettefx

## Keycode database
- Bundled in LazyQMK binary (categories/*.json + languages/*.json)
- Discover: lazyqmk keycodes --category <id> --json

## CLI cheatsheet
- See skill references/0000-cli-cheatsheet.md
```

### `lessons/<date>-session.md`

```markdown
# Session YYYY-MM-DD: <one-line summary>

## Phase(s) executed
- Phase N: <title>

## Skill lessons followed
- 0001-pipeline-overview.md
- 0002-install-and-doctor.md

## Changes made
- Created <keyboard>/layouts/<name>.json
- Added category: navigation (#4ADE80)
- Enabled idle effect: breathing, 60s timeout
- Wrote build: builds/<kb>_<keymap>_<ts>/keymap.c

## Next session should
- Phase 7: configure ripple overlay
- Phase 9: rebuild and verify UF2
```

## Platform Paths Quick Reference

| OS | Config | Layouts | Builds |
|---|---|---|---|
| macOS | `~/Library/Application Support/LazyQMK/` | `…/layouts/` | `…/builds/` |
| Linux | `~/.config/LazyQMK/` | `…/layouts/` | `…/builds/` |
| Windows | `%APPDATA%\LazyQMK\` | `…\layouts\` | `…\builds\` |

Templates: `~/Library/Application Support/LazyQMK/templates/`

Per-keyboard workspace: `~/Library/Application Support/LazyQMK/workspaces/<slug>/`