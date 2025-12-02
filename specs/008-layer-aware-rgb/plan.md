# Implementation Plan: 005-layer-aware-rgb

**Branch**: `feat/layer-aware-rgb` | **Date**: 2025-11-26 | **Spec**: _TBD (inline design for now)_
**Input**: Need to add layer-aware firmware lighting driven by TUI layout colors.

## Summary

We will implement **device-side, layer-aware baked RGB colors** so that the keyboard firmware reflects the color semantics defined in the TUI layout editor. The generator will compute per-layer, per-LED base colors from the existing four-level color priority system and embed them into generated firmware. A small custom QMK RGB effect will then select, at runtime, the correct color for each LED based on the current layer state, including momentary (`MO`) and other stacked layers. As a later extension, we will add per-key animation support (e.g., flashing Enter) on top of these base colors.

## Technical Context

**Language/Version**: Rust 1.88 (app), C for QMK firmware
**Primary Dependencies**: Ratatui 0.26, Crossterm 0.27, Serde 1.0, embedded QMK fork `vial-qmk-keebart`
**Storage**: Local Markdown layout files, TOML config files, generated C files in QMK tree
**Testing**: `cargo test` for Rust; `make`/QMK build for integration (manual)
**Target Platform**: macOS/Linux terminals for TUI; QMK-compatible microcontrollers for firmware
**Project Type**: Single Rust binary + vendored QMK firmware
**Performance Goals**: Negligible overhead in firmware RGB update loop; generator runtime uncritical
**Constraints**:
- Must not break existing firmware generation/build workflows
- Must support keyboards with and without RGB matrix
- Must respect existing three-coordinate mapping (visual, matrix, LED)
**Scale/Scope**:
- Typical boards up to ~100 LEDs and ~8–12 layers

## Constitution Check

- Work is localized to:
  - Rust generator and models (`src/firmware/*`, `src/models/*`)
  - Small, isolated changes or additions under `vial-qmk-keebart/` for a custom RGB mode
- No new external services or databases
- Complexity is justified by the need to track dynamic layer state in firmware while preserving TUI-driven colors.

## Project Structure

### Documentation (this feature)

```text
specs/008-layer-aware-rgb/
  plan.md          # This file
  spec.md          # (optional) Formal user stories and requirements
  tasks.md         # Task breakdown for implementation
```

### Source Code (repository root)

```text
src/
  models/
    layout.rs              # Already contains resolve_key_color
    keyboard_geometry.rs   # LED index and matrix mappings
    visual_layout_mapping.rs
  firmware/
    generator.rs           # Extend to emit RGB color data tables
    builder.rs             # No major changes expected
    mod.rs                 # Re-export if we add helper types

vial-qmk-keebart/
  keyboards/
    [target keyboard(s)]/  # Optional: keymap glue for custom effect
  quantum/
    rgb_matrix/            # Add or extend custom effect using baked colors
```

**Structure Decision**: Single-project Rust app remains unchanged; we only extend existing modules plus a small QMK lighting hook in the vendored firmware.

## Design Overview

### 1. Data Model: Base Colors per Layer and LED

- Define a conceptual structure (Rust-side, not necessarily a new public type) representing:
  - `base_color[layer_index][led_index] -> (r, g, b)`
- Use existing pieces:
  - `Layout::layers` and `Layer::keys` for per-layer keys
  - `Layout::resolve_key_color(layer_idx, key)` for semantic color resolution
  - `VisualLayoutMapping::visual_to_led_index(row, col)` to go from key position to LED index
- For each layer and for each LED index:
  - Initialize with a global fallback (e.g., off or default white)
  - For every key in that layer, compute its color via `resolve_key_color` and assign it to the corresponding LED index.

### 2. Generator Responsibilities

Extend `FirmwareGenerator` to:

- Build the per-layer, per-LED `base_color` table using the three-coordinate mapping.
- Emit this table into generated C code alongside `keymaps`:
  - Option A: A 2D array `rgb_colors[layer][led]` of 8-bit RGB triplets.
  - Option B: Separate arrays per layer; the exact layout is a QMK-side choice.
- Guard emission behind a check:
  - Only generate the table when the target keyboard has an RGB matrix (based on geometry/info.json or config);
  - No changes for boards without RGB.
- Optionally emit a small piece of metadata that describes:
  - Number of layers with color data
  - Number of LEDs

### 3. Firmware (QMK) Responsibilities

Add a small, self-contained lighting mode in `vial-qmk-keebart` that:

- Understands the baked color table symbol emitted by the generator.
- On each RGB Matrix update tick:
  - For each LED index:
    - Determine the *effective layer* for the key at that matrix position, using QMK's existing layer resolution rules (aligning with how keycodes are chosen for `MO`, `LT`, etc.).
    - Use that layer index and LED index to pick a base color from the baked table.
    - Write the resulting RGB value into the RGB matrix buffer.
- Ensures it respects QMK's normal power and brightness controls (global brightness, suspend, etc.).

### 4. Handling Dynamic Layer State (Momentary Layers, etc.)

- The QMK-side effect is responsible for reading `layer_state` and resolving effective layers.
- The Rust generator does *no* runtime decisions; it simply supplies colors per layer & LED.
- This design guarantees:
  - Holding `MO(1)` or any other momentary/stacked layer key immediately changes the effective layer for that key's matrix position.
  - The lighting effect automatically follows that effective layer and uses its color.

### 5. Later Extension: Per-Key Animations (e.g., Flashing Enter)

We design the plan to be forward-compatible with per-key animations:

- Additional metadata from generator:
  - Optional per-LED flag table or list (e.g., LEDs that belong to special keys like Enter, Esc, Space, etc.).
  - Flags can encode "effect groups" (e.g., group for Enter flash, group for confirmation keys, etc.).
- Extended firmware effect:
  - Maintains simple animation state per LED or per group (e.g., timer since last key press).
  - Modulates base color from the baked table (brightness, hue, on/off) based on animation state.
  - Uses QMK's key event hooks to trigger animations (e.g., on Enter press, start a flash timer for the corresponding LED).
- The base architecture (per-layer color tables + layer-aware effect) stays unchanged; we only add a derived "overlay" step.

## Phases

### Phase 0: Clarify Requirements & Scope

- Confirm which keyboards / layouts we will target first (e.g., keebart/corne_choc_pro variants).
- Decide on the C-side representation of `base_color` that best aligns with QMK patterns.
- Decide how we detect "RGB-capable" keyboards (info.json, keyboard.json, or config flags).

### Phase 1: Rust-Side Color Table Generation

- Implement internal logic in `FirmwareGenerator` to:
  - Iterate layers and keys, computing `base_color[layer][led]` using `resolve_key_color` and `visual_to_led_index`.
  - Handle missing LED indices or mismatches robustly (warnings or validation errors).
- Add focused unit tests around this logic using small synthetic layouts/geometries.

### Phase 2: C Code Emission in keymap.c

- Extend `generate_keymap_c` output to include the baked color table as static PROGMEM data.
- Ensure names and linkage are stable and discoverable from QMK code.
- Keep the generated keymap valid for boards without RGB (guarded by `#ifdef RGB_MATRIX_ENABLE` or similar).

### Phase 3: QMK Lighting Mode Integration

- Add a new RGB Matrix effect (or extend an existing one) in `vial-qmk-keebart` that:
  - References the baked color table from `keymap.c`.
  - For each LED, resolves effective layer and selects the correct base color.
  - Plays nicely with global brightness and on/off state.
- Wire this effect as the default or as a selectable mode for the generated keymap.

### Phase 4: Validation and UX Polish

- Manually test on at least one split keyboard with RGB:
  - Verify colors follow layout per layer.
  - Verify that holding a momentary layer key updates colors in real time.
- Ensure firmware generation still works for non-RGB boards (no extra artifacts, no build failures).
- Optionally expose a way (in TUI or config) to disable this lighting mode if users prefer stock QMK animations.

### Phase 5: Future Extension – Per-Key Animations

- Add generator metadata for special keys / effect groups.
- Extend QMK lighting effect to:
  - Maintain animation state per LED or per effect group.
  - Combine base colors with animation overlays.
- Optional: add TUI controls to enable/disable or preview these effects.

## Complexity Tracking

At this stage, no constitution violations beyond the planned custom RGB Matrix effect and additional static tables are anticipated. If the QMK lighting integration proves significantly more complex than expected, we will revisit this section to justify additional abstractions.

## Concrete Implementation Steps (QMK RGB integration)

1. Inspect RGB matrix and existing custom effects in `vial-qmk-keebart` (especially `quantum/rgb_matrix*` and keyboard-specific hooks).
2. Finalize the data contract for `layer_base_colors` and `layer_base_colors_layer_count` between the Rust generator and QMK.
3. Design a new `RGB_MATRIX_TUI_LAYER_COLORS` effect that, per LED, resolves the effective layer (respecting `layer_state` and transparent keys) and reads colors from `layer_base_colors[layer][led]`.
4. Implement `RGB_MATRIX_TUI_LAYER_COLORS` in the forked QMK tree (new `tui_layer_colors` animation file) and register it in the RGB matrix effect list.
5. Reconcile the generated `rgb_matrix_indicators_user` / `layout_colors` section so that, in `layout_static` mode, the firmware relies primarily on the new effect and baked per-layer colors.
6. Ensure `config.h` generation only sets `RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_TUI_LAYER_COLORS` when RGB is present, custom colors exist, and `lighting_mode = layout_static`.
7. Rebuild firmware from the TUI, compile with QMK `make`, and validate both key behaviour and layer-aware colors on hardware.
