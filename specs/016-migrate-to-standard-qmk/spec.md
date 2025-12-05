# Feature Specification: Migrate from Vial QMK to Standard QMK

**Feature Branch**: `016-migrate-to-standard-qmk`  
**Created**: 2025-12-05  
**Status**: Draft  
**Input**: User request to migrate from vial-qmk-keebart fork to standard QMK firmware

## Background & Motivation

The keyboard-configurator currently uses a Vial QMK fork (`vial-qmk-keebart`) which includes Vial-specific modifications. This has caused issues:

1. **Keycode compatibility bugs**: The Vial fork's `vial_ensure_keycode.h` defines then `#undef`s macros like `LCG()`, `RCG()`, causing compilation errors
2. **Maintenance burden**: Keeping a forked QMK repository in sync with upstream is ongoing work
3. **Unnecessary complexity**: The configurator doesn't use Vial's live-editing features (USB HID communication)
4. **Standard compatibility**: Users expect standard QMK keycodes to work per QMK documentation

### What We Lose

- **Vial app compatibility**: No live keymap editing via Vial desktop/web app
- **VIA compatibility**: Unless explicitly enabled in standard QMK
- **Dynamic keymap storage**: All changes require reflashing

### What We Gain

- **Full QMK keycode support**: All documented QMK keycodes work correctly
- **Simpler maintenance**: Use upstream QMK directly
- **Better documentation alignment**: QMK docs match actual behavior
- **Smaller firmware size**: No Vial overhead

## User Scenarios & Testing

### User Story 1 - Standard QMK Keycode Support (Priority: P1)

As a keyboard configurator user, I need all standard QMK keycodes (like `LCG()`, `RCG()`, `LCS()`) to compile correctly so that I can use any keycode documented in QMK's official documentation.

**Why this priority**: This is the primary motivation for the migration - fixing keycode compilation errors.

**Independent Test**: Can be tested by generating a keymap with `LCG(KC_Q)` and successfully compiling firmware.

**Acceptance Scenarios**:

1. **Given** a layout with `LCG(KC_Q)` keycode, **When** firmware is generated and compiled, **Then** compilation succeeds without errors
2. **Given** a layout with any standard QMK modifier combo, **When** firmware is compiled, **Then** all keycodes are recognized
3. **Given** the QMK documentation lists a keycode, **When** that keycode is used, **Then** it compiles successfully

---

### User Story 2 - Seamless Migration Experience (Priority: P1)

As a user with existing keyboard configurations, I need my existing layouts to continue working after the migration so that I don't lose my keyboard customizations.

**Why this priority**: Breaking existing user workflows would be unacceptable.

**Independent Test**: Can be tested by loading an existing layout file and generating firmware successfully.

**Acceptance Scenarios**:

1. **Given** an existing layout.md file, **When** loaded after migration, **Then** all keys are correctly parsed
2. **Given** an existing config.toml, **When** QMK path is updated, **Then** firmware generation works
3. **Given** a user's existing keymap, **When** compiled with standard QMK, **Then** keyboard functions identically

---

### User Story 3 - Clear Documentation (Priority: P2)

As a new user setting up the keyboard configurator, I need documentation that correctly references standard QMK so that I can set up my environment without confusion.

**Why this priority**: Documentation accuracy is important but not blocking.

**Independent Test**: Can be tested by following QUICKSTART.md on a fresh system.

**Acceptance Scenarios**:

1. **Given** QUICKSTART.md documentation, **When** followed step-by-step, **Then** user successfully sets up with standard QMK
2. **Given** README.md, **When** reading setup instructions, **Then** no references to Vial remain
3. **Given** inline code comments, **When** reading generator code, **Then** no Vial-specific comments remain

---

### User Story 4 - Simplified Codebase (Priority: P3)

As a developer maintaining the keyboard configurator, I need Vial-specific code removed so that the codebase is simpler and easier to understand.

**Why this priority**: Code cleanliness is valuable but not user-facing.

**Independent Test**: Can be tested by searching codebase for "vial" references.

**Acceptance Scenarios**:

1. **Given** the firmware generator, **When** reviewing code, **Then** no vial.json generation exists
2. **Given** the validator, **When** reviewing code, **Then** no Vial-specific checks exist
3. **Given** the full codebase, **When** searching for "vial", **Then** only historical/migration notes found

---

### Edge Cases

- What happens when a user has `vial-qmk-keebart` configured but tries to use the migrated app?
- How does the system handle keyboards that only have Vial keymaps as examples?
- What happens if a user's config.toml points to the old Vial QMK path?
- How do we handle the git submodule transition?

## Requirements

### Functional Requirements

**Core Migration**
- **FR-001**: System MUST generate only `keymap.c` and `config.h` (remove `vial.json`)
- **FR-002**: System MUST work with standard `qmk_firmware` repository
- **FR-003**: System MUST NOT include Vial unlock combo macros in generated `config.h`
- **FR-004**: System MUST NOT check for deprecated Vial options (`VIAL_ENABLE`, `VIAL_KEYBOARD_UID`)

**Compatibility**
- **FR-005**: System MUST continue to generate valid QMK `keymap.c` files
- **FR-006**: System MUST preserve all existing keycode support
- **FR-007**: System MUST support all existing keyboard geometries
- **FR-008**: System MUST maintain RGB matrix color table generation

**Configuration**
- **FR-009**: Config auto-detection SHOULD recognize `qmk_firmware` directory name
- **FR-010**: Config SHOULD provide helpful error if old Vial path is configured

### Key Entities

- **FirmwareGenerator**: Generates `keymap.c` and `config.h` (no longer `vial.json`)
- **Config**: QMK firmware path configuration (updated defaults)
- **FirmwareValidator**: Validates QMK setup (Vial checks removed)

## Success Criteria

### Measurable Outcomes

- **SC-001**: All 27 modifier combo keycodes from QMK docs compile successfully
- **SC-002**: Existing test suite passes without modification to test expectations
- **SC-003**: Zero references to "vial" in generated firmware files
- **SC-004**: `cargo test` passes with all Vial-specific tests removed/updated
- **SC-005**: Documentation contains zero Vial setup instructions

## Out of Scope

- Adding VIA support (separate feature if needed)
- Changing the keyboard geometry or layout format
- Modifying keycode database structure
- Adding new features beyond migration

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Users have Vial-only keyboards | Medium | Document how to add standard QMK support |
| Breaking existing workflows | High | Thorough testing, clear migration guide |
| Missing QMK keycodes | Low | Keycode DB already uses standard QMK codes |
| Git submodule complications | Low | Clear instructions, can use any QMK clone |
