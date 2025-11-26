---

description: "Task list for 008-layer-aware-rgb: device-side, layer-aware RGB lighting"
---

# Tasks: 008-layer-aware-rgb

**Input**: This plan (`specs/008-layer-aware-rgb/plan.md`) plus existing TUI + firmware architecture.
**Prerequisites**: `src/models/layout.rs` color system, `src/models/visual_layout_mapping.rs`, `src/firmware/generator.rs`, vendored `vial-qmk-keebart` tree.

## Format: `[ID] [P?] [Story] Description`

(We group tasks by implementation phase instead of user stories, since this is a cross-cutting infrastructure feature.)

---

## Phase 1: Rust-Side Color Table Generation

- [ ] T001 [P] [P1] Analyze `Layout`, `Layer`, `VisualLayoutMapping` to confirm mapping from keys to LED indices (`src/models/layout.rs`, `src/models/keyboard_geometry.rs`, `src/models/visual_layout_mapping.rs`).
- [ ] T002 [P] [P1] Add internal helper in `src/firmware/generator.rs` to compute `base_color[layer][led]` using `Layout::resolve_key_color` and `VisualLayoutMapping::visual_to_led_index`.
- [ ] T003 [P] [P1] Handle error cases where a key's visual position cannot be mapped to an LED index, with clear error messages surfaced via firmware validation.
- [ ] T004 [P] [P1] Add unit tests in `src/firmware/generator.rs` tests module to verify color table generation for a small synthetic layout and geometry.

---

## Phase 2: Emit Color Tables into keymap.c

- [ ] T005 [P] [P1] Extend `FirmwareGenerator::generate_keymap_c` in `src/firmware/generator.rs` to emit static PROGMEM color tables (per-layer, per-LED) after the keymaps.
- [ ] T006 [P] [P1] Guard color table emission with appropriate `#ifdef` checks (e.g., `RGB_MATRIX_ENABLE`) so non-RGB keyboards still build without changes.
- [ ] T007 [P] [P1] Ensure generated symbol names are stable and discoverable from QMK code (document expected names in comments).
- [ ] T008 [P] [P1] Update existing generator unit tests (or add new ones) to assert that the color table is present in the generated `keymap.c` output when RGB is enabled.

---

## Phase 3: QMK Lighting Mode Integration

- [ ] T009 [P] [P1] Inspect `vial-qmk-keebart/quantum/rgb_matrix` to identify the cleanest extension point for a custom effect that consumes the baked color table.
- [ ] T010 [P] [P1] Add a new RGB Matrix effect implementation in `vial-qmk-keebart` that:
  - Uses the baked `base_color[layer][led]` data.
  - Reads QMK `layer_state` to determine effective layer per key.
  - Writes colors into the RGB matrix buffer.
- [ ] T011 [P] [P1] Wire the new effect into the relevant keyboard(s) so that the generated keymap selects it by default (or as a dedicated mode), updating `rules.mk`/`config.h` as needed.
- [ ] T012 [ ] [P1] Build QMK firmware for at least one RGB-capable keyboard and verify that the new effect compiles and links correctly.

---

## Phase 4: End-to-End Validation

- [ ] T013 [ ] [P1] Generate firmware from the TUI for an RGB keyboard and flash it.
- [ ] T014 [ ] [P1] Validate that base-layer colors on the hardware match those shown in the TUI keyboard view.
- [ ] T015 [ ] [P1] Press and hold a momentary layer key (e.g., `MO(1)`) and verify that keys affected by layer 1 show that layer's colors while held, reverting when released.
- [ ] T016 [ ] [P1] Confirm that non-RGB keyboards (or keyboards without RGB_MATRIX enabled) still generate and build firmware without color tables or lighting changes.

---

## Phase 5: Future Extension – Per-Key Animations (Flashing Keys)

- [ ] T017 [P] [P2] Define a minimal metadata format in `plan.md` for marking special keys/LEDs (e.g., Enter, Esc) and effect groups.
- [ ] T018 [P] [P2] Extend `FirmwareGenerator` in `src/firmware/generator.rs` to emit this metadata into `keymap.c` alongside the base color tables.
- [ ] T019 [P] [P2] Add QMK-side state and logic in the custom effect to support simple per-key animations (e.g., flashing a key for a short duration after it is pressed) using the base colors as a palette.
- [ ] T020 [ ] [P2] Manually test flashing behavior for at least one special key (e.g., Enter) across multiple layers, ensuring layer-aware colors are still respected.

---

## Phase N: Documentation & Cleanup

- [ ] T021 [P] [P2] Document the new lighting behavior and its limitations in `README.md` or an appropriate docs file.
- [ ] T022 [P] [P2] Add comments in `src/firmware/generator.rs` and the QMK effect implementation summarizing the data contract between generator and firmware.
- [ ] T023 [P] [P2] Run `cargo test` and a representative QMK build to confirm there are no regressions.

---

## Dependencies & Execution Order

- Phase 1 (Rust color table generation) must complete before Phase 2 and 3.
- Phase 2 (emitting tables) and Phase 3 (QMK effect) can proceed partially in parallel once the data shape is stable.
- Phase 4 (validation) depends on both generator and QMK effect being integrated.
- Phase 5 (per-key animations) is strictly optional and can be scheduled after the main feature ships.
- Documentation and cleanup can be done incrementally but should complete after functional work.
