# Feature Specification: Theme Consistency System

**Feature ID:** 003-theme-consistency  
**Status:** Planning  
**Created:** 2025-11-25  
**Branch:** `fix/theme-consistency`

---

## Problem Statement

The TUI currently has no centralized theme management system, resulting in:
- 40+ hardcoded colors in the help menu alone
- Inconsistent highlight styles across components (DarkGray vs Cyan vs Yellow backgrounds)
- No support for light terminal backgrounds (all colors assume dark mode)
- Colors scattered across 15+ component files making global changes impossible
- The `config.ui.theme` field exists but is completely unused

### User Impact
Users with light-themed terminals cannot read the interface properly, and there's no way to customize the color scheme. The help menu mentioned specifically has jarring inconsistencies.

---

## Solution Overview

Implement a centralized theme system with:
1. **Core `Theme` struct** with semantic color roles (primary, accent, success, error, etc.)
2. **Dark and light variants** that are readable on appropriate terminal backgrounds
3. **Component refactoring** to use theme colors instead of hardcoded values
4. **Runtime theme switching** via Ctrl+T keyboard shortcut
5. **Persistent theme preference** saved to config file

---

## Architecture

### Theme Structure

```rust
pub struct Theme {
    // Primary UI colors
    pub primary: Color,        // Borders, titles, emphasis
    pub accent: Color,         // Highlights, selections
    pub success: Color,        // Success states
    pub error: Color,          // Error states
    pub warning: Color,        // Warning states
    
    // Text hierarchy
    pub text: Color,           // Primary text
    pub text_secondary: Color, // Secondary text
    pub text_muted: Color,     // Muted/help text
    
    // Backgrounds
    pub background: Color,     // Main background
    pub highlight_bg: Color,   // Selection background
    pub surface: Color,        // Panel backgrounds
    
    // States
    pub active: Color,         // Active elements
    pub inactive: Color,       // Inactive elements
}
```

### Theme Variants

| Color Role | Dark Theme | Light Theme |
|------------|------------|-------------|
| primary | Cyan | Blue |
| accent | Yellow | Dark Orange |
| success | Green | Dark Green |
| error | Red | Red |
| text | White | Black |
| text_muted | DarkGray | Gray |
| background | Black | White |
| highlight_bg | DarkGray | Light Gray |

### Integration Points

1. **Config System** (`src/config.rs`)
   - Validate `theme` field values ("dark" or "light")
   - Remove "future feature" comment
   
2. **AppState** (`src/tui/mod.rs`)
   - Add `theme: Theme` field
   - Initialize from config on startup
   - Pass to all render functions

3. **Components** (15 files)
   - Update render signatures to accept `&Theme`
   - Replace all `Color::X` with `theme.x`
   - Maintain semantic meaning (e.g., RGB for color picker channels)

---

## User Stories

### US1: Dark Theme (Default)
**As a user with a dark terminal,**  
**I want** the TUI to use colors optimized for dark backgrounds,  
**So that** all text and UI elements are clearly readable.

**Acceptance Criteria:**
- Default theme is "dark"
- All components use light text on dark backgrounds
- Borders and highlights use bright colors (Cyan, Yellow)
- Status indicators use semantic colors (Green=success, Red=error)

### US2: Light Theme Support
**As a user with a light terminal,**  
**I want** the TUI to use colors optimized for light backgrounds,  
**So that** all text and UI elements are clearly readable without eye strain.

**Acceptance Criteria:**
- Light theme available via config or toggle
- All components use dark text on light backgrounds
- Borders and highlights use darker colors
- All color contrasts meet WCAG AA standards (4.5:1 minimum)

### US3: Runtime Theme Switching
**As a user,**  
**I want** to switch between dark and light themes without restarting,  
**So that** I can adapt to changing lighting conditions quickly.

**Acceptance Criteria:**
- Ctrl+T toggles between dark and light themes
- UI updates immediately (no restart required)
- Current theme shown in status bar or help menu
- Theme preference saved automatically

### US4: Persistent Theme Preference
**As a user,**  
**I want** my theme choice to persist across sessions,  
**So that** I don't have to reconfigure it every time I launch the app.

**Acceptance Criteria:**
- Theme choice saved to config file
- Theme loaded on next app startup
- Config file validates theme values

### US5: Consistent Help Menu
**As a user,**  
**I want** the help menu to use consistent colors with the rest of the UI,  
**So that** it feels like a cohesive part of the application.

**Acceptance Criteria:**
- Help menu uses theme colors (not hardcoded)
- Borders match main UI borders
- Section headers consistent with other dialogs
- Keybindings use success color (like other actions)

---

## Technical Design

### File Structure
```
src/
  tui/
    theme.rs        # NEW: Theme struct and variants
    mod.rs          # MODIFIED: Add theme to AppState
    help_overlay.rs # MODIFIED: Use theme colors
    status_bar.rs   # MODIFIED: Use theme colors
    keyboard.rs     # MODIFIED: Use theme colors
    [... 12 more component files to modify]
  config.rs         # MODIFIED: Validate theme field
```

### Color Mapping Strategy

**Current → Theme Field:**
- `Color::Cyan` → `theme.primary` (most borders, titles)
- `Color::Yellow` → `theme.accent` (highlights, warnings)
- `Color::Green` → `theme.success` (confirmations, success states)
- `Color::Red` → `theme.error` (errors, destructive actions)
- `Color::White` → `theme.text` (primary text)
- `Color::Gray` → `theme.text_secondary` (labels)
- `Color::DarkGray` → `theme.text_muted` (help text, dim)
- `Color::Black` → `theme.background` (explicit backgrounds)
- `Color::DarkGray` (bg) → `theme.highlight_bg` (selection bg)

**Exceptions (Keep As-Is):**
- RGB colors from layout data (user-defined key colors)
- Red/Green/Blue labels in color picker (semantic meaning)

---

## Implementation Phases

### Phase 1: Foundation (6 hours)
- Create `Theme` struct with dark/light variants
- Update config validation
- Add theme to `AppState`
- Pass theme to render functions

### Phase 2: High Priority Components (5 hours)
- Help overlay (biggest impact - 40+ colors)
- Status bar (always visible)
- Main UI title bar

### Phase 3: Pickers & Editors (9 hours)
- Keycode picker
- Color picker
- Category picker & manager
- Metadata editor

### Phase 4: Complex Components (6 hours)
- Keyboard widget (selection contrast)
- Template browser
- Build log

### Phase 5: Runtime Features (4 hours)
- Ctrl+T theme toggle keybinding
- Auto-save theme preference
- Status feedback on toggle

### Phase 6: Documentation & Testing (8 hours)
- Update README with theme docs
- Add theme unit tests
- Manual testing checklist (both themes)
- Verify no hardcoded colors remain

**Total Estimate:** ~38 hours

---

## Testing Strategy

### Unit Tests
- `Theme::dark()` returns expected colors
- `Theme::light()` returns expected colors
- `Theme::from_name()` parses config correctly
- Invalid theme names fall back to dark
- Config validation rejects invalid themes

### Integration Tests
- AppState initializes with correct theme from config
- Theme toggle updates all components
- Theme persists after save and reload

### Manual Testing
- **Visual verification** of all 15+ components in both themes
- **Contrast testing** for readability (WCAG standards)
- **Edge cases:** invalid configs, missing config, theme toggle spam
- **Regression:** Ensure existing functionality unchanged

### Accessibility Checks
- Dark theme: Light text (white/cyan) on dark background (black) = ∞ contrast
- Light theme: Dark text (black/blue) on light background (white) = 15-21:1 contrast
- Status colors visible in both themes

---

## Migration Path

### Backward Compatibility
- Default theme remains "dark" (no change for existing users)
- Existing configs without `theme` field load default ("dark")
- No breaking changes to config file format
- Visual appearance identical until user switches theme

### Rollout Plan
1. **Phase 1:** Deploy infrastructure (no visual changes)
2. **Phase 2-4:** Deploy component refactors (maintain dark theme appearance)
3. **Phase 5:** Enable theme toggle (opt-in feature)
4. **Phase 6:** Document and announce light theme availability

---

## Success Criteria

### Must Have (P0)
- ✅ All components use theme colors (no hardcoded `Color::X`)
- ✅ Dark theme maintains current appearance
- ✅ Light theme is readable on light terminals
- ✅ Theme loaded from config on startup
- ✅ No regressions in existing functionality

### Should Have (P1)
- ✅ Ctrl+T toggles theme at runtime
- ✅ Theme preference persisted to config
- ✅ Help menu documents theme feature
- ✅ Unit tests for theme functionality

### Nice to Have (P2)
- ✅ Status message on theme toggle
- ✅ Theme indicator in UI
- ✅ README screenshots of both themes
- ⬜ Custom theme support (future enhancement)

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Color contrast insufficient in light theme | High | Medium | WCAG testing, adjust colors |
| Components missed during refactor | Medium | Medium | Grep for `Color::`, manual testing checklist |
| Performance impact from theme passing | Low | Low | Theme is lightweight struct, passed by ref |
| User confusion with theme toggle | Low | Low | Document in help menu, show status message |
| Terminal color palette varies | Medium | High | Test on multiple terminals, use RGB where needed |

---

## Open Questions

1. **Q:** Should we support custom user-defined themes in the future?  
   **A:** Not in this phase - keep scope focused. Add as enhancement later.

2. **Q:** Should key colors from layout data be themed?  
   **A:** No - those are user-defined semantic colors, not UI chrome.

3. **Q:** What about terminal 256-color vs 16-color mode?  
   **A:** Use basic colors where possible, RGB for light theme specifics.

4. **Q:** Should theme affect the keyboard keys' RGB colors?  
   **A:** No - keys display user-configured colors from layout file.

5. **Q:** What if terminal doesn't support true color?  
   **A:** Ratatui handles fallback to 256-color palette automatically.

---

## References

- **Related Issues:** Help menu inconsistencies (user report)
- **Design Inspiration:** Helix editor themes, Neovim color schemes
- **Standards:** WCAG 2.1 contrast requirements
- **Dependencies:** Ratatui 0.26, Crossterm 0.27

---

## Appendix: Component Color Audit

### Components with Hardcoded Colors (15 files)

1. **help_overlay.rs** - 40+ colors (Priority: P0)
2. **status_bar.rs** - 8 colors (Priority: P1)
3. **keyboard.rs** - 5 colors (Priority: P3)
4. **keycode_picker.rs** - 4 colors (Priority: P2)
5. **color_picker.rs** - 10 colors (Priority: P2)
6. **category_picker.rs** - 6 colors (Priority: P2)
7. **category_manager.rs** - 12 colors (Priority: P2)
8. **template_browser.rs** - 12 colors (Priority: P3)
9. **build_log.rs** - 6 colors (Priority: P3)
10. **metadata_editor.rs** - 10 colors (Priority: P2)
11. **layout_picker.rs** - Unknown (Priority: P3)
12. **onboarding_wizard.rs** - Unknown (Priority: P3)
13. **config_dialogs.rs** - Unknown (Priority: P2)
14. **mod.rs** (main UI) - 2 colors (Priority: P1)

**Total Hardcoded Colors:** ~115+ across all components

---

**Status:** Specification complete, ready for implementation  
**Next Action:** Begin Phase 1, Task 1.1 (Create theme.rs)
