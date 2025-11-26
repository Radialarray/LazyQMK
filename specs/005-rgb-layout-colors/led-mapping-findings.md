# LED-to-Switch Mapping Findings

## Scope

This document summarizes what we know so far about how physical RGB LEDs are mapped to switch (matrix) positions for the Corne Choc Pro in QMK, and how that relates to the TUI's geometry and mapping code.

Keyboard of interest:
- QMK keyboard: `keebart/corne_choc_pro/standard`
- Layout variant used by keyboard_tui: `LAYOUT_split_3x6_3_ex2`

## Sources

Primary files inspected:

- QMK / Vial repo
  - `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/info.json`
  - `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/standard/keyboard.json`
  - `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/mini/keyboard.json`
  - Generated keymap from keyboard_tui:
    - `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/standard/keymaps/default/keymap.c`

- keyboard_tui repo
  - `src/parser/keyboard_json.rs`
  - `src/models/keyboard_geometry.rs`
  - `src/models/visual_layout_mapping.rs`
  - `src/firmware/generator.rs`
  - `src/config.rs`
  - `src/tui/mod.rs`

## What QMK knows about the Corne Choc Pro LEDs

### 1. RGB Matrix capabilities and LED counts

From `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/info.json`:

- `features.rgb_matrix = true`
- `ws2812` is used as the LED driver:
  - `"ws2812": { "driver": "vendor", "pin": "GP10" }`
- Both `rgblight` and `rgb_matrix` are enabled in JSON metadata:
  - `rgblight.led_count = 46`, `split = true`, `split_count = [23, 23]`
  - `rgb_matrix` has a rich set of animations enabled.

Important: this JSON describes LED **counts** and some high-level behavior, but **not** the order of LEDs. The ordering is provided in the variant-specific `keyboard.json`.

### 2. Variant-specific RGB Matrix layout (LED ordering)

The Corne Choc Pro uses QMK's newer `keyboard.json` mechanism to define the physical RGB Matrix layout and derive `g_led_config`.

- Standard full-size variant:
  - File: `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/standard/keyboard.json`
  - Section: `"rgb_matrix": { "split_count": [23, 23], "layout": [ ... ] }`

- Mini variant:
  - File: `vial-qmk-keebart/keyboards/keebart/corne_choc_pro/mini/keyboard.json`
  - Section: `"rgb_matrix": { "split_count": [20, 20], "layout": [ ... ] }`

For the **standard** variant, the `rgb_matrix.layout` array has 46 entries. Each entry is of the form:

```json
{"matrix": [row, col], "x": <u8>, "y": <u8>, "flags": 4}
```

Example (first part of the array):

```json
{"matrix": [3, 5], "x": 95, "y": 63, "flags": 4},
{"matrix": [2, 5], "x": 85, "y": 39, "flags": 4},
{"matrix": [1, 5], "x": 85, "y": 21, "flags": 4},
{"matrix": [0, 5], "x": 85, "y": 4,  "flags": 4},
{"matrix": [0, 4], "x": 68, "y": 2,  "flags": 4},
{"matrix": [1, 4], "x": 68, "y": 19, "flags": 4},
{"matrix": [2, 4], "x": 68, "y": 37, "flags": 4},
{"matrix": [3, 4], "x": 80, "y": 58, "flags": 4},
...
```

**Key point:** the position of an entry in this array (its index 0..N-1) is the **LED index** used by QMK in `g_led_config`. QMK's JSON tools translate `rgb_matrix.layout` into a `led_config_t g_led_config` where:

- `g_led_config.matrix_co[row][col]` gives the LED index for a switch at matrix position `(row, col)`
- `g_led_config.point[led_index]` uses the `x`/`y` coordinates from the JSON
- `g_led_config.flags[led_index]` uses the `flags` field from the JSON

Thus, for `keebart/corne_choc_pro/standard`, the **authoritative LED index order** is:

- LED 0: matrix `(3,5)`
- LED 1: matrix `(2,5)`
- LED 2: matrix `(1,5)`
- LED 3: matrix `(0,5)`
- LED 4: matrix `(0,4)`
- LED 5: matrix `(1,4)`
- LED 6: matrix `(2,4)`
- LED 7: matrix `(3,4)`
- LED 8: matrix `(3,3)`
- LED 9: matrix `(2,3)`
- LED 10: matrix `(1,3)`
- LED 11: matrix `(0,3)`
- LED 12: matrix `(0,2)`
- LED 13: matrix `(1,2)`
- LED 14: matrix `(2,2)`
- LED 15: matrix `(2,1)`
- LED 16: matrix `(1,1)`
- LED 17: matrix `(0,1)`
- LED 18: matrix `(0,0)`
- LED 19: matrix `(1,0)`
- LED 20: matrix `(2,0)`
- LED 21: matrix `(0,6)`
- LED 22: matrix `(1,6)`
- LED 23: matrix `(7,5)`
- LED 24: matrix `(6,5)`
- LED 25: matrix `(5,5)`
- LED 26: matrix `(4,5)`
- LED 27: matrix `(4,4)`
- LED 28: matrix `(5,4)`
- LED 29: matrix `(6,4)`
- LED 30: matrix `(7,4)`
- LED 31: matrix `(7,3)`
- LED 32: matrix `(6,3)`
- LED 33: matrix `(5,3)`
- LED 34: matrix `(4,3)`
- LED 35: matrix `(4,2)`
- LED 36: matrix `(5,2)`
- LED 37: matrix `(6,2)`
- LED 38: matrix `(6,1)`
- LED 39: matrix `(5,1)`
- LED 40: matrix `(4,1)`
- LED 41: matrix `(4,0)`
- LED 42: matrix `(5,0)`
- LED 43: matrix `(6,0)`
- LED 44: matrix `(4,6)`
- LED 45: matrix `(5,6)`

This is **not** the same as row-major or column-major order over the switch matrix, and it is also **not** the same as the order of keys in the `info.json` layout definition.

### 3. Relationship to `LAYOUT_split_3x6_3_ex2`

The Corne Choc Pro's `info.json` defines several layouts, including:

- `LAYOUT_split_3x5_3`
- `LAYOUT_split_3x6_3`
- `LAYOUT_split_3x5_3_ex2`
- `LAYOUT_split_3x6_3_ex2`

Each layout's `layout` array:

- Assigns a `matrix` position to each key
- Provides `x`/`y` visual coordinates in keyboard units
- Does **not** encode any LED index or LED order

Our generator currently uses `LAYOUT_split_3x6_3_ex2` to instantiate QMK's `LAYOUT_split_3x6_3_ex2(...)` macro with keycodes and colors.

The RGB Matrix layout in `standard/keyboard.json` is **consistent** with the matrix dimensions from `info.json`, but the **ordering** of LED entries in `rgb_matrix.layout` is specific to how the WS2812 strip is wired on the PCB.

## What keyboard_tui currently assumes

### 1. Geometry construction from info.json

`src/parser/keyboard_json.rs:295` builds `KeyboardGeometry` from the **info.json** layout definition:

- For each key in `info.layouts[layout_name].layout`:
  - `matrix_position` is taken directly from `key_pos.matrix`
  - `visual_x`/`visual_y` are taken from `key_pos.x`/`key_pos.y`
  - **`led_index` is currently set to the index in the layout array**:

```rust
for (led_index, key_pos) in layout_def.layout.iter().enumerate() {
    let matrix_position = key_pos.matrix.unwrap();

    let key_geometry = KeyGeometry {
        matrix_position: (matrix_position[0], matrix_position[1]),
        led_index: led_index as u8,
        visual_x: key_pos.x,
        visual_y: key_pos.y,
        width: key_pos.w,
        height: key_pos.h,
        rotation: key_pos.r,
    };

    keys.push(key_geometry);
}
```

This means:

- LED index 0 is **always** the first key in `layout_def.layout` for the selected layout.
- For Corne layouts, this is typically the top-left alpha key, whereas QMK's RGB Matrix LED 0 is a thumb or edge key depending on how the strip is wired.

### 2. Visual layout mapping

`src/models/visual_layout_mapping.rs` builds the `VisualLayoutMapping` from `KeyboardGeometry`:

- `led_to_matrix[led_index] = matrix_position`
- `matrix_to_led[matrix_position] = led_index`
- `matrix_to_visual` and `visual_to_matrix` are derived from the `visual_x`/`visual_y` coordinates rounded to a grid.

Consequences:

- `visual_to_led_index(row, col)` looks up `visual_to_matrix` then `matrix_to_led`.
- Because `matrix_to_led` used the **info.json key index** as LED index, the resulting mapping corresponds to "info.json key order", not to QMK's `g_led_config` LED order.

### 3. Firmware generator usage

`src/firmware/generator.rs` uses the mapping as follows:

- For keycodes (`generate_layer_keys_by_led`):
  - For each key in a layer:
    - Visual → matrix → LED (via `VisualLayoutMapping`)
    - Place the keycode string at index `led_idx` in `keys_by_led`.
  - Emit a `LAYOUT_*()` call with arguments in **LED index order**.

- For colors (`generate_layer_colors_by_led`):
  - Same mapping, but place `RgbColor` values into `colors_by_led[led_idx]`.
  - These become `layer_colors[layer][led]` in the generated `keymap.c`.

This means that both keycodes and colors are ordered by our **internal LED index**, which currently equals **info.json layout index**, not QMK's LED index.

### 4. Interaction with QMK's RGB Matrix

The generated `keymap.c` for Corne Choc Pro (`standard` variant) contains:

- Correct keymaps: keys type correctly, because the `LAYOUT_split_3x6_3_ex2(...)` macro uses matrix positions, and those we pass are consistent with `info.json`.
- RGB Matrix indicators:

```c
const rgb_t PROGMEM layer_colors[][RGB_MATRIX_LED_COUNT] = {
    [0] = { {255, 0, 0}, {128, 128, 128}, ... }
};

bool rgb_matrix_indicators_user(void) {
    uint8_t layer = get_highest_layer(layer_state | default_layer_state);
    if (layer >= (sizeof(layer_colors) / sizeof(layer_colors[0]))) {
        return false;
    }

    for (uint8_t i = 0; i < RGB_MATRIX_LED_COUNT; i++) {
        rgb_matrix_set_color(i,
            layer_colors[layer][i].r,
            layer_colors[layer][i].g,
            layer_colors[layer][i].b);
    }

    return false;
}
```

Here, `i` is QMK's **RGB Matrix LED index**, which expects the same ordering as in `g_led_config` / `keyboard.json`'s `rgb_matrix.layout`.

Because our `layer_colors[layer][i]` is built with a **different LED ordering** (info.json index), the right colors are applied to the wrong physical LEDs. This matches the observed behavior: colors appear, but are misaligned relative to keys.

## Correct LED-to-switch mapping for Corne Choc Pro (standard)

Based on `standard/keyboard.json`, the mapping between **QMK LED indices** and **matrix positions** for `keebart/corne_choc_pro/standard` is:

- LED 0 → matrix (3, 5)
- LED 1 → matrix (2, 5)
- LED 2 → matrix (1, 5)
- LED 3 → matrix (0, 5)
- LED 4 → matrix (0, 4)
- LED 5 → matrix (1, 4)
- LED 6 → matrix (2, 4)
- LED 7 → matrix (3, 4)
- LED 8 → matrix (3, 3)
- LED 9 → matrix (2, 3)
- LED 10 → matrix (1, 3)
- LED 11 → matrix (0, 3)
- LED 12 → matrix (0, 2)
- LED 13 → matrix (1, 2)
- LED 14 → matrix (2, 2)
- LED 15 → matrix (2, 1)
- LED 16 → matrix (1, 1)
- LED 17 → matrix (0, 1)
- LED 18 → matrix (0, 0)
- LED 19 → matrix (1, 0)
- LED 20 → matrix (2, 0)
- LED 21 → matrix (0, 6)
- LED 22 → matrix (1, 6)
- LED 23 → matrix (7, 5)
- LED 24 → matrix (6, 5)
- LED 25 → matrix (5, 5)
- LED 26 → matrix (4, 5)
- LED 27 → matrix (4, 4)
- LED 28 → matrix (5, 4)
- LED 29 → matrix (6, 4)
- LED 30 → matrix (7, 4)
- LED 31 → matrix (7, 3)
- LED 32 → matrix (6, 3)
- LED 33 → matrix (5, 3)
- LED 34 → matrix (4, 3)
- LED 35 → matrix (4, 2)
- LED 36 → matrix (5, 2)
- LED 37 → matrix (6, 2)
- LED 38 → matrix (6, 1)
- LED 39 → matrix (5, 1)
- LED 40 → matrix (4, 1)
- LED 41 → matrix (4, 0)
- LED 42 → matrix (5, 0)
- LED 43 → matrix (6, 0)
- LED 44 → matrix (4, 6)
- LED 45 → matrix (5, 6)

We can equivalently think of this as:

- A **left half** chain of 23 LEDs (indices 0–22), starting at the lower thumb/outer columns and snaking through the left keys.
- A **right half** chain of 23 LEDs (indices 23–45), similarly arranged.

This ordering corresponds to the actual WS2812 daisy-chain on the PCB, and is the order used by QMK for all RGB Matrix effects.

## Implications for keyboard_tui

1. Our current assumption that `led_index == info.json layout index` is **invalid** for boards like Corne Choc Pro where `keyboard.json` defines a distinct RGB Matrix layout.

2. For such boards, we must align `KeyboardGeometry.led_index` with the **QMK LED index** from `keyboard.json`'s `rgb_matrix.layout`, not with the `info.json` layout order.

3. Concretely for Corne Choc Pro standard:
   - `KeyboardGeometry.matrix_position` should still come from `info.json` / selected layout (e.g., `LAYOUT_split_3x6_3_ex2`).
   - `KeyboardGeometry.led_index` should be derived by looking up the matrix position in `standard/keyboard.json`'s `rgb_matrix.layout` array and using that array index as LED index.

4. Once `KeyboardGeometry.led_index` is corrected:
   - `VisualLayoutMapping::build` will create correct `matrix_to_led` and `visual_to_led_index` mappings.
   - `FirmwareGenerator::generate_layer_keys_by_led` and `generate_layer_colors_by_led` will order keycodes and colors by QMK's real LED indices.
   - The generated `layer_colors[layer][i]` will align with QMK's expectation in `rgb_matrix_set_color(i, ...)`, so physical LEDs will show the intended per-key layout colors.

## Next steps (implementation-oriented)

To fix the mismatch for Corne Choc Pro (and similar boards) in the generator:

1. **Augment geometry building with keyboard.json LED data**
   - When building `KeyboardGeometry` for a keyboard that has variant directories and `keyboard.json`, read the appropriate `keyboard.json` file (e.g., `keebart/corne_choc_pro/standard/keyboard.json`).
   - Extract its `rgb_matrix.layout` array.
   - Build a map: `matrix_position (row, col) -> led_index` based on the order in that array.

2. **Override `led_index` for matching matrix positions**
   - Keep the existing `build_keyboard_geometry` logic that derives `matrix_position`, `visual_x`, `visual_y`, etc., from `info.json`.
   - For each key, if we find a matching `(row, col)` in the `rgb_matrix.layout` mapping:
     - Assign `key_geometry.led_index = <QMK LED index>`.
   - For matrix positions that have no LED (e.g., non-backlit keys, or extra matrix positions), either:
     - Set `led_index` to some sentinel value and exclude them from LED-based features, or
     - Continue using the sequential index but ensure they are never included in RGB Matrix loops.

3. **Scope and generalization**
   - The mechanism above should be designed to work for any keyboard that:
     - Has a `keyboard.json` with an `rgb_matrix.layout` section
     - And for which our selected `info.json` layout uses the same matrix coordinate system.
   - Corne Choc Pro standard is the first concrete board we will support; others can follow the same pattern.

4. **Testing and validation**
   - Add unit tests that:
     - Load the Corne Choc Pro `info.json` + `standard/keyboard.json` fixtures.
     - Build `KeyboardGeometry` for `LAYOUT_split_3x6_3_ex2`.
     - Assert that for a few representative keys, `(row, col)` maps to the expected `led_index` from the table above.
   - On hardware:
     - Regenerate firmware using keyboard_tui.
     - Flash onto Corne Choc Pro standard.
     - Verify that per-key colors now align with the correct physical switches on all layers.

This document should be kept in sync with any future changes to how we consume QMK's `keyboard.json` or how we derive LED mappings for other keyboards.