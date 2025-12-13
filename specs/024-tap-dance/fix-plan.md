# Tap Dance Fix Plan: Architecture Alignment & Multi-Picker Flow

**Branch**: `feat/024-tap-dance` | **Date**: 2024-12-13 | **Type**: Architecture Fix

## Problem Analysis

### Critical Issues Found

1. **Data Persistence Broken** ❌
   - `TapDanceEditor::new(layout)` clones `layout.tap_dances` into local state
   - Created tap dances only exist in component memory, never written to `AppState.layout.tap_dances`
   - No `SaveOrUpdate` event to persist changes back to layout
   - After closing editor, all new tap dances disappear

2. **No Multi-Stage Picker Flow** ❌
   - Current implementation uses plain text input for keycodes
   - Should use existing `KeycodePicker` for each field (single_tap, double_tap, hold)
   - Does not follow established layer-tap/mod-tap "double picker" pattern
   - User never sees the keycode picker popup

3. **Apply Flow Doesn't Validate** ❌
   - `TapDanceEditorEvent::Selected(name)` writes `TD(name)` to key without checking if definition exists
   - Can create orphaned references if definition wasn't persisted
   - No feedback when applying undefined tap dance

4. **No Edit Support** ❌
   - Only supports create/select/delete
   - Cannot edit existing tap dance definitions
   - Forces users to delete and recreate

5. **Architecture Divergence** ❌
   - Doesn't reuse `PendingKeycodeState` pattern for multi-stage input
   - Component manages data mutations instead of AppState
   - Hardcoded text entry instead of picker-based keycode selection

### Root Cause

The implementation diverged from LazyQMK's established patterns for parameterized keycodes:
- **Layer-Tap (LT)**: Opens LayerPicker → KeycodePicker, builds `LT(layer, keycode)`
- **Mod-Tap (MT)**: Opens ModifierPicker → KeycodePicker, builds `MT(mod, keycode)`
- **Tap Dance**: Should follow similar pattern: Edit fields → KeycodePicker per field → Save to layout

## Fix Strategy

### Architectural Alignment

1. **Use AppState as Source of Truth**
   - Component operates on read-only view of `layout.tap_dances`
   - All mutations go through AppState handlers
   - Component refreshes from layout after changes

2. **Implement Multi-Stage Picker Flow**
   - Reuse existing `KeycodePicker` component
   - Track active editing state in AppState (which field, which tap dance)
   - Sequential flow: Name → SingleTap picker → DoubleTap picker → Hold picker → Save

3. **Follow Existing Patterns**
   - Study `key_editor.rs`: `ComboEditPart` enum for multi-field editing
   - Study `handlers/popups.rs`: How LT/MT handle sequential pickers
   - Reuse `PendingKeycodeState`-like pattern for draft state

4. **Event-Driven Persistence**
   - Add `TapDanceEditorEvent::SaveOrUpdate(TapDanceAction)`
   - Handler writes to `layout.tap_dances`, marks dirty, refreshes editor
   - Persistence happens immediately, not on editor close

## Implementation Plan

### Phase 1: Add Edit State to AppState

**Goal**: Create proper state management for tap dance editing in AppState.

**Files to Modify**:
- `src/tui/mod.rs` - Add `TapDanceEditState` to AppState

**Tasks**:
1. Add enum for tracking which field is being edited:
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub enum TapDanceEditField {
       Name,        // Entering tap dance name (text input)
       SingleTap,   // Selecting single tap keycode (picker)
       DoubleTap,   // Selecting double tap keycode (picker)
       Hold,        // Selecting hold keycode (picker)
   }
   ```

2. Add state struct for tap dance editing:
   ```rust
   #[derive(Debug, Clone)]
   pub struct TapDanceEditState {
       /// Tap dance being created/edited (None = not editing)
       pub draft: Option<TapDanceAction>,
       /// Current field being edited
       pub active_field: TapDanceEditField,
       /// Whether we're editing existing (Some(index)) or creating new (None)
       pub editing_index: Option<usize>,
       /// Input buffer for name field
       pub name_buffer: String,
   }
   ```

3. Add field to AppState:
   ```rust
   pub struct AppState {
       // ... existing fields ...
       /// Tap dance editing state
       pub tap_dance_edit_state: Option<TapDanceEditState>,
   }
   ```

4. Add helper methods to AppState:
   - `start_tap_dance_create()` - Initialize new tap dance creation
   - `start_tap_dance_edit(index)` - Load existing for editing
   - `advance_tap_dance_field()` - Move to next field, open picker if needed
   - `save_tap_dance()` - Persist to layout, mark dirty, clear edit state
   - `cancel_tap_dance_edit()` - Clear edit state without saving

### Phase 2: Refactor TapDanceEditor Component

**Goal**: Make component a view-only browser/selector, not an editor.

**Files to Modify**:
- `src/tui/tap_dance_editor.rs`

**Tasks**:
1. Simplify component to Select mode only:
   - Remove `EditorMode::Edit` and all edit mode rendering
   - Remove `editing: Option<TapDanceAction>`
   - Remove `edit_field: EditField` 
   - Remove `input_buffer: String`
   - Keep only list selection and navigation

2. Update events:
   ```rust
   pub enum TapDanceEditorEvent {
       Selected(String),      // Apply TD(name) to current key
       CreateNew,             // Start creating new tap dance
       Edit(usize),           // Edit tap dance at index
       Delete(String),        // Delete tap dance
       Cancelled,             // Close editor
   }
   ```

3. Update input handling:
   - Up/Down: Navigate list
   - Enter: Emit `Selected(name)`
   - 'n': Emit `CreateNew`
   - 'e': Emit `Edit(index)` if item selected
   - 'd': Emit `Delete(name)`
   - Esc: Emit `Cancelled`

4. Simplify rendering:
   - Show list of tap dances with details
   - Show help text for shortcuts
   - Remove all edit mode UI

### Phase 3: Implement Multi-Stage Picker Flow

**Goal**: Add sequential keycode picker flow for tap dance fields.

**Files to Modify**:
- `src/tui/handlers/tap_dance.rs` - Update event handling
- `src/tui/handlers/popups.rs` - Wire picker results to tap dance fields

**Tasks**:
1. Handle `CreateNew` event:
   - Call `state.start_tap_dance_create()`
   - Close tap dance editor popup
   - Open name entry popup (simple text input, or re-purpose metadata editor)

2. Handle name entry completion:
   - Validate name (C identifier, not empty, not duplicate)
   - Store in `tap_dance_edit_state.draft.name`
   - Set `active_field = SingleTap`
   - Open `KeycodePicker` for single_tap field

3. Handle `KeycodePickerEvent::KeycodeSelected`:
   - Check if `tap_dance_edit_state` is active
   - Store keycode in appropriate field based on `active_field`
   - Advance to next field:
     - SingleTap → DoubleTap (open picker again, or allow skip)
     - DoubleTap → Hold (open picker again, or allow skip)
     - Hold → Save tap dance and close
   - If user cancels picker, allow skip for optional fields

4. Handle skip for optional fields:
   - DoubleTap: If skipped, mark as 2-way (no double), proceed to Hold
   - Hold: If skipped, finalize and save

5. Handle `Edit(index)` event:
   - Call `state.start_tap_dance_edit(index)`
   - Load existing tap dance into draft
   - Start at SingleTap field (skip name since it's immutable)
   - Open picker flow same as create

6. Save tap dance:
   - Validate complete tap dance
   - If creating new: `layout.add_tap_dance(draft)`
   - If editing: Replace at index, mark dirty
   - Clear `tap_dance_edit_state`
   - Re-open tap dance editor to show updated list

### Phase 4: Add Name Entry Dialog

**Goal**: Create a simple text input dialog for entering tap dance name.

**Files to Modify**:
- `src/tui/mod.rs` - Add `PopupType::TapDanceNameEntry`
- Create new file: `src/tui/tap_dance_name_entry.rs`

**Tasks**:
1. Create simple component:
   ```rust
   pub struct TapDanceNameEntry {
       input: String,
       error: Option<String>,
   }
   
   pub enum TapDanceNameEntryEvent {
       Confirmed(String),
       Cancelled,
   }
   ```

2. Render:
   - Title: "Enter Tap Dance Name"
   - Input field with cursor
   - Validation hint: "Alphanumeric and underscores only"
   - Error message if validation fails
   - Help: "Enter to confirm, Esc to cancel"

3. Input handling:
   - Char: Append to input (if valid character)
   - Backspace: Remove last character
   - Enter: Validate and emit `Confirmed(input)`
   - Esc: Emit `Cancelled`

4. Validation:
   - Not empty
   - Only alphanumeric + underscore
   - Not duplicate name (check against `layout.tap_dances`)

### Phase 5: Wire Everything Together

**Goal**: Connect all pieces into cohesive flow.

**Files to Modify**:
- `src/tui/handlers/tap_dance.rs`
- `src/tui/handlers/popups.rs`
- `src/tui/mod.rs`

**Tasks**:
1. Update tap dance handler to process new events:
   - `CreateNew`: Start creation flow
   - `Edit(index)`: Start edit flow
   - `Selected(name)`: Validate name exists, apply `TD(name)` to key
   - `Delete(name)`: Confirm, remove, refresh editor

2. Update popup handler for keycode picker results:
   - Check if `tap_dance_edit_state` is active
   - Route keycode to appropriate tap dance field
   - Advance to next field or complete

3. Add cancel handling:
   - Allow Esc at any stage to cancel entire operation
   - Confirm before discarding draft if user entered data

4. Update status messages:
   - "Editing tap dance: [name] - Select single tap keycode"
   - "Tap dance created: [name]"
   - "Tap dance updated: [name]"
   - "Error: Tap dance '[name]' not found"

### Phase 6: Add Skip Support for Optional Fields

**Goal**: Allow users to skip double_tap and hold fields.

**Files to Modify**:
- `src/tui/keycode_picker.rs` (if needed for skip hint)
- `src/tui/handlers/popups.rs`

**Tasks**:
1. Show hint in status bar when picker is open for optional field:
   - "Select double tap keycode (Esc to skip)"
   - "Select hold keycode (Esc to skip)"

2. Handle Esc in optional field:
   - If DoubleTap: Leave as None, advance to Hold
   - If Hold: Leave as None, save tap dance

3. Validation:
   - Must have single_tap
   - Can have just single_tap (1-way? or require at least double_tap)
   - Per plan: 2-way = single+double, 3-way = single+double+hold

4. Enforce minimum:
   - Require at least double_tap for valid tap dance
   - If user skips double_tap, show error: "Tap dance must have at least single and double tap"

### Phase 7: Update Tests

**Goal**: Ensure all flows work correctly.

**Files to Modify**:
- `tests/tap_dance_tests.rs`
- Add new integration-style tests if needed

**Tasks**:
1. Test tap dance CRUD via AppState methods:
   - Create, read, update, delete
   - Duplicate name detection
   - Invalid name validation

2. Test multi-stage flow state machine:
   - Start create → name entry → single → double → hold → save
   - Start edit → single → double → hold → update
   - Cancel at each stage

3. Test apply flow:
   - Apply existing tap dance to key
   - Try to apply non-existent tap dance (should error)

4. Test persistence:
   - Create tap dance, save layout, reload, verify present
   - Edit tap dance, save layout, reload, verify updated

5. Test firmware generation with updated tap dances

### Phase 8: Polish & Documentation

**Goal**: Complete user experience and documentation.

**Files to Modify**:
- `docs/FEATURES.md`
- `src/data/help.toml`

**Tasks**:
1. Update FEATURES.md with correct workflow:
   - "Press Shift+D to open tap dance browser"
   - "Press 'n' to create new (multi-stage picker flow)"
   - "Press 'e' to edit existing"
   - "Press Enter to apply to current key"
   - "Press 'd' to delete"

2. Update help.toml:
   - Tap dance browser shortcuts
   - Multi-stage picker hints
   - Field skip instructions

3. Test complete user flow:
   - Open TUI
   - Shift+D → browser
   - 'n' → name entry → pickers → save
   - Apply to key
   - Save layout
   - Reload layout
   - Edit tap dance
   - Generate firmware

## Success Criteria

- ✅ Tap dances persist in `layout.tap_dances` immediately on save
- ✅ Multi-stage picker opens sequentially for each field
- ✅ User never types keycode strings manually (always use picker)
- ✅ Edit existing tap dances works
- ✅ Skip optional fields (double_tap, hold) works
- ✅ Apply validates tap dance exists before inserting `TD(name)`
- ✅ Save layout → reload layout → tap dances still present
- ✅ Generated firmware includes tap dance definitions
- ✅ All tests pass, zero clippy warnings
- ✅ Documentation accurate and complete

## Architecture Compliance

✅ **Follows layer-tap/mod-tap patterns**  
✅ **AppState is source of truth**  
✅ **Component trait pattern maintained**  
✅ **Multi-stage picker reused**  
✅ **Event-driven mutations**  
✅ **No hardcoded strings in UI**  

## Timeline

- Phase 1: AppState edit state - 2 hours
- Phase 2: Refactor component - 2 hours
- Phase 3: Multi-stage picker flow - 3 hours
- Phase 4: Name entry dialog - 1 hour
- Phase 5: Wire everything - 2 hours
- Phase 6: Skip support - 1 hour
- Phase 7: Tests - 2 hours
- Phase 8: Polish - 1 hour

**Total**: ~14 hours (2 work days)

## Dependencies

- Phase 2 depends on Phase 1 (need edit state)
- Phase 3 depends on Phase 2 (need refactored events)
- Phase 4 independent (can parallelize)
- Phase 5 depends on Phases 2, 3, 4
- Phase 6 depends on Phase 5
- Phase 7 depends on Phases 1-6
- Phase 8 depends on Phase 7

## Notes

- Keep existing firmware generator unchanged (already works)
- Keep existing data model unchanged (already works)
- Focus fixes on TUI flow and state management
- Maintain backward compatibility with existing layouts
