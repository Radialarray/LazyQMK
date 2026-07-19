# Continuation Prompt 01 — Plan the deferred work in `bd`

> **Use this prompt to plan first, then implement.** Drop this into a
> fresh pi session (or `pi -p "$(cat .pi-handoff/prompt-01-plan.md)"`).

---

## Context

You are continuing a previous LazyQMK session that closed all 371 open
`bd` issues. The session left 12 items in `closed` status with close
notes describing the work as "deferred to a dedicated session" — these
are the items that follow.

This is **prompt 1 of 2**: **planning only.** You must read, think, write
into `bd`, and stop. **Do not implement anything yet.** Implementation is
prompt 2.

## Required reading (read these files before doing anything else)

1. `/Users/svenlochner/dev/LazyQMK/.pi-handoff/README.md` — session
   summary, our workflow conventions, quality gates, agent+chain choices
2. `/Users/svenlochner/dev/LazyQMK/.pi-handoff/deferred-issues.md` — the
   12 items grouped by tier (execution order), with one-line summaries
3. `/Users/svenlochner/dev/LazyQMK/AGENTS.md` — the project's quality
   rules, file-size caps, pre-release checklist, commit format
4. Each `bd show <id>` for the deferred items — read the full
   description and acceptance criteria. Don't skip this; the bd
   descriptions have the target file structure and risk notes baked in.

## Required tools & commands

- `bd` CLI (see `.pi-handoff/README.md` for workflow conventions)
- `git` to confirm current branch is clean: `git status`, `git log --oneline -3`
- `cargo test --tests --lib` and `cargo clippy --all-features -- -D warnings`
  to confirm the current state is the green baseline (1446 passed,
  0 errors)
- `wc -l <file>` to verify line counts in `bd show` output match reality
- `rg`/`grep`/`find` (scoped) for source reconnaissance

## Workflow (mandatory)

Follow this exact loop. Do not skip steps.

### Step 1 — Reconnaissance (15 min, no writes)

1. `cd /Users/svenlochner/dev/LazyQMK && git status` — confirm clean
   tree on `main`. If not, **stop and report**.
2. `cargo test --tests --lib | tail -3` — confirm 1446 passed baseline.
3. Read `.pi-handoff/README.md` and `.pi-handoff/deferred-issues.md`.
4. For each tier in `deferred-issues.md`:
   - Run `bd show <id>` for each item in the tier
   - Verify the described file path and line count match the current
     tree (`wc -l <path>`)
   - Note any drift between the close reason and reality
5. Decide: do any items overlap, conflict, or invalidate each other?

### Step 2 — Re-open in `bd` (use the correct API)

The 12 items are `closed` but the work is not done. You have two options:

- **Option A (preferred):** create new epics/issues with new IDs that
  reference the closed originals. This keeps the audit trail of the
  closure reason intact.
- **Option B:** reopen closed issues with `bd update <id> --status
  in_progress` after adding a `bd comments add <id> "Re-opening for
  dedicated session per close reason"`.

**Decision rule:** use Option A (new IDs) for any item whose close reason
explicitly said "deferred to a dedicated session" (all 12 qualify), and
keep the closed original for audit. Use Option B only if the user
explicitly asks you to.

### Step 3 — Build the bead structure

Create **one parent epic per tier** and **one child issue per deferred
item**, with proper dependencies. Concretely:

```text
[EPIC] LazyQMK-tier2-file-splits       P3 — closed-as-deferred items 4.5, 4.4, 4.7, 4.2
  ├── [task]  LazyQMK-fsplit-parser-layout      P3 — split parser/layout.rs
  ├── [task]  LazyQMK-fsplit-generator-fn      P2 — split generator mega-functions
  ├── [task]  LazyQMK-fsplit-popups-dispatch   P3 — refactor popups handle_popup_input
  └── [task]  LazyQMK-fsplit-settings-manager  P2 — split settings_manager.rs

[EPIC] LazyQMK-tier3-tests-tui-mod     P3 — aopx.6, aopx.4.6
  ├── [task]  LazyQMK-extract-embedded-tests  P3 — 55 #[cfg(test)] mod blocks
  └── [task]  LazyQMK-fsplit-tui-mod          P3 — split tui/mod.rs

[EPIC] LazyQMK-tier4-web-mod-split     P2 — aopx.4.1, 6u01
  ├── [task]  LazyQMK-fsplit-web-mod          P2 — split web/mod.rs (4181 lines)
  └── [task]  LazyQMK-web-apperror-refactor   P3 — remove ApiError boilerplate
        ↑ blocked-by: LazyQMK-fsplit-web-mod

[EPIC] LazyQMK-tier5-cross-cutting    P3 — aopx.4.3, aopx.7
  ├── [task]  LazyQMK-fsplit-models-layout    P2 — split models/layout.rs
  └── [task]  LazyQMK-tui-domain-reorg        P3 — reorganize src/tui/
        ↑ blocked-by: LazyQMK-fsplit-tui-mod

[task]  LazyQMK-fsplit-key-ops       P3 — note in key_ops.rs (no action)
```

`bd` CLI reminder (no need to memorize; look it up if unsure):

```bash
# Create epic
bd create --title="..." --type=epic --priority=3 --description="..."

# Create task under epic
bd create --title="..." --type=task --priority=3 \
    --description="..." --parent=<epic-id>

# Add dependency
bd dep add <blocked-id> <blocker-id>

# View created structure
bd list --parent=<epic-id>
```

For each new issue, write a description with three sections:

```markdown
## Origin
References LazyQMK-aopx.4.X closed 2026-07-19 with reason: "<paste>"

## Acceptance
- [ ] file <1000 lines (AGENTS.md cap)
- [ ] cargo test --tests --lib passes (1446 baseline preserved)
- [ ] cargo clippy --all-features -- -D warnings clean
- [ ] public API unchanged (callers in src/ + tests/ compile unchanged)
- [ ] golden fixtures unchanged if generator output

## Risk
<file-specific risk from close reason>
```

### Step 4 — Sequencing report

Write a short note (file or `bd comments add` on the parent epic) that
records the dependency graph and **which epic should be tackled first
when implementation begins.** Match the tier order in
`deferred-issues.md` (Tier 1 → 5). Justify deviations, if any.

### Step 5 — Stop

After Step 4, **stop**. Hand control back to the human with:

1. `bd stats` output (now showing N open issues)
2. The list of new epic IDs and their child task IDs
3. The dependency graph (`bd blocked` and `bd dep list <id>`)
4. The recommended starting issue for prompt 2

## Hard constraints

- **Do not edit any source files.** This prompt is planning only.
- **Do not edit AGENTS.md, README.md, or any docs in `.pi-handoff/`.**
- Use `bd create` for new issues, **not** `bd reopen`. Keep the closed
  history.
- All new issue titles must match `LazyQMK-XXXX-short-slug` style and
  must be searchable via `bd search`.
- Use `bd comments add <id> "<note>"` to record planning rationale on
  the parent epic — this preserves context for prompt 2.

## Output format (when you stop)

Reply with a single markdown block:

```markdown
## Plan complete

**New beads created:** N epics, M tasks
**New IDs:** [list epic IDs and task IDs]
**Dependency graph:**
- LazyQMK-fsplit-parser-layout — no blockers
- LazyQMK-fsplit-generator-fn   — no blockers
- LazyQMK-fsplit-popups-dispatch — depends on LazyQMK-fsplit-tui-mod (or: no blockers)
- …
**Recommended starting issue:** LazyQMK-fsplit-X (rationale: …)

**Baseline preserved:**
- `cargo test --tests --lib` → 1446 passed, 5 ignored
- `cargo clippy --all-features -- -D warnings` → 0 errors
- Working tree clean on `main`
```
