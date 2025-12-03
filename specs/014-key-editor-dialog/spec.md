# 014: Key Editor Dialog

## Overview

Add a comprehensive key editor dialog that opens when clicking on an already-assigned key. The dialog shows the current assignment, allows reassignment, and supports per-key descriptions that persist in the layout markdown file.

## Goals

1. **View current assignment**: Show keycode details including tap-hold breakdown
2. **Reassign key**: Open keycode picker to change the assignment
3. **Per-key descriptions**: Add optional description field saved to markdown
4. **Description display**: Show description in status bar when navigating to key

## User Flow

### Opening the Editor
- Navigate to a key with arrow keys
- Press `Enter` on an **already assigned** key (not `KC_NO` or `KC_TRNS`)
- Key Editor dialog opens showing current assignment

### Dialog Actions
- `Enter` - Open keycode picker to reassign
- `D` - Edit description field
- `C` - Open color picker (existing functionality)
- `Esc` - Close dialog without changes

### Description Display
- When navigating the keyboard, if the selected key has a description, it appears in a dedicated line above the status bar

## UI Design

```
┌───────────────────── Edit Key ─────────────────────┐
│ Position: Row 1, Col 3                              │
├─────────────────────────────────────────────────────┤
│                                                     │
│  CURRENT ASSIGNMENT                                 │
│  ┌───────────────────────────────────────────────┐ │
│  │  Keycode: LT(@uuid-123, KC_SPC)               │ │
│  │                                               │ │
│  │     ┌──────i┐                                 │ │
│  │     │▼L1   │  ← Visual preview               │ │
│  │     │ SPC  │                                 │ │
│  │     └──────┘                                 │ │
│  │                                               │ │
│  │  Hold: Layer 1 (Symbols)                      │ │
│  │  Tap:  KC_SPC (Space)                         │ │
│  └───────────────────────────────────────────────┘ │
│                                                     │
│  DESCRIPTION (optional)                             │
│  ┌───────────────────────────────────────────────┐ │
│  │ Primary thumb key. Hold for symbols layer,   │ │
│  │ tap for space.                               │ │
│  └───────────────────────────────────────────────┘ │
│                                                     │
├─────────────────────────────────────────────────────┤
│  [Enter] Reassign   [D] Description   [C] Color    │
│  [Esc] Close                                        │
└─────────────────────────────────────────────────────┘
```

### Status Bar with Description

```
┌──────────────────────────────────────────────────────────────────┐
│  ┌──────i┐  ┌──────┐  ┌──────┐                                   │
│  │▼L1   │  │ Q    │  │ W    │  ...                               │
│  │ SPC  │  │      │  │      │                                    │
│  └──────┘  └──────┘  └──────┘                                    │
│     ▲ Selected                                                    │
├──────────────────────────────────────────────────────────────────┤
│ Description: Primary thumb key - hold for symbols, tap for space │
├──────────────────────────────────────────────────────────────────┤
│ Status: Ready                                                     │
│ Help: ↑↓←→: Navigate | Enter: Edit | Shift+C: Color | ?: Help    │
└──────────────────────────────────────────────────────────────────┘
```

## Data Model Changes

### KeyDefinition (src/models/layer.rs)

Add `description` field:

```rust
pub struct KeyDefinition {
    pub position: Position,
    pub keycode: String,
    pub label: Option<String>,
    pub color_override: Option<RgbColor>,
    pub category_id: Option<String>,
    pub combo_participant: bool,
    pub description: Option<String>,  // NEW
}
```

### Markdown Format

Extend keycode syntax with description in square brackets:

```markdown
| LT(@uuid,KC_SPC)["Primary thumb key"] | KC_Q | KC_W |
```

Or for longer descriptions, use the existing format with a new section:

```markdown
## Key Descriptions

- 0:0:0: Primary thumb key - hold for symbols, tap for space
- 0:1:3: Home row mod - Ctrl on hold, F on tap
```

Format: `layer:row:col: description text`

## Implementation Plan

### Phase 1: Data Model
1. Add `description` field to `KeyDefinition`
2. Update parser to read descriptions from markdown
3. Update serializer to write descriptions to markdown

### Phase 2: Key Editor Dialog
1. Create `KeyEditorState` struct
2. Add `PopupType::KeyEditor` variant
3. Implement dialog rendering
4. Implement input handling

### Phase 3: Description Editing
1. Add description edit mode to dialog
2. Handle text input for description
3. Save description on confirm

### Phase 4: Status Bar Enhancement
1. Add description display line to status bar
2. Show description when key has one
3. Handle layout adjustments

### Phase 5: Integration
1. Wire Enter key to open editor (when key is assigned)
2. Connect to existing color picker flow
3. Connect to keycode picker for reassignment

## Files to Modify

- `src/models/layer.rs` - Add description field
- `src/parser/layout.rs` - Parse descriptions
- `src/parser/template_gen.rs` - Serialize descriptions
- `src/tui/mod.rs` - Add PopupType, state, handlers
- `src/tui/key_editor.rs` - NEW: Dialog implementation
- `src/tui/status_bar.rs` - Add description display
- `src/tui/keyboard.rs` - Minor: key preview in dialog

## Testing

1. Open editor on assigned key
2. View tap-hold breakdown correctly
3. Edit and save description
4. Description persists after save/reload
5. Description shows in status bar
6. Reassign key via picker
7. Color picker accessible from dialog
