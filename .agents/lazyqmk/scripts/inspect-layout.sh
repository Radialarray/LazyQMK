#!/usr/bin/env bash
#
# scripts/inspect-layout.sh — Dump every section of a layout as JSON.
#
# Runs `lazyqmk inspect` for every section (metadata, layers, categories,
# tap-dances, settings) plus `lazyqmk layer-refs`. Pretty-prints each.
#
# Usage:
#   bash scripts/inspect-layout.sh <path-to-layout.json>
#
# Exit codes:
#   0  all sections read successfully
#   1  one or more sections failed (layout likely invalid)
#   2  jq/lazyqmk missing or layout file missing

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: bash scripts/inspect-layout.sh <path-to-layout.json>"
    exit 2
fi

LAYOUT="$1"

if [[ ! -f "$LAYOUT" ]]; then
    echo "FAIL: layout file not found: $LAYOUT"
    exit 2
fi

if ! command -v lazyqmk >/dev/null 2>&1; then
    echo "FAIL: lazyqmk not installed"
    exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "WARN: jq not installed, output will be raw JSON"
fi

echo "=========================================="
echo "Layout: $LAYOUT"
echo "=========================================="
echo ""

# Track section failures
SECTION_FAILURES=0

run_section() {
    local section="$1"
    echo "── $section ──"
    local out
    out=$(mktemp)
    local status=0
    lazyqmk inspect --layout "$LAYOUT" --section "$section" --json >"$out" 2>&1 || status=$?
    if [[ $status -eq 0 ]] && [[ -s "$out" ]]; then
        if command -v jq >/dev/null 2>&1; then
            jq . "$out"
        else
            cat "$out"
        fi
    else
        echo "  (error)"
        sed 's/^/    /' "$out" | head -5
        SECTION_FAILURES=$((SECTION_FAILURES + 1))
    fi
    rm -f "$out"
    echo ""
}

run_section "metadata"
run_section "layers"
run_section "categories"
run_section "tap-dances"
run_section "settings"

echo "── layer-refs ──"
LR_OUT=$(mktemp)
LR_STATUS=0
lazyqmk layer-refs --layout "$LAYOUT" --json >"$LR_OUT" 2>&1 || LR_STATUS=$?
if [[ $LR_STATUS -eq 0 ]] && [[ -s "$LR_OUT" ]]; then
    if command -v jq >/dev/null 2>&1; then
        jq . "$LR_OUT"
    else
        cat "$LR_OUT"
    fi
else
    echo "  (error)"
    sed 's/^/    /' "$LR_OUT" | head -5
    SECTION_FAILURES=$((SECTION_FAILURES + 1))
fi
rm -f "$LR_OUT"

echo ""
echo "=========================================="
if [[ "$SECTION_FAILURES" -gt 0 ]]; then
    echo "Done with $SECTION_FAILURES section failure(s)"
else
    echo "Done"
fi
echo "=========================================="

exit "$SECTION_FAILURES"