# Task for pi-orchestrator.coder-mid-minimax

Fix bd issue LazyQMK-aopx.1 in the LazyQMK TUI main loop. This is the first step in the implement-and-review chain.

LOCATION: /Users/svenlochner/dev/LazyQMK/src/tui/mod.rs, function run_tui, around lines 935-942 (inside the loop).

CURRENT BUGGY CODE:
```rust
if event::poll(Duration::from_millis(100))? {
    if let Event::Key(key) = event::read()? {
        if handle_key_event(state, key)? {
            break; // User quit
        }
    } else if let Event::Resize(_, _) = event::read()? {
        // Terminal resized, will re-render on next loop
    }
}
```

PROBLEM: When the queue holds a non-Key event (Resize, Mouse, Focus), the first event::read() consumes it but the if-let does not match; then the else-if calls event::read() AGAIN, which blocks indefinitely waiting for the next event. Bug class: input lag.

FIX (apply this):
```rust
if event::poll(Duration::from_millis(100))? {
    match event::read()? {
        Event::Key(key) => {
            if handle_key_event(state, key)? {
                break;
            }
        }
        Event::Resize(_, _) => {
            // Terminal resized, will re-render on next loop
        }
        _ => {}
    }
}
```

CONTEXT:
- LazyQMK Rust codebase, Rust 1.91.1+
- `event` is `crossterm::event` (use as `event::poll`, `event::read`)
- `Event` is `crossterm::event::Event` (already in scope via `use crossterm::event::{...}`)
- cargo test + cargo clippy --all-features -- -D warnings must pass
- AGENTS.md forbids #[allow] attributes

ACCEPTANCE:
1. cargo test passes
2. cargo clippy --all-features -- -D warnings clean
3. No scope creep beyond the run_tui event loop fix
4. No new #[allow] attributes
5. Do not edit other files

EVIDENCE TO RETURN (in your final response):
- changed-files: exact file + line numbers modified (use git diff output)
- commands-run: output of `cargo test` and `cargo clippy --all-features -- -D warnings`
- validation-output: confirm both green
- residual-risks: any concerns remaining

Output cwd when running cargo: /Users/svenlochner/dev/LazyQMK

---
**Output:**
Write your findings to exactly this path: /tmp/chain-implementer-result.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: attested
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: review-findings, residual-risks

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