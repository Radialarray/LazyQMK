# Tasks: Per-Key RGB Export from Layout Colors

**Status**: Planning  
**Input**: `/specs/005-per-key-rgb-export/spec.md`, `/specs/005-per-key-rgb-export/plan.md`  
**Prerequisites**: Existing color model, firmware generator, and QMK submodule

## Format: `- [ ] [ID] [P?] Description`

- **[P]**: Can run in parallel (different files, low coupling)
- All paths are from repository root

---

## Phase 1: Config Surface and Modes

**Goal**: Introduce an opt-in lighting mode without changing behavior by default.

- [ ] R001 [P] Add `lighting_mode` field to `BuildConfig` or a dedicated lighting config struct in `src/config.rs` with serde support and a default value equivalent to current behavior (e.g., `"qmk_default"`).
- [ ] R002 [P] Extend config validation (if needed) to ensure `lighting_mode` is one of the supported values.
- [ ] R003 Wire `lighting_mode` into the TUI configuration dialogs (e.g., in `src/tui/config_dialogs.rs` or onboarding wizard) so it can be inspected/changed from the UI.

---

## Phase 2: Color → LED Mapping Helper

**Goal**: Compute per-LED colors from the existing layout model and geometry.

- [ ] R010 [P] In `src/firmware/generator.rs`, add a helper method (e.g., `generate_layer_colors_by_led(&self, layer_idx: usize) -> Result<Vec<RgbColor>>`) that:
  - Uses `self.layout.layers[layer_idx]`.
  - For each key, calls `self.layout.resolve_key_color(layer_idx, key)`.
  - Maps `key.position` to LED index via `self.mapping.visual_to_led_index`.
  - Returns a `Vec<RgbColor>` sized to `self.mapping.key_count()` ordered by LED index.
- [ ] R011 [P] Add unit tests for this helper in the existing test module at the bottom of `src/firmware/generator.rs`.

---

## Phase 3: Generated C Code for Static RGB

**Goal**: Add QMK-compatible static RGB code to the generated keymap when lighting mode is enabled.

- [ ] R020 Extend `generate_keymap_c()` in `src/firmware/generator.rs` to optionally append a `#ifdef RGB_MATRIX_ENABLE` section when `lighting_mode == "layout_static"`.
- [ ] R021 [P] Design the C representation of colors (RGB vs HSV) and implement Rust-side formatting to emit a `const` array sized to `RGB_MATRIX_LED_COUNT`.
- [ ] R022 Implement a generated `rgb_matrix_indicators_user()` (or `_kb`) function that sets each LED color from the array, using standard QMK APIs like `rgb_matrix_set_color` or `rgb_matrix_set_color_hsv`.
- [ ] R023 Ensure the generated C code compiles in the existing QMK fork for at least one representative keyboard (e.g., the default dev keyboard), adjusting function names or includes as needed while keeping changes confined to the keymap directory.
- [ ] R024 Add generator tests in `tests/firmware_gen_tests.rs` (or a new test) that assert the presence of the RGB section when `lighting_mode == "layout_static"` and its absence otherwise.

---

## Phase 4: Vial JSON and config.h Integration

**Goal**: Reflect lighting capabilities in `vial.json` and keep `config.h` safe and compatible.

- [ ] R030 [P] Update `generate_vial_json()` in `src/firmware/generator.rs` to set the `"lighting"` field based on `lighting_mode` (e.g., keep `"none"` for `qmk_default`, set an appropriate non-`"none"` value for `layout_static` after confirming Vial schema).
- [ ] R031 [P] Ensure `generate_merged_config_h()` in `src/firmware/generator.rs` does not introduce conflicting RGB-related defines; if necessary, add small, conservative defines or comments only when `lighting_mode == "layout_static"`.

---

## Phase 5: TUI Integration and UX

**Goal**: Make lighting mode discoverable and clearly documented to users.

- [ ] R040 [P] Update relevant help or quickstart text (e.g., `QUICKSTART.md` or help overlay content in `src/tui/help_overlay.rs`) to mention the new lighting mode and its effect on firmware.
- [ ] R041 [P] Update status or build messages (e.g., in `src/tui/build_log.rs` or builder UI) to indicate when a build is using layout-driven static lighting.

---

## Phase 6: Validation and Guardrails

**Goal**: Ensure robust behavior across different keyboards and configs.

- [ ] R050 [P] Add checks in firmware generation to detect when the selected keyboard does not have RGB Matrix support and either:
  - Skip generating static lighting code with a clear warning, or
  - Generate code guarded with appropriate `#ifdef` so builds still succeed.
- [ ] R051 Add at least one integration or contract test that runs firmware generation end-to-end with `lighting_mode == "layout_static"` on a small test geometry and verifies that both `keymap.c` and `vial.json` look as expected.

---

## Phase 7: Follow-Ups (Optional / Future)

**Goal**: Prepare for richer lighting features without committing to them now.

- [ ] R060 Investigate how `RGB_MATRIX_EFFECT_VIALRGB_DIRECT` and `quantum/vialrgb.c` work in the bundled QMK fork and document options for v2 (e.g., exporting layout colors into a Vial-compatible profile).
- [ ] R061 Prototype multi-layer color behavior (different static colors when switching layers) by extending the color → LED helper and generated C code, ensuring it reads from `layer_state`.
