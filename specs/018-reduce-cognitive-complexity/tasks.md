# Implementation Tasks: Reduce Cognitive Complexity

## Overview

This document provides detailed, actionable tasks for reducing cognitive complexity in the codebase. Each task is rated for:
- **Complexity**: Low (simple, repetitive) or High (requires reasoning, understanding context)
- **Isolation**: Isolated (independent, can parallelize) or Integrated (dependencies, needs coordination)
- **Agent**: Recommended agent type (@coder-low or @coder-high)

---

## Task Classification Guide

### Complexity Ratings
- **Low**: Simple, repetitive, well-defined changes. Clear pattern to follow.
- **High**: Requires understanding context, making decisions, handling edge cases.

### Isolation Ratings
- **Isolated**: Can be done independently without affecting other tasks.
- **Integrated**: Has dependencies or requires coordination with other tasks.

### Agent Assignments
- **@coder-low**: For Low complexity tasks (fast, efficient for simple work)
- **@coder-high**: For High complexity tasks (deeper reasoning required)

---

## Phase 1: Setup (Sequential)

### Task 1.1: Create Handler Module Structure
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 15 minutes

**Description**: Create the directory and file structure for action handlers.

**Actions**:
1. Create directory: `src/tui/handlers/action_handlers/`
2. Create `src/tui/handlers/action_handlers/mod.rs` with:
   ```rust
   //! Action handlers organized by category
   
   pub mod navigation;
   pub mod popups;
   pub mod file_ops;
   pub mod key_ops;
   pub mod selection;
   pub mod color;
   pub mod category;
   pub mod firmware;
   pub mod layout;
   ```
3. Create empty files:
   - `navigation.rs`
   - `popups.rs`
   - `file_ops.rs`
   - `key_ops.rs`
   - `selection.rs`
   - `color.rs`
   - `category.rs`
   - `firmware.rs`
   - `layout.rs`

**Success Criteria**:
- All files created
- `cargo check` passes
- No compilation errors

---

## Phase 2: Navigation Handlers (Parallel)

### Task 2.1: Implement Simple Navigation Handlers (4 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/navigation.rs`

**Description**: Extract 4 simple navigation actions that just call existing AppState methods.

**Actions**:
Implement these handlers:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle navigate up action
pub fn handle_navigate_up(state: &mut AppState) -> Result<bool> {
    state.move_selection_up();
    Ok(false)
}

/// Handle navigate down action
pub fn handle_navigate_down(state: &mut AppState) -> Result<bool> {
    state.move_selection_down();
    Ok(false)
}

/// Handle navigate left action
pub fn handle_navigate_left(state: &mut AppState) -> Result<bool> {
    state.move_selection_left();
    Ok(false)
}

/// Handle navigate right action
pub fn handle_navigate_right(state: &mut AppState) -> Result<bool> {
    state.move_selection_right();
    Ok(false)
}
```

**Success Criteria**:
- All 4 functions implemented
- `cargo check` passes
- Functions match existing behavior in `dispatch_action()`

---

### Task 2.2: Implement Jump Navigation Handlers (2 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 15 minutes

**File**: `src/tui/handlers/action_handlers/navigation.rs`

**Description**: Extract jump-to-first and jump-to-last navigation actions.

**Actions**:
Add these handlers to `navigation.rs`:

```rust
/// Handle jump to first key action
pub fn handle_jump_to_first(state: &mut AppState) -> Result<bool> {
    state.jump_to_first_key();
    Ok(false)
}

/// Handle jump to last key action
pub fn handle_jump_to_last(state: &mut AppState) -> Result<bool> {
    state.jump_to_last_key();
    Ok(false)
}
```

**Success Criteria**:
- Both functions implemented
- `cargo check` passes

---

### Task 2.3: Implement Layer Navigation Handlers (2 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/navigation.rs`

**Description**: Extract next-layer and previous-layer actions with boundary checking.

**Actions**:
Add these handlers to `navigation.rs`:

```rust
/// Handle next layer action
pub fn handle_next_layer(state: &mut AppState) -> Result<bool> {
    if state.current_layer < state.layout.layers.len() - 1 {
        state.current_layer += 1;
        state.set_status(format!("Layer {}", state.current_layer));
    }
    Ok(false)
}

/// Handle previous layer action
pub fn handle_previous_layer(state: &mut AppState) -> Result<bool> {
    if state.current_layer > 0 {
        state.current_layer -= 1;
        state.set_status(format!("Layer {}", state.current_layer));
    }
    Ok(false)
}
```

**Reference**: Lines 287-299 in current `dispatch_action()`

**Success Criteria**:
- Both functions implemented with boundary checks
- Status message updated correctly
- `cargo check` passes

---

## Phase 3: Simple Popup Handlers (Parallel)

### Task 3.1: Implement Simple Popup Openers (6 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 45 minutes

**File**: `src/tui/handlers/action_handlers/popups.rs`

**Description**: Extract simple popup actions that call one method and set status.

**Actions**:
Implement these handlers in `popups.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle open layer manager action
pub fn handle_open_layer_manager(state: &mut AppState) -> Result<bool> {
    state.open_layer_manager();
    state.set_status("Layer Manager - n: new, d: delete, r: rename");
    Ok(false)
}

/// Handle open category manager action
pub fn handle_open_category_manager(state: &mut AppState) -> Result<bool> {
    state.open_category_manager();
    state.set_status("Category Manager - n: new, r: rename, c: color, d: delete");
    Ok(false)
}

/// Handle open settings action
pub fn handle_open_settings(state: &mut AppState) -> Result<bool> {
    state.open_settings_manager();
    state.set_status("Settings Manager - Enter: edit, Esc: close");
    Ok(false)
}

/// Handle edit metadata action
pub fn handle_edit_metadata(state: &mut AppState) -> Result<bool> {
    state.open_metadata_editor();
    state.set_status("Edit Metadata - Tab: next field, Enter: save");
    Ok(false)
}

/// Handle browse templates action
pub fn handle_browse_templates(state: &mut AppState) -> Result<bool> {
    state.open_template_browser();
    state.set_status("Template Browser - Enter: load, /: search");
    Ok(false)
}

/// Handle toggle help action
pub fn handle_toggle_help(state: &mut AppState) -> Result<bool> {
    use crate::tui::PopupType;
    
    if state.active_popup == Some(PopupType::HelpOverlay) {
        state.close_component();
    } else {
        state.open_help_overlay();
    }
    Ok(false)
}
```

**Reference**: Lines 195-214, 256-259, 163-169 in current `dispatch_action()`

**Success Criteria**:
- All 6 functions implemented
- Status messages match existing behavior
- `cargo check` passes

---

### Task 3.2: Implement View Build Log Handler
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/popups.rs`

**Description**: Extract ViewBuildLog action with state checking and toggle logic.

**Actions**:
Add this handler to `popups.rs`:

```rust
use crate::tui::ActiveComponent;

/// Handle view build log action
pub fn handle_view_build_log(state: &mut AppState) -> Result<bool> {
    if state.build_state.is_some() {
        if matches!(state.active_component, Some(ActiveComponent::BuildLog(_))) {
            state.close_component();
        } else {
            state.open_build_log();
        }
        state.set_status("Build log toggled");
    } else {
        state.set_error("No build active");
    }
    Ok(false)
}
```

**Reference**: Lines 243-254 in current `dispatch_action()`

**Success Criteria**:
- Toggles build log correctly
- Error message when no build active
- `cargo check` passes

---

### Task 3.3: Implement Setup Wizard Handler
**Complexity**: High  
**Isolation**: Isolated  
**Agent**: @coder-high  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/popups.rs`

**Description**: Extract SetupWizard action with QMK path validation and error handling.

**Actions**:
Add this handler to `popups.rs`:

```rust
use crate::tui::{onboarding_wizard, PopupType};

/// Handle setup wizard action
pub fn handle_setup_wizard(state: &mut AppState) -> Result<bool> {
    let qmk_path = match &state.config.paths.qmk_firmware {
        Some(path) => path.clone(),
        None => {
            state.set_error("QMK firmware path not configured");
            return Ok(false);
        }
    };

    match onboarding_wizard::OnboardingWizardState::new_for_keyboard_selection(&qmk_path) {
        Ok(wizard_state) => {
            state.wizard_state = wizard_state;
            state.active_popup = Some(PopupType::SetupWizard);
            state.set_status("Setup Wizard - Follow prompts to configure");
        }
        Err(e) => {
            state.set_error(format!("Failed to start wizard: {e}"));
        }
    }
    Ok(false)
}
```

**Reference**: Lines 215-233 in current `dispatch_action()`

**Why High Complexity**: 
- Error handling logic
- Multiple code paths
- State initialization

**Success Criteria**:
- QMK path validation works
- Error messages match existing behavior
- Wizard initializes correctly
- `cargo check` passes

---

### Task 3.4: Implement Open Keycode Picker Handler
**Complexity**: High  
**Isolation**: Isolated  
**Agent**: @coder-high  
**Estimated Time**: 45 minutes

**File**: `src/tui/handlers/action_handlers/popups.rs`

**Description**: Extract the complex OpenKeycodePicker action with nested if-let-else logic.

**Actions**:
Add these handlers to `popups.rs`:

```rust
use crate::tui::{key_editor, PopupType};
use crate::models::Key;

/// Handle open keycode picker action
pub fn handle_open_keycode_picker(state: &mut AppState) -> Result<bool> {
    match get_selected_key_info(state) {
        Some((key, true)) => {
            // Key is assigned - open key editor
            open_key_editor(state, &key);
        }
        Some((_, false)) | None => {
            // Key is empty or no key selected - open keycode picker
            open_keycode_picker_for_empty_key(state);
        }
    }
    Ok(false)
}

/// Get selected key and whether it's assigned
fn get_selected_key_info(state: &AppState) -> Option<(Key, bool)> {
    state
        .get_selected_key()
        .map(|key| (key.clone(), key_editor::is_key_assigned(&key.keycode)))
}

/// Open key editor for an assigned key
fn open_key_editor(state: &mut AppState, key: &Key) {
    state.key_editor_state.init_for_key(key, state.current_layer);
    state.active_popup = Some(PopupType::KeyEditor);
    state.set_status("Key editor - Enter: Reassign, D: Description, C: Color");
}

/// Open keycode picker for an empty/unassigned key
fn open_keycode_picker_for_empty_key(state: &mut AppState) {
    state.open_keycode_picker();
    state.set_status("Select keycode - Type to search, Enter to apply");
}
```

**Reference**: Lines 171-193 in current `dispatch_action()`

**Why High Complexity**:
- Nested if-let-else logic
- Multiple code paths based on key state
- Helper functions needed for clarity
- Decision logic about which picker to open

**Success Criteria**:
- Opens key editor for assigned keys
- Opens keycode picker for empty keys
- Opens keycode picker when no key selected
- Status messages correct for each path
- `cargo check` passes

---

## Phase 4: File Operations (Parallel)

### Task 4.1: Implement Quit Handler
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 15 minutes

**File**: `src/tui/handlers/action_handlers/file_ops.rs`

**Description**: Extract Quit action with dirty state checking.

**Actions**:
Implement in `file_ops.rs`:

```rust
use crate::tui::{AppState, PopupType};
use anyhow::Result;

/// Handle quit action
pub fn handle_quit(state: &mut AppState) -> Result<bool> {
    if state.dirty {
        state.active_popup = Some(PopupType::UnsavedChangesPrompt);
        Ok(false)
    } else {
        Ok(true)
    }
}
```

**Reference**: Lines 145-151 in current `dispatch_action()`

**Success Criteria**:
- Shows unsaved changes prompt if dirty
- Returns true (quit) if not dirty
- `cargo check` passes

---

### Task 4.2: Implement Save Handler
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/file_ops.rs`

**Description**: Extract Save action with error handling.

**Actions**:
Add to `file_ops.rs`:

```rust
use crate::services::LayoutService;

/// Handle save action
pub fn handle_save(state: &mut AppState) -> Result<bool> {
    if let Some(path) = &state.source_path.clone() {
        LayoutService::save(&state.layout, path)?;
        state.mark_clean();
        state.set_status("Saved");
    } else {
        state.set_error("No file path set");
    }
    Ok(false)
}
```

**Reference**: Lines 153-161 in current `dispatch_action()`

**Success Criteria**:
- Saves layout to existing path
- Marks state clean after save
- Shows error if no path set
- `cargo check` passes

---

### Task 4.3: Implement Save As Template Handler
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/file_ops.rs`

**Description**: Extract SaveAsTemplate action.

**Actions**:
Add to `file_ops.rs`:

```rust
use crate::tui::{PopupType, TemplateSaveDialogState};

/// Handle save as template action
pub fn handle_save_as_template(state: &mut AppState) -> Result<bool> {
    state.template_save_dialog_state =
        TemplateSaveDialogState::new(state.layout.metadata.name.clone());
    state.active_popup = Some(PopupType::TemplateSaveDialog);
    state.set_status("Save as Template - Tab: next field, Enter: save");
    Ok(false)
}
```

**Reference**: Lines 261-266 in current `dispatch_action()`

**Success Criteria**:
- Initializes template save dialog
- Opens correct popup
- Sets correct status message
- `cargo check` passes

---

## Phase 5: Key Operations (Parallel)

### Task 5.1: Implement Simple Key Operations (3 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/key_ops.rs`

**Description**: Extract ClearKey, CopyKey, CutKey actions.

**Actions**:
Implement in `key_ops.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle clear key action
pub fn handle_clear_key(state: &mut AppState) -> Result<bool> {
    state.clear_selected_keys();
    Ok(false)
}

/// Handle copy key action
pub fn handle_copy_key(state: &mut AppState) -> Result<bool> {
    state.copy_selected_keys();
    Ok(false)
}

/// Handle cut key action
pub fn handle_cut_key(state: &mut AppState) -> Result<bool> {
    state.cut_selected_keys();
    Ok(false)
}
```

**Reference**: Lines 301-313 in current `dispatch_action()`

**Success Criteria**:
- All 3 functions implemented
- Delegate to existing AppState methods
- `cargo check` passes

---

### Task 5.2: Implement Paste and Undo Paste Handlers
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/key_ops.rs`

**Description**: Extract PasteKey and UndoPaste actions.

**Actions**:
Add to `key_ops.rs`:

```rust
/// Handle paste key action
pub fn handle_paste_key(state: &mut AppState) -> Result<bool> {
    state.paste_keys();
    Ok(false)
}

/// Handle undo paste action
pub fn handle_undo_paste(state: &mut AppState) -> Result<bool> {
    state.undo_paste();
    Ok(false)
}
```

**Reference**: Lines 315-321 in current `dispatch_action()`

**Success Criteria**:
- Both functions implemented
- Delegate to existing AppState methods
- `cargo check` passes

---

### Task 5.3: Implement Toggle Current Key Handler
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 15 minutes

**File**: `src/tui/handlers/action_handlers/key_ops.rs`

**Description**: Extract ToggleCurrentKey action.

**Actions**:
Add to `key_ops.rs`:

```rust
/// Handle toggle current key action
pub fn handle_toggle_current_key(state: &mut AppState) -> Result<bool> {
    state.toggle_current_key_in_selection();
    Ok(false)
}
```

**Reference**: Lines 372-375 in current `dispatch_action()`

**Success Criteria**:
- Function implemented
- Delegates to existing AppState method
- `cargo check` passes

---

## Phase 6: Selection, Color, Category Handlers (Parallel)

### Task 6.1: Implement Selection Handlers (2 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/selection.rs`

**Description**: Extract ToggleSelectionMode and StartRectangleSelect actions.

**Actions**:
Implement in `selection.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle toggle selection mode action
pub fn handle_toggle_selection_mode(state: &mut AppState) -> Result<bool> {
    state.toggle_selection_mode();
    Ok(false)
}

/// Handle start rectangle select action
pub fn handle_start_rectangle_select(state: &mut AppState) -> Result<bool> {
    state.start_rectangle_selection();
    Ok(false)
}
```

**Reference**: Lines 363-370 in current `dispatch_action()`

**Success Criteria**:
- Both functions implemented
- Delegate to existing AppState methods
- `cargo check` passes

---

### Task 6.2: Implement Color Handlers (4 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/color.rs`

**Description**: Extract all 4 color-related actions.

**Actions**:
Implement in `color.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle set individual key color action
pub fn handle_set_individual_key_color(state: &mut AppState) -> Result<bool> {
    state.open_color_picker_for_key();
    Ok(false)
}

/// Handle set layer color action
pub fn handle_set_layer_color(state: &mut AppState) -> Result<bool> {
    state.open_color_picker_for_layer();
    Ok(false)
}

/// Handle toggle layer colors action
pub fn handle_toggle_layer_colors(state: &mut AppState) -> Result<bool> {
    state.toggle_show_layer_colors();
    Ok(false)
}

/// Handle toggle all layer colors action
pub fn handle_toggle_all_layer_colors(state: &mut AppState) -> Result<bool> {
    state.toggle_all_layer_colors();
    Ok(false)
}
```

**Reference**: Lines 377-390 in current `dispatch_action()`

**Success Criteria**:
- All 4 functions implemented
- Delegate to existing AppState methods
- `cargo check` passes

---

### Task 6.3: Implement Category Assignment Handlers (2 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/category.rs`

**Description**: Extract AssignCategoryToKey and AssignCategoryToLayer actions.

**Actions**:
Implement in `category.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle assign category to key action
pub fn handle_assign_category_to_key(state: &mut AppState) -> Result<bool> {
    state.open_category_picker_for_key();
    Ok(false)
}

/// Handle assign category to layer action
pub fn handle_assign_category_to_layer(state: &mut AppState) -> Result<bool> {
    state.open_category_picker_for_layer();
    Ok(false)
}
```

**Reference**: Lines 323-330 in current `dispatch_action()`

**Success Criteria**:
- Both functions implemented
- Delegate to existing AppState methods
- `cargo check` passes

---

## Phase 7: Firmware and Layout Handlers (Parallel)

### Task 7.1: Implement Firmware Handlers (2 actions)
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 20 minutes

**File**: `src/tui/handlers/action_handlers/firmware.rs`

**Description**: Extract BuildFirmware and GenerateFirmware actions.

**Actions**:
Implement in `firmware.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle build firmware action
pub fn handle_build_firmware(state: &mut AppState) -> Result<bool> {
    super::super::handle_firmware_build(state)?;
    Ok(false)
}

/// Handle generate firmware action
pub fn handle_generate_firmware(state: &mut AppState) -> Result<bool> {
    super::super::handle_firmware_generation(state)?;
    Ok(false)
}
```

**Reference**: Lines 235-241 in current `dispatch_action()`

**Success Criteria**:
- Both functions implemented
- Delegate to existing helper functions
- Error propagation works correctly
- `cargo check` passes

---

### Task 7.2: Implement Switch Layout Variant Handler
**Complexity**: High  
**Isolation**: Isolated  
**Agent**: @coder-high  
**Estimated Time**: 30 minutes

**File**: `src/tui/handlers/action_handlers/layout.rs`

**Description**: Extract SwitchLayoutVariant action with QMK path validation and error handling.

**Actions**:
Implement in `layout.rs`:

```rust
use crate::tui::AppState;
use anyhow::Result;

/// Handle switch layout variant action
pub fn handle_switch_layout_variant(state: &mut AppState) -> Result<bool> {
    let qmk_path = match &state.config.paths.qmk_firmware {
        Some(path) => path.clone(),
        None => {
            state.set_error("QMK firmware path not configured");
            return Ok(false);
        }
    };

    let keyboard = state.layout.metadata.keyboard.as_deref().unwrap_or("");
    let base_keyboard = AppState::extract_base_keyboard(keyboard);

    if let Err(e) = state.open_layout_variant_picker(&qmk_path, &base_keyboard) {
        state.set_error(format!("Failed to load layouts: {e}"));
        return Ok(false);
    }

    state.set_status("Select layout variant - ↑↓: Navigate, Enter: Select");
    Ok(false)
}
```

**Reference**: Lines 268-285 in current `dispatch_action()`

**Why High Complexity**:
- Multiple error paths
- QMK path validation
- Keyboard name extraction
- Error handling with Result

**Success Criteria**:
- QMK path validation works
- Error messages match existing behavior
- Layout picker opens correctly
- `cargo check` passes

---

## Phase 8: Integration (Sequential)

### Task 8.1: Update dispatch_action to Use Handlers
**Complexity**: High  
**Isolation**: Integrated  
**Agent**: @coder-high  
**Estimated Time**: 1 hour

**File**: `src/tui/handlers/actions.rs`

**Description**: Replace the 647-line dispatch_action implementation with simple handler calls.

**Actions**:
1. Add import at top of file:
   ```rust
   mod action_handlers;
   use action_handlers::*;
   ```

2. Replace the entire match statement (lines 144-788) with:
   ```rust
   pub fn dispatch_action(state: &mut AppState, action: Action) -> Result<bool> {
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

3. Remove the `#[allow(clippy::cognitive_complexity)]` attribute from line 142

**Why High Complexity**:
- Must ensure all 38 actions are mapped correctly
- Must maintain exact behavior
- Critical integration point
- Requires careful testing

**Success Criteria**:
- All 38 actions mapped to handlers
- `cargo check` passes
- `cargo clippy` shows NO cognitive complexity warning for dispatch_action
- All tests pass (`cargo test`)
- Manual testing confirms no behavioral changes

---

## Phase 9: Test Refactoring (Sequential)

### Task 9.1: Split test_tap_hold_settings_round_trip
**Complexity**: Low  
**Isolation**: Isolated  
**Agent**: @coder-low  
**Estimated Time**: 1 hour

**File**: `src/parser/template_gen.rs`

**Description**: Split the complex test function into 3 separate tests with helper functions.

**Actions**:

1. **Create helper functions** (add before the tests):
   ```rust
   #[cfg(test)]
   mod test_helpers {
       use super::*;
       use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};

       pub fn assert_home_row_mods_markdown(markdown: &str) {
           assert!(markdown.contains("## Settings"));
           assert!(markdown.contains("**Tap-Hold Preset**: Home Row Mods"));
           assert!(markdown.contains("**Tapping Term**: 175ms"));
           assert!(markdown.contains("**Retro Tapping**: On"));
           assert!(markdown.contains("**Flow Tap Term**: 150ms"));
           assert!(markdown.contains("**Chordal Hold**: On"));
       }

       pub fn assert_home_row_mods_settings(settings: &TapHoldSettings) {
           assert_eq!(settings.preset, TapHoldPreset::HomeRowMods);
           assert_eq!(settings.tapping_term, 175);
           assert!(settings.retro_tapping);
           assert_eq!(settings.flow_tap_term, Some(150));
           assert!(settings.chordal_hold);
       }

       pub fn assert_custom_settings_markdown(markdown: &str) {
           assert!(markdown.contains("**Tap-Hold Preset**: Custom"));
           assert!(markdown.contains("**Tapping Term**: 180ms"));
           assert!(markdown.contains("**Quick Tap Term**: 100ms"));
           assert!(markdown.contains("**Hold Mode**: Hold On Other Key Press"));
           assert!(markdown.contains("**Retro Tapping**: On"));
           assert!(markdown.contains("**Tapping Toggle**: 3 taps"));
           assert!(markdown.contains("**Flow Tap Term**: 120ms"));
           assert!(markdown.contains("**Chordal Hold**: On"));
       }

       pub fn assert_custom_settings(settings: &TapHoldSettings) {
           assert_eq!(settings.preset, TapHoldPreset::Custom);
           assert_eq!(settings.tapping_term, 180);
           assert_eq!(settings.quick_tap_term, Some(100));
           assert_eq!(settings.hold_mode, HoldDecisionMode::HoldOnOtherKeyPress);
           assert!(settings.retro_tapping);
           assert_eq!(settings.tapping_toggle, 3);
           assert_eq!(settings.flow_tap_term, Some(120));
           assert!(settings.chordal_hold);
       }
   }
   ```

2. **Replace the existing test** (lines 622-691) with 3 separate tests:
   ```rust
   #[test]
   fn test_tap_hold_home_row_mods_preset_round_trip() {
       use crate::models::{TapHoldPreset, TapHoldSettings};
       use test_helpers::*;

       let mut layout = create_test_layout();
       layout.tap_hold_settings = TapHoldSettings::from_preset(TapHoldPreset::HomeRowMods);
       
       let markdown = generate_markdown(&layout).unwrap();
       assert_home_row_mods_markdown(&markdown);
       
       let parsed = parse_markdown_layout_str(&markdown).unwrap();
       assert_home_row_mods_settings(&parsed.tap_hold_settings);
   }

   #[test]
   fn test_tap_hold_custom_settings_round_trip() {
       use crate::models::{HoldDecisionMode, TapHoldPreset, TapHoldSettings};
       use test_helpers::*;

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
       use crate::models::TapHoldSettings;

       let mut layout = create_test_layout();
       layout.tap_hold_settings = TapHoldSettings::default();
       
       let markdown = generate_markdown(&layout).unwrap();
       assert!(!markdown.contains("Tap-Hold"));
       assert!(!markdown.contains("Tapping Term"));
       
       let parsed = parse_markdown_layout_str(&markdown).unwrap();
       assert_eq!(parsed.tap_hold_settings, TapHoldSettings::default());
   }
   ```

3. **Remove the `#[allow(clippy::cognitive_complexity)]`** attribute from line 623

**Success Criteria**:
- Original test deleted
- 3 new tests created
- Helper functions in test_helpers module
- All tests pass (`cargo test`)
- `cargo clippy` shows NO cognitive complexity warning
- Test coverage maintained (same scenarios tested)

---

## Phase 10: Verification (Sequential)

### Task 10.1: Final Verification
**Complexity**: High  
**Isolation**: Integrated  
**Agent**: Manual (human verification)  
**Estimated Time**: 1 hour

**Description**: Comprehensive verification that all changes work correctly.

**Actions**:
1. **Run Clippy**: `cargo clippy --all-targets`
   - ✅ Zero cognitive complexity warnings
   - ✅ No new warnings introduced
   
2. **Run Tests**: `cargo test`
   - ✅ All 244+ tests pass
   - ✅ New split tests pass individually
   
3. **Build**: `cargo build --release`
   - ✅ Successful compilation
   - ✅ No errors or warnings
   
4. **Manual Testing**: Test critical workflows
   - ✅ Navigation works (arrow keys, page up/down)
   - ✅ Opening popups works (all managers, pickers, etc.)
   - ✅ File operations work (save, quit with/without dirty state)
   - ✅ Key operations work (copy, paste, clear)
   - ✅ Firmware build/generate works
   - ✅ Color and category assignment works
   
5. **Performance Check**: Compare startup time and responsiveness
   - ✅ No degradation (function calls have negligible overhead)
   
6. **Code Quality Check**:
   - ✅ All handler functions ≤25 lines
   - ✅ Clear, descriptive function names
   - ✅ Consistent patterns across handlers
   - ✅ Good error messages

**Success Criteria**:
- All verification steps pass
- No regressions found
- Code quality improved
- Ready to commit

---

## Summary

### Task Count by Phase
- **Phase 1** (Setup): 1 task
- **Phase 2** (Navigation): 3 tasks
- **Phase 3** (Popups): 4 tasks
- **Phase 4** (File Ops): 3 tasks
- **Phase 5** (Key Ops): 3 tasks
- **Phase 6** (Selection/Color/Category): 3 tasks
- **Phase 7** (Firmware/Layout): 2 tasks
- **Phase 8** (Integration): 1 task
- **Phase 9** (Test Split): 1 task
- **Phase 10** (Verification): 1 task

**Total: 22 tasks**

### Complexity Breakdown
- **Low Complexity**: 17 tasks (77%)
- **High Complexity**: 5 tasks (23%)

### Isolation Breakdown
- **Isolated**: 20 tasks (91%) - Can parallelize
- **Integrated**: 2 tasks (9%) - Must run sequentially

### Agent Assignment
- **@coder-low**: 17 tasks (simple, repetitive)
- **@coder-high**: 4 tasks (complex logic, error handling)
- **Manual**: 1 task (final verification)

### Parallelization Strategy
**Phases 2-7 can run in parallel** (20 isolated tasks):
- Launch all @coder-low tasks simultaneously
- Launch all @coder-high tasks simultaneously
- Estimated wall-clock time: ~1-2 hours (vs 6-7 hours sequential)

**Phases 1, 8, 9, 10 must run sequentially**:
- Setup → Extract handlers → Integration → Test split → Verification
- Estimated wall-clock time: ~3 hours

**Total estimated wall-clock time: 4-5 hours** (with parallelization)

---

## Expected Outcomes

### Code Quality Improvements
- **647-line function** → **45-line dispatcher** (93% reduction)
- **Cognitive complexity**: 73 → ~10 (87% reduction)
- **38 inline handlers** → **38 dedicated functions** in 9 organized files
- **Test function**: 31 → ~8 per test (74% reduction per test)

### Maintainability Benefits
- Actions grouped by category (easy to find)
- Each handler function ≤25 lines (easy to understand)
- Clear separation of concerns (easy to modify)
- Easy to add new actions (add function + add match arm)

### Zero Technical Debt
- No `#[allow(clippy::cognitive_complexity)]` suppressions
- All Clippy warnings resolved properly
- Clean, maintainable architecture
