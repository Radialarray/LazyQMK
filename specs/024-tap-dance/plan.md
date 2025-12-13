# Implementation Plan: Tap Dance Support

**Branch**: `024-tap-dance` | **Date**: 2024-12-13 | **Spec**: Internal Analysis

## Summary

Add Tap Dance support to LazyQMK, allowing users to configure keys that perform different actions based on tap count (single tap, double tap) and hold duration. This feature integrates with the existing keycode database, picker system, and firmware generator to emit QMK tap dance code.

**Primary Requirement**: Enable users to define and use tap dance actions in their keyboard layouts with a UI flow similar to existing tap-hold/layer-tap editors.

**Technical Approach**: Extend the data model to store tap dance definitions, add a new keycode category and picker flow, implement a per-part editor for defining tap dance actions, and generate QMK `tap_dance_actions` arrays in the firmware output.

## Technical Context

**Language/Version**: Rust 1.91.1+  
**Primary Dependencies**: Ratatui 0.29 (TUI), Crossterm 0.29 (terminal), Serde 1.0 (serialization)  
**Storage**: Markdown files (YAML frontmatter), TOML configuration, JSON keycode database  
**Testing**: cargo test (unit + integration)  
**Target Platform**: Cross-platform (macOS, Linux, Windows) terminal application  
**Project Type**: Single project (TUI application)  
**Performance Goals**: 60fps UI rendering, <16ms/frame event handling  
**Constraints**: Human-readable file formats, version control friendly, backward compatible with existing layouts  
**Scale/Scope**: ~10 tap dance definitions per layout typical, scales to QMK's `TAP_DANCE_MAX_SIMULTANEOUS` limit

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

✅ **No new dependencies**: Uses existing Rust ecosystem (Serde, Ratatui)  
✅ **Follows existing patterns**: Mirrors layer-tap and mod-tap editor flows  
✅ **Human-readable formats**: Stored in markdown frontmatter, diffable  
✅ **Backward compatible**: No breaking changes to existing layouts  
✅ **Testing coverage**: Unit tests for parser, generator, and editor logic  

## Project Structure

### Documentation (this feature)

```text
specs/024-tap-dance/
└── plan.md              # This file
```

### Source Code (repository root)

```text
src/
├── models/
│   └── layout.rs        # Add TapDanceAction struct
├── keycode_db/
│   ├── mod.rs          # Add tap dance parsing logic
│   └── categories/
│       └── tap_dance.json  # New category file
├── parser/
│   ├── layout.rs       # Parse tap dance from frontmatter
│   └── template_gen.rs # Serialize tap dance to frontmatter
├── firmware/
│   └── generator.rs    # Generate QMK tap dance code
└── tui/
    ├── key_editor.rs   # Add TapDance editing state
    ├── tap_dance_editor.rs  # New component for tap dance management
    └── handlers/
        └── popups.rs   # Wire tap dance editor into popup system

tests/
├── firmware_gen_tests.rs      # Add tap dance generation tests
└── tap_dance_tests.rs         # New test file for tap dance parsing/round-trip
```

**Structure Decision**: Single project structure (default). All tap dance logic integrates into existing modules following established patterns (models, parser, generator, TUI).

## Implementation Phases

### Phase 1: Data Model & Parsing (Foundation)

**Goal**: Add tap dance data structures and enable storage/retrieval in layout files.

**Tasks**:
1. Add `TapDanceAction` struct to `src/models/layout.rs`:
   - Fields: `name: String`, `single_tap: String`, `double_tap: Option<String>`, `hold: Option<String>`
   - Validation methods: ensure at least `single_tap` is set, no duplicate names
2. Add `tap_dances: Vec<TapDanceAction>` to `Layout` struct
3. Update `src/parser/layout.rs` to parse tap dance from YAML frontmatter:
   - Section: `tap_dances:` array in frontmatter
   - Format: `{ name: "example", single_tap: "KC_A", double_tap: "KC_B", hold: "KC_C" }`
4. Update `src/parser/template_gen.rs` to serialize tap dance back to frontmatter
5. Add validation: detect duplicate tap dance names, orphaned `TD()` references
6. Add round-trip tests in `tests/tap_dance_tests.rs`

**Validation Criteria**:
- Parser correctly reads tap dance definitions from markdown frontmatter
- Template generator emits tap dance definitions that round-trip cleanly
- Validation errors are clear and actionable

### Phase 2: Keycode Database Integration

**Goal**: Register tap dance as a keycode category and enable picker interaction.

**Tasks**:
1. Create `src/keycode_db/categories/tap_dance.json`:
   - Category: `{ "id": "tap_dance", "name": "Tap Dance", "description": "Keys with different actions on single/double tap or hold" }`
   - Keycode template: `{ "code": "TD()", "name": "Tap Dance", "category": "tap_dance", "pattern": "^TD\\(([^)]+)\\)$", "params": [{"type": "tapdance", "name": "action"}] }`
2. Add `ParamType::TapDance` to `src/keycode_db/mod.rs` (or reuse existing mechanism)
3. Update keycode loading in `KeycodeDb::load()` to include tap_dance.json
4. Add parsing support for `TD(name)` pattern in `src/keycode_db/mod.rs`
5. Add tests for tap dance keycode validation

**Validation Criteria**:
- `TD(name)` keycodes are recognized and validated by KeycodeDb
- Tap dance category appears in keycode picker
- Pattern matching correctly extracts tap dance name from `TD(...)` syntax

### Phase 3: TUI Editor Component

**Goal**: Create a tap dance editor UI component for defining and managing tap dance actions.

**Tasks**:
1. Create `src/tui/tap_dance_editor.rs`:
   - Component struct: `TapDanceEditor` implementing `ContextualComponent`
   - State: list of tap dances, selected index, edit mode (viewing list vs editing fields)
   - Fields editor: single_tap (required), double_tap (optional), hold (optional)
   - CRUD operations: Create, Rename, Edit, Delete with confirmation
2. Add `ComboEditPart::TapDance` and `TapDanceEditState` to `src/tui/key_editor.rs`:
   - Parse `TD(name)` into editable parts
   - Track which field is being edited (single_tap, double_tap, hold)
3. Wire tap dance editor into popup system in `src/tui/handlers/popups.rs`:
   - Open tap dance editor when user presses shortcut (e.g., Shift+D for "Dance")
   - Handle keycode picker results for each field
   - Apply `TD(name)` to selected key when user confirms
4. Add help overlay entries for tap dance shortcuts
5. Update status bar hints to show tap dance editing options

**Validation Criteria**:
- User can create/edit/delete tap dance actions via TUI
- Each field (single_tap, double_tap, hold) opens keycode picker
- `TD(name)` is correctly inserted into selected key position
- UI follows existing patterns (consistent with layer-tap/mod-tap editors)

### Phase 4: Firmware Generation

**Goal**: Generate QMK tap dance code from layout definitions.

**Tasks**:
1. Update `src/firmware/generator.rs`:
   - Emit `enum tap_dance_ids { TD_NAME1, TD_NAME2, ... };` with stable ordering
   - Emit `tap_dance_action_t tap_dance_actions[] = { ... };` array
   - For two-way (single/double): use `ACTION_TAP_DANCE_DOUBLE(single, double)`
   - For three-way (single/double/hold): generate helper function and use `ACTION_TAP_DANCE_FN_ADVANCED`
   - Substitute `TD(name)` in keymap with `TD(TD_NAME_UPPER)`
2. Add include for QMK tap dance headers in generated keymap
3. Generate helper functions for advanced tap dances (hold detection logic)
4. Add golden tests in `tests/firmware_gen_tests.rs`:
   - Test case with 2-way tap dance (single/double)
   - Test case with 3-way tap dance (single/double/hold)
   - Verify correct enum, actions array, and helper function generation

**Validation Criteria**:
- Generated keymap.c compiles with QMK firmware
- Tap dance actions are correctly indexed and stable across builds
- Helper functions correctly handle single/double/hold logic
- `TD()` references are correctly translated to `TD(enum_value)`

### Phase 5: Validation & Polish

**Goal**: Add comprehensive validation and improve user experience.

**Tasks**:
1. Add validation in layout save flow:
   - Every `TD(name)` in keymap references a defined tap dance action
   - No duplicate tap dance names
   - At least `single_tap` is defined for each action
2. Add warnings:
   - Orphaned tap dance definitions (defined but not used)
   - Multiple tap dances on same key (not supported)
   - Exceeding `TAP_DANCE_MAX_SIMULTANEOUS` limit
3. Update documentation:
   - Add tap dance section to `docs/FEATURES.md`
   - Update in-app help overlay with tap dance shortcuts
   - Add example layout with tap dance to `examples/`
4. Add clippy/rustfmt compliance checks
5. Update `AGENTS.md` with tap dance feature notes

**Validation Criteria**:
- All validation errors have clear, actionable messages
- Documentation accurately describes feature and limitations
- All tests pass with zero clippy warnings
- Example layouts demonstrate common tap dance patterns

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Complex QMK tap dance API | High | Limit MVP to simple double/hold patterns using QMK macros |
| Parser backward compatibility | Medium | Make `tap_dances:` section optional, default to empty array |
| Generator C code correctness | High | Add golden tests, verify with actual QMK compilation |
| UI complexity (multiple pickers) | Medium | Reuse existing layer-tap/mod-tap editor patterns |
| Name stability in generated code | Medium | Use stable sort by name for enum generation |
| User confusion (tap-hold vs tap-dance) | Low | Clear labeling in picker and editor ("Tap Dance: single/double/hold") |

## Success Metrics

- [ ] Users can create/edit/delete tap dance actions via TUI
- [ ] Tap dance definitions persist in markdown files and round-trip correctly
- [ ] Generated firmware compiles and tap dances work on physical keyboards
- [ ] All tests pass with zero clippy warnings
- [ ] Documentation covers feature capabilities and limitations
- [ ] No breaking changes to existing layouts without tap dance

## Timeline Estimate

- Phase 1 (Data Model & Parsing): ~4-6 hours
- Phase 2 (Keycode Database): ~2-3 hours  
- Phase 3 (TUI Editor): ~6-8 hours
- Phase 4 (Firmware Generation): ~4-6 hours
- Phase 5 (Validation & Polish): ~2-3 hours

**Total**: ~18-26 hours (3-4 work days)

## Dependencies

**Blocked By**: None (all prerequisites exist in codebase)  
**Blocks**: Future enhancements (custom tap dance callbacks, timing configuration)

## Notes

- MVP focuses on simple double/hold patterns; custom C callbacks deferred to future work
- Tap dance names must be valid Rust/C identifiers (alphanumeric + underscore)
- Consider adding `TAPPING_TERM` configuration per tap dance in future iteration
- QMK limitation: max 3 simultaneous tap dances by default (warn users)
