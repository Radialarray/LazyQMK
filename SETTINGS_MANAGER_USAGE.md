# SettingsManager Component - Usage Guide

## Overview
The SettingsManager component provides a clean interface for the settings dialog, following the Component trait pattern established in Waves 1-3.

## Basic Usage

### 1. Create the Component

```rust
let mut settings_manager = SettingsManager::new();
```

### 2. Build Context

```rust
let context = SettingsManagerContext {
    rgb_enabled: layout.rgb_enabled,
    rgb_brightness: layout.rgb_brightness,
    rgb_timeout_ms: layout.rgb_timeout_ms,
    uncolored_key_behavior: layout.uncolored_key_behavior,
    tap_hold_settings: layout.tap_hold_settings.clone(),
    config: config.clone(),
    layout: layout.clone(),
};
```

### 3. Handle Input

```rust
if let Some(event) = settings_manager.handle_input_with_context(key_event, &context) {
    match event {
        SettingsManagerEvent::SettingsUpdated => {
            // Extract values from settings_manager.state() and apply them
            apply_settings(&settings_manager.state(), &mut layout, &mut config);
        }
        SettingsManagerEvent::Cancelled => {
            // User cancelled - close dialog without saving
        }
        SettingsManagerEvent::Closed => {
            // Dialog closed naturally
        }
    }
}
```

### 4. Render

```rust
settings_manager.render_with_context(f, area, theme, &context);
```

## Event Handling Pattern

### SettingsUpdated Event
When `SettingsUpdated` is emitted, the parent should:
1. Extract the current mode from `settings_manager.state().mode`
2. Apply the setting based on the mode
3. Update the appropriate field in Layout or Config
4. Mark dirty if needed
5. Close the dialog

Example:
```rust
SettingsManagerEvent::SettingsUpdated => {
    let state = settings_manager.state();
    match &state.mode {
        ManagerMode::EditingNumeric { setting, .. } => {
            if let Some(value) = state.get_numeric_value() {
                apply_numeric_setting(*setting, value, &mut layout);
            }
        }
        ManagerMode::SelectingTapHoldPreset { .. } => {
            if let Some(idx) = state.get_selected_option() {
                if let Some(&preset) = TapHoldPreset::all().get(idx) {
                    layout.tap_hold_settings.apply_preset(preset);
                }
            }
        }
        // ... handle other modes
        _ => {}
    }
    state_mut.cancel(); // Reset mode to Browsing
}
```

## State Access

### Read-Only Access
```rust
let state = settings_manager.state();
let selected = state.selected;
let mode = &state.mode;
```

### Mutable Access
```rust
let state = settings_manager.state_mut();
state.selected = 5;
state.mode = ManagerMode::Browsing;
```

## Complex Settings

Some settings require special handling:

### Keyboard Selection
Opens a separate wizard - handled by parent:
```rust
KeyCode::Enter => {
    let settings = SettingItem::all();
    if let Some(&SettingItem::Keyboard) = settings.get(state.selected) {
        // Open keyboard picker wizard
        state.active_popup = Some(PopupType::SetupWizard);
    }
}
```

### Layout Variant Selection
Opens layout picker - handled by parent:
```rust
if let Some(&SettingItem::LayoutVariant) = settings.get(state.selected) {
    // Open layout picker
    state.active_popup = Some(PopupType::LayoutPicker);
}
```

## Migration from Legacy Code

### Before (Direct State Access)
```rust
// In AppState
pub settings_manager_state: SettingsManagerState,

// Usage
settings_manager_state.select_next(count);

// Rendering
settings_manager::render_settings_manager(
    f, area, &state.settings_manager_state,
    rgb_enabled, rgb_brightness, ...
);
```

### After (Component Pattern)
```rust
// In AppState
pub settings_manager: SettingsManager,

// Usage
if let Some(event) = settings_manager.handle_input_with_context(key, &context) {
    // Handle event
}

// Rendering
settings_manager.render_with_context(f, area, theme, &context);
```

## Key Differences from Standard Component Trait

SettingsManager does NOT implement the standard `Component` trait because:

1. **Requires Complex Context**: Needs both Config and Layout data
2. **Custom Method Signatures**: 
   - `handle_input_with_context(key, context)` instead of `handle_input(key)`
   - `render_with_context(f, area, theme, context)` instead of `render(f, area, theme)`

This is intentional - some components are too complex for the simple trait.

## Backward Compatibility

The legacy functions still work:
```rust
// Still valid - for gradual migration
settings_manager::render_settings_manager(
    f, area, &state,
    rgb_enabled, rgb_brightness, rgb_timeout_ms,
    uncolored_key_behavior, &tap_hold_settings,
    &config, &layout, theme
);
```

## Testing Checklist

- [ ] Open settings (Shift+S)
- [ ] Navigate all setting groups
- [ ] Edit numeric settings (tapping term, RGB brightness)
- [ ] Toggle boolean settings (RGB enabled, retro tapping)
- [ ] Edit string settings (keymap name)
- [ ] Edit path settings (QMK path, output dir)
- [ ] Select presets (tap-hold preset)
- [ ] Select enum values (hold mode, output format)
- [ ] Verify Esc cancels properly
- [ ] Verify Enter applies changes
- [ ] Verify navigation works (Up/Down, k/j)
