# Tasks: Key Copy/Cut/Paste Clipboard

## Phase 1: Simple Single-Key Clipboard

### Setup
- [x] Create spec document
- [ ] Create feature branch
- [ ] Commit spec

### Implementation
- [ ] Create `src/tui/clipboard.rs` module
  - [ ] Define `ClipboardContent` struct
  - [ ] Define `KeyClipboard` struct
  - [ ] Implement `copy()` method
  - [ ] Implement `cut()` method
  - [ ] Implement `paste()` method
  - [ ] Implement `cancel_cut()` method
  - [ ] Implement `is_cut_source()` for visual feedback

- [ ] Integrate into AppState (`src/tui/mod.rs`)
  - [ ] Add `clipboard: KeyClipboard` field
  - [ ] Initialize in `new()`

- [ ] Add keybindings (`src/tui/mod.rs`)
  - [ ] `y` - copy key
  - [ ] `d` - cut key  
  - [ ] `p` - paste key
  - [ ] Ctrl+C - copy key
  - [ ] Ctrl+X - cut key
  - [ ] Ctrl+V - paste key
  - [ ] Esc - cancel cut (extend existing handler)

- [ ] Visual feedback for cut source (`src/tui/keyboard.rs`)
  - [ ] Pass clipboard state to render
  - [ ] Apply DIM modifier to cut source key
  - [ ] Clear on paste/cancel

- [ ] Update help overlay (`src/tui/help_overlay.rs`)
  - [ ] Add clipboard section
  - [ ] Document y/d/p keybindings
  - [ ] Document Ctrl+C/X/V alternatives

### Testing
- [ ] Manual test: copy and paste within layer
- [ ] Manual test: cut and paste within layer
- [ ] Manual test: cross-layer paste
- [ ] Manual test: cancel cut with Esc
- [ ] Manual test: verify status messages
- [ ] Verify build passes
- [ ] Verify clippy passes

### Completion
- [ ] Commit implementation
- [ ] Merge to main
