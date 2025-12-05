# Keyboard Configurator - Shortcut System Analysis & Refactoring Plan

## Executive Summary

The current system has **fragmented shortcut definitions** spread across:
1. `src/data/help.toml` - Help text and status bar hints
2. `src/tui/mod.rs` - Actual keyboard event handling (154 KeyCode references)
3. Various component files - Picker dialogs, managers, etc.

**Problem**: When changing a shortcut, you must update multiple locations, risking inconsistency.

**Solution**: Create a centralized shortcut registry with a structured, hierarchical naming system.

---

## Current State Analysis

### 1. Shortcut Distribution

#### Main View (Primary Context)
**File Operations (Ctrl+)**
- `Ctrl+S` - Save layout
- `Ctrl+Q` - Quit
- `Ctrl+Z` - Undo paste
- `Ctrl+G` - Generate firmware
- `Ctrl+B` - Build firmware
- `Ctrl+L` - View build log
- `Ctrl+W` - Setup wizard
- `Ctrl+Y` - Switch layout variant
- `Ctrl+T` - Category manager

**Shift Modifiers**
- `Shift+E` - Edit metadata

**Standard Copy/Paste**
- `Ctrl+C` / `y` - Copy
- `Ctrl+X` / `d` - Cut
- `Ctrl+V` / `p` - Paste

**Navigation (No modifiers)**
- `↑↓←→` / `hjkl` - Navigate keys
- `Tab` / `Shift+Tab` - Next/previous layer
- `Home` / `End` - Jump to first/last key

**Editing (Single keys)**
- `Enter` - Open keycode picker
- `x` / `Delete` - Clear key
- `c` - Set individual key color
- `v` - Toggle layer colors
- `t` - Browse templates

**Shift Modifiers**
- `Shift+C` - Set layer default color
- `Shift+K` - Assign category to key
- `Shift+L` - Assign category to layer
- `Shift+N` - Layer manager
- `Shift+S` - Settings
- `Shift+T` - Save as template
- `Shift+V` - Selection mode
- `Shift+R` - Rectangle select

**Alt Modifiers**
- `Alt+V` - Toggle all layer colors

**Dialogs/Managers**
- `?` - Toggle help

### 2. Shortcut Conflicts & Issues

#### Current Conflicts:
1. ❌ **`Ctrl+C` collision**: Copy key vs. Cancel (in some dialogs)
2. ❌ **`Ctrl+L` collision**: View build log vs. Vim 'l' navigation
3. ⚠️ **Vim keys**: h/j/k/l cause issues in text input contexts
4. ⚠️ **Inconsistent patterns**: Some use Shift+Letter, others use Ctrl+Letter

#### Mnemonic Issues:
- `Shift+K` = Assign category to **k**ey (good)
- `Shift+L` = Assign category to **l**ayer (good)
- `Shift+N` = Layer manager (why not 'L' for **l**ayers?)
- `Ctrl+T` = Category manager (**t**ags? not obvious)
- `Shift+S` = Settings (good)
- `Shift+R` = **R**ectangle select (good)

### 3. Current Hierarchical Structure

```
Main View
├── File Operations (Ctrl+)
│   ├── Ctrl+S (Save)
│   ├── Ctrl+Q (Quit)
│   └── Ctrl+W (Wizard)
├── Edit Operations
│   ├── Copy/Cut/Paste (Ctrl+ or Vim)
│   ├── Enter (Keycode picker)
│   └── x/Delete (Clear)
├── View/Navigation
│   ├── Arrows / hjkl (Navigate)
│   ├── Tab (Layers)
│   └── ? (Help)
├── Color Operations
│   ├── c (Individual)
│   ├── Shift+C (Layer)
│   └── v (Toggle)
├── Manager Dialogs (Mixed)
│   ├── Shift+N (Layers)
│   ├── Ctrl+T (Categories)
│   └── Shift+S (Settings)
└── Special
    ├── t (Templates browse)
    ├── Shift+T (Template save)
    └── Shift+V (Selection)
```

---

## Industry Best Practices Research

### 1. Common TUI Application Patterns

#### Vim/Neovim
- **Modal editing**: Different keys in different modes
- **Leader key**: Space or comma for extended commands
- **Single letters**: No modifiers for common actions
- **`:` commands**: For infrequent operations

#### Emacs
- **Ctrl prefix**: `Ctrl+x` then letter for file operations
- **Meta prefix**: `Alt` for extended commands
- **Consistent patterns**: `Ctrl+x Ctrl+s` = save

#### Modern TUI Apps (lazygit, k9s, bottom)
- **Single keys**: For most common actions
- **Shift**: For destructive or important actions
- **Ctrl**: For app-level operations
- **`?`**: Universal help
- **`q`**: Universal quit
- **Numbers**: Quick navigation/selection

### 2. Recommended Hierarchy

```
Level 1: Single Keys (Frequent, Safe)
  - Navigation (arrows, hjkl, tab)
  - Enter (select/edit)
  - Esc (cancel)
  - ? (help)

Level 2: Shift+Key (Important, Less Frequent)
  - Shift+C (Color)
  - Shift+L (Layers manager)
  - Shift+K (Categories manager)
  - Shift+S (Settings)
  - Shift+T (Template)

Level 3: Ctrl+Key (App-Level, File Operations)
  - Ctrl+S (Save)
  - Ctrl+Q (Quit)
  - Ctrl+B (Build)
  - Ctrl+C/X/V (Copy/Cut/Paste - standard)
  - Ctrl+Z (Undo)
  - Ctrl+W (Wizard)

Level 4: Alt+Key (Global Toggles, Rare Actions)
  - Alt+V (Toggle all colors)
```

---

## Proposed Refactoring Plan

### Phase 1: Create Centralized Shortcut Registry

**New File**: `src/shortcuts.rs`

```rust
pub struct ShortcutRegistry {
    bindings: HashMap<Context, Vec<Shortcut>>,
}

pub struct Shortcut {
    pub keys: Vec<KeyBinding>,
    pub action: Action,
    pub description: String,
    pub hint: Option<String>,
    pub priority: u32,
}

pub enum Action {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    
    // File operations
    Save,
    Quit,
    
    // Editing
    OpenKeycodePicker,
    ClearKey,
    CopyKey,
    CutKey,
    PasteKey,
    
    // Colors
    SetKeyColor,
    SetLayerColor,
    ToggleLayerColors,
    
    // Managers
    OpenLayerManager,
    OpenCategoryManager,
    OpenSettings,
    
    // ... etc
}

pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: KeyModifiers,
}
```

### Phase 2: Unified Shortcut Definition

**Enhanced `help.toml`** with action IDs:

```toml
[[contexts.main.bindings]]
action_id = "save"
keys = ["Ctrl+S"]
action = "Save layout"
hint = "Save"
priority = 7
```

### Phase 3: Shortcut Reorganization

#### Proposed New Scheme

**Single Keys (Context-Aware)**
- `e` → **E**dit keycode (instead of Enter)
- `c` → Set **c**olor (individual key, context-aware)
- `v` → **V**iew/toggle colors
- `t` → **T**emplates
- `x` → Delete/clear
- `?` → Help
- `/` → Search (in lists/pickers)

**Shift+Key (Managers & Important Actions)**
- `Shift+E` → **E**dit metadata
- `Shift+L` → **L**ayers manager (changed from Shift+N)
- `Shift+K` → **K**ategories manager (changed from Ctrl+T)
- `Shift+C` → **C**olor layer default (flipped with 'c')
- `Shift+S` → **S**ettings
- `Shift+T` → **T**emplate save
- `Shift+V` → **V**isual selection mode
- `Shift+R` → **R**ectangle select
- `Shift+B` → **B**uild log

**Ctrl+Key (File & App Operations)**
- `Ctrl+S` → **S**ave
- `Ctrl+Q` → **Q**uit
- `Ctrl+N` → **N**ew layer
- `Ctrl+W` → **W**izard
- `Ctrl+B` → **B**uild firmware
- `Ctrl+G` → **G**enerate firmware
- `Ctrl+C/X/V/Z` → Standard copy/cut/paste/undo

**Alt+Key (Global Toggles)**
- `Alt+V` → Toggle all layer colors
- `Alt+C` → Color picker mode toggle (palette/RGB)

### Phase 4: Migration Strategy

1. ✅ Create `shortcuts.rs` with Action enum and registry
2. ✅ Add `action_id` field to `help.toml`
3. ✅ Create mapping from KeyCode+Modifiers → Action
4. ✅ Update main event loop to use Action enum
5. ✅ Update all component event handlers
6. ✅ Add shortcut validation (detect conflicts)
7. ✅ Add runtime shortcut customization (future)

---

## Detailed Shortcut Mapping (Proposed)

### Main View - Comprehensive

| Key | Action | Priority | Notes |
|-----|--------|----------|-------|
| **NAVIGATION** |
| ↑↓←→ / hjkl | Navigate keys | 1 | Core |
| Tab | Next layer | 2 | Core |
| Shift+Tab | Previous layer | - | |
| Home/End | First/last key | - | |
| PgUp/PgDn | Scroll (in lists) | - | |
| **EDITING** |
| e / Enter | Edit keycode | 3 | Changed from Enter only |
| x / Delete | Clear key | - | |
| c | Set key color | 4 | Individual key (changed) |
| Shift+C | Set layer color | 5 | Layer default (changed) |
| **CLIPBOARD** |
| y / Ctrl+C | Copy | 6 | Vim + Standard |
| d / Ctrl+X | Cut | - | Vim + Standard |
| p / Ctrl+V | Paste | - | Vim + Standard |
| Ctrl+Z | Undo | - | Standard |
| **MANAGERS** |
| Shift+L | Layers manager | 7 | Changed from Shift+N |
| Shift+K | Categories manager | 8 | Changed from Ctrl+T |
| Shift+S | Settings | 9 | |
| Shift+E | Edit metadata | - | Changed from Ctrl+E |
| **FILE & BUILD** |
| Ctrl+S | Save | 10 | |
| Ctrl+B | Build firmware | 11 | |
| Shift+B | Build log | 12 | Changed from Ctrl+L |
| Ctrl+G | Generate | - | |
| Ctrl+W | Wizard | - | |
| Ctrl+Q | Quit | - | |
| **TEMPLATES** |
| t | Browse templates | 13 | |
| Shift+T | Save template | - | |
| **VISUAL/SELECTION** |
| Shift+V | Selection mode | - | |
| Shift+R | Rectangle select | - | |
| **COLORS** |
| v | Toggle layer colors | 14 | |
| Alt+V | Toggle all colors | - | |
| **HELP** |
| ? | Toggle help | 100 | |
| Esc | Cancel/Close | 101 | |

---

## Benefits of Refactoring

### 1. Consistency
- ✅ All shortcuts defined in one place
- ✅ Mnemonic patterns (c=color, l=layers, k=categories)
- ✅ No more Shift+N for layers (Shift+L makes sense)

### 2. Discoverability
- ✅ Help system auto-updates from central registry
- ✅ Status bar hints auto-update
- ✅ No documentation drift

### 3. Maintainability
- ✅ Change a shortcut once, updates everywhere
- ✅ Detect conflicts at compile time
- ✅ Easy to add new shortcuts

### 4. Future Features
- ✅ User-customizable shortcuts
- ✅ Export/import shortcut schemes
- ✅ Context-sensitive shortcuts
- ✅ Shortcut chords (like Emacs `Ctrl+x Ctrl+s`)

---

## Implementation Phases

### Phase 1: Foundation (Current)
- ✅ `help.toml` exists with all contexts
- ✅ `HelpRegistry` loads and serves help text
- ❌ No action enum
- ❌ No centralized event dispatch

### Phase 2: Centralization (Proposed)
- ✨ Create `Action` enum for all possible actions
- ✨ Create `ShortcutRegistry` to map keys → actions
- ✨ Add `action_id` to `help.toml` bindings
- ✨ Generate registry from TOML at build time

### Phase 3: Refactor Event Loop (Proposed)
- ✨ Main event loop: `KeyEvent` → `Action`
- ✨ Match on `Action` enum instead of raw `KeyCode`
- ✨ Each component takes `Action` and decides how to handle

### Phase 4: Reorganize Shortcuts (Proposed)
- ✨ Apply new mnemonic scheme
- ✨ Update `help.toml`
- ✨ Update event handlers
- ✨ Update documentation

### Phase 5: Validation & Testing (Proposed)
- ✨ Build-time conflict detection
- ✨ Runtime shortcut display
- ✨ Integration tests for all shortcuts

### Phase 6: Customization (Future)
- 🔮 User config file for overrides
- 🔮 Shortcut editor UI
- 🔮 Export/import schemes

---

## Questions for User

1. **Shortcut Changes**: Do you approve the proposed changes?
   - `e` for edit keycode (instead of Enter)
   - `Shift+L` for Layers (instead of Shift+N)
   - `Shift+K` for Categories (instead of Ctrl+T)
   - `Shift+B` for Build log (instead of Ctrl+L)
   - Flip `c` and `Shift+C` (individual vs layer color)

2. **Scope**: Should we implement all phases, or start with Phase 2 only?

3. **Breaking Changes**: The shortcut reorganization is a breaking change for muscle memory. Ship as v0.4.0?

4. **Vim Keys**: Keep h/j/k/l in main view only (removed from pickers)?

5. **Alternative**: Keep current shortcuts, just centralize implementation?

---

## Recommendation

**Start with Phase 2 (Centralization) WITHOUT changing shortcuts.**

This gives us:
- ✅ Single source of truth
- ✅ No breaking changes for users
- ✅ Foundation for future improvements
- ✅ Easy conflict detection
- ✅ Can change shortcuts later in v0.4.0

Then propose shortcut reorganization to user for v0.4.0.
