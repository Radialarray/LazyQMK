# Implementation Plan: Per-Key RGB Export

**Branch**: `005-per-key-rgb-export` | **Date**: 2025-11-26 | **Spec**: _TBD (this plan documents the initial design; a full spec can be added later if needed)_

**Input**: Current behavior of `keyboard_tui` where TUI colors are purely visual and are not exported into QMK/Vial firmware.

## Summary

Today, the color system is end-to-end inside the editor and Markdown (layers, categories, and per-key overrides) but is completely ignored by firmware generation. `keymap.c`, `config.h`, and `vial.json` contain only keycodes and geometry; the keyboard firmware therefore boots with whatever RGB matrix defaults the underlying QMK keyboard defines (often a rainbow flow effect).

Goal of this feature: add an optional, QMK-compatible export path so that the layout's colors can drive on-keyboard lighting. At minimum this means:

- Being able to generate a static per-key color mapping that matches the LED order (`RGB_MATRIX_LED_COUNT` / LED index mapping).
- Wiring that color mapping into the QMK/Vial stack in a way that is robust across boards that already have RGB features.
- Preserving backwards compatibility and making this behavior opt-in so existing users are not surprised.

## Design Constraints

- All behavior (including lighting mode) must be controlled via the TUI and configuration files that `keyboard_tui` owns; users should not need to manually edit C files or JSON inside the QMK/Vial tree.
- We MUST NOT modify existing tracked source files in the `vial-qmk-keebart` submodule (e.g., `quantum/*.c`, `quantum/*.h`, base keyboard `config.h`, `rules.mk`, or `info.json`).
- All new lighting integration must be implemented by generating or updating files only under the keymap-specific directory, e.g. `vial-qmk-keebart/keyboards/{keyboard}/keymaps/{keymap}/`.
- The design should allow the `vial-qmk-keebart` submodule to be updated freely; regenerating firmware from the TUI should re-apply layout and lighting on top of the updated QMK tree without manual conflict resolution.

## What is Missing Right Now

1. **No representation of lighting in firmware outputs**
   - `src/firmware/generator.rs` only emits:
     - `keymap.c` with `const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS]` (keycodes only).
     - `vial.json` with keyboard geometry and layout (`"lighting": "none"`).
     - `config.h` with a header and optional Vial unlock combo.
   - There is no generated data structure that contains per-key HSV/RGB values for QMK to consume.

2. **Vial config explicitly disables lighting**
   - `generate_vial_json` hardcodes `"lighting": "none"`, so Vial treats the generated keymap as having no controllable lighting.
   - The TUI has no concept of which lighting mode is active or whether the keyboard supports RGB matrix vs. underglow vs. none.

3. **No integration with QMK RGB APIs**
   - The project includes a full QMK/Vial fork under `vial-qmk-keebart/` that implements `RGB_MATRIX_ENABLE`, `VIALRGB_ENABLE`, etc.
   - However, `keyboard_tui` never:
     - Enables `VIALRGB_DIRECT` or any other static-mode integration in `config.h` or `rules.mk` for the generated keymap.
     - Writes any C code (e.g., custom `rgb_matrix_indicators_kb()` or VialRGB direct key colors) that would apply layout-based colors at runtime.

4. **No configuration surface for lighting behavior**
   - `Config` (`src/config.rs`) has no fields for "lighting strategy" (e.g., static layout colors vs. QMK default effects).
   - There is no way in the UI to say: "Use my layout colors as the default static color scheme" vs. "Leave RGB alone".

5. **No tests or contracts around lighting**
   - Existing tests validate keymap generation, Vial JSON structure, and geometry, but there are no contract tests asserting how lighting should behave.

## High-Level Solution Approach

We need a minimal but extensible path from `Layout` colors → QMK lighting. A reasonable first increment:

1. **Introduce an opt-in lighting mode in config**
   - Extend `BuildConfig` or `Config` to include a new field, e.g.:
     - `lighting_mode: String` with values like `"none" | "layout_static" | "qmk_default"` (final naming TBD).
   - Default to current behavior (`"qmk_default"` / no special export) for backward compatibility.
   - This mode will drive:
     - How `vial.json` sets its `"lighting"` field.
     - Whether additional C code for static colors is generated.

2. **Define an internal representation for per-LED colors**
   - Reuse the existing `Layout::resolve_key_color(layer_idx, key)` logic.
   - Add a helper in firmware generation to compute a per-LED color array:
     - Input: `Layout`, `VisualLayoutMapping`, active layer index (or multiple layers for advanced features later).
     - Output: `Vec<RgbColor>` ordered by LED index `0..RGB_MATRIX_LED_COUNT`.
   - This helper will be used by any export path (QMK static effect, Vial direct mode, debugging output, etc.).

3. **Choose a QMK integration strategy (v1: static RGB matrix effect)**

   For a first version, we can target a simple and broadly compatible approach:

   - Generate a custom `rgb_matrix_indicators_kb()` or `rgb_matrix_indicators_user()` implementation that:
     - Runs when RGB Matrix is active.
     - Sets each LED's color to the layout-based value for the active layer (or just layer 0 in v1).
   - Wire this in as a small C file in the generated keymap directory, e.g. `rgb_matrix_static.c` or `keymap.c` section guarded by `#ifdef RGB_MATRIX_ENABLE`.
   - Implementation sketch:
     - Expose a `const hsv_t layout_colors[RGB_MATRIX_LED_COUNT] PROGMEM` array in C.
     - Convert `RgbColor` (RGB) to HSV offline (in Rust) or inline in C.
     - In `rgb_matrix_indicators_user()`, loop over LEDs and call `rgb_matrix_set_color(i, r, g, b)` or `rgb_matrix_set_color_hsv()`.
   - Advantages:
     - Uses standard QMK hooks; compatible with many boards.
     - Localized to the generated keymap directory; does not touch the shared QMK fork.
   - Limitations (acceptable for v1):
     - Might conflict with some animated effects (we can specify that `lighting_mode = "layout_static"` implies that the default mode is a static base effect).
     - Multi-layer color schemes (different colors when you change layers) would require additional state and integration with keymaps or `layer_state` callbacks.

4. **Optional: Explore VialRGB direct mode as v2**
   - The bundled QMK fork includes `quantum/vialrgb.c` and `RGB_MATRIX_EFFECT_VIALRGB_DIRECT`, which support per-LED colors driven from host.
   - A future extension (not necessarily in v1) could:
     - Export layout colors into a Vial lighting profile so that the host UI (Vial) can reflect or adjust them.
     - Set `"lighting": "vialrgb"` (or similar) in `vial.json` when the keyboard and keymap support it.
   - This requires more cross-repo contract work and careful testing, so it should be a separate phase.

5. **Update Vial JSON and config.h based on lighting mode**

   - When `lighting_mode = "layout_static"`:
     - `generate_vial_json` should set `"lighting"` to a non-`"none"` value that indicates the board has RGB matrix lighting (exact value depends on Vial's schema; to be researched in implementation).
     - `generate_merged_config_h` should:
       - Optionally define a sane `RGB_MATRIX_DEFAULT_MODE` if the base keymap does not already do so, or
       - At least avoid conflicting defines and leave the effect selection to our generated `rgb_matrix_indicators_*` code.
   - When `lighting_mode = "qmk_default"`:
     - Preserve status quo: `"lighting": "none"` and no extra lighting code.

6. **Add tests and example artifacts**

   - Unit-level tests for the new Rust helper that computes `Vec<RgbColor>` in LED order.
   - Contract/integration tests that:
     - Generate firmware with `lighting_mode = "layout_static"` for a small test keyboard geometry.
     - Assert that the generated C file contains the expected color array and uses the expected QMK hooks.
     - (Optionally) compile the generated keymap against a tiny test QMK target in CI to catch obvious breakages.

## Implementation Steps (High Level)

1. **Config surface**
   - Extend `BuildConfig` or introduce a new `LightingConfig` in `src/config.rs`.
   - Provide sensible defaults and, if needed, a minimal UI to toggle the mode.

2. **Color-to-LED mapping helper**
   - In `src/firmware/generator.rs` (or a small helper module), implement:
     - `fn generate_layer_colors_by_led(&self, layer_idx: usize) -> Result<Vec<RgbColor>>` using `Layout::resolve_key_color` and `VisualLayoutMapping::visual_to_led_index`.

3. **C code generation for static RGB**
   - Extend `generate_keymap_c` or add a new function to append a `#ifdef RGB_MATRIX_ENABLE` section that:
     - Declares a color array in LED order.
     - Implements `rgb_matrix_indicators_user()` (or `_kb`) to apply those colors.
   - Decide whether to store colors as RGB or HSV in C (HSV is more idiomatic in QMK, but RGB is simpler to generate).

4. **Vial JSON & config.h updates**
   - Modify `generate_vial_json` to choose `"lighting"` based on `lighting_mode`.
   - Modify `generate_merged_config_h` to avoid conflicts with existing keyboard-level RGB settings and, if necessary, add small, safe defines.

5. **Testing & documentation**
   - Add tests in `tests/firmware_gen_tests.rs` (or new tests) covering:
     - Color mapping function.
     - Presence of the generated C lighting code under the appropriate mode.
   - Update `QUICKSTART.md` and/or help overlay spec to mention the new lighting mode and its implications.

## Risks and Open Questions

- **Hardware diversity**: Different keyboards use different LED drivers and may have custom RGB matrix code; our static-indicator approach must not break these.
- **Performance**: Updating every LED each matrix scan may have performance implications on low-power MCUs; we might need to restrict updates to when the layout or active layer changes.
- **Vial expectations**: The exact semantics of `"lighting"` in `vial.json` need to be confirmed so we don't mis-advertise capabilities.
- **User expectations**: Some users may still prefer animated effects; we must keep our feature opt-in and document that enabling layout-driven static lighting will override some default animations.

This plan focuses the first iteration on a conservative, static RGB implementation using standard QMK hooks and the existing layout color model. Further iterations can refine the integration with VialRGB and add richer per-layer or reactive lighting.
