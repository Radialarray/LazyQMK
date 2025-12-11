# Implementation Plan: Legacy Code Cleanup

**Branch**: `refactor/legacy-code-cleanup` | **Date**: 2024-12-09 | **Spec**: N/A (refactoring)

## Summary

Remove legacy code patterns accumulated during the component migration (017-tui-architecture-refactor). The codebase has successfully migrated most components to use the Component trait pattern with `ActiveComponent`, but retains backward-compatibility shims, unused state fields, and dead code that should now be cleaned up.

## Technical Context

**Language/Version**: Rust 1.88.0  
**Primary Dependencies**: Ratatui 0.26, Crossterm 0.27, Serde 1.0  
**Testing**: `cargo test`, `cargo clippy`  
**Target Platform**: macOS/Linux terminal  

## Goals

1. **Remove dead code** - Functions/types marked `#[allow(dead_code)]` that are no longer needed
2. **Remove legacy AppState fields** - State fields that have been superseded by component-internal state
3. **Remove legacy rendering functions** - Wrapper functions kept for "backward compatibility"
4. **Remove legacy handlers** - Fallback handlers that are no longer called
5. **Clean up unused imports and dependencies**

## Inventory of Legacy Code

### 1. Legacy AppState Fields (src/tui/mod.rs)

| Field | Status | Used By | Action |
|-------|--------|---------|--------|
| `category_picker_context` | Possibly dead | Needs investigation | Remove if unused |
| `category_manager_state` | Synced with component | handlers/category.rs | Keep (state preservation across color picker) |
| `template_save_dialog_state` | Legacy | handlers/templates.rs | Migrate to component-internal |
| `wizard_state` | Legacy | app/onboarding.rs | Keep (complex wizard flow) |
| `layer_picker_state` | Has fallback rendering | handlers/popups.rs | Remove fallback, use component only |
| `pending_keycode` | Functional | handlers/popups.rs | Keep (multi-stage keycode building) |
| `key_editor_state` | Legacy | key_editor.rs | Migrate to component-internal |

### 2. Dead Code by File

#### `config_dialogs.rs` - Heavily deprecated
- `PathConfigDialogState` - entire struct unused
- `KeyboardPickerState` - entire struct unused  
- `KeyboardPicker` component - unused
- `LayoutVariantPickerState` - needs investigation
- `render_path_dialog()` - unused
- `render_keyboard_picker()` - unused
- `render_keyboard_picker_component()` - unused
- `handle_path_dialog_input()` - unused
- `handle_keyboard_picker_input()` - unused

**Recommendation**: Consider removing entire file or extracting only used parts.

#### `layer_picker.rs` - Partially migrated
- `render_layer_picker()` - legacy fallback rendering
- `handle_layer_picker_legacy()` in handlers/popups.rs
- `state_mut()`, `handle_navigation()` - unused component methods

#### `onboarding_wizard.rs` - Module marked dead
- Entire module has `#[allow(dead_code)]`
- Used by `app/onboarding.rs` but may be deprecated

#### Other files with `#[allow(dead_code)]` methods
- `build_log.rs`: `toggle()`, `state()`, `state_mut()`
- `category_manager.rs`: `is_browsing()`, `get_input()`, `get_input_mut()`
- `color_picker.rs`: `render_rgb_instructions()`
- `help_overlay.rs`: `state()`, `state_mut()`
- `keyboard.rs`: `TapHoldInfo.kind` field
- `layer_manager.rs`: `is_browsing()`, `state()`, `state_mut()`, `layers()`
- `layout_picker.rs`: `get_selected_layout()`, `state()`, `state_mut()`, `LayoutSelected` variant
- `metadata_editor.rs`: `Closed` variant, `state()`, `state_mut()`
- `modifier_picker.rs`: `is_left()`, `with_modifiers()`, `state()`, `state_mut()`, `get_mod_string()`
- `settings_manager.rs`: `all()`, `is_browsing()`, `Closed` variant
- `template_browser.rs`: `SaveAsTemplate` variant, `state()`, `state_mut()`
- `theme.rs`: `ThemeVariant` enum, `from_variant()`, `variant()`

### 3. Unused Enums/Variants

| Location | Item | Status |
|----------|------|--------|
| `component.rs` | `CategoryPickerContext::SingleKey` | Dead |
| `component.rs` | `CategoryPickerContext::MultipleKeys` | Dead |
| `mod.rs` | `ColorPickerContext` enum | Needs investigation |
| `mod.rs` | Some `PopupType` variants | Needs investigation |
| `mod.rs` | Some `ActiveComponent` variants | Needs investigation |

## Implementation Phases

### Phase 1: Safe Removals (Low Risk)
Remove code that is definitively unused:

1. **config_dialogs.rs cleanup**
   - Remove `PathConfigDialogState` and related functions
   - Remove unused `KeyboardPicker` component
   - Keep only `LayoutVariantPicker` if still used

2. **Remove unused `state()`/`state_mut()` methods**
   - These were for legacy patterns; components now own their state

3. **Remove unused enum variants**
   - `CategoryPickerContext::SingleKey/MultipleKeys`
   - Investigate and remove unused `PopupType` variants

### Phase 2: Layer Picker Migration (Medium Risk)
Complete the layer picker component migration:

1. Remove `layer_picker_state` from `AppState`
2. Remove `handle_layer_picker_legacy()` fallback handler
3. Remove `render_layer_picker()` legacy function
4. Ensure component-based flow works for all layer picker use cases

### Phase 3: Template Save Dialog Migration (Medium Risk)
Migrate template save dialog to component-internal state:

1. Remove `template_save_dialog_state` from `AppState`
2. Update handlers to use component
3. Test template save workflow

### Phase 4: Key Editor State Migration (Medium Risk)
Migrate key editor to component-internal state:

1. Analyze `key_editor_state` usage patterns
2. Remove from `AppState` if component can own state
3. Update handlers

### Phase 5: Major Cleanup (Higher Risk)
After phases 1-4 are stable:

1. Evaluate `onboarding_wizard` module - keep or remove?
2. Clean up `ColorPickerContext` if unused
3. Remove any remaining `#[allow(dead_code)]` annotations
4. Final clippy cleanup

## Testing Strategy

After each phase:
1. `cargo build` - ensure no compilation errors
2. `cargo clippy` - check for new warnings
3. `cargo test` - ensure no test regressions
4. Manual testing of affected UI flows

## Success Criteria

- [ ] No `#[allow(dead_code)]` annotations on production code
- [ ] `AppState` contains only actively-used fields
- [ ] No legacy wrapper functions for "backward compatibility"
- [ ] All handlers use component-based patterns
- [ ] `cargo clippy` passes with no warnings
- [ ] All tests pass
- [ ] Manual testing confirms no regressions

## Estimated Effort

| Phase | Effort | Risk |
|-------|--------|------|
| Phase 1 | 1-2 hours | Low |
| Phase 2 | 1 hour | Medium |
| Phase 3 | 30 min | Medium |
| Phase 4 | 1 hour | Medium |
| Phase 5 | 1-2 hours | Higher |

**Total**: ~5-7 hours

## Files Affected

Primary files to modify:
- `src/tui/mod.rs` - Remove legacy AppState fields
- `src/tui/config_dialogs.rs` - Major cleanup or removal
- `src/tui/layer_picker.rs` - Remove legacy functions
- `src/tui/handlers/popups.rs` - Remove legacy handlers
- `src/tui/component.rs` - Remove dead enum variants

Secondary files (remove unused methods):
- `src/tui/build_log.rs`
- `src/tui/category_manager.rs`
- `src/tui/color_picker.rs`
- `src/tui/help_overlay.rs`
- `src/tui/keyboard.rs`
- `src/tui/layer_manager.rs`
- `src/tui/layout_picker.rs`
- `src/tui/metadata_editor.rs`
- `src/tui/modifier_picker.rs`
- `src/tui/settings_manager.rs`
- `src/tui/template_browser.rs`
- `src/tui/theme.rs`
