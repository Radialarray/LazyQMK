# Reference 0009: Platform-Specific Special Keys

> QMK "special" keycodes split into three behavior classes. Most users only encounter HID Consumer codes 0x00A8–0x00C2 — these are the macOS/Windows media & launcher keys. **Always ask the user which host platform they're targeting before recommending any of these.** See `SKILL.md` Mission Gate for the prompt.

---

## A. Cross-platform keys (work on every host)

These keycodes produce the same effect regardless of OS:

- Letters, digits, punctuation (KC_A–KC_Z, KC_0–KC_9, KC_MINS, KC_SLSH, …)
- Modifiers (KC_LSFT, KC_LCTL, KC_LALT, KC_LGUI + right-hand variants)
- Layer switching (`MO()`, `LT()`, `TG()`, `TO()`, `TT()`, `OSL()`, `DF()`)
- Mod-tap (`LCTL_T()`, `LALT_T()`, `LSFT_T()`, `LGUI_T()`, `MT()`, one-shot `OS_*`)
- Tap dance (`TD(name)`)
- Combos (`COMBO(n, kc)` style; see `references/0006-tap-hold-and-combos.md`)
- `KC_TRNS`, `KC_NO`
- `QK_BOOT` (`RESET`), `QK_REBOOT`, `EE_CLR` (system boot/eeprom — see B for OS-dependent system keys)
- Mouse keys (`MS_UP`, `MS_BTN1`, …) — require `MOUSEKEY_ENABLE = yes`, but behavior is host-agnostic

Modifiers are OS-agnostic **as keys**, but most modifier→key combos (e.g. `LGUI(KC_C)`, `LCG(KC_Q)`) only do something useful on the matching host OS. `LCG(KC_Q)` works as a lock shortcut on macOS (Ctrl+Cmd+Q); Windows uses `LGUI(KC_L)` for the same.

---

## B. Apple / HID Consumer codes

These travel over the USB HID **Consumer** usage page (`0x0C`). Hosts map them to OS-level functions. Most modern hosts accept them, but **the behavior diverges by OS**. Always confirm the host before suggesting.

| Alias        | Canonical name          | Hex    | macOS effect                  | Windows effect            | Linux effect                | Notes |
|--------------|-------------------------|--------|-------------------------------|---------------------------|-----------------------------|-------|
| `KC_MCTL`    | `KC_MISSION_CONTROL`    | `0xC1` | Mission Control               | noop (no remap) / iTunes  | noop (needs xkb/hwdb)       | Apple's Fn+F3. Treat as macOS-only. |
| `KC_LPAD`    | `KC_LAUNCHPAD`          | `0xC2` | Launchpad                     | noop                      | noop                        | Apple's Fn+F4. Treat as macOS-only. |
| `KC_ASST`    | `KC_ASSISTANT`          | `0xC0` | Siri (or noop if disabled)    | Cortana (Win10/11)        | noop                        | Cross-platform idea, OS-specific result. |
| `KC_BRIU`    | `KC_BRIGHTNESS_UP`      | `0xBD` | Brightness up                 | Brightness up             | Brightness up (usually)     | Often works on Linux laptops via hwdb. |
| `KC_BRID`    | `KC_BRIGHTNESS_DOWN`    | `0xBE` | Brightness down               | Brightness down           | Brightness down (usually)   | Same caveats as `KC_BRIU`. |
| `KC_MPLY`    | `KC_MEDIA_PLAY_PAUSE`   | `0xAE` | Play/Pause                    | Play/Pause                | Play/Pause                  | Universal media transport. |
| `KC_MSTP`    | `KC_MEDIA_STOP`         | `0xAD` | Stop                          | Stop                      | Stop                        | Universal. |
| `KC_MNXT`    | `KC_MEDIA_NEXT_TRACK`   | `0xAB` | Next                          | Next                      | Next                        | Universal. |
| `KC_MPRV`    | `KC_MEDIA_PREV_TRACK`   | `0xAC` | Previous                      | Previous                  | Previous                    | Universal. |
| `KC_VOLU`    | `KC_AUDIO_VOL_UP`       | `0xA9` | Volume up                     | Volume up                 | Volume up                   | Universal. |
| `KC_VOLD`    | `KC_AUDIO_VOL_DOWN`     | `0xAA` | Volume down                   | Volume down               | Volume down                 | Universal. |
| `KC_MUTE`    | `KC_AUDIO_MUTE`         | `0xA8` | Mute                          | Mute                      | Mute                        | Universal. |
| `KC_MFFD`    | `KC_MEDIA_FAST_FORWARD` | `0xBB` | Fast-forward                  | Fast-forward              | Fast-forward                | Universal transport. |
| `KC_MRWD`    | `KC_MEDIA_REWIND`       | `0xBC` | Rewind                        | Rewind                    | Rewind                      | Universal transport. |
| `KC_EJCT`    | `KC_MEDIA_EJECT`        | `0xB0` | Eject optical (often noop)    | Eject                     | Eject                       | Usually noop on SSD-only macs. |
| `KC_MAIL`    |                         | `0xB1` | Mail (opens default mail app) | Mail (opens Outlook)      | varies                      | Mac/Win target. |
| `KC_CALC`    | `KC_CALCULATOR`         | `0xB2` | Calculator                    | Calculator                | Calculator                  | Mac/Win target. |
| `KC_MYCM`    | `KC_MY_COMPUTER`        | `0xB3` | Finder at Home                | File Explorer             | File manager                | Mac/Win target. |
| `KC_WSCH`    | `KC_WWW_SEARCH`         | `0xB4` | Browser Search (or noop)      | Browser Search            | Browser Search              | Cross-platform-ish. |
| `KC_WHOM`    | `KC_WWW_HOME`           | `0xB5` | Browser Home                  | Browser Home              | Browser Home                | Universal. |
| `KC_WBAK`    | `KC_WWW_BACK`           | `0xB6` | Browser Back                  | Browser Back              | Browser Back                | Universal. |
| `KC_WFWD`    | `KC_WWW_FORWARD`        | `0xB7` | Browser Forward               | Browser Forward           | Browser Forward             | Universal. |
| `KC_WSTP`    | `KC_WWW_STOP`           | `0xB8` | Browser Stop                  | Browser Stop              | Browser Stop                | Universal. |
| `KC_WREF`    | `KC_WWW_REFRESH`        | `0xB9` | Browser Refresh               | Browser Refresh           | Browser Refresh             | Universal. |
| `KC_WFAV`    | `KC_WWW_FAVORITES`      | `0xBA` | Browser Favorites             | Browser Favorites         | Browser Favorites           | Universal. |
| `KC_CPNL`    | `KC_CONTROL_PANEL`      | `0xBF` | System Settings               | Control Panel / Settings  | System Settings (some DEs)  | Mac/Win target. |
| `KC_PWR`     | `KC_SYSTEM_POWER`       | `0xA5` | Power dialog (rare; lock-style)| Power menu (rare)        | Power dialog (rare)         | Use with caution — may shut off the user's box. |
| `KC_SLEP`    | `KC_SYSTEM_SLEEP`       | `0xA6` | Sleep                         | Sleep                     | Sleep                       | Use with caution. |
| `KC_WAKE`    | `KC_SYSTEM_WAKE`        | `0xA7` | Wake (works if lid open)      | Wake                      | Wake                        | Use with caution. |
| `KC_CPNL`   | `KC_CONTROL_PANEL`      | `0xBF` | System Settings               | Settings                  | DE Settings                 | "Control Panel" naming is Windows legacy. |

> Source: `qmk_firmware/quantum/keycodes.h` lines 268–302 (enum) and 925–960 (alias defines). Also reflected in `src/keycode_db/categories/media.json` (lines 189–220) and `system.json` (lines 36–58).

### Sub-section: Unicode / OS-specific input method

| Alias      | Canonical name           | Notes |
|------------|--------------------------|-------|
| `UC_MAC`   | `QK_UNICODE_MODE_MACOS`  | Switch Unicode input mode to macOS. Required before using Unicode hex entry on macOS. |
| `UC_LNX`   | (variants exist)         | Same idea, Linux variants. |
| `UC_WIN`   | (variants exist)         | Same idea, Windows variants. |

See `qmk_firmware/quantum/keycodes.h` line 726 (`QK_UNICODE_MODE_MACOS`) and line 1418 (`UC_MAC`).

---

## C. Common review pitfalls

These are the mistakes the skill has explicitly hit before. Encode them as guardrails.

1. **Never flag a `KC_*` keycode as "non-standard" without checking.** If you think a keycode is invalid:
   - Step 1: `rg "KC_<SUSPECT>" qmk_firmware/quantum/keycodes.h` — most aliases live here.
   - Step 2: `rg "<aliased-name>" qmk_firmware/quantum/keycodes.h` — also check the canonical form.
   - Step 3: `rg "<code>" src/keycode_db/categories/*.json` — the bundled db.
   - If ANY of those answers YES, the keycode is valid. Don't flag it.

2. **Compile success ≠ runtime effect.** `BL_TOG`/`RGB_TOG`/`KC_MCTL` all compile, but:
   - `BL_TOG` does nothing without `BACKLIGHT_ENABLE = yes` in the keyboard's `info.json`.
   - `KC_MCTL` does nothing on a Windows host without a consumer-remap driver.
   - `RGB_TOG` on an RGB-Matrix-only keyboard toggles the matrix immediately (works).
   Treat "does this work for the user" as a config question, never as a keycode-validity question.

3. **Don't suggest `LGUI(KC_UP)` as a "fix" for `KC_MCTL` without checking intent.** A macOS user who explicitly chose `KC_MCTL` may have Spotlight/3rd-party launchers listening for the raw HID keycode. Replacing it breaks that. Ask first.

4. **Don't assume QMK defines a keycode just because it doesn't appear in your session's mental shortlist.** The bundled db (`src/keycode_db/categories/`) is comprehensive; the aliases at `qmk_firmware/quantum/keycodes.h:925-960` are also canonical. When in doubt, both files are the source of truth.

5. **Don't conflate "feature enable" with "keycode enable".** If a user says "BL_TOGG does nothing on my Corne Choc Pro," the answer is rarely "BL_TOGG is invalid" — it's "your keyboard has no backlight driver." Reach for `info.json` first.

---

## D. Validating a "weird" keycode before flagging it

```bash
# 1. Does the bundled database know about it?
LAYOUT=~/Library/Application\ Support/LazyQMK/layouts/<file>.json
lazyqmk keycode --layout "$LAYOUT" --expr "KC_<SUSPECT>" --json

# 2. Does qmk_firmware recognize it (enum + aliases)?
rg "KC_<SUSPECT>" qmk_firmware/quantum/keycodes.h
rg "<lowercase-alias>" qmk_firmware/quantum/keycodes.h

# 3. Is it in the bundled category JSONs?
rg "<code>" src/keycode_db/categories/*.json

# If 1, 2, or 3 says YES → the keycode is valid. Don't flag it.
# Only an actual `keycode not found` from the CLI/parser is a real flag,
# and even then prefer to fix the spelling (e.g. KC_LSFT, not KC_LSHIFT).
```

If the layout references a keycode that NONE of the above recognize, it's a **custom `USER<N>` keycode** defined in the keyboard's own `keymap.c` (via `enum ... { MY_KEY = SAFE_RANGE };`). THAT is the only kind of "weird keycode" worth flagging — and the right response is "this requires a keyboard-side patch to compile on this fork," not "this keycode doesn't exist."

---

## E. Asking the platform question (Mission Gate)

The exact wording to use when prompting during the mission gate:

> **"Which host are you targeting?** macOS, Windows, Linux, iOS/iPadOS, or cross-platform (designed to work on any)? This decides which Mission Control / Control Panel / Spotlight / browser-launcher keys make sense, so I don't suggest `KC_MCTL` for a Windows user or `LGUI(KC_E)` for a Mac user."

Persist the answer in MISSION.md (new `## Host` block) and NOTES.md (`## User preferences: Host` line). If a previous session recorded a host, **always reuse it** unless the user explicitly says they've changed hosts.

When a host-specific recommendation is needed but the user hasn't answered, ask the question once and store the answer; do not re-ask on every layer decision.

---

## See also

- `references/0001-keycode-categories.md` — overall category table + media/system row
- `references/0007-tap-dance.md` — tap-dance on special keys (`TD(KC_MCTL)` is valid as the keycode argument)
- `qmk_firmware/quantum/keycodes.h` — canonical QMK keycode enumeration + aliases
- `src/keycode_db/categories/media.json`, `system.json` — bundled LazyQMK database
