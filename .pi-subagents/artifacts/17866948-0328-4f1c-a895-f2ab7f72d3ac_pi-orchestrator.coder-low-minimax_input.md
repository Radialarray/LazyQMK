# Task for pi-orchestrator.coder-low-minimax

Move one file in the LazyQMK Rust codebase. Mechanical task.

TASK: Move /Users/svenlochner/dev/LazyQMK/src/tui/key_editor.rs to /Users/svenlochner/dev/LazyQMK/src/tui/editor/key_editor.rs and create the directory structure.

STEPS (all in cwd /Users/svenlochner/dev/LazyQMK):

1. Create directory: `mkdir -p src/tui/editor`
2. Use `git mv` to move the file (preserves history):
   `git mv src/tui/key_editor.rs src/tui/editor/key_editor.rs`
3. Verify the file is in the new location: `ls src/tui/editor/`

4. Update src/tui/mod.rs to:
   - Replace `pub mod key_editor;` with `pub mod editor;` (and put editor module near other editor-related stuff)
   - Inside `pub mod editor;` directory, add a `mod key_editor;` declaration
   - Verify the `pub use key_editor::KeyEditorState;` re-export still works (it should since the module is `crate::tui::editor::key_editor`)

5. Update all import sites that currently use `crate::tui::key_editor` or `key_editor::`:
   - `src/tui/mod.rs` lines using `key_editor::KeyEditorState`, `key_editor::render_key_editor`, `key_editor::is_key_assigned`, `key_editor::handle_input`, `key_editor::ComboEditPart`, `key_editor::ComboKeycodeType`
   - `src/tui/handlers/popups.rs` lines using `crate::tui::key_editor::*` and `key_editor::*`
   - Any other imports found via: `rg "key_editor::" src/ tests/ --type rust`

   The path changes from `crate::tui::key_editor` to `crate::tui::editor::key_editor`.

6. After edits, validate:
   - `cargo check --lib 2>&1 | tail -3` — must show 0 errors, 2 expected warnings
   - `cargo test --lib 2>&1 | tail -3` — must show 560 passed, 1 ignored
   - `cargo clippy --all-features -- -D warnings 2>&1 | tail -3` — must be clean

CONSTRAINTS:
- DO NOT modify src/tui/key_editor.rs internals — just move the file
- DO NOT add new dependencies
- DO NOT touch tests/ unless an import there references the moved module
- DO NOT use `mv` — use `git mv` to preserve history
- DO NOT trigger rustfmt cascades — if an edit triggers 50+ lines of unrelated formatting changes, `git restore --staged <file> && git checkout -- <file>` and use a more targeted patch via python heredoc
- If you encounter rustfmt auto-fixing noise (the edit tool sometimes runs rustfmt on save), restore affected files with `git checkout -- <file>` and re-apply only the targeted change

OUTPUT FINAL RESPONSE WITH:
- ls src/tui/editor/ output (single line `key_editor.rs`)
- git status --short output
- `git diff --stat src/tui/mod.rs src/tui/handlers/popups.rs 2>&1` (only the moved-file targets)
- cargo check result
- cargo test --lib result
- cargo clippy result
- the count of all import updates you made

Use `context: "fresh"` to start fresh. Begin now.

---
**Output:**
Write your findings to exactly this path: /tmp/minimax-move-keyeditor.md
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