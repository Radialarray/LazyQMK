#!/usr/bin/env bash
#
# scripts/doctor.sh — Environment health gate.
#
# Runs `lazyqmk doctor --json`, prints a 1-line summary, exits non-zero
# if any dependency is missing or unknown.
#
# Usage:
#   bash scripts/doctor.sh           # basic
#
# Exit codes:
#   0  all dependencies available
#   1  one or more dependencies missing/unknown
#   2  lazyqmk or jq not installed
#
# JSON shape (real):
#   {
#     "status": "ready|warning|error",
#     "passed": <int>,
#     "failed": <int>,
#     "unknown": <int>,
#     "dependencies": [
#       {"name": "QMK CLI"|"ARM GCC"|"AVR GCC"|"QMK Firmware",
#        "status": "available"|"missing"|"unknown",
#        "version": "...",
#        "message": "...",
#        "installation_hint": "..." | null}
#     ],
#     "platform": "macOS"|"Linux"|"Windows"
#   }

set -euo pipefail

# Check lazyqmk is installed
if ! command -v lazyqmk >/dev/null 2>&1; then
    echo "FAIL: lazyqmk not installed"
    echo "Install: brew install Radialarray/lazyqmk/lazyqmk"
    exit 2
fi

# jq is required
if ! command -v jq >/dev/null 2>&1; then
    echo "FAIL: jq not installed (required to parse doctor JSON)"
    echo "Install: brew install jq"
    exit 2
fi

# Run doctor — capture stdout/stderr separately
DOCTOR_STDOUT=$(mktemp)
DOCTOR_STDERR=$(mktemp)
trap 'rm -f "$DOCTOR_STDOUT" "$DOCTOR_STDERR"' EXIT

if ! lazyqmk doctor --json >"$DOCTOR_STDOUT" 2>"$DOCTOR_STDERR"; then
    echo "FAIL: lazyqmk doctor exited non-zero"
    if [[ -s "$DOCTOR_STDERR" ]]; then
        cat "$DOCTOR_STDERR"
    fi
    exit 1
fi

# Parse real JSON shape
STATUS=$(jq -r '.status' "$DOCTOR_STDOUT")
PASSED=$(jq '.passed' "$DOCTOR_STDOUT")
FAILED=$(jq '.failed' "$DOCTOR_STDOUT")
UNKNOWN=$(jq '.unknown' "$DOCTOR_STDOUT")
TOTAL=$(jq '.dependencies | length' "$DOCTOR_STDOUT")

echo "Doctor: $STATUS ($PASSED/$TOTAL passed, failed=$FAILED, unknown=$UNKNOWN)"

# List non-available dependencies
if [[ "$FAILED" -gt 0 ]] || [[ "$UNKNOWN" -gt 0 ]] || [[ "$STATUS" != "ready" ]]; then
    echo ""
    echo "Issues:"
    jq -r '.dependencies[] | select(.status != "available") | "  ✗ \(.name): \(.status) — \(.message)\(if .installation_hint then "\n    Fix: \(.installation_hint)" else "" end)"' \
      "$DOCTOR_STDOUT"
    exit 1
fi

exit 0