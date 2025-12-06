# Phase C.1 Pilot Implementation Guide

## ColorPicker Component Trait Migration

This guide provides a detailed approach for migrating the ColorPicker to use the Component trait pattern, serving as a template for all other components.

### Current Architecture

**File**: `src/tui/color_picker.rs`

**State**: `ColorPickerState` (lines 60-79)
- Embedded directly in `AppState`
- Contains: mode, RGB values, active channel, palette data, selection state

**Input Handling**: `handle_input()` function (line 705)
- Takes `&mut AppState` and modifies it directly
- Calls `apply_color()` and `clear_color()` which directly mutate `AppState`

**Rendering**: `render_color_picker()` function (line 254)
- Takes `&AppState` and reads from `color_picker_state` field

**Context**: Uses `ColorPickerContext` enum in `AppState`
- Determines what is being colored (IndividualKey, LayerDefault, Category)

### Target Architecture

**Component Struct**:
```rust
pub struct ColorPicker {
    state: ColorPickerState,
    context: ColorPickerContext,
}

impl Component for ColorPicker {
    type Event = ColorPickerEvent;
    
    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        // Handle input, return events instead of mutating AppState
    }
    
    fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        // Render without needing full AppState
    }
}
```

**Event Type**:
```rust
pub enum ColorPickerEvent {
    ColorSelected(RgbColor),
    ColorCleared,
    Cancelled,
}
```

### Migration Steps

#### Step 1: Define ColorPickerEvent
- Add to `color_picker.rs`
- Map to `ComponentEvent` variants in event processing

#### Step 2: Create ColorPicker Component Struct
- Wrap `ColorPickerState` 
- Add `context` field
- Add constructor methods

#### Step 3: Implement Component Trait
- Move input handling logic from `handle_input()` to trait method
- Instead of calling `apply_color(state)`, return `Some(ColorPickerEvent::ColorSelected(color))`
- Instead of setting `state.active_popup = None`, component just completes
- Instead of calling `clear_color(state)`, return `Some(ColorPickerEvent::ColorCleared)`

#### Step 4: Refactor Rendering
- Change `render_color_picker(f, state)` signature
- Extract only theme from AppState
- Make rendering methods take `&ColorPicker` instead of `&AppState`

#### Step 5: Update Handler Integration
- In `handlers/popups.rs`, change from:
  ```rust
  color_picker::handle_input(state, key)
  ```
  To:
  ```rust
  if let Some(component) = state.active_component.as_mut() {
      if let Some(event) = component.handle_input(key) {
          return handle_color_picker_event(state, event);
      }
  }
  ```

#### Step 6: Update Component Opening
- Change from:
  ```rust
  state.active_popup = Some(PopupType::ColorPicker);
  state.color_picker_context = Some(context);
  state.color_picker_state = ColorPickerState::with_color(current_color);
  ```
  To:
  ```rust
  let component = ColorPicker::new(context, current_color);
  state.open_component(Box::new(component));
  ```

### Event Processing

Create `handle_color_picker_event()` function that processes events:

```rust
fn handle_color_picker_event(state: &mut AppState, event: ColorPickerEvent) -> Result<bool> {
    match event {
        ColorPickerEvent::ColorSelected(color) => {
            // Apply color based on context (from component)
            // Mark dirty, set status
        }
        ColorPickerEvent::ColorCleared => {
            // Clear color based on context
        }
        ColorPickerEvent::Cancelled => {
            // Just close, no changes
        }
    }
    state.close_component();
    Ok(false)
}
```

### Files to Modify

1. **src/tui/color_picker.rs**
   - Add `ColorPickerEvent` enum
   - Add `ColorPicker` struct
   - Implement `Component` trait
   - Refactor rendering functions
   - Update input handling to return events

2. **src/tui/handlers/popups.rs**
   - Update `handle_popup_input()` for ColorPicker case
   - Add `handle_color_picker_event()` function

3. **src/tui/handlers/actions.rs**
   - Update component opening logic (2 locations)

4. **src/tui/handlers/category.rs**
   - Update component opening logic (2 locations)

5. **src/tui/key_editor.rs**
   - Update component opening logic (1 location)

### Testing Strategy

After migration:
1. Test individual key coloring
2. Test layer default coloring
3. Test category coloring
4. Test palette mode navigation
5. Test custom RGB mode
6. Test color clearing
7. Test cancellation
8. Verify no behavior changes

### Success Criteria

- [ ] ColorPicker implements Component trait
- [ ] All input handling returns events instead of mutating state
- [ ] Rendering works without full AppState
- [ ] All 5 call sites updated
- [ ] All color picker flows work identically
- [ ] No compilation errors
- [ ] Manual testing passes

### Lessons for Other Components

Document after pilot:
- What worked well?
- What was challenging?
- Should trait design be adjusted?
- Is the pattern worth continuing?
