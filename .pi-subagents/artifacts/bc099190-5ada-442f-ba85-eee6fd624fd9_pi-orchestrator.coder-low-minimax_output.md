# OnceLock Migration — src/parser/layout.rs

## Summary
Migrated 6 hardcoded `Regex::new(...).unwrap()` sites in `src/parser/layout.rs` to use
`std::sync::OnceLock<Regex>` via module-scope helper functions, achieving compile-once
semantics without introducing `lazy_static`/`once_cell` crates.

## Sites Migrated
| Line | Helper | Original Literal |
|------|--------|------------------|
| 184  | `tag_regex()` | `r"^[a-z0-9-]+$"` |
| 257  | `layer_regex()` | `r"^##\s+Layer\s+(\d+):\s+(.+)$"` |
| 449  | `keycode_regex()` | multi-line keycode cell syntax |
| 484  | `category_regex()` | `r"^-\s+([a-z][a-z0-9-]*):\s+(.+?)\s+\(#([0-9A-Fa-f]{6})\)$"` |
| 528  | `combo_regex()` | multi-line combo syntax |
| 1178 | `desc_regex()` | `r"^-\s+(\d+):(\d+):(\d+):\s+(.+)$"` |

All 6 regex literals are unique → each gets its own helper. All helpers are placed at
module scope (right after the `use` block), so they're shared if any function ever needs
them later.

## Diff Stats
```
src/parser/layout.rs | 82 ++++++++++++++++++++++++++++++++++++----------------
 1 file changed, 57 insertions(+), 25 deletions(-)
```

## Validation Output

| Command | Result |
|---------|--------|
| `cargo check --lib` | 0 errors, 2 warnings (web-frontend build-script, unrelated) |
| `cargo test --lib` | **560 passed**, 1 ignored |
| `cargo test` | **1474 passed**, 5 ignored |
| `cargo clippy --all-features -- -D warnings` | exit 0, clean |
| `rg 'Regex::new\(' src/parser/layout.rs \| wc -l` | **6** (all inside `OnceLock::get_or_init` closures) |
| `rg 'let .*_regex = Regex' src/parser/layout.rs \| wc -l` | **0** |

## Acceptance Notes

- **Pattern preserved**: every original regex literal was copied byte-for-byte into its
  helper's `OnceLock::get_or_init` closure. No semantics change.
- **Call sites updated**: every local `let re = Regex::new(...).unwrap();` is removed.
  Each usage (`is_match` or `captures`) now invokes the corresponding helper:
  `tag_regex().is_match(tag)`, `layer_regex().captures(...)`, etc.
- **No third-party crates**: `std::sync::OnceLock` only (stable since Rust 1.70; project
  uses 1.91.1+).
- **Scope hygiene**: helpers live at module scope (above all `impl`/`fn`), so they don't
  pollute any specific function. Comments near the call sites were tightened to note the
  OnceLock caching.
- **No business-logic changes**: parser behavior is identical; the only difference is
  compilation cost amortization.