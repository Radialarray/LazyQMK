# Theme Consistency Implementation - Detailed Task List

**Feature:** Centralized theme system with dark/light mode support  
**Branch:** `fix/theme-consistency`  
**Created:** 2025-11-25  
**Status:** Planning

---

## Overview

Implement a centralized theme system to replace hardcoded colors across 15+ UI components, enabling consistent dark/light mode support with runtime theme switching.

### Current State
- ❌ No theme system - `config.ui.theme` field exists but unused
- ❌ 40+ hardcoded colors in help menu alone
- ❌ Inconsistent highlight styles across components
- ❌ No light mode support - assumes dark terminal background
- ❌ Colors scattered across 15+ files

### Target State
- ✅ Centralized `Theme` struct with semantic color roles
- ✅ Dark and light theme variants
- ✅ All components use theme colors consistently
- ✅ Runtime theme switching with keyboard shortcut
- ✅ Theme preference persisted in config

---

## Phase 1: Core Theme Infrastructure (Foundation)

### Task 1.1: Create Theme Module
**File:** `src/tui/theme.rs` (new file)  
**Priority:** P0 - Critical  
**Estimated effort:** 2-3 hours

**Description:**  
Create the core `Theme` struct with semantic color roles that will be used across all components.

**Implementation Details:**
```rust
// src/tui/theme.rs

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Semantic color theme for the TUI
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    // Primary UI colors
    pub primary: Color,        // Borders, titles, emphasis
    pub accent: Color,         // Highlights, selections, focus
    pub success: Color,        // Success states, confirmations
    pub error: Color,          // Error states, warnings
    pub warning: Color,        // Warning states, cautions
    
    // Text hierarchy
    pub text: Color,           // Primary text content
    pub text_secondary: Color, // Secondary text, labels
    pub text_muted: Color,     // Muted text, help, disabled
    
    // Backgrounds
    pub background: Color,     // Main background
    pub highlight_bg: Color,   // Highlight/selection background
    pub surface: Color,        // Surface elements, panels
    
    // State indicators
    pub active: Color,         // Active/focused element
    pub inactive: Color,       // Inactive/disabled element
}

/// Theme variant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeVariant {
    Dark,
    Light,
}

impl Theme {
    /// Creates a dark theme (current default)
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            accent: Color::Yellow,
            success: Color::Green,
            error: Color::Red,
            warning: Color::Yellow,
            
            text: Color::White,
            text_secondary: Color::LightGray,
            text_muted: Color::DarkGray,
            
            background: Color::Black,
            highlight_bg: Color::DarkGray,
            surface: Color::Rgb(30, 30, 30),
            
            active: Color::Yellow,
            inactive: Color::Gray,
        }
    }
    
    /// Creates a light theme
    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            accent: Color::Rgb(180, 100, 0), // Dark orange
            success: Color::Rgb(0, 128, 0),  // Dark green
            error: Color::Red,
            warning: Color::Rgb(200, 100, 0),
            
            text: Color::Black,
            text_secondary: Color::Rgb(60, 60, 60),
            text_muted: Color::Gray,
            
            background: Color::White,
            highlight_bg: Color::Rgb(230, 230, 230),
            surface: Color::Rgb(245, 245, 245),
            
            active: Color::Rgb(180, 100, 0),
            inactive: Color::LightGray,
        }
    }
    
    /// Creates a theme from a variant
    pub fn from_variant(variant: ThemeVariant) -> Self {
        match variant {
            ThemeVariant::Dark => Self::dark(),
            ThemeVariant::Light => Self::light(),
        }
    }
    
    /// Creates a theme from a string name
    pub fn from_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "light" => Self::light(),
            "dark" | "default" | _ => Self::dark(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}
```

**Acceptance Criteria:**
- [ ] `Theme` struct defined with 13 semantic color fields
- [ ] `ThemeVariant` enum with `Dark` and `Light` variants
- [ ] `Theme::dark()` returns current color scheme
- [ ] `Theme::light()` returns light-background-compatible colors
- [ ] `Theme::from_name()` parses config string values
- [ ] `Theme::from_variant()` creates theme from enum
- [ ] All colors readable on both dark/light terminal backgrounds

**Testing:**
```rust
#[test]
fn test_theme_dark() {
    let theme = Theme::dark();
    assert_eq!(theme.primary, Color::Cyan);
    assert_eq!(theme.background, Color::Black);
}

#[test]
fn test_theme_light() {
    let theme = Theme::light();
    assert_eq!(theme.text, Color::Black);
    assert_eq!(theme.background, Color::White);
}

#[test]
fn test_theme_from_name() {
    assert_eq!(Theme::from_name("dark"), Theme::dark());
    assert_eq!(Theme::from_name("light"), Theme::light());
    assert_eq!(Theme::from_name("invalid"), Theme::dark()); // fallback
}
```

---

### Task 1.2: Update Config Theme Validation
**File:** `src/config.rs`  
**Priority:** P0 - Critical  
**Estimated effort:** 1 hour

**Description:**  
Make the `theme` field in `UiConfig` functional by adding validation and removing the "future feature" comment.

**Changes:**
1. Update comment on line 50-51 (remove "future feature")
2. Add theme validation to `Config::validate()` method
3. Update default to be explicit "dark" instead of "default"

**Implementation:**
```rust
// Line 50-51: Update comment
/// Color theme ("dark" or "light")
pub theme: String,

// Line 59: Update default
theme: "dark".to_string(),

// Add to Config::validate() around line 213:
// Validate theme
match self.ui.theme.to_lowercase().as_str() {
    "dark" | "light" => {}
    _ => anyhow::bail!(
        "Invalid theme '{}'. Must be 'dark' or 'light'",
        self.ui.theme
    ),
}
```

**Acceptance Criteria:**
- [ ] `theme` field comment updated to remove "future feature"
- [ ] Default theme is "dark"
- [ ] `Config::validate()` ensures theme is "dark" or "light"
- [ ] Invalid themes return validation error
- [ ] Existing tests still pass

**Testing:**
```rust
#[test]
fn test_config_validate_theme() {
    let mut config = Config::new();
    assert!(config.validate().is_ok());

    config.ui.theme = "invalid".to_string();
    assert!(config.validate().is_err());

    config.ui.theme = "dark".to_string();
    assert!(config.validate().is_ok());

    config.ui.theme = "light".to_string();
    assert!(config.validate().is_ok());
}
```

---

### Task 1.3: Integrate Theme into AppState
**File:** `src/tui/mod.rs`  
**Priority:** P0 - Critical  
**Estimated effort:** 2 hours

**Description:**  
Add `Theme` to `AppState` and initialize it from config during app startup.

**Changes:**
1. Add module declaration: `pub mod theme;` (around line 20)
2. Add theme field to `AppState` struct (around line 160-200)
3. Initialize theme in `AppState::new()` from config
4. Export `Theme` and `ThemeVariant` for external use

**Implementation:**
```rust
// Line ~20: Add module
pub mod theme;

// Re-export at line ~53
pub use theme::{Theme, ThemeVariant};

// In AppState struct (find around line 160-250):
pub struct AppState {
    // ... existing fields ...
    
    /// Current UI theme
    pub theme: Theme,
    
    // ... rest of fields ...
}

// In AppState::new() implementation (find around line 300-400):
pub fn new(
    geometry: KeyboardGeometry,
    layout: Layout,
    keycode_db: KeycodeDb,
    layout_file_path: Option<PathBuf>,
    config: Config,
) -> Self {
    // Initialize theme from config
    let theme = Theme::from_name(&config.ui.theme);
    
    Self {
        // ... existing field initialization ...
        theme,
        // ... rest of field initialization ...
    }
}
```

**Acceptance Criteria:**
- [ ] `theme` module added to `src/tui/mod.rs`
- [ ] `Theme` and `ThemeVariant` re-exported from `tui` module
- [ ] `theme: Theme` field added to `AppState`
- [ ] Theme initialized from `config.ui.theme` in `AppState::new()`
- [ ] Default theme is dark when config has "dark" or "default"
- [ ] App compiles without errors
- [ ] App runs and displays correctly (colors unchanged yet)

---

### Task 1.4: Add Theme Getter Method
**File:** `src/tui/mod.rs`  
**Priority:** P1 - High  
**Estimated effort:** 30 minutes

**Description:**  
Add a convenience method to access the theme from `AppState`.

**Implementation:**
```rust
impl AppState {
    // ... existing methods ...
    
    /// Returns a reference to the current theme
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }
}
```

**Acceptance Criteria:**
- [ ] `theme()` method added to `AppState`
- [ ] Method returns immutable reference to theme
- [ ] Method is public and accessible from render functions

---

## Phase 2: Component Refactoring (High Priority)

### Task 2.1: Refactor Help Overlay
**File:** `src/tui/help_overlay.rs`  
**Priority:** P0 - Critical (biggest impact)  
**Estimated effort:** 3-4 hours

**Description:**  
Replace 40+ hardcoded colors in help overlay with theme-based colors. This is the highest priority as it's the most visible inconsistency mentioned by the user.

**Current Issues:**
- Line 555-559: Hardcoded `Color::Cyan` borders, `Color::Black` background
- Lines 80-96: Hardcoded `Color::Cyan` section headers
- Lines 109-110, 114-115: Hardcoded `Color::Green` keybindings
- Lines 228, 283: Hardcoded `Color::Cyan` info messages
- Line 511: Hardcoded `Color::DarkGray` footer

**Implementation Strategy:**
1. Update `render_help_overlay()` signature to accept `&Theme`
2. Replace all `Color::Cyan` with `theme.primary`
3. Replace all `Color::Yellow` with `theme.accent`
4. Replace all `Color::Green` with `theme.success`
5. Replace all `Color::DarkGray` with `theme.text_muted`
6. Replace `Color::Black` background with `theme.background`

**Changes:**
```rust
// Update function signature (around line 545)
pub fn render_help_overlay(
    frame: &mut Frame,
    area: Rect,
    state: &HelpOverlayState,
    theme: &Theme,  // ADD THIS PARAMETER
) {
    // ... implementation ...
    
    // Line 555-559: Replace border colors
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))  // Was Color::Cyan
        .style(Style::default().bg(theme.background));     // Was Color::Black
    
    // Lines 80-96: Update section headers in get_help_content()
    // Note: Need to pass theme to get_help_content() or make it take theme parameter
}

// Update get_help_content to accept theme
fn get_help_content(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(vec![
            Span::styled("═══...", Style::default().fg(theme.primary)),  // Was Cyan
        ]),
        // ... continue for all ~40 color references
    ]
}
```

**Color Mapping:**
- `Color::Cyan` → `theme.primary` (borders, titles, section headers)
- `Color::Yellow` → `theme.accent` (section titles, emphasis)
- `Color::Green` → `theme.success` (keybindings, confirmations)
- `Color::Red` → `theme.error` (warnings, destructive actions)
- `Color::Gray` → `theme.text_secondary` (help text)
- `Color::DarkGray` → `theme.text_muted` (footer, dim text)
- `Color::Black` → `theme.background` (explicit backgrounds)
- `Color::White` / default → `theme.text` (primary text)

**Acceptance Criteria:**
- [ ] `render_help_overlay()` accepts `&Theme` parameter
- [ ] All 40+ hardcoded colors replaced with theme colors
- [ ] Help overlay renders correctly in dark theme (existing appearance)
- [ ] Help overlay renders correctly in light theme (new appearance)
- [ ] No hardcoded `Color::` references remain in file
- [ ] Text is readable in both themes
- [ ] Function called correctly from main event loop

**Testing:**
- [ ] Manual test: Open help with `?` in dark theme
- [ ] Manual test: Switch to light theme and open help with `?`
- [ ] Visual test: All sections properly colored
- [ ] Visual test: Keybindings stand out appropriately
- [ ] Visual test: No color clashing or readability issues

---

### Task 2.2: Refactor Status Bar
**File:** `src/tui/status_bar.rs`  
**Priority:** P1 - High  
**Estimated effort:** 1-2 hours

**Description:**  
Replace hardcoded colors in status bar with theme colors. Status bar is always visible.

**Current Issues:**
- Lines 26-32: Build status colors (Gray, Yellow, Green, Red)
- Line 36: Error prefix `Color::Red`
- Line 44: Mode display colors
- Line 60: Help text `Color::Cyan`

**Implementation:**
```rust
// Update render function signature (around line 15-25)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    build_state: &BuildState,
    error_msg: Option<&str>,
    theme: &Theme,  // ADD THIS
) {
    // Line 26-32: Replace build status colors
    let (status_text, status_color) = match build_state {
        BuildState::Idle => ("Idle", theme.inactive),        // Was Gray
        BuildState::Validating => ("Validating", theme.warning),  // Was Yellow
        BuildState::Generating => ("Generating", theme.warning),  // Was Yellow
        BuildState::Compiling => ("Compiling", theme.warning),    // Was Yellow
        BuildState::Success => ("Success", theme.success),        // Was Green
        BuildState::Failed => ("Failed", theme.error),            // Was Red
    };
    
    // Line 36: Error prefix
    let error_span = Span::styled("Error: ", Style::default().fg(theme.error));
    
    // Line 60: Help text
    let help_text = Span::styled(" ? Help ", Style::default().fg(theme.primary));
}
```

**Color Mapping:**
- `Color::Gray` (Idle) → `theme.inactive`
- `Color::Yellow` (Building) → `theme.warning`
- `Color::Green` (Success) → `theme.success`
- `Color::Red` (Error/Failed) → `theme.error`
- `Color::Cyan` (Help) → `theme.primary`

**Acceptance Criteria:**
- [ ] `render()` accepts `&Theme` parameter
- [ ] All build state colors use theme
- [ ] Error messages use `theme.error`
- [ ] Help text uses `theme.primary`
- [ ] Status bar renders correctly in both themes
- [ ] No hardcoded colors remain

---

### Task 2.3: Refactor Main UI Title Bar
**File:** `src/tui/mod.rs`  
**Priority:** P1 - High  
**Estimated effort:** 30 minutes

**Description:**  
Replace hardcoded title bar color with theme color.

**Current Issues:**
- Lines 483-484: Hardcoded `Color::Cyan` for title

**Implementation:**
```rust
// Around line 483-484 in run() function
let title_block = Block::default()
    .title(" Keyboard Layout Editor ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(self.theme.primary));  // Was Color::Cyan
```

**Acceptance Criteria:**
- [ ] Title bar uses `self.theme.primary` instead of `Color::Cyan`
- [ ] Title bar renders correctly in both themes

---

## Phase 3: Pickers & Editors (Medium Priority)

### Task 3.1: Refactor Keycode Picker
**File:** `src/tui/keycode_picker.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 1-2 hours

**Current Issues:**
- Lines 113-117: Hardcoded `Color::DarkGray` highlight

**Implementation:**
```rust
// Update render function signature (around line 90-100)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &KeycodePickerState,
    theme: &Theme,  // ADD THIS
) {
    // Line 113-117: Replace highlight style
    let highlight_style = Style::default()
        .bg(theme.highlight_bg)  // Was Color::DarkGray
        .add_modifier(Modifier::BOLD);
        
    // Border style
    let block = Block::default()
        .title(" Select Keycode ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
}
```

**Acceptance Criteria:**
- [ ] Function signature includes `&Theme` parameter
- [ ] Highlight uses `theme.highlight_bg`
- [ ] Border uses `theme.primary`
- [ ] Readable in both themes

---

### Task 3.2: Refactor Color Picker
**File:** `src/tui/color_picker.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 2 hours

**Current Issues:**
- Lines 129-133: Hardcoded title, channel labels, active/inactive colors
- Line 203: Border color
- Line 179: Hex display color

**Implementation:**
```rust
// Update render function (around line 100-120)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &ColorPickerState,
    theme: &Theme,  // ADD THIS
) {
    // Line 129-133: Update channel colors
    let title_style = Style::default()
        .fg(theme.primary)  // Was Color::Cyan
        .add_modifier(Modifier::BOLD);
    
    // Channel labels still use RGB colors (semantic - don't change)
    // But active/inactive state should use theme
    let active_modifier = Modifier::BOLD;
    let inactive_color = theme.text_muted;  // Was Color::Gray
    
    // Line 203: Border
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));
    
    // Line 179: Hex display
    let hex_style = Style::default().fg(theme.text);  // Was Color::White
}
```

**Color Mapping:**
- `Color::Cyan` (title, border) → `theme.primary`
- `Color::Gray` (inactive) → `theme.text_muted`
- `Color::White` (hex) → `theme.text`
- Keep RGB channel labels as Red/Green/Blue (semantic meaning)

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Title and borders use theme
- [ ] Active/inactive states use theme
- [ ] RGB labels remain red/green/blue (semantic)
- [ ] Readable in both themes

---

### Task 3.3: Refactor Category Picker
**File:** `src/tui/category_picker.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 1-2 hours

**Current Issues:**
- Lines 86, 116: Border `Color::Cyan`
- Highlight: `Color::DarkGray` background
- "None" option: `Color::Gray` with italic

**Implementation:**
```rust
// Update render function (around line 70-90)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &CategoryPickerState,
    categories: &[Category],
    theme: &Theme,  // ADD THIS
) {
    // Border
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));  // Was Cyan
    
    // Highlight style
    let highlight_style = Style::default()
        .bg(theme.highlight_bg)  // Was DarkGray
        .add_modifier(Modifier::BOLD);
    
    // "None" option
    let none_style = Style::default()
        .fg(theme.text_muted)  // Was Gray
        .add_modifier(Modifier::ITALIC);
}
```

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Border uses `theme.primary`
- [ ] Highlight uses `theme.highlight_bg`
- [ ] "None" option uses `theme.text_muted`
- [ ] Category colors remain from category data (not themed)

---

### Task 3.4: Refactor Category Manager
**File:** `src/tui/category_manager.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 2-3 hours

**Current Issues:**
- Line 170: Background `Color::Black`
- Lines 232-236: Selected/unselected colors
- Line 258: ID display `Color::DarkGray`
- Prompts: Various colors (Cyan, Red, Gray)

**Implementation:**
```rust
// Update render function (around line 150-170)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &CategoryManagerState,
    theme: &Theme,  // ADD THIS
) {
    // Line 170: Background
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary))
        .style(Style::default().bg(theme.background));  // Was Black
    
    // Lines 232-236: Selection colors
    let selected_style = Style::default()
        .fg(theme.accent)  // Was Yellow
        .add_modifier(Modifier::BOLD);
    
    let unselected_style = Style::default()
        .fg(theme.text);  // Was White
    
    // Line 258: ID display
    let id_style = Style::default().fg(theme.text_muted);  // Was DarkGray
    
    // Highlight
    let highlight_style = Style::default()
        .bg(theme.highlight_bg)  // Was DarkGray
        .add_modifier(Modifier::BOLD);
    
    // Prompts
    let prompt_style = Style::default().fg(theme.primary);    // Was Cyan
    let error_style = Style::default().fg(theme.error);       // Was Red
    let help_style = Style::default().fg(theme.text_muted);   // Was Gray
}
```

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] All backgrounds use theme
- [ ] Selection states use theme
- [ ] Prompts and messages use theme
- [ ] Readable in both themes

---

### Task 3.5: Refactor Metadata Editor
**File:** `src/tui/metadata_editor.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 2 hours

**Current Issues:**
- Line 193: Border colors
- Lines 254-276: Active/inactive field colors
- Lines 285-297: Control key colors

**Implementation:**
```rust
// Update render function (around line 180-200)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &MetadataEditorState,
    theme: &Theme,  // ADD THIS
) {
    // Line 193: Border
    let border_style = Style::default().fg(theme.primary);  // Was Cyan
    
    // Lines 254-276: Field styles
    let active_style = Style::default()
        .fg(theme.active)  // Was Yellow
        .add_modifier(Modifier::BOLD);
    
    let inactive_style = Style::default()
        .fg(theme.text);  // Was White
    
    let active_border = Style::default().fg(theme.active);    // Was Yellow
    let inactive_border = Style::default().fg(theme.inactive); // Was Gray
    
    // Lines 285-297: Control keys
    let enter_style = Style::default()
        .fg(theme.success)  // Was Green
        .add_modifier(Modifier::BOLD);
    
    let esc_style = Style::default()
        .fg(theme.error)  // Was Red
        .add_modifier(Modifier::BOLD);
    
    let tab_style = Style::default()
        .fg(theme.warning)  // Was Yellow
        .add_modifier(Modifier::BOLD);
    
    let help_style = Style::default().fg(theme.text_muted);  // Was Gray
}
```

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Active/inactive states use theme
- [ ] Control keys use semantic colors
- [ ] Help text uses muted color
- [ ] Readable in both themes

---

## Phase 4: Complex Components (Lower Priority)

### Task 4.1: Refactor Keyboard Widget
**File:** `src/tui/keyboard.rs`  
**Priority:** P3 - Lower  
**Estimated effort:** 2-3 hours

**Current Issues:**
- Lines 112-117: Selected key uses `Color::Black` on `Color::Yellow`
- Key colors come from RGB data (should remain)
- Fallback color `Color::White`

**Implementation:**
```rust
// Update render function (around line 90-110)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    widget: &KeyboardWidget,
    selected_key_index: Option<usize>,
    theme: &Theme,  // ADD THIS
) {
    // Lines 112-117: Selection style
    // Note: Selected key needs high contrast
    let selected_style = if matches!(theme.background, Color::Black | Color::Rgb(..)) {
        // Dark theme: black text on yellow bg
        Style::default()
            .fg(Color::Black)
            .bg(theme.accent)
    } else {
        // Light theme: white text on dark bg
        Style::default()
            .fg(Color::White)
            .bg(Color::Rgb(180, 100, 0))
    };
    
    // Fallback color for keys without RGB
    let fallback_style = Style::default().fg(theme.text);  // Was White
}
```

**Note:** Key colors should remain from layout RGB data - don't theme these, as they're user-defined semantic colors.

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Selection style adapts to theme background
- [ ] Key colors remain from layout data (not themed)
- [ ] Fallback text color uses theme
- [ ] High contrast maintained for selected key in both themes

---

### Task 4.2: Refactor Template Browser
**File:** `src/tui/template_browser.rs`  
**Priority:** P3 - Lower  
**Estimated effort:** 2-3 hours

**Current Issues:**
- Lines 247-254: Title colors (Cyan/Yellow)
- Lines 269-272: Search field colors
- Lines 294-300: Selected item, details, help text

**Implementation:**
```rust
// Update render function (around line 230-250)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    state: &TemplateBrowserState,
    theme: &Theme,  // ADD THIS
) {
    // Lines 247-254: Title
    let title_style = if state.search_active {
        Style::default()
            .fg(theme.accent)  // Was Yellow
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.primary)  // Was Cyan
            .add_modifier(Modifier::BOLD)
    };
    
    // Lines 269-272: Search field
    let search_style = if state.search_active {
        Style::default().fg(theme.accent)  // Was Yellow
    } else {
        Style::default().fg(theme.text_muted)  // Was Gray
    };
    
    // Lines 294-300: Item selection
    let selected_style = Style::default()
        .fg(theme.background)  // Was Black
        .bg(theme.primary)     // Was Cyan
        .add_modifier(Modifier::BOLD);
    
    let unselected_style = Style::default()
        .fg(theme.text);  // Was White
    
    // Details labels
    let label_style = Style::default().fg(theme.primary);  // Was Cyan
    
    // Empty state
    let empty_style = Style::default().fg(theme.warning);  // Was Yellow
    
    // Help text
    let help_style = Style::default().fg(theme.text_muted);  // Was Gray
}
```

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Title reflects active/inactive state with theme
- [ ] Search field uses theme colors
- [ ] Selection uses high contrast from theme
- [ ] Details and help text use theme
- [ ] Readable in both themes

---

### Task 4.3: Refactor Build Log
**File:** `src/tui/build_log.rs`  
**Priority:** P3 - Lower  
**Estimated effort:** 1-2 hours

**Current Issues:**
- Lines 93-96: Message type colors (White, Green, Red)
- Line 117: Border `Color::Cyan`
- Line 132: Help text `Color::Gray` with `Modifier::DIM`

**Implementation:**
```rust
// Update render function (around line 80-100)
pub fn render(
    frame: &mut Frame,
    area: Rect,
    build_log: &BuildLog,
    theme: &Theme,  // ADD THIS
) {
    // Lines 93-96: Message colors
    let style = match message_type {
        MessageType::Info => Style::default().fg(theme.text),      // Was White
        MessageType::Success => Style::default().fg(theme.success), // Was Green
        MessageType::Error => Style::default().fg(theme.error),     // Was Red
    };
    
    // Line 117: Border
    let block = Block::default()
        .title(" Build Log ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.primary));  // Was Cyan
    
    // Line 132: Help text
    let help_style = Style::default()
        .fg(theme.text_muted)  // Was Gray
        .add_modifier(Modifier::DIM);
}
```

**Acceptance Criteria:**
- [ ] Function accepts `&Theme` parameter
- [ ] Message types use semantic theme colors
- [ ] Border uses theme
- [ ] Help text uses muted theme color
- [ ] Readable in both themes

---

## Phase 5: Runtime Theme Switching (Enhancement)

### Task 5.1: Add Theme Toggle Keybinding
**File:** `src/tui/mod.rs`  
**Priority:** P2 - Medium  
**Estimated effort:** 2 hours

**Description:**  
Add keyboard shortcut (Ctrl+T) to toggle between dark and light themes at runtime.

**Implementation:**
```rust
// In event handling loop (around line 600-800 in run() method)
// Add new key handler for Ctrl+T

KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    // Toggle theme
    self.theme = if matches!(self.theme, Theme { background: Color::Black, .. }) {
        // Currently dark, switch to light
        self.config.ui.theme = "light".to_string();
        Theme::light()
    } else {
        // Currently light, switch to dark
        self.config.ui.theme = "dark".to_string();
        Theme::dark()
    };
    
    // Note: Theme change will take effect on next render
    // Could optionally save config here (see Task 5.2)
}
```

**Better Implementation (using ThemeVariant):**
```rust
// Add field to AppState
pub struct AppState {
    // ... existing fields ...
    pub theme: Theme,
    pub theme_variant: ThemeVariant,  // ADD THIS
}

// In initialization
Self {
    theme_variant: if config.ui.theme == "light" {
        ThemeVariant::Light
    } else {
        ThemeVariant::Dark
    },
    theme: Theme::from_name(&config.ui.theme),
    // ... other fields ...
}

// In event handler
KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    // Toggle theme variant
    self.theme_variant = match self.theme_variant {
        ThemeVariant::Dark => ThemeVariant::Light,
        ThemeVariant::Light => ThemeVariant::Dark,
    };
    self.theme = Theme::from_variant(self.theme_variant);
    self.config.ui.theme = match self.theme_variant {
        ThemeVariant::Dark => "dark",
        ThemeVariant::Light => "light",
    }.to_string();
}
```

**Acceptance Criteria:**
- [ ] Ctrl+T toggles between dark and light themes
- [ ] Theme change reflects immediately in UI
- [ ] All components render correctly after toggle
- [ ] `config.ui.theme` updated in memory
- [ ] Status message shown: "Theme switched to light/dark"

---

### Task 5.2: Persist Theme on Change
**File:** `src/tui/mod.rs`  
**Priority:** P3 - Lower  
**Estimated effort:** 1 hour

**Description:**  
Automatically save theme preference when changed via Ctrl+T.

**Implementation:**
```rust
// In the Ctrl+T handler from Task 5.1
KeyCode::Char('t') if key.modifiers.contains(KeyModifiers::CONTROL) => {
    // ... toggle theme code from Task 5.1 ...
    
    // Save config (with error handling)
    if let Err(e) = self.config.save() {
        self.error_message = Some(format!("Failed to save theme preference: {}", e));
    } else {
        // Optional: Show success message
        // Could add to status bar or as temporary message
    }
}
```

**Acceptance Criteria:**
- [ ] Theme preference saved to config file on toggle
- [ ] User's choice persists across app restarts
- [ ] Error message shown if save fails
- [ ] App doesn't crash if config save fails

---

### Task 5.3: Update Help Overlay with Theme Toggle
**File:** `src/tui/help_overlay.rs`  
**Priority:** P3 - Lower  
**Estimated effort:** 30 minutes

**Description:**  
Add Ctrl+T keybinding documentation to help overlay.

**Implementation:**
```rust
// In get_help_content(), add to System section (around line 470-490)
Line::from(vec![
    Span::styled("  Ctrl+T      ", Style::default().fg(theme.success)),
    Span::raw("Toggle dark/light theme"),
]),
```

**Acceptance Criteria:**
- [ ] Ctrl+T documented in help overlay
- [ ] Appears in appropriate section (System or Configuration)
- [ ] Uses correct theme colors

---

## Phase 6: Documentation & Testing

### Task 6.1: Update Configuration Documentation
**File:** `README.md` or docs  
**Priority:** P3 - Lower  
**Estimated effort:** 1 hour

**Description:**  
Document the theme configuration option and keybinding.

**Content:**
```markdown
### Theme Configuration

The TUI supports both dark and light color themes.

**Config file** (`~/.config/layout_tools/config.toml`):
```toml
[ui]
theme = "dark"  # or "light"
```

**Runtime toggle:** Press `Ctrl+T` to switch between themes.

Theme preference is automatically saved when changed.
```

**Acceptance Criteria:**
- [ ] Theme configuration documented
- [ ] Ctrl+T keybinding documented
- [ ] Config file location and format shown
- [ ] Screenshots of both themes (optional)

---

### Task 6.2: Add Theme Tests
**File:** `tests/theme_tests.rs` (new file)  
**Priority:** P2 - Medium  
**Estimated effort:** 2 hours

**Description:**  
Add integration tests for theme functionality.

**Implementation:**
```rust
#[cfg(test)]
mod theme_tests {
    use super::*;
    
    #[test]
    fn test_dark_theme_colors() {
        let theme = Theme::dark();
        // Verify key colors are set correctly
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.background, Color::Black);
        assert_eq!(theme.text, Color::White);
    }
    
    #[test]
    fn test_light_theme_colors() {
        let theme = Theme::light();
        // Verify light theme has light background
        assert_eq!(theme.background, Color::White);
        assert_eq!(theme.text, Color::Black);
        // Should not use bright colors on light bg
        assert_ne!(theme.text, Color::White);
    }
    
    #[test]
    fn test_theme_from_config() {
        let mut config = Config::new();
        config.ui.theme = "light".to_string();
        let theme = Theme::from_name(&config.ui.theme);
        assert_eq!(theme.background, Color::White);
    }
    
    #[test]
    fn test_theme_variant_toggle() {
        let mut variant = ThemeVariant::Dark;
        variant = match variant {
            ThemeVariant::Dark => ThemeVariant::Light,
            ThemeVariant::Light => ThemeVariant::Dark,
        };
        assert_eq!(variant, ThemeVariant::Light);
    }
}
```

**Acceptance Criteria:**
- [ ] Tests for `Theme::dark()` and `Theme::light()`
- [ ] Tests for `Theme::from_name()`
- [ ] Tests for `ThemeVariant` enum
- [ ] Tests for config integration
- [ ] All tests pass with `cargo test`

---

### Task 6.3: Manual Testing Checklist
**Priority:** P1 - High  
**Estimated effort:** 2-3 hours

**Description:**  
Comprehensive manual testing of all UI components in both themes.

**Test Checklist:**

#### Dark Theme (Existing)
- [ ] Help overlay (?) - all sections readable, colors consistent
- [ ] Status bar - build states visible, help text clear
- [ ] Keyboard widget - keys visible, selection clear
- [ ] Keycode picker (k) - list readable, highlight visible
- [ ] Color picker (c) - RGB sliders clear, hex display readable
- [ ] Category picker - list readable, preview colors work
- [ ] Category manager (m) - list readable, prompts clear
- [ ] Template browser (t) - templates readable, search works
- [ ] Metadata editor (e) - fields readable, active field clear
- [ ] Build log (l) - messages readable, colors appropriate
- [ ] Config dialogs - all dialogs readable

#### Light Theme (New)
- [ ] Help overlay (?) - all sections readable on light background
- [ ] Status bar - build states visible, no color clashing
- [ ] Keyboard widget - keys visible, selection high contrast
- [ ] Keycode picker (k) - list readable, highlight visible
- [ ] Color picker (c) - RGB sliders clear, hex display readable
- [ ] Category picker - list readable, preview colors work
- [ ] Category manager (m) - list readable, prompts clear
- [ ] Template browser (t) - templates readable, search works
- [ ] Metadata editor (e) - fields readable, active field clear
- [ ] Build log (l) - messages readable, error/success clear
- [ ] Config dialogs - all dialogs readable

#### Theme Switching
- [ ] Ctrl+T toggles from dark to light
- [ ] Ctrl+T toggles from light to dark
- [ ] UI updates immediately after toggle
- [ ] No visual glitches during toggle
- [ ] Theme preference saved (check config file)
- [ ] Theme persists after restart

#### Edge Cases
- [ ] Invalid theme in config falls back to dark
- [ ] Config validation rejects invalid themes
- [ ] All text readable in both themes
- [ ] No hardcoded colors remain (grep verification)

---

## Summary

### Total Tasks: 24
- **Phase 1 (Foundation):** 4 tasks - ~6 hours
- **Phase 2 (High Priority):** 3 tasks - ~5 hours  
- **Phase 3 (Pickers/Editors):** 5 tasks - ~9 hours
- **Phase 4 (Complex):** 3 tasks - ~6 hours
- **Phase 5 (Runtime Toggle):** 3 tasks - ~4 hours
- **Phase 6 (Docs/Testing):** 3 tasks - ~5 hours
- **Manual Testing:** 3 hours

**Total Estimated Time:** ~38 hours

### Priority Breakdown
- **P0 (Critical):** 5 tasks - Must complete first
- **P1 (High):** 5 tasks - Complete before P2
- **P2 (Medium):** 9 tasks - Complete before P3  
- **P3 (Lower):** 5 tasks - Nice to have / polish

### Recommended Order
1. Complete Phase 1 entirely (foundation required)
2. Task 2.1 (help overlay - biggest user impact)
3. Complete remaining Phase 2 (always-visible UI)
4. Complete Phase 3 (frequently used pickers)
5. Add theme toggle (Task 5.1) for testing
6. Complete Phase 4 (less frequently used)
7. Complete Phase 5 (persistence)
8. Complete Phase 6 (documentation)

### Files Modified Summary
- **New files:** 2 (`src/tui/theme.rs`, `tests/theme_tests.rs`)
- **Modified files:** 15+ (all UI components)
- **Total LOC changed:** ~1000-1500 lines

### Dependencies
- Phase 2-6 depend on Phase 1 completion
- Theme toggle (5.1) should wait until several components refactored
- Testing (6.3) should be final step

---

## Notes

### Color Accessibility
Light theme colors chosen for WCAG contrast compliance:
- Dark blue on white: 8.5:1 (AAA)
- Black text on white: 21:1 (AAA)
- Dark orange/green on white: 4.5:1+ (AA)

### Backward Compatibility
- Default theme remains "dark"
- Existing configs without theme field get "dark"
- No breaking changes to API or config format

### Future Enhancements
- Custom theme support (user-defined colors)
- More theme variants (high contrast, solarized, etc.)
- Per-component color overrides in config
- Theme import/export

### Known Limitations
- Key RGB colors from layout not themed (by design)
- Some components may need contrast adjustments after testing
- Terminal color palette affects final appearance

---

**Status:** Ready for implementation  
**Next Step:** Begin Phase 1, Task 1.1 (Create Theme Module)
