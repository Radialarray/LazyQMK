# Implementation Tasks: Complete QMK Keycodes

## Phase 1: Database Expansion (Basic Keycodes)

### 1.1 Add New Categories
- [ ] Add `mouse` category - "Mouse cursor, buttons, wheel"
- [ ] Add `mod_combo` category - "Modifier + keycode combinations"
- [ ] Add `one_shot` category - "One-shot modifiers and layers"
- [ ] Add `rgb` category - "RGB lighting controls"
- [ ] Add `backlight` category - "Backlight controls"
- [ ] Add `audio` category - "Audio and music mode"
- [ ] Add `shifted` category - "US ANSI shifted symbols"
- [ ] Add `mod_tap` category - "Hold for mod, tap for key"
- [ ] Add `advanced` category - "Leader key, autocorrect, macros"
- [ ] Rename `special` to `layers` for clarity
- [ ] Add category shortcuts (0-9) where applicable

### 1.2 Add Missing Basic Keycodes (~60)
- [ ] International keys: KC_INT1 through KC_INT9
- [ ] Language keys: KC_LNG1 through KC_LNG9
- [ ] Locking keys: KC_LCAP, KC_LNUM, KC_LSCR
- [ ] Non-US keys: KC_NUHS, KC_NUBS
- [ ] Editing keys: KC_EXEC, KC_HELP, KC_SLCT, KC_STOP, KC_AGIN, KC_UNDO, KC_CUT, KC_COPY, KC_PSTE, KC_FIND
- [ ] Numpad extras: KC_PCMM, KC_PEQL
- [ ] System keys: KC_PWR, KC_SLEP, KC_WAKE

### 1.3 Add Media & Application Keys
- [ ] KC_EJCT, KC_MAIL, KC_CALC, KC_MYCM
- [ ] Browser keys: KC_WSCH, KC_WHOM, KC_WBAK, KC_WFWD, KC_WSTP, KC_WREF, KC_WFAV
- [ ] macOS keys: KC_MCTL, KC_LPAD, KC_ASST, KC_CPNL
- [ ] Media extras: KC_MFFD, KC_MRWD

### 1.4 Add Mouse Keys (19)
- [ ] Cursor: MS_UP, MS_DOWN, MS_LEFT, MS_RGHT
- [ ] Buttons: MS_BTN1 through MS_BTN8
- [ ] Wheel: MS_WHLU, MS_WHLD, MS_WHLL, MS_WHLR
- [ ] Acceleration: MS_ACL0, MS_ACL1, MS_ACL2

### 1.5 Add US ANSI Shifted Symbols (21)
- [ ] KC_TILD, KC_EXLM, KC_AT, KC_HASH, KC_DLR
- [ ] KC_PERC, KC_CIRC, KC_AMPR, KC_ASTR
- [ ] KC_LPRN, KC_RPRN, KC_UNDS, KC_PLUS
- [ ] KC_LCBR, KC_RCBR, KC_PIPE, KC_COLN
- [ ] KC_DQUO, KC_LABK, KC_RABK, KC_QUES

---

## Phase 2: One-Shot & RGB Keys

### 2.1 Add One-Shot Keys (35)
- [ ] Toggle: OS_TOGG, OS_ON, OS_OFF
- [ ] Left mods: OS_LCTL, OS_LSFT, OS_LALT, OS_LGUI
- [ ] Left combos: OS_LCS, OS_LCA, OS_LCG, OS_LSA, OS_LSG, OS_LAG, OS_LCSG, OS_LCAG, OS_LSAG
- [ ] Right mods: OS_RCTL, OS_RSFT, OS_RALT, OS_RGUI
- [ ] Right combos: OS_RCS, OS_RCA, OS_RCG, OS_RSA, OS_RSG, OS_RAG, OS_RCSG, OS_RCAG, OS_RSAG
- [ ] Special: OS_MEH, OS_HYPR
- [ ] Parameterized: OSM() with pattern

### 2.2 Add RGB Keys (25)
- [ ] Underglow: UG_TOGG, UG_NEXT, UG_PREV, UG_HUEU, UG_HUED, UG_SATU, UG_SATD, UG_VALU, UG_VALD, UG_SPDU, UG_SPDD
- [ ] RGB Matrix: RM_ON, RM_OFF, RM_TOGG, RM_NEXT, RM_PREV, RM_HUEU, RM_HUED, RM_SATU, RM_SATD, RM_VALU, RM_VALD, RM_SPDU, RM_SPDD

### 2.3 Add Backlight Keys (7)
- [ ] BL_TOGG, BL_STEP, BL_ON, BL_OFF, BL_UP, BL_DOWN, BL_BRTG

### 2.4 Add Audio Keys (15)
- [ ] Audio: AU_ON, AU_OFF, AU_TOGG
- [ ] Clicky: CK_TOGG, CK_ON, CK_OFF, CK_UP, CK_DOWN, CK_RST
- [ ] Music: MU_ON, MU_OFF, MU_TOGG, MU_NEXT
- [ ] Voice: AU_NEXT, AU_PREV

### 2.5 Add Advanced Keys (10)
- [ ] Auto Shift: AS_DOWN, AS_UP, AS_RPT, AS_ON, AS_OFF, AS_TOGG
- [ ] Autocorrect: AC_ON, AC_OFF, AC_TOGG
- [ ] Leader: QK_LEAD
- [ ] Layer Lock: QK_LOCK, QK_LLCK

---

## Phase 3: Modifier Combos & Mod-Tap

### 3.1 Add Modifier Combo Keys (30)
- [ ] Left single: LCTL(), LSFT(), LALT(), LGUI()
- [ ] Left combos: LCS(), LCA(), LCG(), LSA(), LSG(), LAG(), LCSG(), LCAG(), LSAG()
- [ ] Right single: RCTL(), RSFT(), RALT(), RGUI()
- [ ] Right combos: RCS(), RCA(), RCG(), RSA(), RSG(), RAG(), RCSG(), RCAG(), RSAG()
- [ ] Special: MEH(), HYPR()
- [ ] Standalone: KC_MEH, KC_HYPR

### 3.2 Add Mod-Tap Keys (28)
- [ ] Left single: LCTL_T(), LSFT_T(), LALT_T(), LGUI_T()
- [ ] Left combos: LCS_T(), LCA_T(), LCG_T(), LSA_T(), LSG_T(), LAG_T(), LCSG_T(), LCAG_T(), LSAG_T()
- [ ] Right single: RCTL_T(), RSFT_T(), RALT_T(), RGUI_T()
- [ ] Right combos: RCS_T(), RCA_T(), RCG_T(), RSA_T(), RSG_T(), RAG_T(), RCSG_T(), RCAG_T(), RSAG_T()
- [ ] Special: MEH_T(), HYPR_T()

### 3.3 Update KeycodeDb for Parameterized Keycodes
- [ ] Add `param_type` field to `KeycodeDefinition`
- [ ] Implement param type detection from patterns
- [ ] Update `is_valid()` for new patterns
- [ ] Update `get()` for new patterns

---

## Phase 4: Enhanced Category UI

### 4.1 Design Tab Component
- [ ] Create `CategoryTabBar` widget
- [ ] Calculate tab widths based on category names
- [ ] Handle overflow with scroll indicators (◄ ►)
- [ ] Highlight active tab

### 4.2 Update KeycodePickerState
- [ ] Add `category_index: usize` (0 = All)
- [ ] Add `tab_scroll: usize` for overflow
- [ ] Add `sub_picker: Option<SubPickerState>`
- [ ] Update `reset()` method

### 4.3 Implement Tab Navigation
- [ ] Left/Right arrows change category
- [ ] Tab/Shift+Tab also works
- [ ] Number keys (0-9) for quick jump
- [ ] Wrap around at ends

### 4.4 Implement Sub-Picker for Parameters
- [ ] Detect when selected keycode needs parameter
- [ ] Open sub-picker with filtered keycodes
- [ ] Build final keycode string (e.g., `LCTL(KC_A)`)
- [ ] Allow cancel to return to parent picker

### 4.5 Update Rendering
- [ ] Render tab bar at top
- [ ] Update help text with new shortcuts
- [ ] Show current category description
- [ ] Handle narrow terminals gracefully

---

## Phase 5: Integration & Testing

### 5.1 Firmware Generator Updates
- [ ] Verify new keycodes output correctly
- [ ] Test parameterized keycodes in keymap output
- [ ] Validate against QMK syntax

### 5.2 Unit Tests
- [ ] Test loading expanded keycodes.json
- [ ] Test all new regex patterns
- [ ] Test alias resolution
- [ ] Test category filtering
- [ ] Test search across new keycodes

### 5.3 Integration Tests
- [ ] Test sub-picker flow
- [ ] Test tab navigation
- [ ] Test firmware compilation with new keycodes

### 5.4 Manual Testing Checklist
- [ ] Browse all 19 categories
- [ ] Select keycode from each category
- [ ] Create mod-tap: `LCTL_T(KC_A)`
- [ ] Create mod combo: `MEH(KC_F1)`
- [ ] Create one-shot: `OSM(MOD_LSFT)`
- [ ] Assign mouse key
- [ ] Assign RGB control key
- [ ] Build firmware with mixed keycodes
- [ ] Verify firmware compiles successfully

### 5.5 Documentation
- [ ] Update help overlay with new categories
- [ ] Update README with keycode coverage
- [ ] Document parameterized keycode usage

---

## Keycode Count Summary

| Category | Current | Added | Total |
|----------|---------|-------|-------|
| basic | 48 | 35 | 83 |
| navigation | 10 | 0 | 10 |
| symbols | 11 | 2 | 13 |
| shifted | 0 | 21 | 21 |
| function | 24 | 0 | 24 |
| numpad | 18 | 2 | 20 |
| modifiers | 8 | 2 | 10 |
| mod_combo | 0 | 30 | 30 |
| mod_tap | 0 | 28 | 28 |
| layers | 7 | 2 | 9 |
| one_shot | 0 | 35 | 35 |
| mouse | 0 | 19 | 19 |
| media | 10 | 15 | 25 |
| rgb | 0 | 25 | 25 |
| backlight | 0 | 7 | 7 |
| audio | 0 | 15 | 15 |
| system | 4 | 3 | 7 |
| advanced | 0 | 10 | 10 |
| **TOTAL** | **~130** | **~260** | **~390** |

---

## Notes

### Priority Order
1. Phase 1.2-1.5: Basic, mouse, shifted (most requested)
2. Phase 2: One-shot, RGB (common features)
3. Phase 3: Mod combos, mod-tap (power user features)
4. Phase 4: UI improvements (polish)

### Breaking Changes
- None expected - all additions are backward compatible
- Existing keycodes unchanged
- Old layouts will continue to work

### Performance Considerations
- 3x more keycodes but search is already O(n)
- Consider indexing by category for faster filtered search
- Pattern matching remains lazy (only when needed)
