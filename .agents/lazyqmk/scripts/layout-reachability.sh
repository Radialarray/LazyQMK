#!/usr/bin/env bash
# lookup <layout> <char> — print every keycode (base/Shift/AltGr) that produces the given
# character on the given layout. Exits 0 if reachable, 1 if not.
#
# Usage:
#   bash scripts/layout-reachability.sh <layout> <char>
#   bash scripts/layout-reachability.sh german_qwertz "_"
#   bash scripts/layout-reachability.sh us_ansi "+"
#   bash scripts/layout-reachability.sh german_mac_iso "€"
#   bash scripts/layout-reachability.sh german_qwertz udia     # named entity
#
# Layouts: german_qwertz, german_mac_iso, us_ansi
#
# Sourced from QMK's keymap_extras headers in the LazyQMK fork. The script greps
# the matching header to find every alias and its recipe, then prints the keycode
# and the character + recipe.

set -euo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Resolve QMK path: explicit env > lazyqmk config show > fallback locations
QMK_PATH="${LAZYQMK_QMK_PATH:-}"
if [[ -z "$QMK_PATH" ]]; then
  if command -v lazyqmk >/dev/null 2>&1; then
    QMK_PATH="$(lazyqmk config show --json 2>/dev/null | jq -r '.paths.qmk_firmware // empty' || true)"
  fi
  if [[ -z "$QMK_PATH" || ! -d "$QMK_PATH" ]]; then
    for candidate in \
      "$HOME/dev/LazyQMK/qmk_firmware" \
      "/Users/svenlochner/dev/LazyQMK/qmk_firmware" \
      "$SKILL_DIR/../../../qmk_firmware"
    do
      if [[ -d "$candidate/quantum/keymap_extras" ]]; then
        QMK_PATH="$candidate"
        break
      fi
    done
  fi
fi

if [[ -z "$QMK_PATH" || ! -d "$QMK_PATH/quantum/keymap_extras" ]]; then
  echo "ERROR: cannot find QMK keymap_extras directory. Set LAZYQMK_QMK_PATH." >&2
  exit 2
fi

LAYOUT="${1:-}"
INPUT_CHAR="${2:-}"

usage() {
  cat >&2 <<EOF
Usage: $(basename "$0") <layout> <char>
       $(basename "$0") <layout> --resolve <keycode>
       $(basename "$0") <layout> --all

Modes:
  <char>                  Print every keycode that produces <char> (base/Shift/AltGr)
  --resolve <keycode>     Print the character that <keycode> produces, plus reachability
  --all                   Print every keycode->character mapping for the layout

Layouts:
  german_qwertz    keymap_german.h          (QWERTZ, Linux/Windows)
  german_mac_iso   keymap_german_mac_iso.h (QWERTZ, macOS)
  us_ansi          keymap_us.h              (US ANSI)

The char can be a literal glyph ('_', '€') or a named entity
(see scripts/layout-reachability-entities.txt).
EOF
  exit 2
}

if [[ -z "$LAYOUT" || -z "$INPUT_CHAR" ]]; then
  usage
fi

case "$LAYOUT" in
  german_qwertz)  HEADER="keymap_german.h" ;;
  german_mac_iso) HEADER="keymap_german_mac_iso.h" ;;
  us_ansi)        HEADER="keymap_us.h" ;;
  *) echo "ERROR: unknown layout '$LAYOUT'. Use one of: german_qwertz, german_mac_iso, us_ansi." >&2; exit 2 ;;
esac

HEADER_PATH="$QMK_PATH/quantum/keymap_extras/$HEADER"
if [[ ! -f "$HEADER_PATH" ]]; then
  echo "ERROR: missing $HEADER_PATH" >&2
  exit 2
fi

# Resolve named entity to a literal char. The entities file is key=value, plain ASCII.
ENTITIES_FILE="$SKILL_DIR/scripts/layout-reachability-entities.txt"
TARGET="$INPUT_CHAR"
if [[ -f "$ENTITIES_FILE" ]]; then
  resolved="$(grep -E "^${INPUT_CHAR}=" "$ENTITIES_FILE" 2>/dev/null | head -1 | cut -d= -f2- | tr -d '\r' || true)"
  if [[ -n "$resolved" ]]; then
    TARGET="$resolved"
  fi
fi

# Mode detection
MODE="char"
KEYCODE_ARG=""
if [[ "$INPUT_CHAR" == "--resolve" ]]; then
  if [[ -z "${3:-}" ]]; then
    echo "ERROR: --resolve requires a keycode argument" >&2
    exit 2
  fi
  MODE="resolve"
  KEYCODE_ARG="$3"
elif [[ "$INPUT_CHAR" == "--all" ]]; then
  MODE="all"
fi

# Resolve base keycode-to-character map for the layout.
BASE_FILE="$SKILL_DIR/scripts/layout-reachability-base.txt"
BASE_MAP=""
if [[ -f "$BASE_FILE" ]]; then
  # Extract the section delimited by "<layout>_base=\"" and the next "\""
  BASE_MAP="$(awk -v target="${LAYOUT}_base" '
    $0 ~ target"=\"" { in_section=1; next }
    in_section && /^"/ { exit }
    in_section { print }
  ' "$BASE_FILE")"
  # Always prepend the universal QMK base keycode map (digits, F-keys, numpad, etc.)
  UNIVERSAL="$(awk '
    $0 == "_universal_base=\"" { in_section=1; next }
    in_section && /^"/ { exit }
    in_section { print }
  ' "$BASE_FILE")"
  BASE_MAP="${UNIVERSAL}${BASE_MAP}"
fi

exec python3 - "$HEADER_PATH" "$TARGET" "$INPUT_CHAR" "$LAYOUT" "$BASE_MAP" "$MODE" "$KEYCODE_ARG" <<'PY'
import re
import sys

path, target, requested, layout, base_map, mode, keycode_arg = sys.argv[1:8]

with open(path, encoding="utf-8") as fh:
    lines = fh.read().splitlines()

base_aliases = {}
char_map = {}

# Translate parenthetical comments like "(backslash)", "(dead)", "(U-Umlaut)"
# into actual characters or markers. The German QWERTZ header uses these
# when the character is hard to type or is a dead key.
NAMED_CHARS = {
    "backslash": "\\",
    "backsp": "\\",
    "hash": "#",
    "sharp": "#",
    "number": "#",
    "section": "§",
    "sect": "§",
    "degree": "°",
    "deg": "°",
    "cent": "¢",
    "pound": "£",
    "currency": "¤",
    "yen": "¥",
    "tilde": "~",
    "grave": "`",
    "acute": "´",
    "circumflex": "^",
    "caret": "^",
    "underscore": "_",
    "plus": "+",
    "minus": "-",
    "equal": "=",
    "equals": "=",
    "asterisk": "*",
    "star": "*",
    "ampersand": "&",
    "percent": "%",
    "dollar": "$",
    "at": "@",
    "question": "?",
    "exclamation": "!",
    "pipe": "|",
    "tilde": "~",
}

def parse_char_from_comment(comment):
    """Extract a printable character from a keymap_extras comment string.
    Examples:
        "Ü" → "Ü"
        "$" → "$"
        "(backslash)" → "\\"
        "(U-Umlaut)" → "Ü" (special-cased)
        "(dead)" → None (dead key)
        "(U-Umlaut)" → "ü" (handle diacritic names)
    """
    comment = comment.strip()
    if not comment:
        return None
    # First try direct single-character match
    if len(comment) == 1:
        return comment
    # Parenthetical description
    if comment.startswith("(") and comment.endswith(")"):
        body = comment[1:-1].strip()
        # Strip trailing qualifiers like "dead", "U-Umlaut"
        body_clean = body.split()[0].strip(",.;:") if body else ""
        # Look up the named character
        if body_clean in NAMED_CHARS:
            return NAMED_CHARS[body_clean]
        # Mappings for diacritic names
        diacritic_map = {
            "U-Umlaut": "ü", "O-Umlaut": "ö", "A-Umlaut": "ä",
            "s-z": "ß", "sharp": "ß",
            "double-quote": "\"", "single-quote": "'",
            "left-single-quote": "‘", "right-single-quote": "’",
        }
        if body_clean in diacritic_map:
            return diacritic_map[body_clean]
        # Dead key — no character
        if body_clean == "dead":
            return None
        # Try first-char fallback
        return body_clean[:1] if body_clean else None
    # Other multi-char: try first token
    first = comment.split()[0].strip(",.;:")
    if len(first) == 1:
        return first
    return first if first else None

def tokenize(rhs):
    rhs = rhs.strip()
    m = re.match(r'^S\(ALGR\((.+)\)\)\s*$', rhs)
    if m:
        return ("S", "ALGR", m.group(1).strip())
    m = re.match(r'^S\((.+)\)\s*$', rhs)
    if m:
        return ("S", m.group(1).strip())
    m = re.match(r'^ALGR\((.+)\)\s*$', rhs)
    if m:
        return ("ALGR", m.group(1).strip())
    return (rhs,)

def extract_rhs_and_comment(line):
    m = re.match(r'^\s*#define\s+([A-Z0-9_]+)\s+(.+?)$', line.rstrip())
    if not m:
        return None
    lhs, rhs_with_comment = m.group(1), m.group(2)
    if "//" in rhs_with_comment:
        rhs, comment = rhs_with_comment.split("//", 1)
        comment = comment.strip()
    else:
        rhs = rhs_with_comment
        comment = ""
    return lhs, rhs.strip(), comment

# First pass: register all keymap_extras aliases.
for line in lines:
    parsed = extract_rhs_and_comment(line)
    if not parsed:
        continue
    lhs, rhs, comment = parsed
    tokens = tokenize(rhs)
    base_aliases[lhs] = tokens
    if comment:
        char = parse_char_from_comment(comment)
        if char:
            char_map[lhs] = char

# Second pass: register the base keycode map (unshifted keycodes).
for line in base_map.splitlines():
    line = line.strip()
    if not line or "=" not in line:
        continue
    kc, char = line.split("=", 1)
    kc = kc.strip()
    char = char.strip()
    base_aliases.setdefault(kc, (kc,))
    char_map.setdefault(kc, char)

def resolve(kc, depth=0):
    if depth > 10:
        return None
    if kc not in base_aliases:
        return (kc,)
    tokens = base_aliases[kc]
    if len(tokens) == 1:
        inner = tokens[0]
        if inner in base_aliases:
            inner_resolved = resolve(inner, depth + 1)
            if inner_resolved is None:
                return None
            return inner_resolved
        return (inner,)
    return tokens

def format_via(path, default_kc):
    if path is None:
        return default_kc
    modifiers = [p for p in path if p in ("S", "ALGR")]
    keys = [p for p in path if p not in ("S", "ALGR")]
    if not modifiers:
        return keys[0] if keys else default_kc
    if len(modifiers) == 1:
        key = keys[0] if keys else ""
        if modifiers[0] == "S":
            return f"S({key})" if key else "S"
        if modifiers[0] == "ALGR":
            return f"ALGR({key})" if key else "ALGR"
    if len(modifiers) == 2:
        key = keys[0] if keys else ""
        return f"S(ALGR({key}))" if key else "S(ALGR)"
    return " ".join(modifiers) + f"({keys[0]})" if keys else " ".join(modifiers)

# --all mode: print every keycode -> character mapping
if mode == "all":
    print(f"# Layout: {layout}")
    print(f"# Source keymap_extras: {path}")
    print(f"# Base keycode map: scripts/layout-reachability-base.txt")
    print()
    seen = set()
    rows = []
    for kc, char in char_map.items():
        path_info = resolve(kc)
        via = format_via(path_info, kc)
        if (kc, via, char) in seen:
            continue
        seen.add((kc, via, char))
        rows.append((kc, via, char))
    rows.sort()
    for kc, via, char in rows:
        print(f"{kc:14}  {char:6}  via {via}")
    sys.exit(0)

# --resolve mode: given a keycode, print what character it produces
if mode == "resolve":
    if not keycode_arg:
        print("ERROR: --resolve requires a keycode argument", file=sys.stderr)
        sys.exit(2)
    kc = keycode_arg

    # Handle wrapped keycodes: S(X), ALGR(X), S(ALGR(X))
    # Walk the wrapper until we find the bare keycode, then look up the
    # modifier-wrapped alias in base_aliases.
    modifiers = []
    inner = kc
    while True:
        m = re.match(r'^S\(ALGR\((.+)\)\)\s*$', inner)
        if m:
            modifiers.append("S")
            inner = m.group(1).strip()
            modifiers.append("ALGR")
            continue
        m = re.match(r'^S\((.+)\)\s*$', inner)
        if m:
            modifiers.append("S")
            inner = m.group(1).strip()
            continue
        m = re.match(r'^ALGR\((.+)\)\s*$', inner)
        if m:
            modifiers.append("ALGR")
            inner = m.group(1).strip()
            continue
        break

    found = []
    # Find the wrapping keycode that produces the modifier-wrapped character
    for lhs_keycode, tokens in base_aliases.items():
        if len(tokens) >= 2:
            token_mods = [t for t in tokens if t in ("S", "ALGR")]
            token_keys = [t for t in tokens if t not in ("S", "ALGR")]
            if token_keys and token_keys[0] == inner and sorted(token_mods) == sorted(modifiers):
                if lhs_keycode in char_map:
                    found.append((lhs_keycode, char_map[lhs_keycode]))
                else:
                    found.append((lhs_keycode, "?"))
    # Also handle the bare keycode
    if inner in char_map:
        found.insert(0, (inner, char_map[inner]))
    if not found:
        print(f"# Layout: {layout}")
        print(f"# Keycode: {kc}")
        print()
        print(f"NOT FOUND: {kc!r} is not a keycode on {layout}")
        sys.exit(1)
    print(f"# Layout: {layout}")
    print(f"# Keycode: {kc}")
    print(f"# Source keymap_extras: {path}")
    print(f"# Base keycode map: scripts/layout-reachability-base.txt")
    print()
    for k, c in found:
        path_info = resolve(k)
        via = format_via(path_info, k)
        print(f"{k:14}  {c:6}  via {via}")
    sys.exit(0)

# Default mode: character -> keycodes
target_norm = target.casefold()

matches = []
for kc, char in char_map.items():
    if char.casefold() == target_norm:
        path = resolve(kc)
        via = format_via(path, kc)
        matches.append((kc, via, char))

seen = set()
out = []
for kc, via, char in matches:
    if (kc, via) in seen:
        continue
    seen.add((kc, via))
    out.append((kc, via, char))

print(f"# Layout: {layout}")
print(f"# Character: {requested!r} (target: {target!r})")
print(f"# Source keymap_extras: {path}")
print(f"# Base keycode map: scripts/layout-reachability-base.txt")
print()

if not out:
    print(f"NOT REACHABLE: {requested!r} on {layout}")
    sys.exit(1)

for kc, via, char in out:
    print(f"{kc:14}  {char:6}  via {via}")

sys.exit(0)
PY
