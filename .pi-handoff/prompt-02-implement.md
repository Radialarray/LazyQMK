# Continuation Prompt 02 — Implement with `implement-and-review` chain (minimax only)

> **Use this prompt after prompt 1 has finished planning in `bd`.** Drop
> this into a fresh pi session (or `pi -p "$(cat .pi-handoff/prompt-02-implement.md)"`).
>
> Implementation uses the `implement-and-review` chain with the
> **minimax provider only**, as requested.

---

## Context

You are continuing LazyQMK work. Prompt 1 already:

- Re-read the deferred issues (see `.pi-handoff/deferred-issues.md`)
- Re-opened or recreated them as fresh `bd` issues with proper
  dependencies
- Recorded the recommended starting issue in the parent epic comments

**Your job:** drive those issues through to green using the
`implement-and-review` chain (see `~/.pi/agent/chains/implement-and-review.chain.json`).

The chain is **Implement → Review → Fix**:

1. `coder-mid-minimax` (minimax/MiniMax-M3) implements the task with
   `acceptance: attested`
2. Three `reviewer-minimax` agents (minimax/MiniMax-M3) review the diff
   in parallel, each from a different angle:
   - correctness & regressions
   - tests & validation
   - simplicity & maintainability
3. A `worker` agent (the chain's `Fix` stage) applies the accepted fixes

> **Provider constraint:** every agent in the chain must use `minimax/MiniMax-M3`. The
> chain's default `worker` agent for the Fix stage may not have the
> minimax model pinned — **see the chain patch below** if it doesn't.

---

## Required reading (read these files before launching the chain)

1. `/Users/svenlochner/dev/LazyQMK/.pi-handoff/README.md` — workflow,
   quality gates, commit format, git author identity
2. `/Users/svenlochner/dev/LazyQMK/.pi-handoff/prompt-01-plan.md` —
   what the planner produced (the new epic/task IDs you should drive)
3. `/Users/svenlochner/dev/LazyQMK/AGENTS.md` — file-size cap, lint rules,
   commit format, pre-release checklist
4. The **bd issue description** for the specific task you're running —
   it has the acceptance criteria, target file structure, and risk notes

## Provider constraint (minimax ONLY)

The user explicitly requested **only the minimax provider**. A
project-local chain has been provided at:

```text
/Users/svenlochner/dev/LazyQMK/.pi/chains/implement-and-review-minimax.chain.json
```

It is identical in shape to `~/.pi/agent/chains/implement-and-review.chain.json`
but every stage uses a `-minimax` agent. **Use this project-local chain
instead of the global one.** Verified agent references:

| Phase | Agent | Model |
|---|---|---|
| Implement | `pi-orchestrator.coder-mid-minimax` | minimax/MiniMax-M3 |
| Review × 3 (parallel) | `pi-orchestrator.reviewer-minimax` | minimax/MiniMax-M3 |
| Fix | `pi-orchestrator.coder-mid-minimax` | minimax/MiniMax-M3 |

(verified `non-minimax agents: []`)

## Workflow (mandatory, per task)

### Step 0 — Pre-flight (5 min, before each task)

```bash
cd /Users/svenlochner/dev/LazyQMK
git status                              # must be clean
git log --oneline -3
cargo test --tests --lib | tail -3       # 1446 passed baseline
cargo clippy --all-features -- -D warnings | tail -3  # 0 errors
```

If anything fails: **stop and report**.

### Step 1 — `bd update <task-id> --claim` (or `--status in_progress`)

This both claims the task in `bd` and signals to the chain that work is
underway.

### Step 2 — Launch the chain (use the project-local minimax-only chain)

The chain is at `.pi/chains/implement-and-review-minimax.chain.json`. Two
ways to invoke it:

**Option A — project-local chain (preferred):**

```bash
pi chain run /Users/svenlochner/dev/LazyQMK/.pi/chains/implement-and-review-minimax.chain.json \
  "<task description>"
```

**Option B — via the orchestrator skill:**

```text
/run-chain /Users/svenlochner/dev/LazyQMK/.pi/chains/implement-and-review-minimax.chain.json
```

`task description` should include:

- The bd issue id and title (so the agents can `bd show <id>` for context)
- The acceptance criteria verbatim from the bd description
- The baseline commands and expected output (`cargo test --tests --lib`
  → 1446 passed, `cargo clippy --all-features -- -D warnings` → 0 errors)
- The project-local chain path (so reviewers can confirm they got the
  right chain)

**Do not** invoke the global `implement-and-review` chain — its Fix
stage uses the non-minimax `worker` agent.

### Step 3 — Validate the chain's output

After the chain reports `acceptance: attested`:

1. `git diff --stat HEAD` — review the changed files for scope creep
2. `cargo test --tests --lib | tail -3` — must still show 1446 passed
3. `cargo clippy --all-features -- -D warnings | tail -3` — must still
   show 0 errors
4. If the task modifies the firmware generator, additionally:
   `cargo test --test firmware_gen_tests | tail -3` — golden fixtures
   must stay byte-identical
5. `git diff --check` — make sure there are no whitespace-only
   conversions in the diff (reformat-noise guard)

If any gate fails, **do not close the bd task.** Instead:

- `bd comments add <task-id> "Validation failed: <details>"` to record why
- Decide whether to re-run the chain (it auto-fixes once via the Fix
  stage; beyond that you need a fresh chain invocation)
- Surface the failure to the human before continuing

### Step 4 — Commit

```bash
git add <specific files>                 # not `git add -A` — never
git status                               # confirm scope
git commit -F /tmp/<task-id>.txt         # conventional commit, file body
git log --oneline -1                     # confirm commit landed
```

Commit message template (must be in `~/.pi/agent/agents/coder-mid-minimax.agent.md`
form):

```text
<type>(<scope>): <imperative summary>

<why>

<what changed>

<verification: tests + clippy>
```

Example for a split:

```text
refactor(firmware): split generator.rs mega-functions per group

The 2956-line generator.rs had four mega-functions (generate_ripple_overlay_code
429 lines, generate_combo_code 372, generate_conditional_encoder_map 343,
generate_merged_config_h 297) that all shared self.layout/self.geometry/self.keycode_db.
Extracting per-group helpers into generator/{keymap,encoder,ripple,idle,combo,
tap_dance,config_h,rules_mk}.rs brings the main file to <400 lines and keeps
each helper independently testable.

Verification: cargo test --tests --lib passes (1446/1446);
firmware_gen_tests golden fixtures unchanged byte-for-byte.
```

### Step 5 — Push and close

```bash
git push origin main
bd close <task-id> --reason="<one-line summary of what was done + verification line>"
```

### Step 6 — Loop

Pick the next ready task from the parent's blocked list:

```bash
bd ready --parent=<epic-id>      # shows unblocked children
```

Follow the recommended order from prompt 1's output. Stop when:

- All tasks in the chosen epic are closed, **or**
- The chain produces a `verification failed` outcome you can't resolve
  in 2 attempts (escalate to human)

---

## Hard constraints (carry over from prompt 1)

- **Never** run `cargo fmt` on the whole repo. The `coder-mid-minimax`
  agent has explicit guidance against reformatting whole files.
- **Never** add `#[allow(...)]` or `#[ignore]` to silence clippy/lints —
  fix the underlying issue. (AGENTS.md §Code Review Checklist)
- **Never** commit `.pi-subagents/`, `.opencode/`, `.pi/`, `node_modules/`,
  scratch files, or untracked build artifacts. The reviewer's
  project-specific rubric flags these as non-blocking but you should
  exclude them in `git add` explicitly.
- **Never** widen scope: if the chain produces a diff that touches
  files outside the bd task's stated scope, `git reset HEAD` and re-run
  with tighter task description.
- File-size cap: 500 lines target, 1000 hard limit. (AGENTS.md)
- One-commit-per-task: don't bundle multiple bd tasks in a single commit.

## Quality gates (re-state for clarity)

```bash
cargo test --tests --lib              # must show 1446 passed, 5 ignored
cargo clippy --all-features -- -D warnings  # must show 0 errors
cargo fmt --check                     # clean (after reformat-noise guard)
```

If any of these regress, **the chain's attestation is invalid** — go
back to Step 3 validation, do not proceed.

## Output format (when you stop)

Reply with a single markdown block:

```markdown
## Implementation report

**Closed in this run:** [list of bd IDs]
**Pushed to:** origin/main (commit SHAs)
**Chain invocations:** N (M successes, K failures)
**Validation gates:**
- `cargo test` → 1446 passed, 5 ignored
- `cargo clippy -D warnings` → 0 errors
- `cargo fmt --check` → clean
- (firmware only) `firmware_gen_tests` → golden fixtures unchanged

**Open blockers (if any):** [next task + reason]

**Recommended next prompt-02 run:** start at LazyQMK-fsplit-<X>
because <reason>
```

## Stopping conditions

Stop the run when any of the following is true:

1. All Tier-1 through Tier-3 tasks are closed and pushed
2. The chain has failed twice on the same task (escalate to human)
3. The 1446-test baseline breaks and you can't restore it in one cycle
4. The session has been running for >4 hours of agent time
5. The human signals a stop (via `Stop` button or message)

If you stop for any reason other than (1), write a status block first so
the next prompt run can resume cleanly.
