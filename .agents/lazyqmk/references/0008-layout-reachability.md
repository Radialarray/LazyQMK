# Reference 0008: Layout Reachability

> Per-layout character → keycode reachability for the three most common European layouts. Use this to identify **duplicates** (same character reachable via two keycodes), **redundant dedicated keys** (already reachable via Shift/AltGr on the base layer), and **missing characters** (commonly-needed ones not currently mapped). Combine with `scripts/layout-reachability.sh <layout> <char>` for CLI lookups and `scripts/audit-layout.sh <layout> <layout.json>` for a full layer-aware audit.

## Auditing with layer context (key principle)

A character is **reachable** if any of the following can produce it:

* The bare keycode on any layer (e.g. `DE_MINS` on Base layer 1,13 produces `-`)
* A **Shift** combination of any base keycode (e.g. `S(DE_MINS)` produces `_`)
* An **AltGr** combination (e.g. `ALGR(DE_M)` produces `µ`)
* A **Shift+AltGr** combination (e.g. `S(ALGR(DE_2))` produces `²`)
* A direct keycode alias on any layer (e.g. `DE_UNDS = S(DE_MINS)` is a keycode the user can place)

The audit answers **two questions** for every character:

1. **On which layer(s) is this character reachable?** (Layer 0 base, 1 Symbols, 2 Navigation, 3 Numbers, 4 Code, 5 Globals)
2. **What's the suggested slot** if the character is missing entirely?

Suggested slot logic:

| Character category | Target layer |
|---|---|
| `!`, `^`, `_`, `{`, `}`, `[`, `]`, `(`, `)`, `\|`, `\\`, `~`, `'`, `"`, `;`, `:`, `<`, `>`, `+` | **Code** |
| `×`, `÷`, `±`, `°`, `µ`, `²`, `³`, `π`, `*`, `=`, `-`, `+`, `.` | **Numbers** |
| `ä`, `ö`, `ü`, `Ä`, `Ö`, `Ü`, `ß`, `€`, `§`, `@` | **Symbols** |
| `←`, `→`, `↑`, `↓`, `Home`, `End`, `PageUp`, `PageDown` | **Navigation** |
| Anything else | **Symbols** (default) |

The first free `KC_TRNS` slot on the target layer is suggested. If no free slot exists, the audit says so — you must re-arrange or settle for a different layer.

## Source of truth

The keycode tables below are derived directly from the QMK `keymap_extras` headers in the LazyQMK fork. These headers are the canonical mappings QMK uses to translate the keycode DB into actual characters. To re-verify any mapping, open the source file.

| Layout | Keymap header |
|---|---|
| German QWERTZ (Linux/Windows) | `qmk_firmware/quantum/keymap_extras/keymap_german.h` |
| German Mac ISO | `qmk_firmware/quantum/keymap_extras/keymap_german_mac_iso.h` |
| US ANSI | `qmk_firmware/quantum/keymap_extras/keymap_us.h` |

The QMK keycode aliasing convention in these headers is:

```c
#define DE_FOO  KC_BAR  // unshifted
#define DE_QQQ  S(DE_BAR)   // Shift + base key
#define DE_RRR  ALGR(DE_BAR) // AltGr + base key
#define DE_SSS  S(ALGR(DE_BAR)) // Shift+AltGr + base key
```

So the tables below show every keycode alias with the formula that produces its character.

## How to read the tables

Each row is one `DE_*` or `KC_*` keycode. The character column shows the printable output. The via column shows the recipe (`S(x)` = Shift, `ALGR(x)` = AltGr). If a keycode is reachable by more than one recipe (e.g. shifted **and** AltGr), the row appears in both tables.

## 1. German QWERTZ (`keymap_german.h`)

### 1.1 Base layer (no modifier)

| Keycode | Character | Physical key | Via |
|---|---|---|---|
| `DE_CIRC` | `^` (dead) | `^` (above Tab) | `KC_GRV` |
| `DE_1` | `1` | 1 | `KC_1` |
| `DE_2` | `2` | 2 | `KC_2` |
| `DE_3` | `3` | 3 | `KC_3` |
| `DE_4` | `4` | 4 | `KC_4` |
| `DE_5` | `5` | 5 | `KC_5` |
| `DE_6` | `6` | 6 | `KC_6` |
| `DE_7` | `7` | 7 | `KC_7` |
| `DE_8` | `8` | 8 | `KC_8` |
| `DE_9` | `9` | 9 | `KC_9` |
| `DE_0` | `0` | 0 | `KC_0` |
| `DE_SS` | `ß` | `ß` (right of 0) | `KC_MINS` |
| `DE_ACUT` | `´` (dead) | `´` (right of ß) | `KC_EQL` |
| `DE_Y` | `Y` | Y (home row, QWERTY Z position) | `KC_Z` |
| `DE_UDIA` | `Ü` | `Ü` (right of P) | `KC_LBRC` |
| `DE_PLUS` | `+` | `+` (right of Ü) | `KC_RBRC` |
| `DE_ODIA` | `Ö` | `Ö` (right of L) | `KC_SCLN` |
| `DE_ADIA` | `Ä` | `Ä` (right of Ö) | `KC_QUOT` |
| `DE_HASH` | `#` | `#` (right of Ä) | `KC_NUHS` |
| `DE_LABK` | `<` | `<` (left of Y) | `KC_NUBS` |
| `DE_Z` | `Z` | Z (home row, QWERTY Y position) | `KC_Y` |
| `DE_COMM` | `,` | `,` | `KC_COMM` |
| `DE_DOT` | `.` | `.` | `KC_DOT` |
| `DE_MINS` | `-` | `-` (right of 0/Y) | `KC_SLSH` |

### 1.2 Shift (uppercase / shifted symbols)

| Keycode | Character | Via |
|---|---|---|
| `DE_DEG` | `°` (degree) | `S(DE_CIRC)` |
| `DE_EXLM` | `!` | `S(DE_1)` |
| `DE_DQUO` | `"` | `S(DE_2)` |
| `DE_SECT` | `§` | `S(DE_3)` |
| `DE_DLR` | `$` | `S(DE_4)` |
| `DE_PERC` | `%` | `S(DE_5)` |
| `DE_AMPR` | `&` | `S(DE_6)` |
| `DE_SLSH` | `/` | `S(DE_7)` |
| `DE_LPRN` | `(` | `S(DE_8)` |
| `DE_RPRN` | `)` | `S(DE_9)` |
| `DE_EQL` | `=` | `S(DE_0)` |
| `DE_QUES` | `?` | `S(DE_SS)` |
| `DE_GRV` | `` ` `` (dead) | `S(DE_ACUT)` |
| `DE_ASTR` | `*` | `S(DE_PLUS)` |
| `DE_QUOT` | `'` | `S(DE_HASH)` |
| `DE_RABK` | `>` | `S(DE_LABK)` |
| `DE_SCLN` | `;` | `S(DE_COMM)` |
| `DE_COLN` | `:` | `S(DE_DOT)` |
| `DE_UNDS` | `_` | `S(DE_MINS)` |

### 1.3 AltGr (right Alt, 3rd-level symbols)

| Keycode | Character | Via |
|---|---|---|
| `DE_SUP2` | `²` | `ALGR(DE_2)` |
| `DE_SUP3` | `³` | `ALGR(DE_3)` |
| `DE_LCBR` | `{` | `ALGR(DE_7)` |
| `DE_LBRC` | `[` | `ALGR(DE_8)` |
| `DE_RBRC` | `]` | `ALGR(DE_9)` |
| `DE_RCBR` | `}` | `ALGR(DE_0)` |
| `DE_BSLS` | `\` (backslash) | `ALGR(DE_SS)` |
| `DE_AT` | `@` | `ALGR(DE_Q)` |
| `DE_EURO` | `€` | `ALGR(DE_E)` |
| `DE_TILD` | `~` | `ALGR(DE_PLUS)` |
| `DE_PIPE` | `\|` | `ALGR(DE_LABK)` |
| `DE_MICR` | `µ` | `ALGR(DE_M)` |

### 1.4 Shift+AltGr (rare, 4th-level)

`_` does **not** appear in this list — it is reachable via `S(DE_MINS)` (Shift+Minus) on the base layer, so add `DE_MINS` to the base layer to get both `-` and `_` for free.

---

## 2. German Mac ISO (`keymap_german_mac_iso.h`)

macOS uses a slightly different AltGr mechanism (`ALGR` in QMK → `LALT` in macOS Unicode input). Most of the same characters are reachable, but the path differs. Use this section when the user runs QMK on macOS with the Mac ISO driver.

### 2.1 Base layer

Same as German QWERTZ base (the Mac ISO header aliases identical physical keys).

### 2.2 Shift

Mostly identical to German QWERTZ Shift. Differences:

| Keycode | Character | Via |
|---|---|---|
| `DE_DEG` | `°` | `S(DE_CIRC)` |
| `DE_GRV` | `` ` `` (dead) | `S(DE_ACUT)` |
| `DE_UNDS` | `_` | `S(DE_MINS)` |
| `DE_TILD` | `~` (dead) | `S(DE_N)` (note: not `ALGR`!) |

### 2.3 Alt / Option (macOS)

These are `ALGR(...)` in QMK, but on macOS the user presses `Option` (a.k.a. Alt). The keycode-wise it's identical to QMK's ALGR.

| Keycode | Character | Via |
|---|---|---|
| `DE_LCBR` | `{` | `ALGR(DE_7)` |
| `DE_LBRC` | `[` | `ALGR(DE_8)` |
| `DE_RBRC` | `]` | `ALGR(DE_9)` |
| `DE_RCBR` | `}` | `ALGR(DE_0)` |
| `DE_BSLS` | `\` | `ALGR(DE_SS)` |
| `DE_AT` | `@` | `ALGR(DE_L)` (note: different from QWERTZ!) |
| `DE_EURO` | `€` | `ALGR(DE_E)` |
| `DE_LSQU` | `‘` (left single quote) | `ALGR(DE_HASH)` |
| `DE_TILD` | `~` (dead) | `ALGR(DE_N)` |

---

## 3. US ANSI (`keymap_us.h`)

The default English layout. QMK includes these as both `KC_*` aliases and bare shifted definitions.

### 3.1 Base layer

| Keycode | Character | Physical key |
|---|---|---|
| `KC_GRV` | `` ` `` | `` ` `` (above Tab) |
| `KC_1` | `1` | 1 |
| `KC_2` | `2` | 2 |
| `KC_3` | `3` | 3 |
| `KC_4` | `4` | 4 |
| `KC_5` | `5` | 5 |
| `KC_6` | `6` | 6 |
| `KC_7` | `7` | 7 |
| `KC_8` | `8` | 8 |
| `KC_9` | `9` | 9 |
| `KC_0` | `0` | 0 |
| `KC_MINS` | `-` | `-` |
| `KC_EQL` | `=` | `=` |
| `KC_LBRC` | `[` | `[` |
| `KC_RBRC` | `]` | `]` |
| `KC_BSLS` | `\` | `\` |
| `KC_SCLN` | `;` | `;` |
| `KC_QUOT` | `'` | `'` |
| `KC_COMM` | `,` | `,` |
| `KC_DOT` | `.` | `.` |
| `KC_SLSH` | `/` | `/` |

### 3.2 Shift

| Keycode | Character | Via |
|---|---|---|
| `KC_TILDE` / `KC_TILD` | `~` | `S(KC_GRV)` |
| `KC_EXLM` / `KC_EXCLAIM` | `!` | `S(KC_1)` |
| `KC_AT` | `@` | `S(KC_2)` |
| `KC_HASH` | `#` | `S(KC_3)` |
| `KC_DLR` / `KC_DOLLAR` | `$` | `S(KC_4)` |
| `KC_PERC` / `KC_PERCENT` | `%` | `S(KC_5)` |
| `KC_CIRC` / `KC_CIRCUMFLEX` | `^` | `S(KC_6)` |
| `KC_AMPR` / `KC_AMPERSAND` | `&` | `S(KC_7)` |
| `KC_ASTR` / `KC_ASTERISK` | `*` | `S(KC_8)` |
| `KC_LPRN` / `KC_LEFT_PAREN` | `(` | `S(KC_9)` |
| `KC_RPRN` / `KC_RIGHT_PAREN` | `)` | `S(KC_0)` |
| `KC_UNDS` / `KC_UNDERSCORE` | `_` | `S(KC_MINS)` |
| `KC_PLUS` | `+` | `S(KC_EQL)` |
| `KC_LCBR` / `KC_LEFT_CURLY_BRACE` | `{` | `S(KC_LBRC)` |
| `KC_RCBR` / `KC_RIGHT_CURLY_BRACE` | `}` | `S(KC_RBRC)` |
| `KC_PIPE` | `\|` | `S(KC_BSLS)` |
| `KC_COLN` / `KC_COLON` | `:` | `S(KC_SCLN)` |
| `KC_DQUO` / `KC_DOUBLE_QUOTE` | `"` | `S(KC_QUOT)` |
| `KC_LABK` / `KC_LEFT_ANGLE_BRACKET` | `<` | `S(KC_COMM)` |
| `KC_RABK` / `KC_RIGHT_ANGLE_BRACKET` | `>` | `S(KC_DOT)` |
| `KC_QUES` / `KC_QUESTION` | `?` | `S(KC_SLSH)` |

---

## Common Pitfalls

- **The same character is reachable via different paths on different layouts.** German QWERTZ reaches `_` via `S(DE_MINS)`, US ANSI reaches it via `S(KC_MINS)`. Both produce the same character but use different physical keys. Always verify against the user's primary layout.
- **Shift on a layer-tap key** — `LT(2, KC_W)` will `W` on tap. To get `W` on a shifted layer, you need `LSFT_T(KC_W)` or set the layer's `W` to `KC_W` and rely on the shift key. The Shift modifier works across all layers transparently because QMK's `S(KC_X)` syntax is a hold-modifier combinator, not a keycode.
- **Dead keys** — `DE_CIRC` (^), `DE_ACUT` (´), `DE_GRV` (`` ` ``) are dead keys: pressing them alone produces nothing; pressing them before another key produces the combined character (e.g. `^` + `a` → `â`). On Mac ISO, `~` is also a dead key.
- **AltGr on split-side combos** — combos on layer 0 use the base layer's `KC_*` aliases. If the base layer has `DE_E` on a given physical key, the combo trigger fires on that physical key regardless of the OS layout. The AltGr mapping doesn't apply to combos.
- **Combo keys must be simple** — the combo generator emits the raw keycode (e.g. `KC_MCTL`) as a `uint16_t`. If the combo keycode is `LSFT(KC_X)` or `LT(@uuid, KC_X)`, it produces invalid C. Always use base `KC_*` (or `DE_*`/`US_*` aliases) on combo trigger positions.

## How to use this for audits

For a practical duplicate/missing audit, run the lookup script against every keycode in your custom layers:

```bash
LAYOUT=~/.../corne_choc_pro_enhanced.json
LAYOUT_NAME="german_qwertz"
for kc in $(jq -r '.layers[].keys[].keycode' "$LAYOUT" | sort -u); do
  echo "=== $kc ==="
  bash scripts/layout-reachability.sh "$LAYOUT_NAME" "$kc"
done
```

For each duplicate character, you get a one-line output:

```text
KC_PMNS - (DE_MINS on German QWERTZ)
DE_UNDS - (DE_UNDS via S(DE_MINS) on German QWERTZ)
```

If two keycodes both produce `-`, the second one is redundant if you already have `DE_MINS` on the base layer.

### Layer-aware audit (recommended)

For a complete picture that includes Shift/AltGr reachability, use the dedicated audit script:

```bash
# Full audit (per-character table + missing-char report)
bash scripts/audit-layout.sh german_qwertz ~/.../corne_choc_pro_enhanced.json

# Single character lookup with layer + modifier context
bash scripts/audit-layout.sh german_qwertz ~/.../corne_choc_pro_enhanced.json "_"
bash scripts/audit-layout.sh german_qwertz ~/.../corne_choc_pro_enhanced.json "adia"
```

The `audit-layout.sh` script outputs:

1. **Per-character reachability table** (modifier-aware). Each character shows every reachable layer + the modifier + keycode combination that produces it. Example:
   ```
   Char    Layers          Paths (layer: modifier+keycode)
   '_'     L0              L0:Shift+DE_MINS
   '|'     L0,L1,L4        L0:AltGr+DE_LABK | L1:AltGr+DE_LABK | L4:DE_PIPE/Shift+DE_PIPE/AltGr+DE_PIPE
   '°'     L1              L1:Shift+DE_CIRC
   ```
2. **Free slots per layer** — where new keycodes can be placed.
3. **Common German/code/math characters** — `[OK]` if reachable on any layer (including via modifiers), `[MISS]` if not, with a suggested slot.
