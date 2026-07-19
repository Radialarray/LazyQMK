# Task for pi-orchestrator.coder-low-minimax

Migrate 6 hardcoded `Regex::new(...).unwrap()` sites in /Users/svenlochner/dev/LazyQMK/src/parser/layout.rs to use `std::sync::OnceLock` for cached compile-once semantics.

Lines to migrate (from `rg 'Regex::new\\(' src/parser/layout.rs`):

- 184: inside `validate_metadata(&self) -> Result<()>` — `let tag_regex = Regex::new(r"^[a-z0-9-]+$").unwrap();`
- 257: inside parsing function — `let layer_regex = Regex::new(r"^##\s+Layer\s+(\d+):\s+(.+)$").unwrap();`
- 449: inside `parse_keycode_section` or similar — `let keycode_regex = Regex::new(...)` (read exact pattern)
- 484: inside category parser — `Regex::new(r"^-\s+([a-z][a-z0-9-]*):\s+(.+?)\s+\(#([0-9A-Fa-f]{6})\)$").unwrap();`
- 528: inside combo parser — `let combo_regex = Regex::new(...)` (read exact pattern)
- 1178: somewhere in deserializer — `let desc_regex = Regex::new(r"^-\s+(\d+):(\d+):(\d+):\s+(.+)$").unwrap();`

Each has a hardcoded string. NO business logic change — only cache the regex so it's compiled once.

PATTERN: for each site, replace the local `let re = Regex::new(LITERAL).unwrap();` with a static cached version. Use `std::sync::OnceLock<Regex>` (Rust 1.70+, our project is 1.91.1+). Define a small helper at file scope:

```rust
fn tag_regex() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[a-z0-9-]+$").unwrap())
}
```

Then at the call site, replace `tag_regex.is_match(tag)` (etc.) — the local `let tag_regex = ...` is gone, replaced by the function call. KEEP all call sites unchanged by using the function call instead. If a site uses `let re = Regex::new(...); re.is_match(x)`, replace with the helper and `re.is_match(x)` → `<helper>().is_match(x)`.

CRITICAL: 
1. Read each line context first. The `unwrap()` is inside a function body. Add the helper INSIDE or OUTSIDE that function — whichever doesn't pollute scope. Recommendation: add helpers at MODULE scope (above the first use), so they're shared if the same regex is used in multiple functions.
2. Preserve EXACT REGEX SEMANTICS — do not change the literal pattern.
3. Do not refactor other code in the file.
4. Do not introduce `lazy_static` or `once_cell` crates — std-only.
5. Add `use std::sync::OnceLock;` near the top of the file (after the existing `use std::path::Path;`) if not already present.

STEPS:
1. `cd /Users/svenlochner/dev/LazyQMK`
2. Read each of the 6 lines and the surrounding 5 lines for context.
3. If any pair of regexes shares an identical literal, share the helper.
4. For 2 of them, write the helpers at module scope (before any `fn` definitions). For the remaining 4, also module-scope helpers.
5. Replace the local `let re = Regex::new(...).unwrap();` with function calls.

VALIDATION after each site:
- `cargo check --lib 2>&1 | tail -3` — must remain clean (0 errors, 2 expected warnings).

FINAL VALIDATION:
- `cargo test --lib 2>&1 | tail -3` — must show 560 passed
- `cargo test 2>&1 | tail -3` — must show 1474 passed
- `cargo clippy --all-features -- -D warnings 2>&1 | tail -3` — must be clean
- `rg 'Regex::new\(' src/parser/layout.rs | wc -l` — must show 6 (the function-call pattern remains; only .unwrap() on hardcoded literal stays in the OnceLock helper, not in call sites)
- `rg 'let .*_regex = Regex' src/parser/layout.rs | wc -l` — must show 0

If a regex literal at line 449 or 528 is long, multi-line, and shared with another site, share the helper. Otherwise each site gets its own helper.

OUTPUT FINAL RESPONSE WITH:
- `git diff --stat src/parser/layout.rs`
- cargo check result
- cargo test --lib result
- cargo test result
- cargo clippy result
- the count of `Regex::new(` and `let .*_regex = Regex` left in the file

Use `context: "fresh"` to start with clean state. cwd is /Users/svenlochner/dev/LazyQMK.

---
**Output:**
Write your findings to exactly this path: /tmp/minimax-oncelock.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: required by reviewer.

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```