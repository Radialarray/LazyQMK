# Idle Effect (Screensaver) Feature Plan

**Branch**: `feat/idle-effect` | **Date**: 2025-12-12 | **Spec**: (this plan)
**Input**: User request to add an idle LED effect (screensaver) before LEDs turn off.

## Summary
Implement an "Idle Effect" for keyboard RGB: after a configurable idle timeout (keypress-based), switch from per-key/TUI colors to a selectable standard RGB matrix effect (default: Breathing) for a configurable duration, then turn LEDs off. Any keypress during idle effect or off restores normal behavior and resets timers. Settings are per-layout and stored in the layout Markdown. Firmware generation emits the necessary QMK code to enforce idle → effect → off.

## Technical Context
- **Language/Version**: Rust (1.91.1+), QMK C output
- **Primary Dependencies**: Ratatui/Crossterm for TUI; Serde/Serde_yaml/json5 for parsing; QMK RGB Matrix on firmware side
- **Storage**: Layout Markdown files (per-layout settings)
- **Testing**: cargo test, cargo clippy --all-features -- -D warnings; integration tests for firmware generation and parser round-trip
- **Target Platform**: QMK keyboards with RGB Matrix; TUI for firmware generation only
- **Project Type**: CLI/TUI + codegen
- **Performance Goals**: Minimal overhead; timers handled in firmware with QMK hooks
- **Constraints**: Must not regress existing RGB timeout behavior; compile guards for non-RGB boards
- **Scale/Scope**: Single feature, per-layout settings

## Constitution Check
- No personal data; no credentials.
- Follow Conventional Commits; run tests and clippy.
- Guard RGB logic with RGB_MATRIX_ENABLE.
- Keep per-layout vs global scopes distinct (store in layout Markdown).

## Project Structure (relevant paths)
```
specs/023-idle-effect/
  plan.md               # This plan

src/
  models/layout.rs      # New idle effect settings
  parser/layout.rs      # Parse idle fields from Markdown
  parser/template_gen.rs# Emit idle fields to Markdown
  tui/settings_manager.rs
  tui/handlers/settings.rs
  firmware/generator.rs # Emit QMK idle effect logic

tests/
  firmware_gen_tests.rs # Integration: generated code contains idle logic
  (add parser/template round-trip tests if needed)
```

**Structure Decision**: Single-project CLI/TUI + firmware codegen; all changes live under existing src/ and tests/ paths; spec under specs/023-idle-effect.

## Plan / Tasks
1) **Data model**: Add per-layout idle effect fields with defaults
   - idle_effect_enabled (bool, default true)
   - idle_timeout_ms (u32, default 60_000)
   - idle_effect_duration_ms (u32, default 300_000)
   - idle_effect_mode (enum/string for standard RGB effects, default Breathing)
2) **Parsing/serialization** (layout Markdown)
   - Parse/save fields under Settings: Idle Effect (on/off), Idle Timeout, Idle Effect Duration, Idle Effect Mode
   - Support min/sec/ms formats; defaults when missing
3) **TUI settings**
   - Add settings in RGB group (grouped visually): enable, timeout, duration, mode selector (standard effects)
   - Display values with human-friendly units; store per-layout
4) **Firmware generation**
   - Emit defines/constants and C logic: idle → effect → off; keypress resets and restores TUI colors
   - Avoid conflict with RGB_MATRIX_TIMEOUT (disable/omit when idle effect enabled)
   - Guard with RGB_MATRIX_ENABLE; fallback to Breathing if mode unsupported
5) **Tests**
   - Integration: generated keymap/config includes idle defines and logic; RGB_MATRIX_TIMEOUT handled correctly
   - Parser/template: round-trip new fields
6) **Docs**
   - Update template output for Settings section; brief mention in FEATURES.md if needed (or inline comments in plan)

## Edge Cases / Decisions
- Keypress-only idle detection.
- idle_effect_duration_ms = 0 → go straight to off after idle timeout.
- idle_timeout_ms = 0 → treat as disabled (never enter idle effect).
- Non-RGB boards: compile out.
- If rgb_enabled is false, idle effect does not run; LEDs remain off.
- Preserve existing rgb_timeout_ms parsing/serialization for backward compatibility; suppress emitting RGB_MATRIX_TIMEOUT when idle effect is enabled.

## Deliverables
- Updated model, parser, template gen, TUI settings, firmware generator idle logic, tests. Commit spec and implementation on branch `feat/idle-effect`.
