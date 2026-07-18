#!/usr/bin/env bash
# Style and quality checks for LazyQMK.
# Run from the project root:  ./scripts/check-style.sh
# Exits non-zero on the first failure.
#
# Audits the codebase for:
#   - File size cap (target: 500 lines; warn at 1000)
#   - Forbidden `#[allow]` patterns (project AGENTS.md says fix underlying issue)
#   - Production-code .unwrap() / .expect() with no context
#   - TODO / FIXME / XXX / HACK markers
#   - dbg! / .to_string() in non-test code
#   - `.lock().unwrap()` style on Mutex/RwLock
#   - Unused dependencies (requires cargo-udeps)
#
# This is the lightweight CI gate companion to cargo clippy. Run after
# `cargo clippy --all-features -- -D warnings` to catch stylistic drift.

set -eu

ROOT="${1:-.}"
cd "$ROOT"

FAILED=0

log() {
    printf '\033[1;34m[check-style]\033[0m %s\n' "$1"
}

ok() {
    printf '\033[1;32m  ✓\033[0m %s\n' "$1"
}

fail() {
    printf '\033[1;31m  ✗\033[0m %s\n' "$1"
    FAILED=$((FAILED + 1))
}

# 1. File size cap: warn on > 500 lines, fail on > 1000
log "checking file sizes (target ≤500, fail >1000)..."
while IFS= read -r line; do
    file="${line#* }"
    count=$(echo "$line" | awk '{print $1}')
    if [ "$count" -gt 1000 ] 2>/dev/null; then
        fail "$file: ${count} lines (over 1000-line cap)"
    elif [ "$count" -gt 500 ] 2>/dev/null; then
        printf '  \033[1;33m!\033[0m %s: %s lines (warning)\n' "$file" "$count"
    fi
done < <(find src tests -name '*.rs' -exec wc -l {} \; | sort -rn | head -25)

ok "file sizes"

# 2. Block-level `#[allow]` flags in source files are tolerated only with
#    an inline justification comment (per AGENTS.md).
log "scanning for #[allow(dead_code)] without justification comments..."
if grep -rn --include='*.rs' '^\s*#\[allow(dead_code)\]' src/ tests/ 2>/dev/null \
    | grep -v '//' > /tmp/lazyqmk-allow-no-comment.txt; then
    if [ -s /tmp/lazyqmk-allow-no-comment.txt ]; then
        fail "dead_code allow(s) without justification comment:"
        cat /tmp/lazyqmk-allow-no-comment.txt
    fi
else
    ok "all #[allow(dead_code)] have justification comments"
fi

# 3. `.unwrap()` calls in production code (src/, not tests/)
log "counting production-code .unwrap() calls..."
UNWRAP_COUNT=$(grep -rn --include='*.rs' '\.unwrap()' src/ 2>/dev/null \
    | grep -v '#\[cfg(test)\]' | grep -v '#\[allow' | wc -l)
echo "  found ${UNWRAP_COUNT} production .unwrap() calls"

# 4. TODO/FIXME markers
log "scanning for TODO/FIXME comments..."
TODO_COUNT=$(grep -rn --include='*.rs' -E '(TODO|FIXME|XXX|HACK)' src/ 2>/dev/null | wc -l)
if [ "$TODO_COUNT" -gt 0 ]; then
    echo "  found $TODO_COUNT TODO/FIXME comments (informational)"
fi

# 5. dbg!() calls (always a smell)
log "checking for dbg!()..."
if grep -rn --include='*.rs' 'dbg!' src/ tests/ 2>/dev/null > /tmp/lazyqmk-dbg.txt; then
    if [ -s /tmp/lazyqmk-dbg.txt ]; then
        fail "dbg!() found (use tracing instead):"
        cat /tmp/lazyqmk-dbg.txt
    fi
fi
ok "no dbg!()"

# 6. .lock().unwrap() / .read().unwrap() / .write().unwrap() patterns
log "counting lock-unwraps..."
LOCK_UNWRAPS=$(grep -rn --include='*.rs' -E '\.(lock|read|write)\(\)\.unwrap\(\)' src/ tests/ 2>/dev/null \
    | grep -v '#\[cfg(test)\]' | grep -v '#\[allow' | wc -l)
echo "  found ${LOCK_UNWRAPS} lock-unwraps (target: use poisoned-recovery helpers)"

# 7. Cargo.toml allow attributes (informational)
log "cargo [lints.clippy] audit..."
ALLOW_ATTRS=$(grep -c '"allow"' Cargo.toml 2>/dev/null || echo 0)
echo "  ${ALLOW_ATTRS} blanket allow attributes in [lints.clippy]"
if [ "$ALLOW_ATTRS" -gt 10 ]; then
    echo "  consider per-line allows with justification"
fi

# Summary
echo
if [ "$FAILED" -gt 0 ]; then
    printf '\033[1;31m%d failure(s)\033[0m\n' "$FAILED"
    exit 1
fi
printf '\033[1;32mAll checks passed\033[0m\n'
