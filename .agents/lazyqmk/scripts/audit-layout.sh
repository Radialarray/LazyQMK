#!/usr/bin/env bash
# audit-layout.sh <layout> <layout.json> [char]
#
# Layer-aware reachability audit. Reads a LazyQMK layout JSON and reports:
#   - For a single character: which layer(s) and positions it is reachable on,
#     the keycode at each position, and a suggested slot if missing.
#   - Without a character: a full audit of all custom-layer keycodes plus a
#     missing-char report for common German/coding/math characters.
#
# Usage:
#   bash scripts/audit-layout.sh german_qwertz ~/Library/Application\ Support/LazyQMK/layouts/corne_choc_pro_enhanced.json
#   bash scripts/audit-layout.sh german_qwertz ~/.../my.json "_"
#   bash scripts/audit-layout.sh german_qwertz ~/.../my.json "adia"
#
# Layouts: german_qwertz, german_mac_iso, us_ansi
#
# The script calls layout-reachability.sh internally for the per-keycode resolve
# step. They must be siblings under scripts/.

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOOKUP="$SKILL_DIR/scripts/layout-reachability.sh"

if [[ ! -x "$LOOKUP" ]]; then
  echo "ERROR: missing $LOOKUP. Both scripts must be installed together." >&2
  exit 2
fi

LAYOUT="${1:-}"
LAYOUT_JSON="${2:-}"
INPUT_CHAR="${3:-}"

usage() {
  cat >&2 <<EOF
Usage: $(basename "$0") <layout> <layout.json> [char]

Layouts: german_qwertz, german_mac_iso, us_ansi

With <char>: report which layer(s) the character is reachable on, with the
keycode(s) and position(s). If missing, recommend a free slot to add it.

Without <char>: full audit (every custom-layer keycode plus a missing-char
report for common German/coding/math characters).
EOF
  exit 2
}

if [[ -z "$LAYOUT" || -z "$LAYOUT_JSON" ]]; then
  usage
fi

if [[ ! -f "$LAYOUT_JSON" ]]; then
  echo "ERROR: layout file not found: $LAYOUT_JSON" >&2
  exit 2
fi

# Suggested-slot heuristics: which layer should hold a missing character?
# Look for the first free slot on the named layer. If none, fall back to any layer.
suggest_layer_and_slot() {
  local char="$1"
  local layout_json="$2"

  # Decide target layer by character category
  local target_layer=""
  case "$char" in
    # Code/programming: {}, [], (), |, \\, !, ^, _, ~, ', ", ;, :, <, >, +
    "{" | "}" | "[" | "]" | "(" | ")" | "|" | "\\" | "!" | "^" | "_" | "~" | "'" | "\"" | ";" | ":" | "<" | ">" | "+")
      target_layer="code" ;;
    # Math: ×, ÷, ±, °, µ, ², ³, π, *
    "×" | "÷" | "±" | "°" | "µ" | "²" | "³" | "π" | "*")
      target_layer="numbers" ;;
    # German letters/diacritics: ä, ö, ü, ß, €
    "ä" | "ö" | "ü" | "Ä" | "Ö" | "Ü" | "ß" | "€" | "ª" | "º")
      target_layer="symbols" ;;
    # German punctuation: ², ³, §, @
    "§" | "@")
      target_layer="symbols" ;;
    # Arrows
    "←" | "→" | "↑" | "↓" | "Home" | "End" | "PageUp" | "PageDown")
      target_layer="navigation" ;;
    # Default: Symbols
    *)
      target_layer="symbols" ;;
  esac

  # Map layer name to layer number
  local target_num
  target_num="$(jq -r --arg n "$target_layer" '.layers[] | select(.name | ascii_downcase == $n) | .number' "$layout_json" | head -1)"

  # Fall back to Symbols if the named layer doesn't exist
  if [[ -z "$target_num" ]]; then
    target_num="$(jq -r '.layers[] | select(.name == "Symbols") | .number' "$layout_json" | head -1)"
    target_layer="symbols"
  fi

  # Find the first TRNS slot on the target layer
  local slot
  slot="$(jq -r --argjson layer "$target_num" '
    .layers[$layer].keys[]
    | select(.keycode == "KC_TRNS")
    | "\(.position.row),\(.position.col)"
  ' "$layout_json" | head -1)"

  if [[ -n "$slot" ]]; then
    printf "  Suggested slot: Layer %s (%s) (%s)\n" "$target_num" "$target_layer" "$slot"
  else
    printf "  No free slot on %s layer; consider re-arranging.\n" "$target_layer"
  fi
}

# Check base layer Shift/AltGr reachability for chars missing from the layout.
# For each base keycode, ask the lookup script "if I Shift/AltGr + this keycode,
# what character do I get?" Then check if the target char is in that list.
check_base_modifier_reach() {
  local char="$1"
  local layout_json="$2"

  # For each base keycode, check what its bare/Shift/AltGr produces
  python3 - "$char" "$layout_json" "$LOOKUP" "$LAYOUT" <<'PY'
import json, subprocess, sys

target_char, layout_path, lookup_script, layout_name = sys.argv[1:5]

with open(layout_path) as fh:
    layout = json.load(fh)

# Walk base layer
base = layout['layers'][0]
target_norm = target_char.casefold()
matches = []

for k in base['keys']:
    kc = k['keycode']
    if kc in ('KC_TRNS', 'KC_NO'):
        continue
    # Get the bare character
    r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', kc],
                       capture_output=True, text=True)
    bare = ''
    for line in r.stdout.splitlines():
        if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
            parts = line.split()
            if len(parts) >= 2 and parts[1] != 'via':
                bare = parts[1]
            break

    # Check Shift + keycode → ALGR + keycode → S(ALGR(keycode))
    # The lookup script reports the recipe, e.g. "DE_UNDS  _  via S(DE_MINS)"
    # For modifier-based reachability, we need to check what the wrapped version produces.
    # The simplest way: ask the lookup script for the wrapped keycode.
    # e.g. "S(DE_MINS)" → DE_UNDS → "_"
    # We'll resolve via the lookup script's character lookup.

    modifier_specs = [
        ("S", "Shift"),
        ("ALGR", "AltGr"),
        ("S(ALGR", "Shift+AltGr"),
    ]
    # Build a list of inner-keycode variants to try. The base layout uses
    # KC_* aliases, but the German QWERTZ header uses DE_* aliases for the
    # same physical keys (e.g. KC_M = DE_M). We try both.
    inner_variants = [kc]
    # Also try the layout-specific aliases (e.g. DE_M for KC_M)
    if layout_name == 'german_qwertz':
        for prefix in ('DE_',):
            alias = f"{prefix}{kc.removeprefix('KC_')}"
            if alias != kc:
                inner_variants.append(alias)
    elif layout_name == 'german_mac_iso':
        for prefix in ('DE_',):
            alias = f"{prefix}{kc.removeprefix('KC_')}"
            if alias != kc:
                inner_variants.append(alias)
    elif layout_name == 'us_ansi':
        for prefix in ('US_',):
            alias = f"{prefix}{kc.removeprefix('KC_')}"
            if alias != kc:
                inner_variants.append(alias)

    for modifier_prefix, label in modifier_specs:
        for inner_kc in inner_variants:
            wrapped_kc = f"{modifier_prefix}({inner_kc})"
            if wrapped_kc.startswith("S(KC_TRNS)") or wrapped_kc.startswith("S(KC_NO)") or wrapped_kc.startswith("ALGR(KC_NO)"):
                continue
            r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', wrapped_kc],
                               capture_output=True, text=True)
            produced_chars = []
            for line in r.stdout.splitlines():
                if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
                    parts = line.split()
                    if len(parts) >= 2 and parts[1] != 'via':
                        produced_chars.append(parts[1])
            if target_norm in (c.casefold() for c in produced_chars):
                pos = f"({k['position']['row']},{k['position']['col']})"
                matches.append((pos, inner_kc, label))
                break  # Don't double-report the same position for the same character

if not matches:
    print(f"  NOT REACHABLE via base Shift/AltGr either ({target_char!r}).")
    sys.exit(0)

print(f"  Reachable via base modifier on Layer 0:")
for pos, kc, label in matches:
    print(f"    {pos}: {label}+{kc} → {target_char!r}")
PY
}

# Print per-layer reachability for the given character
char_where() {
  local char="$1"
  local layout_json="$2"

  # Build a per-(layer,position,keycode,char) index
  python3 - "$char" "$layout_json" "$LOOKUP" "$LAYOUT" <<'PY'
import json, subprocess, sys

target_char, layout_path, lookup_script, layout_name = sys.argv[1:5]

with open(layout_path) as fh:
    layout = json.load(fh)

LAYER_NAMES = {0: 'Base', 1: 'Symbols', 2: 'Navigation', 3: 'Numbers', 4: 'Code', 5: 'Globals'}

# Walk every keycode, resolve to its character AND its modifier variants
# (S(X), ALGR(X), S(ALGR(X))). Each produces a char that the user can reach
# on that layer when holding the appropriate modifier.
target_norm = target_char.casefold()
hits = []  # (layer, pos, keycode, char, via_label)

for layer in layout['layers']:
    ln = layer['number']
    lname = LAYER_NAMES.get(ln, layer['name'])
    for k in layer['keys']:
        kc = k['keycode']
        if kc in ('KC_TRNS', 'KC_NO'):
            continue
        # Build the inner-keycode variants to try (KC_* is the base, plus
        # layout-specific aliases like DE_*, US_*).
        inner_variants = [kc]
        if layout_name in ('german_qwertz', 'german_mac_iso'):
            inner_variants.append('DE_' + kc.removeprefix('KC_'))
        elif layout_name == 'us_ansi':
            inner_variants.append('US_' + kc.removeprefix('KC_'))
        # Bare keycode (no modifier)
        for inner in inner_variants:
            r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', inner],
                               capture_output=True, text=True)
            for line in r.stdout.splitlines():
                if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
                    parts = line.split()
                    if len(parts) >= 2 and parts[1] != 'via':
                        produced = parts[1]
                        if produced and produced.casefold() == target_norm:
                            pos = f"({k['position']['row']},{k['position']['col']})"
                            hits.append((ln, lname, pos, kc, 'bare', produced))
        # Modifier variants
        for prefix, label in [('S', 'Shift'), ('ALGR', 'AltGr'), ('S(ALGR', 'Shift+AltGr')]:
            for inner in inner_variants:
                wrapped = f"{prefix}({inner})"
                r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', wrapped],
                                   capture_output=True, text=True)
                for line in r.stdout.splitlines():
                    if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
                        parts = line.split()
                        if len(parts) >= 2 and parts[1] != 'via':
                            produced = parts[1]
                            if produced and produced.casefold() == target_norm:
                                pos = f"({k['position']['row']},{k['position']['col']})"
                                hits.append((ln, lname, pos, kc, label, produced))

if not hits:
    print(f"NOT REACHABLE: {target_char!r} on any layer of this layout JSON")
    sys.exit(0)

# Group by layer
from collections import defaultdict
by_layer = defaultdict(list)
for ln, lname, pos, kc, via, produced in hits:
    by_layer[ln].append((lname, pos, kc, via, produced))

for ln in sorted(by_layer.keys()):
    rows = by_layer[ln]
    lname = rows[0][0]
    print(f"  Layer {ln} ({lname}):")
    for _, pos, kc, via, produced in rows:
        if via == 'bare':
            label = f"  {pos}: {kc} (bare)"
        else:
            label = f"  {pos}: {via}+{kc} (via {via})"
        print(label)
PY
}

# Resolve a single character via the lookup script (handle named entities)
resolve_target() {
  local char="$1"
  local layout_json="$2"
  bash "$LOOKUP" "$LAYOUT" --resolve "$char" 2>&1 | grep -E "^(DE_|KC_)" | head -1 | awk '{print $2}'
}

# === Main ===
if [[ -n "$INPUT_CHAR" ]]; then
  # Resolve named entity if needed
  ENTITIES_FILE="$SKILL_DIR/scripts/layout-reachability-entities.txt"
  TARGET="$INPUT_CHAR"
  if [[ -f "$ENTITIES_FILE" ]]; then
    resolved="$(grep -E "^${INPUT_CHAR}=" "$ENTITIES_FILE" 2>/dev/null | head -1 | cut -d= -f2- | tr -d '\r' || true)"
    if [[ -n "$resolved" ]]; then
      TARGET="$resolved"
    fi
  fi

  echo "=== Layer-aware reachability for $INPUT_CHAR ==="
  if [[ "$TARGET" != "$INPUT_CHAR" ]]; then
    echo "    Named entity resolves to: $TARGET"
  fi
  echo
  # Capture the where output for layout-level reachability (use TARGET, the resolved char)
  RAW_WHERE="$(char_where "$TARGET" "$LAYOUT_JSON")"
  printf '%s\n' "$RAW_WHERE"
  echo
  # If nothing was found on any layer-keycode, check base Shift/AltGr.
  if ! printf '%s\n' "$RAW_WHERE" | grep -q "^  Layer "; then
    check_base_modifier_reach "$TARGET" "$LAYOUT_JSON"
  fi
  echo
  suggest_layer_and_slot "$TARGET" "$LAYOUT_JSON"
  exit 0
fi

# Full audit: build full per-layer reachability table + missing-char report
echo "=== Per-character reachability (layer-aware, modifier-aware) ==="
echo

python3 - "$LAYOUT_JSON" "$LOOKUP" "$LAYOUT" <<'PY'
import json, subprocess, sys

layout_path, lookup_script, layout_name = sys.argv[1:4]

with open(layout_path) as fh:
    layout = json.load(fh)

LAYER_NAMES = {0: 'Base', 1: 'Symbols', 2: 'Navigation', 3: 'Numbers', 4: 'Code', 5: 'Globals'}

# Build a per-character, per-layer index — INCLUDING modifier (Shift/AltGr)
# reachability. For each keycode we check its bare, S(), ALGR(), and S(ALGR())
# forms. Each form is a separate "reachability path".
from collections import defaultdict
char_layers = defaultdict(set)  # char -> set of layer numbers
char_paths = defaultdict(list)  # char -> [(layer, pos, kc, via_label), ...]

def inner_variants_for(kc):
    variants = [kc]
    if layout_name in ('german_qwertz', 'german_mac_iso'):
        variants.append('DE_' + kc.removeprefix('KC_'))
    elif layout_name == 'us_ansi':
        variants.append('US_' + kc.removeprefix('KC_'))
    return variants

def resolve_chars(kc_wrapped):
    """Return all characters that the wrapped keycode can produce."""
    r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', kc_wrapped],
                       capture_output=True, text=True)
    chars = []
    for line in r.stdout.splitlines():
        if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
            parts = line.split()
            if len(parts) >= 2 and parts[1] != 'via':
                chars.append(parts[1])
    return chars

for layer in layout['layers']:
    ln = layer['number']
    for k in layer['keys']:
        kc = k['keycode']
        if kc in ('KC_TRNS', 'KC_NO'):
            continue
        pos = f"({k['position']['row']},{k['position']['col']})"
        variants = inner_variants_for(kc)
        # Bare
        for v in variants:
            for ch in resolve_chars(v):
                char_layers[ch].add(ln)
                char_paths[ch].append((ln, pos, kc, 'bare'))
        # Modifier variants
        for prefix, label in [('S', 'Shift'), ('ALGR', 'AltGr'), ('S(ALGR', 'Shift+AltGr')]:
            for v in variants:
                wrapped = f"{prefix}({v})"
                for ch in resolve_chars(wrapped):
                    char_layers[ch].add(ln)
                    char_paths[ch].append((ln, pos, kc, label))

# Print per-character table — show all reachable paths, not just the layer
print(f"{'Char':6}  {'Layers':14}  Paths (layer: modifier+keycode)")
print("-" * 90)
for char in sorted(char_paths.keys(), key=lambda c: (len(c), c)):
    paths = char_paths[char]
    layers = sorted({p[0] for p in paths})
    layer_short = ','.join(f"L{l}" for l in layers)
    # Group paths by layer
    by_layer = defaultdict(list)
    for ln, pos, kc, via in paths:
        by_layer[ln].append((pos, kc, via))
    path_summary = []
    for ln in sorted(by_layer.keys()):
        ps = by_layer[ln]
        # Deduplicate (layer, modifier, keycode) tuples
        seen = set()
        uniq = []
        for pos, kc, via in ps:
            t = (via, kc)
            if t not in seen:
                seen.add(t)
                uniq.append((via, kc))
        seg = f"L{ln}:" + "/".join(f"{v}+{kc}" if v != 'bare' else kc for v, kc in uniq)
        path_summary.append(seg)
    print(f"{char!r:6}  {layer_short:14}  {' | '.join(path_summary)}")
PY

echo
echo "=== Free slots per layer ==="
echo
python3 -c "
import json, sys
layout = json.load(open('$LAYOUT_JSON'))
LAYER_NAMES = {0: 'Base', 1: 'Symbols', 2: 'Navigation', 3: 'Numbers', 4: 'Code', 5: 'Globals'}
for layer in layout['layers']:
    ln = layer['number']
    name = LAYER_NAMES.get(ln, layer['name'])
    trns = [(k['position']['row'], k['position']['col']) for k in layer['keys'] if k['keycode'] == 'KC_TRNS']
    if trns:
        print(f'  Layer {ln} ({name}): {len(trns)} free slot(s) at {trns}')
    else:
        print(f'  Layer {ln} ({name}): no free slots')
"
echo
echo "=== Common German/code/math characters ==="
echo "(missing = char not on any layer; suggested slot shows where to add it)"
echo

# Common chars per category
COMMON_CHARS="
code:! ^ _ { } [ ] ( ) | \\ ~ ' \" ; : < > +
numbers:° × ÷ ± µ ² ³ π * = - + .
symbols:ä ö ü ß € Ä Ö Ü § @
navigation:← ↑ ↓ → Home End PageUp PageDown
"
python3 - "$LAYOUT_JSON" "$LOOKUP" "$LAYOUT" "$COMMON_CHARS" <<'PY'
import json, subprocess, sys

layout_path, lookup_script, layout_name, common_text = sys.argv[1:5]

with open(layout_path) as fh:
    layout = json.load(fh)

LAYER_NAMES = {0: 'Base', 1: 'Symbols', 2: 'Navigation', 3: 'Numbers', 4: 'Code', 5: 'Globals'}

# Build per-char -> layer set index (case-insensitive, modifier-aware).
# For each keycode we check bare, S(), ALGR(), S(ALGR()) forms so the index
# is consistent with the per-character table above.
from collections import defaultdict
char_layers = defaultdict(set)

def inner_variants_for(kc):
    variants = [kc]
    if layout_name in ('german_qwertz', 'german_mac_iso'):
        variants.append('DE_' + kc.removeprefix('KC_'))
    elif layout_name == 'us_ansi':
        variants.append('US_' + kc.removeprefix('KC_'))
    return variants

def resolve_chars(kc_wrapped):
    r = subprocess.run(['bash', lookup_script, layout_name, '--resolve', kc_wrapped],
                       capture_output=True, text=True)
    chars = []
    for line in r.stdout.splitlines():
        if line.startswith(('DE_', 'KC_', 'UK_', 'US_', 'FR_', 'IT_', 'ES_', 'NO_', 'DK_', 'SE_')):
            parts = line.split()
            if len(parts) >= 2 and parts[1] != 'via':
                chars.append(parts[1])
    return chars

for layer in layout['layers']:
    ln = layer['number']
    for k in layer['keys']:
        kc = k['keycode']
        if kc in ('KC_TRNS', 'KC_NO'):
            continue
        for v in inner_variants_for(kc):
            for ch in resolve_chars(v):
                char_layers[ch.casefold()].add(ln)
        for prefix, _ in [('S', ''), ('ALGR', ''), ('S(ALGR', '')]:
            for v in inner_variants_for(kc):
                for ch in resolve_chars(f"{prefix}({v})"):
                    char_layers[ch.casefold()].add(ln)

# Find the first free slot on each layer
def first_free_slot(target_num):
    for layer in layout['layers']:
        if layer['number'] != target_num:
            continue
        for k in layer['keys']:
            if k['keycode'] == 'KC_TRNS':
                return f"({k['position']['row']},{k['position']['col']})"
    return None

def suggest_layer_and_slot(char):
    """Pick a target layer based on character category."""
    if char in '{}[]().|\\!^_~\'";:<>+':
        target_name = 'code'
    elif char in '×÷±°µ²³π*=.-':
        target_name = 'numbers'
    elif char in 'äöüÄÖÜß€@§':
        target_name = 'symbols'
    elif char in '←↑↓→HomeEndPageUpPageDown':
        target_name = 'navigation'
    else:
        target_name = 'symbols'
    target_num = None
    for layer in layout['layers']:
        if LAYER_NAMES.get(layer['number'], layer['name']).lower() == target_name:
            target_num = layer['number']
            break
    if target_num is None:
        # Fall back to symbols
        for layer in layout['layers']:
            if LAYER_NAMES.get(layer['number'], layer['name']).lower() == 'symbols':
                target_num = layer['number']
                target_name = 'symbols'
                break
    slot = first_free_slot(target_num) if target_num is not None else None
    if target_num is not None and slot:
        return f"Layer {target_num} ({target_name}) {slot}"
    elif target_num is not None:
        return f"Layer {target_num} ({target_name}) (no free slot — re-arrange)"
    return "no suitable layer"

# Parse common chars
for line in common_text.strip().splitlines():
    cat, chars = line.split(':', 1)
    print(f"  {cat}:")
    for ch in chars.split():
        ch_key = {'←': 'Left', '→': 'Right', '↑': 'Up', '↓': 'Down'}.get(ch, ch)
        ch_norm = ch_key.casefold()
        if ch_norm in char_layers:
            layers = sorted(char_layers[ch_norm])
            lnames = ', '.join(f"L{l}({LAYER_NAMES.get(l, '?')})" for l in layers)
            print(f"    [OK] {ch!r:6}  reachable on {lnames}")
        else:
            # Check base Shift/AltGr reachability
            base_reach = subprocess.run(
                ['python3', '-c', f'''
import json, subprocess, sys
with open({layout_path!r}) as fh:
    layout = json.load(fh)
target_norm = {ch_key!r}.casefold()
base = layout["layers"][0]
hits = []
for k in base["keys"]:
    kc = k["keycode"]
    if kc in ("KC_TRNS", "KC_NO"):
        continue
    inner_variants = [kc]
    if {layout_name!r} in ("german_qwertz", "german_mac_iso"):
        inner_variants.append("DE_" + kc.removeprefix("KC_"))
    elif {layout_name!r} == "us_ansi":
        inner_variants.append("US_" + kc.removeprefix("KC_"))
    for prefix, label in [("S", "Shift"), ("ALGR", "AltGr"), ("S(ALGR", "Shift+AltGr")]:
        for inner in inner_variants:
            wrapped = f"{{prefix}}({{inner}})"
            r = subprocess.run(["bash", {lookup_script!r}, {layout_name!r}, "--resolve", wrapped],
                               capture_output=True, text=True)
            for line in r.stdout.splitlines():
                if line.startswith(("DE_", "KC_", "UK_", "US_", "FR_", "IT_", "ES_", "NO_", "DK_", "SE_")):
                    parts = line.split()
                    if len(parts) >= 2 and parts[1] != "via" and parts[1].casefold() == target_norm:
                        pos = f"({{k[\'position\'][\'row\']}},{{k[\'position\'][\'col\']}})"
                        hits.append((pos, label, inner))
                        break
if hits:
    for pos, label, inner in hits:
        print(f"      base {{pos}}: {{label}}+{{inner}}")
else:
    print("      (not reachable via base Shift/AltGr either)")
'''], capture_output=True, text=True)
            base_reach_output = base_reach.stdout.strip()
            if base_reach_output:
                print(f"    [MISS] {ch!r:6}  NOT on any layer")
                print(f"      Base modifier reachability:")
                for line in base_reach_output.splitlines():
                    print(f"      {line}")
            else:
                print(f"    [MISS] {ch!r:6}  NOT on any layer  →  not reachable via base Shift/AltGr either")
            suggestion = suggest_layer_and_slot(ch_key)
            print(f"      Suggested slot: {suggestion}")
PY
echo
echo "=== End of audit ==="
