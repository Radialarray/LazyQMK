# Spec: 008-layer-aware-rgb

## Feature Overview

Provide **layer-aware, TUI-driven RGB lighting** for supported QMK keyboards. The TUI layout editor already defines rich per-layer, per-key colors using a four-level priority system (global default, layer default, category, key override). This feature ensures that those semantics are baked into the generated firmware and reflected live on the keyboard, including when layers change via `MO`, `TG`, and other QMK mechanisms.

## Goals

- **G1**: When a user edits key colors in the TUI (per-key, per-layer, via categories or layer defaults), the flashed firmware should display matching colors on the physical keyboard.
- **G2**: Colors must be **layer-aware**: holding `MO(1)` or activating other layers should cause affected keys to show the active layer's color, following QMK's normal layer resolution rules.
- **G3**: The solution must work with existing QMK RGB Matrix infrastructure (brightness, suspend, global mode selection) and not break boards without RGB.
- **G4**: Users must be able to opt out and keep stock QMK lighting (via `lighting_mode = "qmk_default"`).

## Non-Goals (for this spec)

- Complex time-based animations (breathing, waves, reactive typing effects) beyond a static, layer-aware color base.
- Automatic per-key animation groups (Enter flashing, etc.); these are a future extension.
- UI-level controls for selecting individual QMK RGB modes; this spec focuses on a single, well-defined TUI-driven default.

## Requirements

### R1: Data Generation (Rust side)

- R1.1: The firmware generator must compute a **per-layer, per-LED base color table** from the layout:
  - Shape: `layer_base_colors[layer_index][led_index][3]` with 8-bit RGB channels.
  - Guarded by `#ifdef RGB_MATRIX_ENABLE` and only emitted when the keyboard has an RGB matrix.
- R1.2: The generator must also emit the number of colored layers:
  - `const uint8_t PROGMEM layer_base_colors_layer_count = <num_layers>;`
- R1.3: Colors must follow the existing resolution order:
  1. Key override color
  2. Key category color
  3. Layer default color
  4. Global default (`#FFFFFF`)
- R1.4: Visual positions from the layout markdown must be correctly mapped to LED indices via `VisualLayoutMapping`, staying consistent with how `keymaps` are generated.

### R2: Firmware Integration (QMK side)

- R2.1: Implement a **new RGB Matrix effect** named `TUI_LAYER_COLORS` and ensure `RGB_MATRIX_TUI_LAYER_COLORS` is a valid mode macro/enum in the forked QMK tree.
- R2.2: The effect must, for each RGB LED:
  - Map LED index → matrix row/col using `g_led_config`.
  - Determine the **effective layer** for that key using QMK's layer resolution (respecting `KC_TRNS` fall-through and `layer_state`).
  - Read the base color from `layer_base_colors[effective_layer][led_index]` and feed it to `rgb_matrix_set_color`.
- R2.3: The effect must respect global RGB Matrix constraints:
  - Honor brightness settings.
  - Honor suspend/power-off behavior.
  - Play nicely with QMK's effect dispatch (`rgb_matrix_check_finished_leds`, etc.).

### R3: Behavior and Modes

- R3.1: When `lighting_mode = "layout_static"` in `config.toml` and the keyboard has RGB:
  - Generated `config.h` must define:
    - `RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_TUI_LAYER_COLORS`.
    - `LAYER_BASE_COLORS_LAYER_COUNT <num_layers>` (used by firmware if needed).
  - Generated `keymap.c` must include `layer_base_colors` and `layer_base_colors_layer_count`.
- R3.2: When `lighting_mode = "qmk_default"`:
  - The generator must **not** override `RGB_MATRIX_DEFAULT_MODE` in `config.h`.
  - Emission of `layer_base_colors` is optional; if present, it must not affect default behavior unless the user selects the mode manually.
- R3.3: Avoid conflicting RGB hooks:
  - `rgb_matrix_indicators_user` and any static `layout_colors` section must be reconciled so they do not fight with `TUI_LAYER_COLORS`.

### R4: Compatibility and Safety

- R4.1: Boards without RGB Matrix must build unchanged:
  - No references to `RGB_MATRIX_LED_COUNT` or RGB APIs when the feature is not enabled.
- R4.2: The generated keymap must remain valid for other QMK tooling (Vial, QMK configurator) and not introduce linker errors.
- R4.3: The solution must tolerate additional layers beyond those colored (e.g., transparent utility layers); uncolored layers should fall back to sensible defaults (e.g., base layer colors or white).

## Design Decisions

- **D1: One source of truth for colors**
  - All color semantics are defined in the TUI layout (markdown) and resolved by Rust; QMK treats `layer_base_colors` as authoritative and does not recompute color semantics.
- **D2: Effect, not indicators**
  - We prefer a full RGB Matrix effect (`RGB_MATRIX_TUI_LAYER_COLORS`) over hard-wiring behavior in `rgb_matrix_indicators_user`, to keep behavior consistent with QMK's mode system and allow users to switch effects.
- **D3: Layer resolution in firmware**
  - The firmware, not the generator, is responsible for resolving which layer is currently active at a matrix position. This keeps runtime behavior correct for momentary/latched layers and custom QMK keycodes.

## Acceptance Criteria

- A1: For a test layout (e.g., `keebart_corne_choc_pro` with two layers and distinct colors per layer), the generated `keymap.c` includes correctly structured `layer_base_colors` that match the editor view.
- A2: Flashing the generated firmware with `lighting_mode = "layout_static"` results in:
  - Base layer keys showing their TUI-defined colors.
  - Holding `MO(1)` (or switching to Layer 1) causes affected keys to change to Layer 1 colors in real time.
- A3: With `lighting_mode = "qmk_default"`, the keyboard's lighting behaves as before (stock QMK modes) unless the user manually selects `TUI_LAYER_COLORS`.
- A4: Non-RGB boards (or boards built without `RGB_MATRIX_ENABLE`) compile successfully and behave the same as before enabling this feature.

## Open Questions / Future Work

- Q1: Should we provide a way to blend base colors with existing QMK animations (e.g., hue from layout, brightness from an effect)?
- Q2: How should we expose TUI control over choosing between `TUI_LAYER_COLORS` and stock QMK modes?
- Q3: For boards with per-key backlight (not matrix), is there a minimal subset of this feature we can reuse?
