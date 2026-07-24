# Lesson 0001: Pipeline Overview

> Read this first. The 10-phase spine for going from zero to a polished UF2 file.

## Mission

Walk the user from blank slate (no keyboard, no firmware) to a polished `*.uf2` file in `~/Library/Application Support/LazyQMK/builds/<keyboard>_<keymap>_<timestamp>/`. All work is CLI-driven. Never touch the TUI or Web UI in agent mode.

## The 10-Phase Pipeline

```
Phase 1: Install              (lazyqmk + qmk fork)         → lesson 0002
Phase 2: Doctor               (env healthy)                → lesson 0002
Phase 3: Pick keyboard        (vendor/kb/variant)          → lesson 0003
Phase 4: Plan layers          (count + names)              → lesson 0004
Phase 5: Semantic categories  (palette + IDs)              → lesson 0005
Phase 6: Populate layers      (CLI mutations per key)      → lesson 0006
Phase 7: Configure features   (8 groups, opt-in)           → lesson 0007
Phase 8: Validate + generate  (keymap.c, config.h)         → lesson 0008
Phase 9: Build UF2            (qmk compile, locate output) → lesson 0009
Phase 10: Flash + verify      (bootloader, qmk flash)      → lesson 0010
```

Each phase has a **discovery gate** (silent commands to run first), **user questions** (only what matters), **concrete steps**, **validation**, and **workspace update** (writes to MISSION.md / NOTES.md / session log).

## Feature Branches

Phase 7 has 8 feature branches (each opt-in):

1. RGB master settings (always)
2. Idle effect / PaletteFX screensaver
3. Ripple overlay (keypress feedback)
4. Tap-hold (LT/MT/TT behavior presets)
5. Combos (up to 32, 3 hardcoded actions)
6. Tap dance (TD(name) actions)
7. Color categories + semantic palette
8. Output format / keymap identity

Default path: enable #1, #4 (HomeRowMods preset), #5 (bootloader combo), #7 (3–5 semantic categories), #8 (uf2).

## Workspace Setup (first session)

Before any phase, establish the user's workspace:

```
~/Library/Application Support/LazyQMK/workspaces/<keyboard-slug>/
├── MISSION.md                            # Why this keyboard, what success looks like
├── NOTES.md                              # Persistent user prefs (DE, HomeRowMods, etc.)
├── RESOURCES.md                          # LazyQMK docs paths, QMK fork, keycode DB
└── lessons/                              # Session logs (one per session)
    └── <date>-session-<NN>.md
```

The agent creates this folder on the first session via the **Mission Gate** (see SKILL.md).

## End State

You're done when:

1. `~/.../builds/<kb>_<keymap>_<ts>/<keymap>.uf2` exists (or .hex / .bin)
2. `lazyqmk validate --layout <path>` returns zero errors
3. All MISSION.md "must-have features" checked off
4. NOTES.md updated with decisions made
5. Optional: `lazyqmk export` produced a shareable `*.md`

## File Location Convention

| File | macOS | Linux | Windows |
|---|---|---|---|
| Config | `~/Library/Application Support/LazyQMK/config.toml` | `~/.config/LazyQMK/config.toml` | `%APPDATA%\LazyQMK\config.toml` |
| Layouts | `~/Library/Application Support/LazyQMK/layouts/` | `~/.config/LazyQMK/layouts/` | `%APPDATA%\LazyQMK\layouts\` |
| Builds | `~/Library/Application Support/LazyQMK/builds/` | `~/.config/LazyQMK/builds/` | `%APPDATA%\LazyQMK\builds\` |
| Templates | `~/Library/Application Support/LazyQMK/templates/` | `~/.config/LazyQMK/templates/` | `%APPDATA%\LazyQMK\templates\` |
| Workspace (per-kbd) | `~/Library/Application Support/LazyQMK/workspaces/<slug>/` | `~/.config/LazyQMK/workspaces/<slug>/` | `%APPDATA%\LazyQMK\workspaces\<slug>\` |

## When to Skip Phases

| User already has... | Skip |
|---|---|
| lazyqmk installed | Phase 1 |
| Doctor passing | Phase 2 |
| Existing layout to modify | Phase 3-5 (jump to Phase 6 with modification intent) |
| Already has RGB/idle/ripple configured | Phase 7 branch |
| Already has UF2 | Phase 8-10 (just verify flash works) |

## Decision Heuristics (when user is undecided)

| Choice | Default if user says "you decide" |
|---|---|
| Output format | `uf2` (RP2040 standard) |
| Language layout | English (DE/FR/Nordic if they say so) |
| Tap-hold preset | HomeRowMods (best for productivity) |
| Idle effect | breathing (gentle, default) |
| PaletteFX | disabled (add later if asked) |
| Ripple | disabled (add later if asked) |
| Semantic categories | 4-5 standard: navigation, delete, space, modtap, symbols |
| Combo | Bootloader on a comfortable key pair, 800ms |
| Tap dance | none by default (add specific TD on request) |

## Hard Constraints

- **Custom QMK fork required** — official QMK won't compile ripple/idle/PaletteFX
- **macOS paths** in all examples — adapt for user's OS in step 1
- **Always validate after any JSON mutation** — `lazyqmk validate --layout <path>`
- **Always generate before compile** — `lazyqmk generate ... --format all`
- **Never touch TUI/Web UI** — CLI only in agent mode

## Next Lesson

After reading this, proceed to `lessons/0002-install-and-doctor.md` (Phase 1 + 2).