# Task for pi-orchestrator.coder-mid-minimax

Add scoped `#[allow(dead_code)]` annotations with brief justification comments to specific items in the LazyQMK Rust codebase. These are bin/lib split artifacts (bin doesn't link test-only public API). DO NOT add `#![allow]` at file/module level. Use per-item allow with justification comment only.

Location: /Users/svenlochner/dev/LazyQMK

For EACH of the following 8 sites:
1. Read the file at the line range
2. Read the function/variable above the line number
3. Add `#[allow(dead_code)] // <brief justification, ~5-15 words>` IMMEDIATELY BEFORE the item
4. Run `cargo check --lib --features web 2>&1 | tail -3` from /Users/svenlochner/dev/LazyQMK after each batch

SITES TO FIX:

1. src/models/keyboard_geometry.rs:78 — `pub const fn with_width` 
2. src/models/keyboard_geometry.rs:85 — `pub const fn with_height`
3. src/models/keyboard_geometry.rs:92 — `pub const fn with_rotation`
4. src/models/keyboard_geometry.rs:200 — `pub fn key_count` and `pub fn get_key_by_led` (both on line 200; check actual line, may be one per)
5. src/models/layer.rs:17 — `pub const DEFAULT_QMK_LAYER_LIMIT: u8 = 8;`
6. src/models/layer.rs:145 — `pub fn with_label` (or wherever it is)
7. src/models/layer.rs around 145 — `pub fn with_description`
8. src/models/layer.rs around 287 — `pub fn set_category`, `pub fn set_default_color`, `pub fn set_name`
9. src/models/rgb.rs:95 — `pub const fn to_ratatui_color` (note: it's behind #[cfg(feature = "ratatui")])
10. src/models/visual_layout_mapping.rs around 131 — `pub fn led_to_matrix_pos`, `pub fn matrix_to_visual_pos`, `pub fn get_bounds`
11. src/shortcuts.rs around 275 — `pub fn matches` (3-arg form on ShortcutRegistry)
12. src/tui/mod.rs around 433 — `LayoutPicker(LayoutPicker)` variant in `ActiveComponent` enum
13. src/tui/metadata_editor.rs:39 — `Closed` variant of `MetadataEditorEvent` (already restored by previous edit; just needs allow)

RUN ORDER:
1. Do all sites 1-13 above by reading the function/variable first, then adding the allow attribute above it.
2. After each site, verify with `cargo check --lib --features web 2>&1 | tail -1`.
3. After all sites done, run `cargo clippy --all-features -- -D warnings 2>&1 | tail -3` to confirm clean.
4. Run `cargo test 2>&1 | tail -3` to confirm 1474 tests still pass.

DO NOT:
- Add `#[allow(dead_code)]` to anything not in the list above
- Add module/file-level allows
- Edit any other files
- Run rustfmt or similar formatters that would touch unrelated lines

OUTPUT FINAL RESPONSE WITH:
- diff stats: `git diff --stat src/ | tail -15`
- clippy result: `cargo clippy --all-features -- -D warnings 2>&1 | tail -3`
- cargo test result: `cargo test 2>&1 | tail -3`
- list of any sites that needed different handling

BEGIN NOW. cwd: /Users/svenlochner/dev/LazyQMK

---
**Output:**
Write your findings to exactly this path: /tmp/minimax-audit-fix.md
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