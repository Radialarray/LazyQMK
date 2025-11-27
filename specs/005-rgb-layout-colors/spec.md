# Feature Specification: RGB Layout-Driven Lighting

**Feature Branch**: `005-rgb-layout-colors`  
**Created**: 2025-11-26  
**Status**: Draft  
**Input**: User description: "Hardware RGB currently shows QMK effects (rainbow). TUI has a rich color system (layers/categories/overrides), but those colors are not pushed into firmware. Add support for layout-driven per-key RGB on supported boards."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Layout Colors on Hardware (Priority: P1)

Users who carefully design color-coded layouts in the TUI (layer defaults, categories, per-key overrides) want their **physical keyboard LEDs** to mirror those colors instead of QMK’s default rainbow or animation effects.

**Why this priority**: The color system is already a core part of the TUI (P1 in 001-tui-complete-features). Without hardware parity, a major part of that value stops at the screen. This story delivers an immediately visible, end-to-end win using existing data.

**Independent Test**: Can be fully tested by configuring a layout with distinct colors per layer/category in the TUI, generating firmware, flashing the board, and verifying that keys light up with the same per-key colors and do **not** show the default QMK RGB effect.

**Acceptance Scenarios**:

1. **Given** a layout file with multiple layers and categories, **When** the user generates and flashes firmware, **Then** each key’s LED color on hardware matches the resolved color from the layout (taking into account overrides, categories, and layer defaults).
2. **Given** a key with an individual color override `{#RRGGBB}`, **When** firmware is flashed, **Then** that key’s LED shows the override color even if its category or layer has a different color.
3. **Given** a layer with a default color and no categories, **When** the board is on that layer, **Then** all keys on that layer show the layer default color (no per-key variation unless explicitly overridden).
4. **Given** a layout that previously produced a TUI-only color scheme while the hardware showed rainbow, **When** new firmware is generated and flashed, **Then** the hardware no longer runs the default RGB effect and instead shows static layout-driven colors.

---

### User Story 2 - Respect QMK Lighting Backend & Board Capabilities (Priority: P1)

Users with different keyboards (RGB Matrix vs RGBLIGHT vs no RGB) expect the tool to **do the right thing per board**: drive per-key colors only where supported and avoid breaking boards that use other lighting setups.

**Why this priority**: QMK has multiple lighting backends and lots of variation between boards. A one-size-fits-all codegen risks breaking builds or clobbering carefully tuned effects. This story ensures we integrate safely and predictably.

**Independent Test**: Can be tested by running firmware generation for (a) a keyboard with `RGB_MATRIX_ENABLE`, (b) a keyboard with `RGBLIGHT_ENABLE`, and (c) a keyboard with no RGB; builds should succeed in all three cases, and layout-driven colors should be active only where supported.

**Acceptance Scenarios**:

1. **Given** a keyboard with `RGB_MATRIX_ENABLE = yes`, **When** firmware is generated, **Then** keymap.c includes appropriate `rgb_matrix_*` hooks (e.g., `rgb_matrix_indicators_user` or a custom effect) and compiles without additional user edits.
2. **Given** a keyboard with `RGBLIGHT_ENABLE = yes` but no per-key matrix, **When** firmware is generated, **Then** TUI colors are mapped to a reasonable fallback (e.g., per-layer color) or the feature is explicitly disabled with a clear warning.
3. **Given** a keyboard with neither `RGB_MATRIX_ENABLE` nor `RGBLIGHT_ENABLE`, **When** firmware is generated, **Then** no lighting code or config is emitted and the build works as before.
4. **Given** a board-specific lighting configuration in `keyboard.json` (e.g., split counts, LED layout), **When** firmware is generated, **Then** the layout-to-LED mapping used for colors is consistent with the mapping we already use for keycodes.

---

### User Story 3 - Non-Destructive Integration with Existing QMK Config (Priority: P2)

Power users who already have custom QMK lighting or complex configs want the TUI-generated color behavior to be **opt-in and non-destructive**, avoiding overwriting keyboard-level configuration in ways that are hard to undo.

**Why this priority**: Many users maintain their own QMK repos or share keyboards with others. If our tool silently overwrites core keyboard files or disables effects globally, it will erode trust. Keeping everything scoped to the generated keymap is safer.

**Independent Test**: Can be tested by generating firmware for a keyboard with pre-existing lighting code and verifying that changes are limited to the generated keymap directory; base keyboard files remain untouched and can still be built with their original keymaps.

**Acceptance Scenarios**:

1. **Given** an upstream QMK keyboard with its own lighting effects, **When** the user generates a TUI keymap, **Then** all layout-driven lighting code lives under `keymaps/{tui-keymap}/` and does not modify base `keyboard.c`, `rules.mk`, or `config.h`.
2. **Given** a user switches back to a non-TUI keymap in QMK, **When** they build that keymap, **Then** the board behaves exactly as before with no residual changes from the TUI feature.
3. **Given** the user deletes the TUI-generated keymap directory from QMK, **When** they rebuild other keymaps, **Then** there are no build failures or lighting regressions.

---

### Edge Cases

- What happens when a layout has **more keys than LEDs** (e.g., non-lit positions or function keys without LEDs)?
- What happens when a layout has **fewer keys than LEDs** (e.g., underglow-only LEDs or decoration LEDs not mapped to keys)?
- How does the system behave if **no RGB configuration** exists in `keyboard.json`, but the user expects lighting?
- What happens when the user changes the **layout’s colors** and regenerates firmware without cleaning the QMK build directory? Do colors update reliably?
- How do we handle **layer changes at runtime**: does the board immediately reflect the new active layer’s colors, and what if multiple layers are active (e.g., momentary or tri-layer logic)?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST export a per-key color table from the TUI layout, using the same LED index order that is already used for keycodes.
- **FR-002**: System MUST integrate with QMK lighting APIs on supported boards (e.g., `rgb_matrix_*` hooks) to apply the exported colors at runtime.
- **FR-003**: System MUST respect the existing `Layout` color priority rules (key override > key category > layer category > layer default) when computing hardware colors.
- **FR-004**: System MUST disable or override the default QMK RGB effect for the TUI-generated keymap so that layout-driven colors are visible instead of the rainbow animation.
- **FR-005**: System MUST leave base keyboard files and global QMK configuration untouched, scoping all new lighting code to the generated keymap.
- **FR-006**: System MUST compile successfully on keyboards with `RGB_MATRIX_ENABLE` defined and with a valid LED layout in `keyboard.json`.
- **FR-007**: System MUST either no-op gracefully or provide a clear warning when run on keyboards without compatible RGB capabilities.
- **FR-008**: Users MUST be able to regenerate firmware after changing layout colors and see updated colors on hardware without manual edits to QMK sources.

### Key Entities *(include if feature involves data)*

- **LayoutColorMap**: Derived, in-memory structure mapping `(layer_index, led_index)` to `RgbColor` based on `Layout`, `KeyboardGeometry`, and `VisualLayoutMapping`.
- **LightingBackend**: Logical enum or inferred mode indicating whether the target keyboard uses `RGB_MATRIX`, `RGBLIGHT`, or has no RGB support.
- **GeneratedLightingHooks**: The C code fragments we emit into `keymap.c` (e.g., `rgb_matrix_indicators_user`, per-key color arrays) to apply layout-driven colors.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: On a supported RGB Matrix board (e.g., Corne with per-key RGB), at least 90% of keys visually match their TUI colors within one firmware flash cycle after changes.
- **SC-002**: Firmware generation and compilation continue to succeed with zero additional QMK warnings or errors attributable to the new lighting code on supported boards.
- **SC-003**: On unsupported boards (no RGB or incompatible lighting), firmware generation and compilation behavior is identical to the current baseline (no regressions).
- **SC-004**: Users can switch between a TUI-generated keymap and a non-TUI keymap without needing to modify or repair QMK core files.
