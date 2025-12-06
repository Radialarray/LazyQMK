# Plan: Reduce Cognitive Complexity

## Overview

This plan addresses the cognitive complexity warnings in the codebase by refactoring complex functions into smaller, more maintainable units. We currently have 2 functions exceeding the complexity threshold of 25:

1. **`dispatch_action()`** - Score: 73/25 (647 lines, 38 action handlers)
2. **`test_tap_hold_settings_round_trip()`** - Score: 31/25 (test function)

**Goal**: Reduce all cognitive complexity scores to ≤25 without using `#[allow]` suppressions.

---

## Problem Analysis

### 1. `dispatch_action()` (Score: 73/25)

**Location**: `src/tui/handlers/actions.rs:143-789`

**Current Structure**:
```rust
pub fn dispatch_action(state: &mut AppState, action: Action) -> Result<bool> {
    match action {
        Action::Quit => { /* 5-10 lines with nested if */ }
        Action::Save => { /* 5-10 lines with error handling */ }
        // ... 36 more actions ...
    }
}
```

**Complexity Sources**:
- **38 action handlers** in one giant match statement
- **Nested conditionals** (e.g., checking `state.dirty`, then checking popup state)
- **Early returns** (`return Ok(false)`)
- **Error handling** with `?` operators
- **Complex logic** in some arms (e.g., `OpenKeycodePicker` has nested if-let-else)

**Action Categories** (analyzed from code):
1. **Navigation** (8): Up/Down/Left/Right, JumpToFirst/Last, NextLayer/PreviousLayer
2. **Popup Management** (9): OpenKeycodePicker, OpenLayerManager, OpenCategoryManager, OpenSettings, EditMetadata, SetupWizard, BrowseTemplates, ViewBuildLog, ToggleHelp
3. **File Operations** (3): Save, SaveAsTemplate, Quit
4. **Key Operations** (6): ClearKey, CopyKey, CutKey, PasteKey, UndoPaste, ToggleCurrentKey
5. **Selection** (2): ToggleSelectionMode, StartRectangleSelect
6. **Color Management** (4): SetIndividualKeyColor, SetLayerColor, ToggleLayerColors, ToggleAllLayerColors
7. **Category Assignment** (2): AssignCategoryToKey, AssignCategoryToLayer
8. **Firmware** (2): BuildFirmware, GenerateFirmware
9. **Layout** (1): SwitchLayoutVariant
10. **Cancel** (1): Cancel

---

### 2. `test_tap_hold_settings_round_trip()` (Score: 31/25)

**Location**: `src/parser/template_gen.rs:623-691`

**Current Structure**:
- Tests 3 scenarios in one function (HomeRowMods, Custom, Default)
- Each scenario has ~15-20 assertions
- Repetitive pattern: setup → generate → assert → parse → assert

**Complexity Sources**:
- Multiple test scenarios in one function
- Deep nesting of assertions
- Error handling with `.unwrap()`
- Multiple variables and intermediate states

---

## Refactoring Strategy

### Phase 1: Extract Action Handlers

**Approach**: Create dedicated handler functions for each action category.

#### 1.1 Create Handler Module Structure

```
src/tui/handlers/
  actions.rs              # dispatch_action (thin dispatcher)
  action_handlers/
    mod.rs                # Re-exports
    navigation.rs         # 8 navigation actions
    popups.rs             # 9 popup management actions
    file_ops.rs           # 3 file operations
    key_ops.rs            # 6 key operations
    selection.rs          # 2 selection actions
    color.rs              # 4 color management actions
    category.rs           # 2 category assignment actions
    firmware.rs           # 2 firmware actions
    layout.rs             # 1 layout action
```

#### 1.2 Define Handler Trait

```rust
/// Trait for action handlers
pub trait ActionHandler {
    /// Handle the action and return whether to quit the app
    fn handle(&self, state: &mut AppState) -> Result<bool>;
}
```

#### 1.3 Implement Handlers

**Example: Navigation Handler**
```rust
// src/tui/handlers/action_handlers/navigation.rs

pub fn handle_navigate_up(state: &mut AppState) -> Result<bool> {
    state.move_selection_up();
    Ok(false)
}

pub fn handle_navigate_down(state: &mut AppState) -> Result<bool> {
    state.move_selection_down();
    Ok(false)
}

pub fn handle_next_layer(state: &mut AppState) -> Result<bool> {
    if state.current_layer < state.layout.layers.len() - 1 {
        state.current_layer += 1;
        state.set_status(format!("Layer {}", state.current_layer));
    }
    Ok(false)
}

// ... 5 more navigation handlers
```

**Example: Complex Handler (Quit)**
```rust
// src/tui/handlers/action_handlers/file_ops.rs

pub fn handle_quit(state: &mut AppState) -> Result<bool> {
    if state.dirty {
        state.active_popup = Some(PopupType::UnsavedChangesPrompt);
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn handle_save(state: &mut AppState) -> Result<bool> {
    let path = state.source_path.clone()
        .ok_or_else(|| anyhow::anyhow!("No file path set"))?;
    
    LayoutService::save(&state.layout, &path)?;
    state.mark_clean();
    state.set_status("Saved");
    Ok(false)
}
```

**Example: Very Complex Handler (OpenKeycodePicker)**
```rust
// src/tui/handlers/action_handlers/popups.rs

pub fn handle_open_keycode_picker(state: &mut AppState) -> Result<bool> {
    match get_selected_key_info(state) {
        Some((key, true)) => open_key_editor(state, &key),
        Some((key, false)) | None => open_keycode_picker_for_empty_key(state),
    }
    Ok(false)
}

fn get_selected_key_info(state: &AppState) -> Option<(Key, bool)> {
    state.get_selected_key()
        .map(|key| (key.clone(), key_editor::is_key_assigned(&key.keycode)))
}

fn open_key_editor(state: &mut AppState, key: &Key) {
    state.key_editor_state.init_for_key(key, state.current_layer);
    state.active_popup = Some(PopupType::KeyEditor);
    state.set_status("Key editor - Enter: Reassign, D: Description, C: Color");
}

fn open_keycode_picker_for_empty_key(state: &mut AppState) {
    state.open_keycode_picker();
    state.set_status("Select keycode - Type to search, Enter to apply");
}
```

#### 1.4 Refactor `dispatch_action()`

**New Structure** (Score: ~10/25):
```rust
pub fn dispatch_action(state: &mut AppState, action: Action) -> Result<bool> {
    use action_handlers::*;
    
    match action {
        // Navigation (8 actions)
        Action::NavigateUp => navigation::handle_navigate_up(state),
        Action::NavigateDown => navigation::handle_navigate_down(state),
        Action::NavigateLeft => navigation::handle_navigate_left(state),
        Action::NavigateRight => navigation::handle_navigate_right(state),
        Action::JumpToFirst => navigation::handle_jump_to_first(state),
        Action::JumpToLast => navigation::handle_jump_to_last(state),
        Action::NextLayer => navigation::handle_next_layer(state),
        Action::PreviousLayer => navigation::handle_previous_layer(state),
        
        // File operations (3 actions)
        Action::Quit => file_ops::handle_quit(state),
        Action::Save => file_ops::handle_save(state),
        Action::SaveAsTemplate => file_ops::handle_save_as_template(state),
        
        // Popup management (9 actions)
        Action::OpenKeycodePicker => popups::handle_open_keycode_picker(state),
        Action::OpenLayerManager => popups::handle_open_layer_manager(state),
        Action::OpenCategoryManager => popups::handle_open_category_manager(state),
        Action::OpenSettings => popups::handle_open_settings(state),
        Action::EditMetadata => popups::handle_edit_metadata(state),
        Action::SetupWizard => popups::handle_setup_wizard(state),
        Action::BrowseTemplates => popups::handle_browse_templates(state),
        Action::ViewBuildLog => popups::handle_view_build_log(state),
        Action::ToggleHelp => popups::handle_toggle_help(state),
        
        // Key operations (6 actions)
        Action::ClearKey => key_ops::handle_clear_key(state),
        Action::CopyKey => key_ops::handle_copy_key(state),
        Action::CutKey => key_ops::handle_cut_key(state),
        Action::PasteKey => key_ops::handle_paste_key(state),
        Action::UndoPaste => key_ops::handle_undo_paste(state),
        Action::ToggleCurrentKey => key_ops::handle_toggle_current_key(state),
        
        // Selection (2 actions)
        Action::ToggleSelectionMode => selection::handle_toggle_selection_mode(state),
        Action::StartRectangleSelect => selection::handle_start_rectangle_select(state),
        
        // Color management (4 actions)
        Action::SetIndividualKeyColor => color::handle_set_individual_key_color(state),
        Action::SetLayerColor => color::handle_set_layer_color(state),
        Action::ToggleLayerColors => color::handle_toggle_layer_colors(state),
        Action::ToggleAllLayerColors => color::handle_toggle_all_layer_colors(state),
        
        // Category assignment (2 actions)
        Action::AssignCategoryToKey => category::handle_assign_category_to_key(state),
        Action::AssignCategoryToLayer => category::handle_assign_category_to_layer(state),
        
        // Firmware (2 actions)
        Action::BuildFirmware => firmware::handle_build_firmware(state),
        Action::GenerateFirmware => firmware::handle_generate_firmware(state),
        
        // Layout (1 action)
        Action::SwitchLayoutVariant => layout::handle_switch_layout_variant(state),
        
        // Cancel (1 action)
        Action::Cancel => Ok(false),
    }
}
```

**Cognitive Complexity Reduction**:
- **Before**: 73 (647 lines, nested logic in match arms)
- **After**: ~10 (38 simple function calls, no nesting)
- **Reduction**: 87% decrease

---

### Phase 2: Split Complex Test Function

#### 2.1 Create Separate Test Functions

```rust
// src/parser/template_gen.rs

#[test]
fn test_tap_hold_home_row_mods_preset_round_trip() {
    let mut layout = create_test_layout();
    layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::HomeRowMods);
    
    let markdown = generate_markdown(&layout).unwrap();
    assert_home_row_mods_markdown(&markdown);
    
    let parsed = parse_markdown_layout_str(&markdown).unwrap();
    assert_home_row_mods_settings(&parsed.tap_hold_settings);
}

#[test]
fn test_tap_hold_custom_settings_round_trip() {
    let mut layout = create_test_layout();
    layout.tap_hold_settings = TapHoldSettings {
        tapping_term: 180,
        quick_tap_term: Some(100),
        hold_mode: HoldDecisionMode::HoldOnOtherKeyPress,
        retro_tapping: true,
        tapping_toggle: 3,
        flow_tap_term: Some(120),
        chordal_hold: true,
        preset: TapHoldPreset::Custom,
    };
    
    let markdown = generate_markdown(&layout).unwrap();
    assert_custom_settings_markdown(&markdown);
    
    let parsed = parse_markdown_layout_str(&markdown).unwrap();
    assert_custom_settings(&parsed.tap_hold_settings);
}

#[test]
fn test_tap_hold_default_settings_not_written() {
    let mut layout = create_test_layout();
    layout.tap_hold_settings = TapHoldSettings::default();
    
    let markdown = generate_markdown(&layout).unwrap();
    assert!(!markdown.contains("Tap-Hold"));
    
    let parsed = parse_markdown_layout_str(&markdown).unwrap();
    assert_eq!(parsed.tap_hold_settings, TapHoldSettings::default());
}

// Helper functions to reduce assertion complexity
fn assert_home_row_mods_markdown(markdown: &str) {
    assert!(markdown.contains("## Settings"));
    assert!(markdown.contains("**Tap-Hold Preset**: Home Row Mods"));
    assert!(markdown.contains("**Tapping Term**: 175ms"));
    assert!(markdown.contains("**Retro Tapping**: On"));
    assert!(markdown.contains("**Flow Tap Term**: 150ms"));
    assert!(markdown.contains("**Chordal Hold**: On"));
}

fn assert_home_row_mods_settings(settings: &TapHoldSettings) {
    assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
    assert_eq!(settings.tapping_term, 175);
    assert!(settings.retro_tapping);
    assert_eq!(settings.flow_tap_term, Some(150));
    assert!(settings.chordal_hold);
}

fn assert_custom_settings_markdown(markdown: &str) {
    assert!(markdown.contains("**Tap-Hold Preset**: Custom"));
    assert!(markdown.contains("**Tapping Term**: 180ms"));
    assert!(markdown.contains("**Quick Tap Term**: 100ms"));
    assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));
    assert!(markdown.contains("**Retro Tapping**: On"));
    assert!(markdown.contains("**Tapping Toggle**: 3 taps"));
    assert!(markdown.contains("**Flow Tap Term**: 120ms"));
    assert!(markdown.contains("**Chordal Hold**: On"));
}

fn assert_custom_settings(settings: &TapHoldSettings) {
    assert_eq!(settings.preset, TapHoldPreset::Custom);
    assert_eq!(settings.tapping_term, 180);
    assert_eq!(settings.quick_tap_term, Some(100));
    assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
    assert!(settings.retro_tapping);
    assert_eq!(settings.tapping_toggle, 3);
    assert_eq!(settings.flow_tap_term, Some(120));
    assert!(settings.chordal_hold);
}
```

**Cognitive Complexity Reduction**:
- **Before**: 31 (one function with 3 scenarios, 40+ assertions)
- **After**: ~8 per test function (3 functions, each with helper functions)
- **Reduction**: 74% decrease per function

---

## Implementation Plan

### Step 1: Create Handler Module Structure
- [ ] Create `src/tui/handlers/action_handlers/` directory
- [ ] Create `mod.rs` with module declarations
- [ ] Create empty handler files (navigation.rs, popups.rs, etc.)

### Step 2: Extract Simple Handlers (Low-Hanging Fruit)
- [ ] **Navigation handlers** (8 actions) - 1-3 lines each
- [ ] **Simple popup handlers** (5 actions) - call one method + set status
- [ ] **Simple key operations** (3 actions) - ClearKey, CopyKey, CutKey
- [ ] **Color handlers** (4 actions) - straightforward state updates

**Estimated Time**: 2-3 hours

### Step 3: Extract Medium Complexity Handlers
- [ ] **File operations** - Quit (with dirty check), Save (with error handling)
- [ ] **Category assignment** (2 actions)
- [ ] **Selection operations** (2 actions)
- [ ] **Firmware handlers** (2 actions) - delegate to existing functions

**Estimated Time**: 2-3 hours

### Step 4: Extract Complex Handlers
- [ ] **OpenKeycodePicker** - nested if-let-else logic
- [ ] **SetupWizard** - QMK path validation + error handling
- [ ] **SwitchLayoutVariant** - complex multi-step logic
- [ ] **ViewBuildLog** - state checking + toggle logic
- [ ] **PasteKey, UndoPaste** - clipboard operations with validation

**Estimated Time**: 3-4 hours

### Step 5: Refactor dispatch_action()
- [ ] Update `dispatch_action()` to simple match with function calls
- [ ] Add imports for handler modules
- [ ] Remove old inline implementations
- [ ] Run tests to verify no behavioral changes

**Estimated Time**: 1 hour

### Step 6: Split Test Function
- [ ] Create 3 separate test functions
- [ ] Extract assertion helper functions
- [ ] Delete original `test_tap_hold_settings_round_trip()`
- [ ] Run tests to verify coverage maintained

**Estimated Time**: 1 hour

### Step 7: Verification
- [ ] Run `cargo clippy --all-targets` - no cognitive complexity warnings
- [ ] Run `cargo test` - all 244+ tests pass
- [ ] Manual testing - verify no behavioral regressions
- [ ] Performance check - no degradation

**Estimated Time**: 1 hour

---

## Success Criteria

1. ✅ **Zero cognitive complexity warnings** in `cargo clippy --all-targets`
2. ✅ **All existing tests pass** with no modifications to test logic
3. ✅ **No behavioral changes** - application works identically
4. ✅ **Code is more maintainable**:
   - Actions grouped by category in separate files
   - Each handler function ≤25 lines
   - Clear, descriptive function names
   - Easy to add new actions (add handler + add to dispatch match)
5. ✅ **Test coverage maintained** - all scenarios still tested

---

## Benefits

### Maintainability
- **Easier to find code**: "Where's the Save handler?" → `file_ops.rs`
- **Easier to add features**: Add new action → add handler function → add match arm
- **Easier to test**: Can unit test individual handlers without full AppState
- **Easier to review**: Small, focused functions in logical files

### Code Quality
- **Reduced cognitive load**: Each function does one thing
- **Better error messages**: Clear function names in stack traces
- **Improved readability**: No 647-line functions
- **Consistent patterns**: All handlers follow same signature

### Testing
- **Isolated tests**: Can test handlers independently
- **Clearer failures**: Test name indicates which scenario failed
- **Better coverage reporting**: Per-function coverage metrics

---

## Risks & Mitigations

### Risk 1: Breaking Changes
**Mitigation**: 
- Run full test suite after each step
- Manual testing of all actions
- Git commits after each working step (easy rollback)

### Risk 2: Performance Regression
**Mitigation**:
- Function call overhead is negligible in Rust (especially with inline)
- Can add `#[inline]` to hot path handlers if needed
- Benchmark critical paths before/after

### Risk 3: Over-Fragmentation
**Mitigation**:
- Keep handlers in same module if closely related
- Balance: not too many tiny files, not too few giant files
- Group by functionality (navigation, popups, etc.)

---

## Alternative Approaches (Rejected)

### 1. Trait-Based Dispatch
```rust
trait ActionHandler {
    fn handle(&self, state: &mut AppState) -> Result<bool>;
}

impl ActionHandler for Action {
    fn handle(&self, state: &mut AppState) -> Result<bool> {
        match self {
            Action::Quit => /* ... */
        }
    }
}
```
**Rejected**: Doesn't reduce complexity, just moves it to a different location.

### 2. Macro-Based Handler Registration
```rust
register_handlers! {
    Action::Quit => handle_quit,
    Action::Save => handle_save,
    // ...
}
```
**Rejected**: Over-engineered for the problem. Simple function calls are clearer.

### 3. Dynamic Dispatch with HashMap
```rust
let handlers: HashMap<Action, Box<dyn Fn(&mut AppState) -> Result<bool>>> = ...;
handlers[&action](state)
```
**Rejected**: Runtime overhead, less type-safe, harder to debug.

---

## Timeline

**Total Estimated Time**: 10-13 hours

- Step 1 (Setup): 30 minutes
- Step 2 (Simple handlers): 2-3 hours
- Step 3 (Medium handlers): 2-3 hours
- Step 4 (Complex handlers): 3-4 hours
- Step 5 (Refactor dispatch): 1 hour
- Step 6 (Split test): 1 hour
- Step 7 (Verification): 1 hour

**Parallelization**: Steps 2-4 can be done independently (extract different handler categories simultaneously).

---

## Next Steps

1. **Approve plan**: Review and approve this refactoring plan
2. **Create feature branch**: `018-reduce-cognitive-complexity`
3. **Execute Step 1**: Set up handler module structure
4. **Execute Steps 2-4**: Extract handlers (can parallelize with subagents)
5. **Execute Step 5**: Refactor dispatch_action()
6. **Execute Step 6**: Split test function
7. **Execute Step 7**: Verify and commit

**Start with**: Step 1 (module setup) + Step 2 (simple handlers) to validate approach.
