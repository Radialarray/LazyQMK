# Spec: Per-Key RGB Export from Layout Colors

**Feature ID:** 005-per-key-rgb-export  
**Status:** Planning  
**Created:** 2025-11-26  
**Updated:** 2025-11-26  
**Priority:** High

## Problem Statement

The TUI provides a rich color system for organizing layouts (layer defaults, categories, per-key overrides) and persists those colors in Markdown. However, generated QMK firmware ignores this color information:

- `keymap.c` contains only keycodes and encoder defaults, no LED color data.
- `vial.json` hardcodes `"lighting": "none"`, so Vial does not expose lighting controls for generated keymaps.
- `config.h` only includes generic comments and optional Vial unlock combo macros.

As a result, when users flash the generated firmware, the keyboard boots with whatever RGB matrix defaults the underlying QMK keyboard provides (typically an animated rainbow effect), instead of a static color scheme matching the layout they designed in the TUI.

## Goals

- Use the existing layout color model (layer defaults, categories, per-key overrides) to drive **on-device per-key RGB lighting**.
- Keep all behavior configurable **through the TUI and its config**, with no manual edits inside the QMK submodule.
- Avoid modifying any core files in the `vial-qmk-keebart` submodule; all integration must happen via generated keymap files.
- Preserve backward compatibility: by default, builds should continue to behave exactly as they do today unless a lighting mode is explicitly enabled.

## Non-Goals

- Implementing fully dynamic lighting editors or animation authoring inside the TUI.
- Replacing or forking the QMK RGB matrix implementation.
- Changing Vial's core behavior or protocol.

## Current Behavior

- TUI renders key colors using `Layout::resolve_key_color` and displays source indicators (`i`, `k`, `L`, `d`) purely for visualization.
- Layout colors are stored in Markdown syntax (`{#RRGGBB}` and `@category-id`) and parsed back correctly.
- Firmware generation (`src/firmware/generator.rs`):
  - Generates `keymap.c` with `keymaps[][MATRIX_ROWS][MATRIX_COLS]` and a default encoder map using RGB modifier keys.
  - Generates `vial.json` with geometry and layout but `"lighting": "none"`.
  - Generates a minimal `config.h` which may copy Vial unlock macros from a base keymap.
- The bundled QMK fork under `vial-qmk-keebart/` supports RGB matrix, Vial, and VialRGB, but the generated keymaps do not take advantage of this to express layout colors.

## Desired Behavior

When a user enables layout-driven lighting (via TUI/config):

- The generator computes static per-key colors in **LED index order** from the current layout and keyboard geometry.
- The generated keymap directory contains additional C code (or extended `keymap.c`) that:
  - Exposes a color array matching `RGB_MATRIX_LED_COUNT` and LED indices.
  - Uses standard QMK hooks (e.g. `rgb_matrix_indicators_user()` or `_kb`) to set each LED to its layout color when RGB Matrix is active.
- `vial.json` advertises that lighting is present (using an appropriate `"lighting"` value) so that tooling can treat the keyboard as RGB-capable.
- All of this is controlled by an opt-in **lighting mode** in the `Config`, defaulting to current behavior (no layout-driven lighting).

## Design Constraints

- All behavior (including lighting mode) must be controlled via the TUI and configuration files that `keyboard_tui` owns; users should not need to manually edit C files or JSON inside the QMK/Vial tree.
- We MUST NOT modify existing tracked source files in the `vial-qmk-keebart` submodule (e.g., `quantum/*.c`, `quantum/*.h`, base keyboard `config.h`, `rules.mk`, or `info.json`).
- All new lighting integration must be implemented by generating or updating files only under the keymap-specific directory, e.g. `vial-qmk-keebart/keyboards/{keyboard}/keymaps/{keymap}/`.
- The design should allow the `vial-qmk-keebart` submodule to be updated freely; regenerating firmware from the TUI should re-apply layout and lighting on top of the updated QMK tree without manual conflict resolution.

## Solution Overview

We will introduce an opt-in **lighting mode** and a new firmware generation path that:

1. Converts the existing layout color model into a per-LED color array in LED index order, for one or more layers.
2. Embeds this array into C code in the generated keymap directory.
3. Hooks into QMK's RGB Matrix callbacks to apply those colors as a static effect.
4. Updates `vial.json` and `config.h` to reflect lighting capabilities without touching the base keyboard or QMK core.

### Lighting Mode

Add a new configuration option, tentatively named `lighting_mode`, with values such as:

- `"qmk_default"` (default): do not generate any extra lighting code; preserve existing RGB behavior.
- `"layout_static"`: generate a static per-key RGB scheme based on layout colors and apply it via QMK hooks.

This will live in `Config` / `BuildConfig` and be editable via the TUI's configuration dialogs.

### Color-to-LED Mapping

We already have:

- `Layout::resolve_key_color(layer_idx, key)` for final color decision per key.
- `VisualLayoutMapping::visual_to_led_index(row, col)` to map visual coordinates to LED indices.

We will add a helper in firmware generation that:

- Iterates keys for a given layer.
- Resolves each key's color.
- Maps each key's visual position to LED index.
- Produces `Vec<RgbColor>` where `index == LED index`.

This function will be reused for:

- Generating C arrays for QMK.
- Potential future integrations (e.g., VialRGB direct mode, debug views).

### QMK Integration (v1: Static RGB Matrix Effect)

We will implement the simplest robust path first:

- In the generated keymap directory, add or extend C code so that when `RGB_MATRIX_ENABLE` is defined:
  - A static array of colors is defined, e.g. `const rgb_t layout_colors[RGB_MATRIX_LED_COUNT];` or equivalent.
  - `rgb_matrix_indicators_user()` (or `_kb`) is implemented to set each LED's color from that array on each update.

Design considerations:

- Colors may be stored in RGB (directly from `RgbColor`) or pre-converted to HSV, depending on which is simpler and more stable across QMK versions.
- The implementation must:
  - Respect the active layer in a minimal way (at least layer 0 in v1; multi-layer could be added later).
  - Avoid conflicting with user-selected modes where possible (document that `layout_static` prefers a static base mode).

### Vial Integration (Future Work)

The bundled QMK fork includes `quantum/vialrgb.c` and `RGB_MATRIX_EFFECT_VIALRGB_DIRECT`, which enable host-driven per-LED colors. For v1 we will not attempt to fully integrate with VialRGB; instead we:

- Ensure `vial.json` accurately reports that lighting exists when `lighting_mode = "layout_static"`.
- Leave deeper Vial-driven lighting control (e.g., Vial editing the same colors) as follow-up work.

## Risks and Trade-offs

- **Hardware diversity:** Different boards may have custom RGB configurations; using standard QMK hooks in keymaps is usually safe, but we must avoid assumptions about drivers.
- **Performance:** Setting every LED's color on each scan may cost CPU time on older MCUs. We may need to optimize to only update on changes (e.g., when layer or layout changes).
- **User expectations:** Some users prefer animated effects. Making layout-based lighting opt-in and clearly documented avoids surprising changes to default behavior.
- **Schema assumptions:** We must confirm the correct `"lighting"` values for `vial.json` to avoid misrepresenting capabilities.

## Acceptance Criteria

1. With `lighting_mode = "qmk_default"`:
   - Generated `keymap.c`, `vial.json`, and `config.h` match current behavior (no additional lighting code, `"lighting": "none"`).
   - Firmware builds and runs exactly as before.

2. With `lighting_mode = "layout_static"` and a keyboard that has RGB Matrix support:
   - Firmware generation produces additional C code in the keymap folder implementing a static RGB effect.
   - After flashing, the keyboard shows a static color scheme that matches the layout's colors for the base layer.
   - No core files in `vial-qmk-keebart` have been modified.

3. The TUI exposes a way to configure or at least display the current lighting mode.

4. Tests cover:
   - Correct mapping from layout colors to LED index order.
   - Presence and basic structure of the generated lighting code when `layout_static` is enabled.
