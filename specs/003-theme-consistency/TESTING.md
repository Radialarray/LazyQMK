# Theme System Testing Guide

**Branch:** `fix/theme-consistency`  
**Phase:** 1 & 2 Complete  
**Date:** 2025-11-25

---

## What Has Been Implemented

### ✅ Phase 1: Core Infrastructure
- `Theme` struct with 13 semantic color roles
- Dark theme (default) - maintains existing appearance
- Light theme - optimized for light terminal backgrounds
- Theme loaded from `config.ui.theme` setting
- All 119 tests passing

### ✅ Phase 2: High Priority Components
- **Help Overlay** - 40+ colors themed
- **Status Bar** - build states, errors, help text
- **Title Bar** - layout name and dirty indicator
- **Template Save Dialog** - field highlighting
- **Unsaved Changes Prompt** - warning styling

---

## Manual Testing Checklist

### Test 1: Basic App Launch (Dark Theme - Default)
```bash
cargo run --release
```

**Expected:**
- App launches successfully
- Title bar shows layout name in Cyan
- Status bar shows help text in Cyan
- All UI elements visible and readable
- No crashes or errors

**Verify:**
- [ ] App launches without errors
- [ ] Title bar is Cyan (dark theme primary color)
- [ ] Status bar help text is Cyan
- [ ] No visual glitches

---

### Test 2: Help Overlay (Dark Theme)
**Action:** Press `?` to open help overlay

**Expected:**
- Help dialog opens centered on screen
- Header border and title in Cyan
- Section headers (NAVIGATION, EDITING, etc.) in Yellow
- Keybindings in Green
- Footer text in Dark Gray
- Black background
- Scrollbar in Cyan

**Verify:**
- [ ] Help overlay opens correctly
- [ ] All text is readable
- [ ] Colors are consistent (no hardcoded colors visible)
- [ ] Section headers stand out (Yellow)
- [ ] Keybindings are highlighted (Green)
- [ ] Border is Cyan
- [ ] Can scroll with arrow keys
- [ ] Press `Esc` to close

---

### Test 3: Status Bar States (Dark Theme)
**Action:** Navigate around the app and observe status bar

**Expected Status Colors:**
- Normal message: White text
- Error message: "ERROR:" prefix in Red
- Build idle: Gray
- Build in progress: Yellow
- Build success: Green
- Build failed: Red
- Help text label: Cyan

**Verify:**
- [ ] Status messages display correctly
- [ ] Different states use appropriate colors
- [ ] Help text is always visible in Cyan

---

### Test 4: Light Theme Configuration
**Action:** Test light theme by modifying config

```bash
# Find your config file
# macOS/Linux: ~/.config/layout_tools/config.toml
# Windows: %APPDATA%\layout_tools\config.toml

# Edit the file and change:
[ui]
theme = "light"
```

Then restart the app:
```bash
cargo run --release
```

**Expected:**
- Title bar shows layout name in Blue (light theme primary)
- Status bar uses darker colors
- All text readable on light background
- Help overlay uses Blue borders
- Section headers in Dark Orange
- Keybindings in Dark Green

**Verify:**
- [ ] App loads with light theme
- [ ] Title bar is Blue (not Cyan)
- [ ] All text is readable (dark colors on light background)
- [ ] Press `?` - help overlay uses light theme
- [ ] No bright colors that are hard to see on light background

---

### Test 5: Theme Validation
**Action:** Test invalid theme value

Edit config:
```toml
[ui]
theme = "invalid_theme_name"
```

**Expected:**
- App should show validation error
- Error message should mention theme must be "dark" or "light"

**Verify:**
- [ ] Invalid theme is rejected
- [ ] Clear error message shown

---

### Test 6: Template Save Dialog (Dark Theme)
**Action:** 
1. Make a change to a layout
2. Press `Ctrl+Shift+T` (or appropriate key to save as template)

**Expected:**
- Dialog title in Cyan with BOLD
- Active field highlighted in Yellow
- Inactive fields in White
- Help text in Gray
- Action buttons in Green

**Verify:**
- [ ] Dialog opens correctly
- [ ] Tab between fields - active field is Yellow
- [ ] Inactive fields are White (not Yellow)
- [ ] Help text is dimmed (Gray)
- [ ] Action text is Green
- [ ] Press `Esc` to cancel

---

### Test 7: Unsaved Changes Prompt (Dark Theme)
**Action:**
1. Make a change to layout
2. Press `Ctrl+Q` to quit

**Expected:**
- Prompt dialog appears
- Border/title in Yellow (warning color)
- Options listed clearly
- Text is readable

**Verify:**
- [ ] Prompt appears on quit with unsaved changes
- [ ] Dialog uses Yellow for warning state
- [ ] Options are clear
- [ ] Press `Esc` to cancel

---

## Known Status

### ✅ Fully Themed (Phase 1 & 2)
- Core theme system
- Help overlay
- Status bar
- Title bar
- Template save dialog
- Unsaved changes prompt

### ⏳ Not Yet Themed (Phase 3+)
These components still use hardcoded colors:
- Keyboard widget (key selection)
- Keycode picker
- Color picker
- Category picker & manager
- Template browser
- Build log
- Metadata editor
- Layout picker
- Onboarding wizard

**Note:** These components will continue to work with hardcoded colors until Phase 3+ implementation.

---

## Testing Results

### Build Status
```
✅ Compiles successfully (release mode)
✅ Binary size: 3.0M
✅ Zero compilation errors
⚠️  6 warnings (expected - unused functions for Phase 5)
```

### Test Suite Status
```
✅ 129 total tests passing
✅ Unit tests: 104 passed
✅ Integration tests: 10 passed
✅ QMK tests: 5 passed
✅ Doc tests: 10 passed
```

---

## Troubleshooting

### Issue: App won't start
**Solution:** Check config file for valid theme value ("dark" or "light")

### Issue: Colors look wrong
**Check:**
- Terminal supports colors (most modern terminals do)
- Config file has correct theme setting
- Try both dark and light themes

### Issue: Help overlay not opening
**Check:**
- Press `?` key (Shift + /)
- Check if another popup is active (press Esc first)

---

## What to Look For

### Good Signs ✅
- All UI elements are readable
- Colors are consistent within each theme
- Section headers stand out appropriately
- Status indicators use semantic colors (Green=success, Red=error)
- No jarring color mismatches

### Potential Issues ⚠️
- If you see bright Yellow on white background (light theme issue)
- If text is unreadable (contrast issue)
- If some parts use different colors than others (missed refactoring)
- If the app crashes on startup (validation issue)

---

## Next Steps After Testing

If testing reveals issues:
1. Note which component has the issue
2. Check if it's a Phase 3+ component (expected to have hardcoded colors)
3. For Phase 1-2 components, report the specific color/component

If testing is successful:
1. ✅ Phase 1 & 2 confirmed working
2. 🚀 Ready to proceed with Phase 3 (Pickers & Editors)
3. 📝 Consider creating before/after screenshots

---

## Quick Test Commands

```bash
# Run app with default (dark) theme
cargo run --release

# Run tests
cargo test

# Build release
cargo build --release

# Check for hardcoded colors (should only be in unthemed components)
rg "Color::" src/tui/*.rs --type rust
```

---

## Summary

**Ready for Testing:**
- ✅ Dark theme (default)
- ✅ Light theme (via config)
- ✅ Help overlay (40+ colors themed)
- ✅ Status bar (all states)
- ✅ Title bar
- ✅ Dialogs (template save, unsaved changes)

**Status:** All Phase 1 & 2 components ready for manual testing. No automated visual tests yet - manual verification required.
