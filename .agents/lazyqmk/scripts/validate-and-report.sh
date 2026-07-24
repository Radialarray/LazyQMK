#!/usr/bin/env bash
#
# scripts/validate-and-report.sh — Run all validations on a layout.
#
# Runs:
#   1. lazyqmk validate (basic or --strict)
#   2. lazyqmk tap-dance validate
#   3. lazyqmk layer-refs
# Then prints a consolidated report.
#
# Usage:
#   bash scripts/validate-and-report.sh <path-to-layout.json> [--strict]
#
# Exit codes:
#   0  all validations pass
#   1  one or more validations failed
#   2  lazyqmk/jq missing or layout file invalid

set -euo pipefail

if [[ $# -lt 1 ]]; then
    echo "Usage: bash scripts/validate-and-report.sh <path-to-layout.json> [--strict]"
    exit 2
fi

LAYOUT="$1"
STRICT_FLAG=""
if [[ "${2:-}" == "--strict" ]]; then
    STRICT_FLAG="--strict"
fi

if [[ ! -f "$LAYOUT" ]]; then
    echo "FAIL: layout file not found: $LAYOUT"
    exit 2
fi

if ! command -v lazyqmk >/dev/null 2>&1; then
    echo "FAIL: lazyqmk not installed"
    exit 2
fi

if ! command -v jq >/dev/null 2>&1; then
    echo "FAIL: jq not installed (required for JSON parsing)"
    echo "Install: brew install jq"
    exit 2
fi

# Helper: run a lazyqmk command, capture stdout+stderr into globals,
# return command exit status. Caller MUST clean up $LAST_STDOUT / $LAST_STDERR.
run_json() {
    local desc="$1"
    shift
    local out err status
    out=$(mktemp)
    err=$(mktemp)
    status=0
    "$@" >"$out" 2>"$err" || status=$?
    LAST_STDOUT="$out"
    LAST_STDERR="$err"
    LAST_STATUS="$status"
    if [[ $status -ne 0 ]]; then
        echo "  ✗ $desc (exit $status)"
        if [[ -s "$err" ]]; then
            sed 's/^/    /' "$err" | head -5
        fi
    fi
    return "$status"
}

TOTAL_ERRORS=0
TOTAL_WARNINGS=0
OVERALL_EXIT=0

echo "=========================================="
echo "Validating: $LAYOUT"
echo "=========================================="
echo ""

# 1. Basic validate
echo "── 1. validate ──"
# `validate` returns exit 1 when there are errors, but stdout still contains the
# detailed JSON report. Always parse stdout when present. Use `|| true` so
# set -e doesn't terminate on the failing validate.
VALIDATE_STATUS=0
run_json "validate" lazyqmk validate --layout "$LAYOUT" $STRICT_FLAG --json || VALIDATE_STATUS=$?

if [[ -s "$LAST_STDOUT" ]]; then
    VALID=$(jq -r '.valid // "false"' "$LAST_STDOUT" 2>/dev/null)
    ERRORS=$(jq '[.errors[]? | select(.severity == "error")] | length' "$LAST_STDOUT" 2>/dev/null || echo 0)
    WARNINGS=$(jq '[.errors[]? | select(.severity == "warning")] | length' "$LAST_STDOUT" 2>/dev/null || echo 0)

    if [[ "$VALID" == "true" && "$VALIDATE_STATUS" -eq 0 ]]; then
        echo "  ✓ validation passed ($ERRORS errors, $WARNINGS warnings)"
        if [[ "$ERRORS" -gt 0 ]]; then
            jq -r '.errors[] | select(.severity == "error") | "    ERROR: \(.message)"' "$LAST_STDOUT"
            TOTAL_ERRORS=$((TOTAL_ERRORS + ERRORS))
        fi
        if [[ "$WARNINGS" -gt 0 ]]; then
            jq -r '.errors[] | select(.severity == "warning") | "    WARN: \(.message)"' "$LAST_STDOUT"
            TOTAL_WARNINGS=$((TOTAL_WARNINGS + WARNINGS))
        fi
    else
        echo "  ✗ validation failed ($ERRORS errors, $WARNINGS warnings)"
        if [[ "$ERRORS" -gt 0 ]] || [[ "$WARNINGS" -gt 0 ]]; then
            jq -r '.errors[]? | "    \(.severity | ascii_upcase): \(.message)"' "$LAST_STDOUT"
            TOTAL_ERRORS=$((TOTAL_ERRORS + ERRORS))
            TOTAL_WARNINGS=$((TOTAL_WARNINGS + WARNINGS))
        else
            TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        fi
    fi
elif [[ "$VALIDATE_STATUS" -ne 0 ]]; then
    echo "  ✗ validate failed (no JSON output)"
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
else
    echo "  ✗ no JSON output"
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
fi
rm -f "$LAST_STDOUT" "$LAST_STDERR"
echo ""

# 2. Tap dance validate
echo "── 2. tap-dance validate ──"
TD_STATUS=0
run_json "tap-dance validate" lazyqmk tap-dance validate --layout "$LAYOUT" --json || TD_STATUS=$?

if [[ -s "$LAST_STDOUT" ]]; then
    VALID=$(jq -r '.valid // "false"' "$LAST_STDOUT" 2>/dev/null)
    ORPHANED=$(jq '.orphaned | length' "$LAST_STDOUT" 2>/dev/null || echo 0)
    UNUSED=$(jq '.unused | length' "$LAST_STDOUT" 2>/dev/null || echo 0)

    if [[ "$VALID" == "true" && "$TD_STATUS" -eq 0 ]]; then
        echo "  ✓ tap dance refs valid (orphaned=$ORPHANED, unused=$UNUSED)"
    else
        echo "  ✗ tap dance refs invalid (orphaned=$ORPHANED)"
        if [[ "$ORPHANED" -gt 0 ]]; then
            jq -r '.orphaned[] | "    ORPHANED: TD(\(.)) used but not defined"' "$LAST_STDOUT"
            TOTAL_ERRORS=$((TOTAL_ERRORS + ORPHANED))
        else
            TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
        fi
    fi
    if [[ "$UNUSED" -gt 0 ]]; then
        jq -r '.unused[] | "    UNUSED: \(.) defined but never used"' "$LAST_STDOUT"
        TOTAL_WARNINGS=$((TOTAL_WARNINGS + UNUSED))
    fi
elif [[ "$TD_STATUS" -ne 0 ]]; then
    echo "  ✗ tap-dance validate failed"
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
else
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
fi
rm -f "$LAST_STDOUT" "$LAST_STDERR"
echo ""

# 3. Layer refs
echo "── 3. layer-refs ──"
LR_STATUS=0
run_json "layer-refs" lazyqmk layer-refs --layout "$LAYOUT" --json || LR_STATUS=$?

if [[ -s "$LAST_STDOUT" ]]; then
    WARNING_COUNT=$(jq '[.layers[].warnings[]?] | length' "$LAST_STDOUT" 2>/dev/null || echo 0)

    if [[ "$WARNING_COUNT" -eq 0 ]]; then
        echo "  ✓ layer refs clean (no warnings)"
    else
        echo "  ⚠ $WARNING_COUNT layer ref warnings:"
        jq -r '.layers[] | select(.warnings | length > 0) | "    Layer \(.number) (\(.name)): \(.warnings | length) warnings"' "$LAST_STDOUT"
        TOTAL_WARNINGS=$((TOTAL_WARNINGS + WARNING_COUNT))
    fi
elif [[ "$LR_STATUS" -ne 0 ]]; then
    echo "  ✗ layer-refs failed"
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
else
    TOTAL_ERRORS=$((TOTAL_ERRORS + 1))
fi
rm -f "$LAST_STDOUT" "$LAST_STDERR"
echo ""

# 4. Tap dance placeholder check (auto-created `TD(name)` with `single_tap = KC_NO`
#    is NOT flagged by `tap-dance validate` as orphan or invalid, but won't type
#    anything in firmware).
echo "── 4. tap-dance placeholders ──"
PLACEHOLDERS=$(jq -r '.tap_dances[]? | select(.single_tap == "KC_NO") | .name' "$LAYOUT" 2>/dev/null)
if [ -z "$PLACEHOLDERS" ]; then
    echo "  ✓ no placeholder tap dances"
else
    PLACEHOLDER_COUNT=$(echo "$PLACEHOLDERS" | wc -l | tr -d ' ')
    echo "  ⚠ $PLACEHOLDER_COUNT placeholder tap dance(s) (single_tap = KC_NO):"
    echo "$PLACEHOLDERS" | sed 's/^/    /'
    TOTAL_WARNINGS=$((TOTAL_WARNINGS + PLACEHOLDER_COUNT))
fi
echo ""

# Summary
echo "=========================================="
echo "Summary: errors=$TOTAL_ERRORS, warnings=$TOTAL_WARNINGS"
echo "=========================================="

if [[ "$TOTAL_ERRORS" -gt 0 ]]; then
    OVERALL_EXIT=1
fi
if [[ -n "$STRICT_FLAG" ]] && [[ "$TOTAL_WARNINGS" -gt 0 ]]; then
    OVERALL_EXIT=1
fi

exit "$OVERALL_EXIT"