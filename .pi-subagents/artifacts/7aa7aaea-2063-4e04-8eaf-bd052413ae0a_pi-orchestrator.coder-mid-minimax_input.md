# Task for pi-orchestrator.coder-mid-minimax

Fix bd issue LazyQMK-aopx.1 in the LazyQMK TUI main loop.

LOCATION: /Users/svenlochner/dev/LazyQMK/src/tui/mod.rs, function run_tui, around lines 935-942.

CURRENT BUGGY CODE inside run_tui (replace this exact block):
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

PROBLEM: The else-if branch calls event::read() a second time when first call consumed a non-Key event. Causes input lag.

FIX (replace with this):
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

STEPS:
1. Read /Users/svenlochner/dev/LazyQMK/src/tui/mod.rs around lines 920-960 to verify exact context
2. Use edit tool to replace the if-let-else-if block with the match block (preserve indentation)
3. Run `cargo test --lib 2>&1 | tail -10` from /Users/svenlochner/dev/LazyQMK
4. Run `cargo clippy --all-features -- -D warnings 2>&1 | tail -10` from /Users/svenlochner/dev/LazyQMK
5. Verify both pass

OUTPUT YOUR FINAL RESPONSE WITH:
- The exact `git diff` of the change
- cargo test result (must show all tests pass)
- cargo clippy result (must be clean)

Do not edit any other file. Do not add #[allow] attributes. Do not widen scope.

cwd: /Users/svenlochner/dev/LazyQMK

---
**Output:**
Write your findings to exactly this path: /tmp/chain-step1.md
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