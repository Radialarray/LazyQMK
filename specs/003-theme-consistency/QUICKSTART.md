# Theme Consistency - Quick Reference

**Branch:** `fix/theme-consistency`  
**Status:** Planning Complete  
**Estimated Time:** ~38 hours  
**Files to Modify:** 17 (2 new, 15 modified)

---

## Quick Start

1. Read `spec.md` for full feature specification
2. Follow `tasks.md` for detailed implementation steps
3. Start with Phase 1 (Foundation) - 4 tasks, ~6 hours

---

## The Problem

- ❌ No centralized theme system
- ❌ 115+ hardcoded colors across 15+ files
- ❌ Help menu alone has 40+ hardcoded colors
- ❌ Inconsistent highlight styles (DarkGray vs Cyan vs Yellow)
- ❌ No light mode support
- ❌ `config.ui.theme` exists but is unused

---

## The Solution

Create centralized `Theme` struct with semantic colors:

```rust
pub struct Theme {
    pub primary: Color,        // Borders, titles
    pub accent: Color,         // Highlights, selections
    pub success: Color,        // Success states
    pub error: Color,          // Error states
    pub warning: Color,        // Warning states
    pub text: Color,           // Primary text
    pub text_secondary: Color, // Secondary text
    pub text_muted: Color,     // Muted text
    pub background: Color,     // Main background
    pub highlight_bg: Color,   // Selection background
    pub surface: Color,        // Panels
    pub active: Color,         // Active elements
    pub inactive: Color,       // Inactive elements
}
```

---

## Implementation Phases

### ✅ Planning (Complete)
- Research completed
- Spec written
- Tasks defined

### 🔄 Phase 1: Foundation (Next)
- [ ] Task 1.1: Create `src/tui/theme.rs`
- [ ] Task 1.2: Update `src/config.rs` validation
- [ ] Task 1.3: Add theme to `AppState`
- [ ] Task 1.4: Add theme getter method

### 📋 Phase 2: High Priority Components
- [ ] Task 2.1: Refactor help overlay (40+ colors!)
- [ ] Task 2.2: Refactor status bar
- [ ] Task 2.3: Refactor main UI title bar

### 📋 Phase 3: Pickers & Editors
- [ ] Task 3.1: Keycode picker
- [ ] Task 3.2: Color picker
- [ ] Task 3.3: Category picker
- [ ] Task 3.4: Category manager
- [ ] Task 3.5: Metadata editor

### 📋 Phase 4: Complex Components
- [ ] Task 4.1: Keyboard widget
- [ ] Task 4.2: Template browser
- [ ] Task 4.3: Build log

### 📋 Phase 5: Runtime Features
- [ ] Task 5.1: Add Ctrl+T theme toggle
- [ ] Task 5.2: Persist theme on change
- [ ] Task 5.3: Update help with Ctrl+T docs

### 📋 Phase 6: Documentation & Testing
- [ ] Task 6.1: Update README
- [ ] Task 6.2: Add theme tests
- [ ] Task 6.3: Manual testing checklist

---

## Color Mappings

| Current Hardcoded | Theme Field |
|-------------------|-------------|
| `Color::Cyan` | `theme.primary` |
| `Color::Yellow` | `theme.accent` |
| `Color::Green` | `theme.success` |
| `Color::Red` | `theme.error` |
| `Color::White` | `theme.text` |
| `Color::Gray` | `theme.text_secondary` |
| `Color::DarkGray` | `theme.text_muted` |
| `Color::Black` (bg) | `theme.background` |
| `Color::DarkGray` (bg) | `theme.highlight_bg` |

---

## Theme Variants

| Color | Dark Theme | Light Theme |
|-------|------------|-------------|
| primary | Cyan | Blue |
| accent | Yellow | Dark Orange |
| success | Green | Dark Green |
| error | Red | Red |
| text | White | Black |
| text_muted | DarkGray | Gray |
| background | Black | White |
| highlight_bg | DarkGray | Light Gray (230) |

---

## Files to Create

1. `src/tui/theme.rs` - Theme struct and variants
2. `tests/theme_tests.rs` - Unit tests

---

## Files to Modify

1. `src/config.rs` - Theme validation
2. `src/tui/mod.rs` - Add theme to AppState
3. `src/tui/help_overlay.rs` - 40+ colors
4. `src/tui/status_bar.rs` - 8 colors
5. `src/tui/keyboard.rs` - 5 colors
6. `src/tui/keycode_picker.rs` - 4 colors
7. `src/tui/color_picker.rs` - 10 colors
8. `src/tui/category_picker.rs` - 6 colors
9. `src/tui/category_manager.rs` - 12 colors
10. `src/tui/template_browser.rs` - 12 colors
11. `src/tui/build_log.rs` - 6 colors
12. `src/tui/metadata_editor.rs` - 10 colors
13. `src/tui/layout_picker.rs` - Unknown
14. `src/tui/onboarding_wizard.rs` - Unknown
15. `src/tui/config_dialogs.rs` - Unknown

---

## Testing Checklist

### Both Themes
- [ ] Help overlay (?)
- [ ] Status bar
- [ ] Keyboard widget
- [ ] Keycode picker (k)
- [ ] Color picker (c)
- [ ] Category picker
- [ ] Category manager (m)
- [ ] Template browser (t)
- [ ] Metadata editor (e)
- [ ] Build log (l)
- [ ] Config dialogs

### Theme Switching
- [ ] Ctrl+T toggles dark→light
- [ ] Ctrl+T toggles light→dark
- [ ] UI updates immediately
- [ ] Config saved
- [ ] Persists after restart

---

## Key Decisions

1. **Default theme:** Dark (no breaking change)
2. **Toggle keybinding:** Ctrl+T
3. **Config field:** `ui.theme = "dark"` or `"light"`
4. **Key colors:** Not themed (user-defined semantic colors)
5. **Fallback:** Invalid theme → dark theme

---

## Success Metrics

- ✅ Zero hardcoded `Color::` in UI components
- ✅ Both themes pass WCAG AA contrast (4.5:1)
- ✅ Help menu consistent with main UI
- ✅ All existing tests pass
- ✅ Manual testing checklist complete

---

## Next Steps

1. **Start Phase 1, Task 1.1:** Create `src/tui/theme.rs`
   - Duration: ~2-3 hours
   - Creates foundation for all other work

2. **After Phase 1 complete:** Begin Phase 2, Task 2.1 (help overlay)
   - Highest user impact
   - Good test case for theme system

3. **Continuous:** Run manual tests after each component refactor

---

## Commands

```bash
# Start development
git checkout fix/theme-consistency

# Create theme module
touch src/tui/theme.rs

# Run tests frequently
cargo test

# Check for remaining hardcoded colors
rg "Color::" src/tui/ --type rust

# Manual testing
cargo run
```

---

**Last Updated:** 2025-11-25  
**Next Task:** Phase 1, Task 1.1 - Create theme.rs
