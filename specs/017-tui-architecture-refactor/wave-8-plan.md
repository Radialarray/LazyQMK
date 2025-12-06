# Wave 8 Implementation Plan: AppState Refactoring

## Overview

Replace 17+ component state fields in AppState with a unified Component trait-based system.

## Current AppState Structure (1192 lines)

**Component States (17 fields to remove)**:
1. `keycode_picker_state: KeycodePickerState`
2. `color_picker_state: ColorPickerState`
3. `color_picker_context: Option<ColorPickerContext>`
4. `category_picker_state: CategoryPickerState`
5. `category_picker_context: Option<CategoryPickerContext>`
6. `category_manager_state: CategoryManagerState`
7. `layer_manager_state: LayerManagerState`
8. `build_log_state: BuildLogState`
9. `template_browser_state: TemplateBrowserState`
10. `template_save_dialog_state: TemplateSaveDialogState`
11. `metadata_editor_state: MetadataEditorState`
12. `help_overlay_state: HelpOverlayState`
13. `layout_picker_state: LayoutPickerState`
14. `wizard_state: OnboardingWizardState`
15. `settings_manager_state: SettingsManagerState`
16. `layer_picker_state: LayerPickerState`
17. `modifier_picker_state: ModifierPickerState`
18. `key_editor_state: KeyEditorState`

## Target AppState Structure

```rust
pub struct AppState {
    // Core data (unchanged)
    pub layout: Layout,
    pub source_path: Option<PathBuf>,
    pub dirty: bool,
    
    // UI state (unchanged)
    pub theme: Theme,
    pub current_layer: usize,
    pub selected_position: Position,
    pub active_popup: Option<PopupType>,
    pub status_message: String,
    pub error_message: Option<String>,
    
    // NEW: Active component system
    pub active_component: Option<ActiveComponent>,
    
    // System resources (unchanged)
    pub keycode_db: KeycodeDb,
    pub geometry: KeyboardGeometry,
    pub mapping: VisualLayoutMapping,
    pub config: Config,
    pub build_state: Option<BuildState>,
    
    // Other fields (unchanged)
    pub clipboard: KeyClipboard,
    pub flash_highlight: Option<(usize, Position, u8)>,
    pub selection_mode: Option<SelectionMode>,
    pub selected_keys: Vec<Position>,
    pub should_quit: bool,
    pub pending_keycode: PendingKeycodeState,
    pub return_to_settings_after_picker: bool,
}

enum ActiveComponent {
    ColorPicker(ColorPicker),
    KeycodePicker(KeycodePicker),
    LayerPicker(LayerPicker),
    CategoryPicker(CategoryPicker),
    ModifierPicker(ModifierPicker),
    CategoryManager(CategoryManager),
    LayerManager(LayerManager),
    SettingsManager(SettingsManager),
    MetadataEditor(MetadataEditor),
    TemplateBrowser(TemplateBrowser),
    LayoutPicker(LayoutPicker),
    KeyboardPicker(KeyboardPicker),
    BuildLog(BuildLog),
    HelpOverlay(HelpOverlay),
    KeyEditor(KeyEditor),
    OnboardingWizard(OnboardingWizardState), // Keep as-is for now
}
```

## Implementation Steps

### Step 1: Create ActiveComponent Enum
- File: `src/tui/mod.rs`
- Add enum with variants for all 16 migrated components
- Add helper methods: `as_color_picker()`, `as_mut_keycode_picker()`, etc.

### Step 2: Add Component Management Methods to AppState
```rust
impl AppState {
    pub fn open_color_picker(&mut self, context: ColorPickerContext, color: RgbColor) {
        let picker = ColorPicker::new(context, color);
        self.active_component = Some(ActiveComponent::ColorPicker(picker));
        self.active_popup = Some(PopupType::ColorPicker);
    }
    
    pub fn open_keycode_picker(&mut self) {
        let picker = KeycodePicker::new();
        self.active_component = Some(ActiveComponent::KeycodePicker(picker));
        self.active_popup = Some(PopupType::KeycodePicker);
    }
    
    // ... similar for all 16 components
    
    pub fn close_component(&mut self) {
        self.active_component = None;
        self.active_popup = None;
    }
}
```

### Step 3: Update Handler Integration
- File: `src/tui/handlers/popups.rs` (32KB)
- Replace direct state access with Component trait calls
- Process ComponentEvents and update AppState accordingly

Example transformation:
```rust
// OLD:
Some(PopupType::ColorPicker) => {
    color_picker::handle_input(state, key)
}

// NEW:
Some(PopupType::ColorPicker) => {
    if let Some(ActiveComponent::ColorPicker(picker)) = &mut state.active_component {
        if let Some(event) = picker.handle_input(key) {
            return handle_color_picker_event(state, event);
        }
    }
    Ok(false)
}
```

### Step 4: Update Rendering
- File: `src/tui/mod.rs` - `render_ui()` function
- Replace direct state rendering with Component trait calls

Example transformation:
```rust
// OLD:
Some(PopupType::ColorPicker) => {
    color_picker::render_color_picker(f, state);
}

// NEW:
Some(PopupType::ColorPicker) => {
    if let Some(ActiveComponent::ColorPicker(picker)) = &state.active_component {
        let area = centered_rect(70, 70, f.size());
        picker.render(f, area, &state.theme);
    }
}
```

### Step 5: Update All Component Opening Sites
Update ~50+ locations across handlers where components are opened:
- `handlers/actions.rs` (34KB) - ~15 locations
- `handlers/category.rs` (10KB) - ~5 locations  
- `handlers/layer.rs` (22KB) - ~8 locations
- `handlers/settings.rs` (26KB) - ~10 locations
- `handlers/templates.rs` (6KB) - ~5 locations
- Other files - ~7 locations

### Step 6: Remove Old State Fields
After all usages updated, remove the 17 state fields from AppState struct.

### Step 7: Update AppState::new() Constructor
Remove initialization of removed state fields.

## Files to Modify (19 files)

1. **src/tui/mod.rs** (1192 lines) - Core changes
2. **src/tui/handlers/popups.rs** (32KB) - Event handling
3. **src/tui/handlers/actions.rs** (34KB) - Component opening
4. **src/tui/handlers/category.rs** (10KB) - Category operations
5. **src/tui/handlers/layer.rs** (22KB) - Layer operations
6. **src/tui/handlers/settings.rs** (26KB) - Settings operations
7. **src/tui/handlers/templates.rs** (6KB) - Template operations
8. **src/tui/handlers/main.rs** (486 bytes) - Main input
9. All component files (for any AppState coupling)

## Testing Strategy

1. **Incremental**: Migrate one component at a time, test, commit
2. **Start simple**: Begin with ColorPicker (already has event handlers)
3. **Compilation**: Ensure code compiles after each component
4. **Manual testing**: Test each component's full workflow
5. **Regression**: Run full test suite after each migration

## Risk Mitigation

- Keep legacy functions during migration
- Test each component independently
- Can rollback individual components if issues arise
- Extensive manual testing after completion

## Success Criteria

- [ ] AppState reduced from 17+ to 1 component field
- [ ] All 247 tests pass
- [ ] No behavioral changes
- [ ] Clean compile with no errors
- [ ] All component workflows functional

## Estimated Effort

- **Time**: 8-12 hours
- **Complexity**: High (touching 19 files)
- **Risk**: Medium (can break existing functionality)
- **Reversibility**: High (can revert per component)
