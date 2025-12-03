# Spec 013: Key Copy/Cut/Paste Clipboard

## Overview

Implement clipboard functionality for keyboard keys, allowing users to copy, cut, and paste keys within the same layer or across layers.

## Problem Statement

Currently, users cannot duplicate keys or move them between positions/layers without manually re-entering the keycode and settings. This makes keyboard layout editing tedious, especially when:
- Setting up similar keys across layers (e.g., gaming layer based on main layer)
- Moving keys to optimize layout
- Duplicating complex keycodes like `LT(1, KC_SPC)` with color overrides

## Solution

Add an internal clipboard system with:
- Single-key copy/cut/paste operations
- Both vim-style and standard keybindings
- Visual feedback for operations
- Cross-layer paste support

## Implementation Phases

### Phase 1: Simple Single-Key Clipboard (This Spec)
- Copy/cut/paste one key at a time
- Dual keybinding support (vim + standard)
- Status bar feedback
- Visual dim for cut source
- Cross-layer paste

### Phase 2: Enhanced Feedback (Future)
- Flash highlight on paste
- Clipboard preview in status
- Undo integration

### Phase 3: Multi-Select (Future)
- Visual selection mode (`v`)
- Rectangle/range select
- Bulk operations

### Phase 4: Layer Operations (Future)
- Duplicate layer
- Copy layer to another
- Swap layers

## Keybindings

| Operation | Vim-style | Standard | Description |
|-----------|-----------|----------|-------------|
| Copy | `y` | Ctrl+C | Copy selected key to clipboard |
| Cut | `d` | Ctrl+X | Copy + clear source to KC_TRNS |
| Paste | `p` | Ctrl+V | Paste clipboard at selection |
| Cancel | Esc | Esc | Clear cut state |

## What Gets Copied

Full `KeyDefinition` data:
- `keycode` - The QMK keycode string
- `color_override` - Individual key color (if set)
- `category_id` - Category assignment (if set)

Position is NOT copied (paste uses target position).

## Behavior Details

### Copy (`y` / Ctrl+C)
1. Read selected key's data
2. Store in clipboard
3. Show status: "Copied: KC_A" or "Copied: LT(1,SPC) (with color)"
4. No visual change to source key

### Cut (`d` / Ctrl+X)
1. Read selected key's data
2. Store in clipboard with cut flag
3. Mark source position for visual feedback
4. Show status: "Cut: KC_A - press p to paste, Esc to cancel"
5. Source key renders dimmed until paste/cancel
6. Source NOT cleared until paste (allows cancel)

### Paste (`p` / Ctrl+V)
1. If clipboard empty: show "Nothing in clipboard"
2. Apply clipboard data to selected position
3. If was cut: clear source to KC_TRNS, remove cut visual
4. Show status: "Pasted: KC_A"
5. Mark layout as dirty

### Cancel (Esc)
1. If in cut state: clear cut visual, keep clipboard content
2. Cut source is preserved (not cleared)

### Cross-Layer Paste
- Paste works regardless of current layer
- Cut source clears on original layer when pasted

## Visual Feedback

### Status Bar Messages
```
[Copy]   "Copied: KC_A"
[Copy]   "Copied: LT(1,SPC) (with color)"
[Cut]    "Cut: KC_A - press p to paste, Esc to cancel"
[Paste]  "Pasted: KC_A"
[Empty]  "Nothing in clipboard"
```

### Cut Source Rendering
- Apply `Modifier::DIM` to cut source key
- Optionally use dashed border characters: `┄` `┆`
- Clear visual on paste or Esc

## Data Structures

### ClipboardContent
```rust
pub struct ClipboardContent {
    pub keycode: String,
    pub color_override: Option<RgbColor>,
    pub category_id: Option<String>,
}
```

### KeyClipboard
```rust
pub struct KeyClipboard {
    /// Stored key data
    content: Option<ClipboardContent>,
    /// Cut source for visual feedback (layer_index, position)
    cut_source: Option<(usize, Position)>,
}
```

## Files to Modify

1. **New: `src/tui/clipboard.rs`** - Clipboard state and operations
2. **`src/tui/mod.rs`** - Add clipboard to AppState, handle keybindings
3. **`src/tui/keyboard.rs`** - Render cut source with dim effect
4. **`src/tui/help_overlay.rs`** - Document new keybindings

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| Copy empty position | Copy KC_TRNS (or show "No key to copy") |
| Paste to same position | Overwrites (no-op if same data) |
| Cut then navigate away | Cut visual persists on source |
| Cut then copy | New copy replaces clipboard, clears cut state |
| Paste after layer delete | Paste still works (data in clipboard) |
| Multiple pastes | Same data pastes multiple times |

## Testing

- Copy key, verify clipboard content
- Paste key, verify target updated
- Cut key, verify source dims
- Paste cut key, verify source cleared
- Cross-layer paste
- Cancel cut, verify source preserved
- Copy over existing cut

## Success Criteria

- [ ] `y` and Ctrl+C copy selected key
- [ ] `d` and Ctrl+X cut selected key
- [ ] `p` and Ctrl+V paste at selection
- [ ] Cut source shows dimmed
- [ ] Esc cancels cut visual
- [ ] Cross-layer paste works
- [ ] Status messages display correctly
- [ ] Help overlay updated
