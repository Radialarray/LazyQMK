# Feature Specification: Complete QMK Keycodes & Enhanced Category Picker

**Feature ID:** 009-complete-qmk-keycodes  
**Status:** Planning  
**Created:** 2025-12-02  
**Branch:** `feature/complete-qmk-keycodes`

---

## Problem Statement

The current keycode database is incomplete, containing only ~130 keycodes while QMK supports 200+ keycodes. Additionally, the category picker UI is basic and could be improved for better usability. This limits users from fully configuring their keyboard firmware.

### Missing Keycode Categories

Based on analysis of `src/keycode_db/keycodes_qmk/`:

| Category | Current Count | Missing |
|----------|---------------|---------|
| Basic Keycodes | ~95 | ~60 (international, locking, system keys) |
| Mouse Keys | 0 | 19 (cursor, buttons, wheel, acceleration) |
| Modifiers (Combos) | 8 | ~30 (LCTL(kc), MEH(kc), HYPR(kc), etc.) |
| One Shot Keys | 0 | ~35 (OS_LCTL, OS_LSFT, OSM(), etc.) |
| RGB/Lighting | 0 | ~40 (UG_TOGG, RM_NEXT, BL_TOGG, etc.) |
| US ANSI Shifted | 0 | 21 (KC_TILD, KC_EXLM, KC_PIPE, etc.) |
| Mod-Tap | 0 | ~28 (LCTL_T(kc), MEH_T(kc), etc.) |
| System/Audio | 0 | ~25 (QK_AUDIO_*, QK_AUTO_SHIFT_*, etc.) |
| Leader Key | 0 | 1 (QK_LEAD) |
| Layer Lock | 0 | 2 (QK_LOCK, QK_LLCK) |

**Total Missing: ~260 keycodes**

### Current Category System Issues

1. **Number-key filtering (1-8)** - Not intuitive, no visual indication of current filter
2. **Category display** - Just shows text, no visual hierarchy
3. **No category tabs** - Users must know number shortcuts
4. **No "All" option** - Can only clear filter by pressing same number again or Esc

---

## Solution Overview

### Part 1: Complete Keycode Database

Expand `keycodes.json` to include all QMK keycodes from the reference docs:

1. **New Categories** (add to existing 8):
   - `mouse` - Mouse keys (cursor, buttons, wheel)
   - `mod_combo` - Modifier combinations (LCTL(kc), MEH(kc))
   - `one_shot` - One-shot modifiers and layers
   - `rgb` - RGB Lighting and Matrix controls
   - `backlight` - Backlight controls
   - `audio` - Audio and music mode
   - `shifted` - US ANSI shifted symbols
   - `mod_tap` - Mod-tap keys
   - `advanced` - Leader key, layer lock, autocorrect

2. **Parameterized Keycodes** (with regex patterns):
   - `OSM(mod)` - One-shot modifier
   - `LCTL(kc)`, `LSFT(kc)`, etc. - Modifier combos
   - `LCTL_T(kc)`, `LSFT_T(kc)`, etc. - Mod-tap

### Part 2: Enhanced Category Picker UI

Redesign the keycode picker with:

1. **Visual category tabs** - Horizontal tabs at top showing all categories
2. **Category icons/abbreviations** - Clear visual distinction
3. **Tab navigation** - Left/Right arrows or Tab/Shift+Tab
4. **"All" tab** - First tab shows all keycodes
5. **Keyboard shortcuts** - Keep number keys as acceleration

---

## Architecture

### New Category Structure

```json
{
  "categories": [
    {"id": "all", "name": "All", "description": "All keycodes", "shortcut": "0"},
    {"id": "basic", "name": "Basic", "description": "Letters, numbers, Enter, Tab", "shortcut": "1"},
    {"id": "symbols", "name": "Symbols", "description": "Punctuation and special characters", "shortcut": "2"},
    {"id": "shifted", "name": "Shifted", "description": "US ANSI shifted symbols (!@#$%)", "shortcut": "3"},
    {"id": "navigation", "name": "Nav", "description": "Arrow keys, Home, End, PgUp/Dn", "shortcut": "4"},
    {"id": "function", "name": "F-Keys", "description": "F1-F24 function keys", "shortcut": "5"},
    {"id": "numpad", "name": "Numpad", "description": "Numeric keypad keys", "shortcut": "6"},
    {"id": "modifiers", "name": "Mods", "description": "Shift, Ctrl, Alt, GUI", "shortcut": "7"},
    {"id": "mod_combo", "name": "Mod+Key", "description": "LCTL(kc), MEH(kc), HYPR(kc)", "shortcut": "8"},
    {"id": "mod_tap", "name": "Mod-Tap", "description": "Hold for mod, tap for key", "shortcut": "9"},
    {"id": "layers", "name": "Layers", "description": "MO, TG, TO, LT, OSL", "shortcut": null},
    {"id": "one_shot", "name": "One-Shot", "description": "One-shot modifiers and layers", "shortcut": null},
    {"id": "mouse", "name": "Mouse", "description": "Mouse cursor, buttons, wheel", "shortcut": null},
    {"id": "media", "name": "Media", "description": "Volume, playback, brightness", "shortcut": null},
    {"id": "rgb", "name": "RGB", "description": "RGB lighting and matrix", "shortcut": null},
    {"id": "backlight", "name": "Backlight", "description": "Backlight controls", "shortcut": null},
    {"id": "audio", "name": "Audio", "description": "Audio, music mode, clicky", "shortcut": null},
    {"id": "system", "name": "System", "description": "Boot, power, sleep, system", "shortcut": null},
    {"id": "advanced", "name": "Advanced", "description": "Leader key, autocorrect, macros", "shortcut": null}
  ]
}
```

### New UI Layout

```
┌─────────────────────────────────────────────────────────────────────────┐
│ ◄ All │ Basic │ Symbols │ Shifted │ Nav │ F-Keys │ Numpad │ Mods │ ... ►│
├─────────────────────────────────────────────────────────────────────────┤
│ Search: [query_________________]                                        │
├─────────────────────────────────────────────────────────────────────────┤
│ >> KC_A          - A                                                    │
│    KC_B          - B                                                    │
│    KC_C          - C                                                    │
│    KC_D          - D                                                    │
│    ...                                                                  │
├─────────────────────────────────────────────────────────────────────────┤
│ ◄►/Tab: Categories │ ↑↓: Navigate │ Enter: Select │ Esc: Cancel        │
└─────────────────────────────────────────────────────────────────────────┘
```

### Parameterized Keycode Handling

For keycodes like `LCTL(kc)`, `LCTL_T(kc)`, `OSM(mod)`:

1. **Selection Flow**:
   - User selects `LCTL()` from Mod+Key category
   - Picker switches to keycode sub-picker
   - User selects inner keycode (e.g., `KC_A`)
   - Final keycode: `LCTL(KC_A)`

2. **Regex Patterns**:
   ```json
   {"code": "LCTL()", "pattern": "LCTL\\(([A-Z0-9_]+)\\)", "param_type": "keycode"}
   {"code": "OSM()", "pattern": "OSM\\((MOD_[LR]?[A-Z]+)\\)", "param_type": "modifier"}
   {"code": "LCTL_T()", "pattern": "LCTL_T\\(([A-Z0-9_]+)\\)", "param_type": "keycode"}
   ```

---

## Keycodes to Add

### 1. Basic Keycodes (Missing ~60)

From `Basic Keycodes.md`:

```json
// International keys
{"code": "KC_INT1", "name": "International 1", "category": "basic", "aliases": ["KC_INTERNATIONAL_1"]},
{"code": "KC_INT2", "name": "International 2", "category": "basic"},
// ... through KC_INT9

// Language keys
{"code": "KC_LNG1", "name": "Language 1", "category": "basic", "aliases": ["KC_LANGUAGE_1"]},
// ... through KC_LNG9

// Locking keys
{"code": "KC_LCAP", "name": "Locking Caps Lock", "category": "basic", "aliases": ["KC_LOCKING_CAPS_LOCK"]},
{"code": "KC_LNUM", "name": "Locking Num Lock", "category": "basic"},
{"code": "KC_LSCR", "name": "Locking Scroll Lock", "category": "basic"},

// Non-US keys
{"code": "KC_NUHS", "name": "Non-US #", "category": "symbols", "aliases": ["KC_NONUS_HASH"]},
{"code": "KC_NUBS", "name": "Non-US \\", "category": "symbols", "aliases": ["KC_NONUS_BACKSLASH"]},

// System keys
{"code": "KC_PWR", "name": "System Power", "category": "system", "aliases": ["KC_SYSTEM_POWER"]},
{"code": "KC_SLEP", "name": "System Sleep", "category": "system", "aliases": ["KC_SYSTEM_SLEEP"]},
{"code": "KC_WAKE", "name": "System Wake", "category": "system", "aliases": ["KC_SYSTEM_WAKE"]},

// Editing keys
{"code": "KC_EXEC", "name": "Execute", "category": "basic", "aliases": ["KC_EXECUTE"]},
{"code": "KC_HELP", "name": "Help", "category": "basic"},
{"code": "KC_SLCT", "name": "Select", "category": "basic", "aliases": ["KC_SELECT"]},
{"code": "KC_STOP", "name": "Stop", "category": "basic"},
{"code": "KC_AGIN", "name": "Again", "category": "basic", "aliases": ["KC_AGAIN"]},
{"code": "KC_UNDO", "name": "Undo", "category": "basic"},
{"code": "KC_CUT", "name": "Cut", "category": "basic"},
{"code": "KC_COPY", "name": "Copy", "category": "basic"},
{"code": "KC_PSTE", "name": "Paste", "category": "basic", "aliases": ["KC_PASTE"]},
{"code": "KC_FIND", "name": "Find", "category": "basic"},

// Numpad extras
{"code": "KC_PCMM", "name": "Numpad ,", "category": "numpad", "aliases": ["KC_KP_COMMA"]},
{"code": "KC_PEQL", "name": "Numpad =", "category": "numpad", "aliases": ["KC_KP_EQUAL"]},

// Media extras
{"code": "KC_EJCT", "name": "Eject", "category": "media", "aliases": ["KC_MEDIA_EJECT"]},
{"code": "KC_MAIL", "name": "Launch Mail", "category": "media"},
{"code": "KC_CALC", "name": "Calculator", "category": "media", "aliases": ["KC_CALCULATOR"]},
{"code": "KC_MYCM", "name": "My Computer", "category": "media", "aliases": ["KC_MY_COMPUTER"]},

// Browser keys
{"code": "KC_WSCH", "name": "Browser Search", "category": "media", "aliases": ["KC_WWW_SEARCH"]},
{"code": "KC_WHOM", "name": "Browser Home", "category": "media", "aliases": ["KC_WWW_HOME"]},
{"code": "KC_WBAK", "name": "Browser Back", "category": "media", "aliases": ["KC_WWW_BACK"]},
{"code": "KC_WFWD", "name": "Browser Forward", "category": "media", "aliases": ["KC_WWW_FORWARD"]},
{"code": "KC_WSTP", "name": "Browser Stop", "category": "media", "aliases": ["KC_WWW_STOP"]},
{"code": "KC_WREF", "name": "Browser Refresh", "category": "media", "aliases": ["KC_WWW_REFRESH"]},
{"code": "KC_WFAV", "name": "Browser Favorites", "category": "media", "aliases": ["KC_WWW_FAVORITES"]},

// macOS specific
{"code": "KC_MCTL", "name": "Mission Control", "category": "media", "aliases": ["KC_MISSION_CONTROL"]},
{"code": "KC_LPAD", "name": "Launchpad", "category": "media", "aliases": ["KC_LAUNCHPAD"]},
{"code": "KC_ASST", "name": "Assistant", "category": "media", "aliases": ["KC_ASSISTANT"]},
{"code": "KC_CPNL", "name": "Control Panel", "category": "media", "aliases": ["KC_CONTROL_PANEL"]},

// Media extras
{"code": "KC_MFFD", "name": "Fast Forward", "category": "media", "aliases": ["KC_MEDIA_FAST_FORWARD"]},
{"code": "KC_MRWD", "name": "Rewind", "category": "media", "aliases": ["KC_MEDIA_REWIND"]}
```

### 2. Mouse Keys (19 keys)

From `Mouse keys.md`:

```json
// Cursor movement
{"code": "MS_UP", "name": "Mouse Up", "category": "mouse", "aliases": ["QK_MOUSE_CURSOR_UP"]},
{"code": "MS_DOWN", "name": "Mouse Down", "category": "mouse", "aliases": ["QK_MOUSE_CURSOR_DOWN"]},
{"code": "MS_LEFT", "name": "Mouse Left", "category": "mouse", "aliases": ["QK_MOUSE_CURSOR_LEFT"]},
{"code": "MS_RGHT", "name": "Mouse Right", "category": "mouse", "aliases": ["QK_MOUSE_CURSOR_RIGHT"]},

// Buttons
{"code": "MS_BTN1", "name": "Mouse Button 1", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_1"]},
{"code": "MS_BTN2", "name": "Mouse Button 2", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_2"]},
{"code": "MS_BTN3", "name": "Mouse Button 3", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_3"]},
{"code": "MS_BTN4", "name": "Mouse Button 4", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_4"]},
{"code": "MS_BTN5", "name": "Mouse Button 5", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_5"]},
{"code": "MS_BTN6", "name": "Mouse Button 6", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_6"]},
{"code": "MS_BTN7", "name": "Mouse Button 7", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_7"]},
{"code": "MS_BTN8", "name": "Mouse Button 8", "category": "mouse", "aliases": ["QK_MOUSE_BUTTON_8"]},

// Wheel
{"code": "MS_WHLU", "name": "Wheel Up", "category": "mouse", "aliases": ["QK_MOUSE_WHEEL_UP"]},
{"code": "MS_WHLD", "name": "Wheel Down", "category": "mouse", "aliases": ["QK_MOUSE_WHEEL_DOWN"]},
{"code": "MS_WHLL", "name": "Wheel Left", "category": "mouse", "aliases": ["QK_MOUSE_WHEEL_LEFT"]},
{"code": "MS_WHLR", "name": "Wheel Right", "category": "mouse", "aliases": ["QK_MOUSE_WHEEL_RIGHT"]},

// Acceleration
{"code": "MS_ACL0", "name": "Accel 0 (Slow)", "category": "mouse", "aliases": ["QK_MOUSE_ACCELERATION_0"]},
{"code": "MS_ACL1", "name": "Accel 1 (Medium)", "category": "mouse", "aliases": ["QK_MOUSE_ACCELERATION_1"]},
{"code": "MS_ACL2", "name": "Accel 2 (Fast)", "category": "mouse", "aliases": ["QK_MOUSE_ACCELERATION_2"]}
```

### 3. One-Shot Keys (35 keys)

From `One Shot Keys.md`:

```json
// Toggle
{"code": "OS_TOGG", "name": "One Shot Toggle", "category": "one_shot", "aliases": ["QK_ONE_SHOT_TOGGLE"]},
{"code": "OS_ON", "name": "One Shot On", "category": "one_shot", "aliases": ["QK_ONE_SHOT_ON"]},
{"code": "OS_OFF", "name": "One Shot Off", "category": "one_shot", "aliases": ["QK_ONE_SHOT_OFF"]},

// Left modifiers
{"code": "OS_LCTL", "name": "One Shot Left Ctrl", "category": "one_shot"},
{"code": "OS_LSFT", "name": "One Shot Left Shift", "category": "one_shot"},
{"code": "OS_LALT", "name": "One Shot Left Alt", "category": "one_shot"},
{"code": "OS_LGUI", "name": "One Shot Left GUI", "category": "one_shot"},

// Left combos
{"code": "OS_LCS", "name": "One Shot Ctrl+Shift", "category": "one_shot"},
{"code": "OS_LCA", "name": "One Shot Ctrl+Alt", "category": "one_shot"},
{"code": "OS_LCG", "name": "One Shot Ctrl+GUI", "category": "one_shot"},
{"code": "OS_LSA", "name": "One Shot Shift+Alt", "category": "one_shot"},
{"code": "OS_LSG", "name": "One Shot Shift+GUI", "category": "one_shot"},
{"code": "OS_LAG", "name": "One Shot Alt+GUI", "category": "one_shot"},
{"code": "OS_LCSG", "name": "One Shot Ctrl+Shift+GUI", "category": "one_shot"},
{"code": "OS_LCAG", "name": "One Shot Ctrl+Alt+GUI", "category": "one_shot"},
{"code": "OS_LSAG", "name": "One Shot Shift+Alt+GUI", "category": "one_shot"},

// Right modifiers
{"code": "OS_RCTL", "name": "One Shot Right Ctrl", "category": "one_shot"},
{"code": "OS_RSFT", "name": "One Shot Right Shift", "category": "one_shot"},
{"code": "OS_RALT", "name": "One Shot Right Alt", "category": "one_shot"},
{"code": "OS_RGUI", "name": "One Shot Right GUI", "category": "one_shot"},

// Right combos
{"code": "OS_RCS", "name": "One Shot RCtrl+RShift", "category": "one_shot"},
{"code": "OS_RCA", "name": "One Shot RCtrl+RAlt", "category": "one_shot"},
{"code": "OS_RCG", "name": "One Shot RCtrl+RGUI", "category": "one_shot"},
{"code": "OS_RSA", "name": "One Shot RShift+RAlt", "category": "one_shot"},
{"code": "OS_RSG", "name": "One Shot RShift+RGUI", "category": "one_shot"},
{"code": "OS_RAG", "name": "One Shot RAlt+RGUI", "category": "one_shot"},
{"code": "OS_RCSG", "name": "One Shot RCtrl+RShift+RGUI", "category": "one_shot"},
{"code": "OS_RCAG", "name": "One Shot RCtrl+RAlt+RGUI", "category": "one_shot"},
{"code": "OS_RSAG", "name": "One Shot RShift+RAlt+RGUI", "category": "one_shot"},

// Special
{"code": "OS_MEH", "name": "One Shot Meh", "category": "one_shot"},
{"code": "OS_HYPR", "name": "One Shot Hyper", "category": "one_shot"},

// Parameterized
{"code": "OSM()", "name": "One Shot Modifier", "category": "one_shot", "pattern": "OSM\\((MOD_[LR]?[A-Z]+(?:\\s*\\|\\s*MOD_[LR]?[A-Z]+)*)\\)", "description": "Hold modifier for one keypress"}
```

### 4. RGB Lighting (25 keys)

From `RGB.md`:

```json
// Underglow
{"code": "UG_TOGG", "name": "RGB Toggle", "category": "rgb", "aliases": ["QK_UNDERGLOW_TOGGLE"]},
{"code": "UG_NEXT", "name": "RGB Mode Next", "category": "rgb", "aliases": ["QK_UNDERGLOW_MODE_NEXT"]},
{"code": "UG_PREV", "name": "RGB Mode Prev", "category": "rgb", "aliases": ["QK_UNDERGLOW_MODE_PREVIOUS"]},
{"code": "UG_HUEU", "name": "RGB Hue Up", "category": "rgb", "aliases": ["QK_UNDERGLOW_HUE_UP"]},
{"code": "UG_HUED", "name": "RGB Hue Down", "category": "rgb", "aliases": ["QK_UNDERGLOW_HUE_DOWN"]},
{"code": "UG_SATU", "name": "RGB Sat Up", "category": "rgb", "aliases": ["QK_UNDERGLOW_SATURATION_UP"]},
{"code": "UG_SATD", "name": "RGB Sat Down", "category": "rgb", "aliases": ["QK_UNDERGLOW_SATURATION_DOWN"]},
{"code": "UG_VALU", "name": "RGB Val Up", "category": "rgb", "aliases": ["QK_UNDERGLOW_VALUE_UP"]},
{"code": "UG_VALD", "name": "RGB Val Down", "category": "rgb", "aliases": ["QK_UNDERGLOW_VALUE_DOWN"]},
{"code": "UG_SPDU", "name": "RGB Speed Up", "category": "rgb", "aliases": ["QK_UNDERGLOW_SPEED_UP"]},
{"code": "UG_SPDD", "name": "RGB Speed Down", "category": "rgb", "aliases": ["QK_UNDERGLOW_SPEED_DOWN"]},

// RGB Matrix
{"code": "RM_ON", "name": "Matrix On", "category": "rgb", "aliases": ["QK_RGB_MATRIX_ON"]},
{"code": "RM_OFF", "name": "Matrix Off", "category": "rgb", "aliases": ["QK_RGB_MATRIX_OFF"]},
{"code": "RM_TOGG", "name": "Matrix Toggle", "category": "rgb", "aliases": ["QK_RGB_MATRIX_TOGGLE"]},
{"code": "RM_NEXT", "name": "Matrix Mode Next", "category": "rgb", "aliases": ["QK_RGB_MATRIX_MODE_NEXT"]},
{"code": "RM_PREV", "name": "Matrix Mode Prev", "category": "rgb", "aliases": ["QK_RGB_MATRIX_MODE_PREVIOUS"]},
{"code": "RM_HUEU", "name": "Matrix Hue Up", "category": "rgb", "aliases": ["QK_RGB_MATRIX_HUE_UP"]},
{"code": "RM_HUED", "name": "Matrix Hue Down", "category": "rgb", "aliases": ["QK_RGB_MATRIX_HUE_DOWN"]},
{"code": "RM_SATU", "name": "Matrix Sat Up", "category": "rgb", "aliases": ["QK_RGB_MATRIX_SATURATION_UP"]},
{"code": "RM_SATD", "name": "Matrix Sat Down", "category": "rgb", "aliases": ["QK_RGB_MATRIX_SATURATION_DOWN"]},
{"code": "RM_VALU", "name": "Matrix Val Up", "category": "rgb", "aliases": ["QK_RGB_MATRIX_VALUE_UP"]},
{"code": "RM_VALD", "name": "Matrix Val Down", "category": "rgb", "aliases": ["QK_RGB_MATRIX_VALUE_DOWN"]},
{"code": "RM_SPDU", "name": "Matrix Speed Up", "category": "rgb", "aliases": ["QK_RGB_MATRIX_SPEED_UP"]},
{"code": "RM_SPDD", "name": "Matrix Speed Down", "category": "rgb", "aliases": ["QK_RGB_MATRIX_SPEED_DOWN"]}
```

### 5. Backlight (7 keys)

```json
{"code": "BL_TOGG", "name": "Backlight Toggle", "category": "backlight", "aliases": ["QK_BACKLIGHT_TOGGLE"]},
{"code": "BL_STEP", "name": "Backlight Step", "category": "backlight", "aliases": ["QK_BACKLIGHT_STEP"]},
{"code": "BL_ON", "name": "Backlight On", "category": "backlight", "aliases": ["QK_BACKLIGHT_ON"]},
{"code": "BL_OFF", "name": "Backlight Off", "category": "backlight", "aliases": ["QK_BACKLIGHT_OFF"]},
{"code": "BL_UP", "name": "Backlight Up", "category": "backlight", "aliases": ["QK_BACKLIGHT_UP"]},
{"code": "BL_DOWN", "name": "Backlight Down", "category": "backlight", "aliases": ["QK_BACKLIGHT_DOWN"]},
{"code": "BL_BRTG", "name": "Backlight Breathing", "category": "backlight", "aliases": ["QK_BACKLIGHT_TOGGLE_BREATHING"]}
```

### 6. Audio/System (15 keys)

```json
// Audio
{"code": "AU_ON", "name": "Audio On", "category": "audio", "aliases": ["QK_AUDIO_ON"]},
{"code": "AU_OFF", "name": "Audio Off", "category": "audio", "aliases": ["QK_AUDIO_OFF"]},
{"code": "AU_TOGG", "name": "Audio Toggle", "category": "audio", "aliases": ["QK_AUDIO_TOGGLE"]},
{"code": "CK_TOGG", "name": "Clicky Toggle", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_TOGGLE"]},
{"code": "CK_ON", "name": "Clicky On", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_ON"]},
{"code": "CK_OFF", "name": "Clicky Off", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_OFF"]},
{"code": "CK_UP", "name": "Clicky Up", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_UP"]},
{"code": "CK_DOWN", "name": "Clicky Down", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_DOWN"]},
{"code": "CK_RST", "name": "Clicky Reset", "category": "audio", "aliases": ["QK_AUDIO_CLICKY_RESET"]},

// Music mode
{"code": "MU_ON", "name": "Music On", "category": "audio", "aliases": ["QK_MUSIC_ON"]},
{"code": "MU_OFF", "name": "Music Off", "category": "audio", "aliases": ["QK_MUSIC_OFF"]},
{"code": "MU_TOGG", "name": "Music Toggle", "category": "audio", "aliases": ["QK_MUSIC_TOGGLE"]},
{"code": "MU_NEXT", "name": "Music Mode Next", "category": "audio", "aliases": ["QK_MUSIC_MODE_NEXT"]},
{"code": "AU_NEXT", "name": "Audio Voice Next", "category": "audio", "aliases": ["QK_AUDIO_VOICE_NEXT"]},
{"code": "AU_PREV", "name": "Audio Voice Prev", "category": "audio", "aliases": ["QK_AUDIO_VOICE_PREVIOUS"]}
```

### 7. Auto Shift & Autocorrect (6 keys)

```json
{"code": "AS_DOWN", "name": "Auto Shift Down", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_DOWN"]},
{"code": "AS_UP", "name": "Auto Shift Up", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_UP"]},
{"code": "AS_RPT", "name": "Auto Shift Report", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_REPORT"]},
{"code": "AS_ON", "name": "Auto Shift On", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_ON"]},
{"code": "AS_OFF", "name": "Auto Shift Off", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_OFF"]},
{"code": "AS_TOGG", "name": "Auto Shift Toggle", "category": "advanced", "aliases": ["QK_AUTO_SHIFT_TOGGLE"]},

{"code": "AC_ON", "name": "Autocorrect On", "category": "advanced", "aliases": ["QK_AUTOCORRECT_ON"]},
{"code": "AC_OFF", "name": "Autocorrect Off", "category": "advanced", "aliases": ["QK_AUTOCORRECT_OFF"]},
{"code": "AC_TOGG", "name": "Autocorrect Toggle", "category": "advanced", "aliases": ["QK_AUTOCORRECT_TOGGLE"]}
```

### 8. US ANSI Shifted Symbols (21 keys)

From `US ANSI Shifted Symbols.md`:

```json
{"code": "KC_TILD", "name": "~", "category": "shifted", "aliases": ["KC_TILDE"]},
{"code": "KC_EXLM", "name": "!", "category": "shifted", "aliases": ["KC_EXCLAIM"]},
{"code": "KC_AT", "name": "@", "category": "shifted"},
{"code": "KC_HASH", "name": "#", "category": "shifted"},
{"code": "KC_DLR", "name": "$", "category": "shifted", "aliases": ["KC_DOLLAR"]},
{"code": "KC_PERC", "name": "%", "category": "shifted", "aliases": ["KC_PERCENT"]},
{"code": "KC_CIRC", "name": "^", "category": "shifted", "aliases": ["KC_CIRCUMFLEX"]},
{"code": "KC_AMPR", "name": "&", "category": "shifted", "aliases": ["KC_AMPERSAND"]},
{"code": "KC_ASTR", "name": "*", "category": "shifted", "aliases": ["KC_ASTERISK"]},
{"code": "KC_LPRN", "name": "(", "category": "shifted", "aliases": ["KC_LEFT_PAREN"]},
{"code": "KC_RPRN", "name": ")", "category": "shifted", "aliases": ["KC_RIGHT_PAREN"]},
{"code": "KC_UNDS", "name": "_", "category": "shifted", "aliases": ["KC_UNDERSCORE"]},
{"code": "KC_PLUS", "name": "+", "category": "shifted"},
{"code": "KC_LCBR", "name": "{", "category": "shifted", "aliases": ["KC_LEFT_CURLY_BRACE"]},
{"code": "KC_RCBR", "name": "}", "category": "shifted", "aliases": ["KC_RIGHT_CURLY_BRACE"]},
{"code": "KC_PIPE", "name": "|", "category": "shifted"},
{"code": "KC_COLN", "name": ":", "category": "shifted", "aliases": ["KC_COLON"]},
{"code": "KC_DQUO", "name": "\"", "category": "shifted", "aliases": ["KC_DOUBLE_QUOTE", "KC_DQT"]},
{"code": "KC_LABK", "name": "<", "category": "shifted", "aliases": ["KC_LEFT_ANGLE_BRACKET", "KC_LT"]},
{"code": "KC_RABK", "name": ">", "category": "shifted", "aliases": ["KC_RIGHT_ANGLE_BRACKET", "KC_GT"]},
{"code": "KC_QUES", "name": "?", "category": "shifted", "aliases": ["KC_QUESTION"]}
```

### 9. Modifier Combinations (30 keys)

From `Modifiers.md`:

```json
// Left modifier + keycode (parameterized)
{"code": "LCTL()", "name": "Ctrl+", "category": "mod_combo", "pattern": "LCTL\\(([A-Z0-9_]+)\\)", "aliases": ["C()"]},
{"code": "LSFT()", "name": "Shift+", "category": "mod_combo", "pattern": "LSFT\\(([A-Z0-9_]+)\\)", "aliases": ["S()"]},
{"code": "LALT()", "name": "Alt+", "category": "mod_combo", "pattern": "LALT\\(([A-Z0-9_]+)\\)", "aliases": ["A()", "LOPT()"]},
{"code": "LGUI()", "name": "GUI+", "category": "mod_combo", "pattern": "LGUI\\(([A-Z0-9_]+)\\)", "aliases": ["G()", "LCMD()", "LWIN()"]},

// Left combos
{"code": "LCS()", "name": "Ctrl+Shift+", "category": "mod_combo", "pattern": "LCS\\(([A-Z0-9_]+)\\)"},
{"code": "LCA()", "name": "Ctrl+Alt+", "category": "mod_combo", "pattern": "LCA\\(([A-Z0-9_]+)\\)"},
{"code": "LCG()", "name": "Ctrl+GUI+", "category": "mod_combo", "pattern": "LCG\\(([A-Z0-9_]+)\\)"},
{"code": "LSA()", "name": "Shift+Alt+", "category": "mod_combo", "pattern": "LSA\\(([A-Z0-9_]+)\\)"},
{"code": "LSG()", "name": "Shift+GUI+", "category": "mod_combo", "pattern": "LSG\\(([A-Z0-9_]+)\\)"},
{"code": "LAG()", "name": "Alt+GUI+", "category": "mod_combo", "pattern": "LAG\\(([A-Z0-9_]+)\\)"},
{"code": "LCSG()", "name": "Ctrl+Shift+GUI+", "category": "mod_combo", "pattern": "LCSG\\(([A-Z0-9_]+)\\)"},
{"code": "LCAG()", "name": "Ctrl+Alt+GUI+", "category": "mod_combo", "pattern": "LCAG\\(([A-Z0-9_]+)\\)"},
{"code": "LSAG()", "name": "Shift+Alt+GUI+", "category": "mod_combo", "pattern": "LSAG\\(([A-Z0-9_]+)\\)"},

// Right modifier + keycode
{"code": "RCTL()", "name": "RCtrl+", "category": "mod_combo", "pattern": "RCTL\\(([A-Z0-9_]+)\\)"},
{"code": "RSFT()", "name": "RShift+", "category": "mod_combo", "pattern": "RSFT\\(([A-Z0-9_]+)\\)"},
{"code": "RALT()", "name": "RAlt+", "category": "mod_combo", "pattern": "RALT\\(([A-Z0-9_]+)\\)", "aliases": ["ROPT()", "ALGR()"]},
{"code": "RGUI()", "name": "RGUI+", "category": "mod_combo", "pattern": "RGUI\\(([A-Z0-9_]+)\\)", "aliases": ["RCMD()", "RWIN()"]},

// Right combos
{"code": "RCS()", "name": "RCtrl+RShift+", "category": "mod_combo", "pattern": "RCS\\(([A-Z0-9_]+)\\)"},
{"code": "RCA()", "name": "RCtrl+RAlt+", "category": "mod_combo", "pattern": "RCA\\(([A-Z0-9_]+)\\)"},
{"code": "RCG()", "name": "RCtrl+RGUI+", "category": "mod_combo", "pattern": "RCG\\(([A-Z0-9_]+)\\)"},
{"code": "RSA()", "name": "RShift+RAlt+", "category": "mod_combo", "pattern": "RSA\\(([A-Z0-9_]+)\\)"},
{"code": "RSG()", "name": "RShift+RGUI+", "category": "mod_combo", "pattern": "RSG\\(([A-Z0-9_]+)\\)"},
{"code": "RAG()", "name": "RAlt+RGUI+", "category": "mod_combo", "pattern": "RAG\\(([A-Z0-9_]+)\\)"},
{"code": "RCSG()", "name": "RCtrl+RShift+RGUI+", "category": "mod_combo", "pattern": "RCSG\\(([A-Z0-9_]+)\\)"},
{"code": "RCAG()", "name": "RCtrl+RAlt+RGUI+", "category": "mod_combo", "pattern": "RCAG\\(([A-Z0-9_]+)\\)"},
{"code": "RSAG()", "name": "RShift+RAlt+RGUI+", "category": "mod_combo", "pattern": "RSAG\\(([A-Z0-9_]+)\\)"},

// Special combos
{"code": "MEH()", "name": "Meh+", "category": "mod_combo", "pattern": "MEH\\(([A-Z0-9_]+)\\)", "description": "Ctrl+Shift+Alt"},
{"code": "HYPR()", "name": "Hyper+", "category": "mod_combo", "pattern": "HYPR\\(([A-Z0-9_]+)\\)", "description": "Ctrl+Shift+Alt+GUI"},

// Standalone Meh/Hyper
{"code": "KC_MEH", "name": "Meh", "category": "modifiers", "description": "Ctrl+Shift+Alt"},
{"code": "KC_HYPR", "name": "Hyper", "category": "modifiers", "description": "Ctrl+Shift+Alt+GUI"}
```

### 10. Mod-Tap Keys (28 keys)

From `Mod Tap.md`:

```json
// Left mod-tap
{"code": "LCTL_T()", "name": "Ctrl-Tap", "category": "mod_tap", "pattern": "LCTL_T\\(([A-Z0-9_]+)\\)", "aliases": ["CTL_T()"]},
{"code": "LSFT_T()", "name": "Shift-Tap", "category": "mod_tap", "pattern": "LSFT_T\\(([A-Z0-9_]+)\\)", "aliases": ["SFT_T()"]},
{"code": "LALT_T()", "name": "Alt-Tap", "category": "mod_tap", "pattern": "LALT_T\\(([A-Z0-9_]+)\\)", "aliases": ["ALT_T()", "LOPT_T()", "OPT_T()"]},
{"code": "LGUI_T()", "name": "GUI-Tap", "category": "mod_tap", "pattern": "LGUI_T\\(([A-Z0-9_]+)\\)", "aliases": ["GUI_T()", "LCMD_T()", "LWIN_T()", "CMD_T()", "WIN_T()"]},

// Left combo mod-tap
{"code": "LCS_T()", "name": "Ctrl+Shift-Tap", "category": "mod_tap", "pattern": "LCS_T\\(([A-Z0-9_]+)\\)"},
{"code": "LCA_T()", "name": "Ctrl+Alt-Tap", "category": "mod_tap", "pattern": "LCA_T\\(([A-Z0-9_]+)\\)"},
{"code": "LCG_T()", "name": "Ctrl+GUI-Tap", "category": "mod_tap", "pattern": "LCG_T\\(([A-Z0-9_]+)\\)"},
{"code": "LSA_T()", "name": "Shift+Alt-Tap", "category": "mod_tap", "pattern": "LSA_T\\(([A-Z0-9_]+)\\)"},
{"code": "LSG_T()", "name": "Shift+GUI-Tap", "category": "mod_tap", "pattern": "LSG_T\\(([A-Z0-9_]+)\\)"},
{"code": "LAG_T()", "name": "Alt+GUI-Tap", "category": "mod_tap", "pattern": "LAG_T\\(([A-Z0-9_]+)\\)"},
{"code": "LCSG_T()", "name": "Ctrl+Shift+GUI-Tap", "category": "mod_tap", "pattern": "LCSG_T\\(([A-Z0-9_]+)\\)"},
{"code": "LCAG_T()", "name": "Ctrl+Alt+GUI-Tap", "category": "mod_tap", "pattern": "LCAG_T\\(([A-Z0-9_]+)\\)"},
{"code": "LSAG_T()", "name": "Shift+Alt+GUI-Tap", "category": "mod_tap", "pattern": "LSAG_T\\(([A-Z0-9_]+)\\)"},

// Right mod-tap
{"code": "RCTL_T()", "name": "RCtrl-Tap", "category": "mod_tap", "pattern": "RCTL_T\\(([A-Z0-9_]+)\\)"},
{"code": "RSFT_T()", "name": "RShift-Tap", "category": "mod_tap", "pattern": "RSFT_T\\(([A-Z0-9_]+)\\)"},
{"code": "RALT_T()", "name": "RAlt-Tap", "category": "mod_tap", "pattern": "RALT_T\\(([A-Z0-9_]+)\\)", "aliases": ["ROPT_T()", "ALGR_T()"]},
{"code": "RGUI_T()", "name": "RGUI-Tap", "category": "mod_tap", "pattern": "RGUI_T\\(([A-Z0-9_]+)\\)", "aliases": ["RCMD_T()", "RWIN_T()"]},

// Right combo mod-tap
{"code": "RCS_T()", "name": "RCtrl+RShift-Tap", "category": "mod_tap", "pattern": "RCS_T\\(([A-Z0-9_]+)\\)"},
{"code": "RCA_T()", "name": "RCtrl+RAlt-Tap", "category": "mod_tap", "pattern": "RCA_T\\(([A-Z0-9_]+)\\)"},
{"code": "RCG_T()", "name": "RCtrl+RGUI-Tap", "category": "mod_tap", "pattern": "RCG_T\\(([A-Z0-9_]+)\\)"},
{"code": "RSA_T()", "name": "RShift+RAlt-Tap", "category": "mod_tap", "pattern": "RSA_T\\(([A-Z0-9_]+)\\)"},
{"code": "RSG_T()", "name": "RShift+RGUI-Tap", "category": "mod_tap", "pattern": "RSG_T\\(([A-Z0-9_]+)\\)"},
{"code": "RAG_T()", "name": "RAlt+RGUI-Tap", "category": "mod_tap", "pattern": "RAG_T\\(([A-Z0-9_]+)\\)"},
{"code": "RCSG_T()", "name": "RCtrl+RShift+RGUI-Tap", "category": "mod_tap", "pattern": "RCSG_T\\(([A-Z0-9_]+)\\)"},
{"code": "RCAG_T()", "name": "RCtrl+RAlt+RGUI-Tap", "category": "mod_tap", "pattern": "RCAG_T\\(([A-Z0-9_]+)\\)"},
{"code": "RSAG_T()", "name": "RShift+RAlt+RGUI-Tap", "category": "mod_tap", "pattern": "RSAG_T\\(([A-Z0-9_]+)\\)"},

// Special mod-tap
{"code": "MEH_T()", "name": "Meh-Tap", "category": "mod_tap", "pattern": "MEH_T\\(([A-Z0-9_]+)\\)"},
{"code": "HYPR_T()", "name": "Hyper-Tap", "category": "mod_tap", "pattern": "HYPR_T\\(([A-Z0-9_]+)\\)"}
```

### 11. Advanced Keys (5 keys)

```json
// Leader key
{"code": "QK_LEAD", "name": "Leader Key", "category": "advanced", "aliases": ["QK_LEADER"]},

// Layer lock
{"code": "QK_LOCK", "name": "Key Lock", "category": "advanced", "description": "Hold down next key until pressed again"},
{"code": "QK_LLCK", "name": "Layer Lock", "category": "advanced", "aliases": ["QK_LAYER_LOCK"], "description": "Lock/unlock highest layer"},

// Macros (placeholder for macro slots)
{"code": "QK_MACRO_0", "name": "Macro 0", "category": "advanced"},
{"code": "QK_MACRO_1", "name": "Macro 1", "category": "advanced"},
// ... up to QK_MACRO_31
```

---

## User Stories

### US1: Complete Basic Keycodes (Priority: P1)

**As a keyboard user,**  
**I want** access to all standard HID keycodes including international and editing keys,  
**So that** I can configure any key function I need.

**Why this priority**: Core functionality - users need basic keycodes to build useful layouts.

**Acceptance Criteria:**
1. All ~200 basic keycodes from QMK reference are available
2. International keys (KC_INT1-9, KC_LNG1-9) available
3. System keys (Power, Sleep, Wake) available
4. Editing keys (Cut, Copy, Paste, Undo) available
5. All keycodes have proper aliases registered

---

### US2: Mouse Keys Support (Priority: P1)

**As a user without a mouse,**  
**I want** to assign mouse functions to keys,  
**So that** I can control my cursor from the keyboard.

**Why this priority**: Essential for users who rely on keyboard-only navigation.

**Acceptance Criteria:**
1. Mouse cursor keys (MS_UP/DOWN/LEFT/RGHT) available
2. Mouse buttons 1-8 available
3. Scroll wheel keys available
4. Acceleration modes available

---

### US3: RGB Control Keys (Priority: P2)

**As a user with RGB keyboard,**  
**I want** to assign RGB control keys,  
**So that** I can change lighting without external software.

**Why this priority**: Important for keyboards with RGB, but not all users have RGB.

**Acceptance Criteria:**
1. Underglow toggle/mode/HSV controls available
2. RGB Matrix toggle/mode/HSV controls available
3. Speed controls available

---

### US4: One-Shot Modifiers (Priority: P2)

**As an ergonomic-conscious user,**  
**I want** to use one-shot modifiers,  
**So that** I can reduce hand strain from holding modifier keys.

**Acceptance Criteria:**
1. All OS_* one-shot modifiers available
2. OSM() parameterized function works
3. One-shot toggle/on/off available

---

### US5: Mod-Tap Keys (Priority: P2)

**As a power user,**  
**I want** keys that act as modifiers when held and regular keys when tapped,  
**So that** I can use my home row for modifiers.

**Acceptance Criteria:**
1. All *_T() mod-tap functions available
2. Parameterized input flow works (select mod-tap, then select keycode)
3. MEH_T and HYPR_T available

---

### US6: Enhanced Category UI (Priority: P3)

**As a user,**  
**I want** a visual category tab bar in the keycode picker,  
**So that** I can easily browse and switch between keycode categories.

**Acceptance Criteria:**
1. Horizontal tab bar shows all categories
2. Left/Right arrows or Tab navigates categories
3. Current category is visually highlighted
4. "All" category shows all keycodes
5. Number key shortcuts still work (0-9)
6. Tab bar scrolls if too many categories

---

### US7: Shifted Symbols (Priority: P3)

**As a user,**  
**I want** to directly assign shifted symbols like @, #, $,  
**So that** I can create symbol layers without shift key.

**Acceptance Criteria:**
1. All 21 US ANSI shifted symbols available
2. Clearly displayed with actual symbol character

---

## Implementation Phases

### Phase 1: Database Expansion (8 hours)

1. **Add new categories** to `keycodes.json`
2. **Add basic keycodes** (60 new keys)
3. **Add mouse keys** (19 keys)
4. **Add shifted symbols** (21 keys)
5. **Update category count** from 8 to 19
6. **Test database loading** and validation

### Phase 2: One-Shot & RGB Keys (6 hours)

1. **Add one-shot keys** (35 keys)
2. **Add RGB keys** (25 keys)
3. **Add backlight keys** (7 keys)
4. **Add audio keys** (15 keys)
5. **Add advanced keys** (10 keys)

### Phase 3: Modifier Combos & Mod-Tap (8 hours)

1. **Add modifier combo patterns** (30 keys)
2. **Add mod-tap patterns** (28 keys)
3. **Implement sub-picker flow** for parameterized keycodes
4. **Handle keycode parameter input**
5. **Test pattern matching**

### Phase 4: Enhanced Category UI (10 hours)

1. **Design category tab component**
2. **Implement horizontal scrolling tabs**
3. **Add tab navigation** (Left/Right, Tab/Shift+Tab)
4. **Update help text** with new shortcuts
5. **Add visual highlighting** for active category
6. **Handle overflow** with scroll indicators

### Phase 5: Integration & Testing (6 hours)

1. **Firmware generator validation** - ensure new keycodes output correctly
2. **Pattern matching tests** for parameterized keycodes
3. **UI navigation tests**
4. **Manual testing checklist**
5. **Documentation updates**

**Total Estimate:** ~38 hours

---

## Technical Changes

### Files to Modify

| File | Changes |
|------|---------|
| `src/keycode_db/keycodes.json` | Add ~260 keycodes, 11 new categories |
| `src/keycode_db/mod.rs` | Update category handling, add param types |
| `src/tui/keycode_picker.rs` | Add tab UI, sub-picker for params |
| `src/firmware/generator.rs` | Validate new keycodes in output |

### New Data Structures

```rust
/// Parameter type for parameterized keycodes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamType {
    /// Layer reference (for MO, TG, LT, etc.)
    Layer,
    /// Keycode (for LCTL(kc), LT(layer, kc), etc.)
    Keycode,
    /// Modifier (for OSM(mod))
    Modifier,
}

/// Extended keycode definition
pub struct KeycodeDefinition {
    // ... existing fields ...
    
    /// Parameter type for parameterized keycodes
    pub param_type: Option<ParamType>,
}
```

### UI State Extension

```rust
pub struct KeycodePickerState {
    // ... existing fields ...
    
    /// Current category tab index (0 = All)
    pub category_index: usize,
    /// Tab scroll offset (for overflow handling)
    pub tab_scroll: usize,
    /// Sub-picker mode for parameterized keycodes
    pub sub_picker: Option<SubPickerState>,
}

pub struct SubPickerState {
    /// The parent keycode being configured (e.g., "LCTL()")
    pub parent_code: String,
    /// Search within sub-picker
    pub search: String,
    /// Selected index in sub-picker
    pub selected: usize,
}
```

---

## Success Criteria

### Must Have (P0)
- [ ] All ~400 keycodes from QMK reference available
- [ ] Mouse keys category fully implemented
- [ ] One-shot modifiers category implemented
- [ ] Mod-tap patterns working
- [ ] All existing functionality preserved
- [ ] Firmware generator accepts new keycodes

### Should Have (P1)
- [ ] Visual category tabs in picker
- [ ] Tab navigation working
- [ ] Parameterized keycode input flow
- [ ] RGB/backlight/audio keys available

### Nice to Have (P2)
- [ ] Category descriptions in tab tooltips
- [ ] Scrollable tab bar for overflow
- [ ] Category icons/abbreviations
- [ ] Quick-jump with number keys preserved

---

## Testing Strategy

### Unit Tests
- Keycode database loading with new entries
- Pattern matching for all parameterized keycodes
- Alias resolution for all keycodes
- Category filtering

### Integration Tests
- Sub-picker flow for mod-tap
- Firmware output with new keycodes
- Navigation between categories

### Manual Testing
- Browse all 19 categories
- Select keycodes from each category
- Create mod-tap keycode (e.g., `LCTL_T(KC_A)`)
- Create modifier combo (e.g., `MEH(KC_F1)`)
- Verify firmware compiles with new keycodes

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Too many categories clutters UI | Medium | Scrollable tabs, category grouping |
| Parameterized input too complex | Medium | Clear prompts, cancel option |
| Pattern regex errors | High | Comprehensive unit tests |
| Firmware rejects new keycodes | High | Test against QMK reference |
| Performance with 400+ keycodes | Low | Lazy loading, efficient search |

---

## References

- **QMK Keycodes Reference**: https://docs.qmk.fm/keycodes
- **Source Docs**: `src/keycode_db/keycodes_qmk/`
- **Current Implementation**: `src/keycode_db/keycodes.json`
- **Keycode Picker**: `src/tui/keycode_picker.rs`

---

**Status:** Specification complete, ready for implementation  
**Next Action:** Begin Phase 1, update keycodes.json with basic keycodes
