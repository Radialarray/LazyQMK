# Implementation Tasks

## Phase 1: Data Model

- [ ] T001: Add `description: Option<String>` field to `KeyDefinition` in `src/models/layer.rs`
- [ ] T002: Update `KeyDefinition::new()` to initialize description as None
- [ ] T003: Add `with_description()` builder method
- [ ] T004: Update parser regex in `parse_keycode_syntax()` to handle `["description"]` suffix
- [ ] T005: Add `## Key Descriptions` section parsing in `parse_content()`
- [ ] T006: Update template_gen.rs to serialize descriptions

## Phase 2: Key Editor Dialog Structure

- [ ] T007: Create `src/tui/key_editor.rs` module file
- [ ] T008: Add `pub mod key_editor;` to `src/tui/mod.rs`
- [ ] T009: Create `KeyEditorState` struct with fields:
  - `position: Position` - which key we're editing
  - `layer_idx: usize` - which layer
  - `mode: KeyEditorMode` - View/EditDescription
  - `description_buffer: String` - for editing
  - `cursor_position: usize` - for text editing
- [ ] T010: Create `KeyEditorMode` enum (View, EditDescription)
- [ ] T011: Add `PopupType::KeyEditor` variant
- [ ] T012: Add `key_editor_state: KeyEditorState` to `AppState`

## Phase 3: Dialog Rendering

- [ ] T013: Implement `render_key_editor()` function
- [ ] T014: Render position header
- [ ] T015: Render current keycode with visual preview
- [ ] T016: Render tap-hold breakdown (reuse `parse_tap_hold_keycode` logic)
- [ ] T017: Render description box (view mode)
- [ ] T018: Render description editor (edit mode) with cursor
- [ ] T019: Render action bar with keybindings
- [ ] T020: Add render call in `render_popup()` match arm

## Phase 4: Input Handling

- [ ] T021: Create `handle_key_editor_input()` function
- [ ] T022: Handle `Esc` - close dialog
- [ ] T023: Handle `Enter` in View mode - open keycode picker
- [ ] T024: Handle `D` - switch to EditDescription mode
- [ ] T025: Handle `C` - open color picker with context
- [ ] T026: Handle text input in EditDescription mode
- [ ] T027: Handle `Enter` in EditDescription mode - save and return to View
- [ ] T028: Handle `Esc` in EditDescription mode - cancel edit, return to View
- [ ] T029: Add handler call in `handle_popup_input()` match arm

## Phase 5: Status Bar Enhancement

- [ ] T030: Modify status bar layout to add description line
- [ ] T031: Get selected key's description in `StatusBar::render()`
- [ ] T032: Render description line when present (above help line)
- [ ] T033: Handle truncation for long descriptions

## Phase 6: Main UI Integration

- [ ] T034: Modify Enter key handling in main input
- [ ] T035: Check if selected key is assigned (not KC_NO/KC_TRNS)
- [ ] T036: If assigned, open KeyEditor instead of KeycodePicker
- [ ] T037: If unassigned, open KeycodePicker directly (current behavior)
- [ ] T038: Initialize `KeyEditorState` with current key data

## Phase 7: Flow Connections

- [ ] T039: After keycode picker closes from editor, return to editor
- [ ] T040: After color picker closes from editor, return to editor
- [ ] T041: Save description changes to key when exiting editor
- [ ] T042: Mark layout dirty when description changes

## Phase 8: Testing & Polish

- [ ] T043: Test opening editor on various key types
- [ ] T044: Test description editing and persistence
- [ ] T045: Test status bar description display
- [ ] T046: Test reassignment flow through editor
- [ ] T047: Test color picker flow through editor
- [ ] T048: Verify markdown save/load round-trip
