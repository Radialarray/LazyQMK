# Review: `run_tui` event-loop fix (src/tui/mod.rs)

## Summary

Bug fix is correct and the smallest possible diff: collapses a double-`event::read()` call (which silently swallowed / blocked on the next event whenever a non-Key variant like Resize/Mouse/Focus arrived) into a single `match` that consumes one event per iteration. Behavior change is local to the main loop. All 1474 tests pass and `cargo clippy --all-features -- -D warnings` exits 0. **No regression test was added**, and the loop itself remains untestable in its current shape; `handle_key_event` (a pure function) is testable but is not the locus of the bug.

## Critical Issues 🔴

None. The fix is correct and minimal.

## Warnings ⚠️

### Missing regression test for this specific bug class
- **Where**: `src/tui/mod.rs:935-942` (the changed block).
- **Risk**: This is a P0 bug per bd issue `LazyQMK-aopx.1`. The fix is correct today, but the bug pattern — "I forgot to bind the second event to the same value as the first" — is exactly the kind of regression that re-appears under refactor pressure (e.g. adding a new event variant in one branch but not the other, or re-introducing nested `if let`/`else if let` chains). No existing test would fail if a future change reintroduced the double-`read()`.
- **Evidence**: grep over `tests/` for `run_tui|handle_key_event|Event::Resize` returns nothing. The only in-module tests in `src/tui/mod.rs:1538` cover `AppState::extract_base_keyboard` and `PendingKeycodeState` — none exercise the event loop or key handling.

### `handle_key_event` is testable but unused in tests
- **Where**: `src/tui/mod.rs:1516` (`fn handle_key_event(state: &mut AppState, key: event::KeyEvent) -> Result<bool>`).
- The function is a pure mapping from `KeyEvent` → state mutation + `bool` (quit-or-continue). It does not touch the terminal. It is **already a clean test seam**, yet zero unit tests target it. Adding a small suite here (Enter/Esc clear error; popup routing; 'q' quit) would cheaply cover the dispatcher half of the loop.

## Suggestions 💡

### Introduce a testable `dispatch_event` seam (recommended)
- **Where**: extract the `match event::read()? { ... }` body into a free function:
  ```rust
  pub(crate) enum DispatchOutcome { Continue, Quit }
  pub(crate) fn dispatch_event(state: &mut AppState, event: event::Event) -> Result<DispatchOutcome> { ... }
  ```
  `run_tui` then becomes:
  ```rust
  if event::poll(Duration::from_millis(100))? {
      if matches!(dispatch_event(state, event::read()?)?, DispatchOutcome::Quit) { break; }
  }
  ```
- **Why**: this lets a unit test feed synthetic `Event::Resize`, `Event::FocusGained`, `Event::Mouse(...)`, `Event::Key(...)` values and assert one event per call, no spurious state mutation, and the `Quit` return path. It directly catches the original bug class (double-consumption / wrong-event-handled) without needing a PTY or terminal harness.
- **Cost**: ~20-line refactor, fully behavior-preserving, easy to back out.

### Alternative: introduce an `EventSource` trait (higher effort)
- Parameterize `run_tui` (or just the inner loop body) over a custom `trait EventSource { fn poll(Duration) -> Result<bool>; fn read() -> Result<Event> }`. Test with a `VecDeque<Event>` mock. Higher payoff (full loop coverage) but bigger blast radius — would touch the public `run_tui` signature or require a generic inner function. Not recommended for a one-line bug fix unless loop testability is a stated goal.

### If no refactor: document the invariant
- Add a one-line comment above the `match` enforcing the one-event-per-iteration invariant:
  ```rust
  // CRITICAL: exactly one `event::read()` per loop iteration.
  // Nested `else if let` patterns must NOT call `event::read()` again
  // (see bd LazyQMK-aopx.1).
  ```
- Cheap insurance against the pattern recurring; not a substitute for a test but reduces future regression odds.

### Add a manual-test note to the bd issue
- Acceptance criteria mention "manual TUI test: no input lag when terminal events interleave." Record the exact reproduction (e.g., `script -q /dev/null cargo run`, send `kill -WINCH $$` between keystrokes) so the next person can verify quickly.

## Positive Notes ✅

- Diff is the smallest correct change: same control flow, same semantics for Key/Resize, just one `read()` per iteration. No drive-by edits, no unrelated changes — easy to review and revert if needed.
- Match-arm exhaustiveness (`_ => {}`) means any future crossterm `Event` variant is silently ignored rather than triggering the original bug — the fix is forward-compatible with new event types.
- `git status` confirms only `src/tui/mod.rs` is modified; no submodule churn, no incidental edits.
- Verified locally: `cargo test` → **1474 passed, 5 ignored**; `cargo clippy --all-features --no-deps -- -D warnings` → exit 0, no warnings; both `--lib` (560 passed) and `--tests` (1446 passed) clean. The "0 errors, 2 warnings" message earlier in the run was wrapper output, not from clippy itself.

## Verification Commands Run

| Command | Result |
|---|---|
| `cargo test` | 1474 passed, 5 ignored (25 suites) |
| `cargo test --lib` | 560 passed, 1 ignored |
| `cargo test --tests` | 1446 passed, 5 ignored |
| `cargo clippy --all-features --no-deps -- -D warnings` | exit 0, no warnings |
| `git diff src/tui/mod.rs` | +9/-5, single contiguous block, no unrelated changes |

## Answer to the explicit checks

1. **Should this bug fix have a regression test? Is there one?** Yes, ideally; **no, none exists**.
2. **Is there a testable seam?** `handle_key_event` (pure function, `src/tui/mod.rs:1516`) is trivially testable today but isn't used by any test. The loop itself is not testable without refactor; introducing a `dispatch_event` helper is the cheapest seam that would actually catch this bug class.
3. **Does cargo test pass / no behavior change visible?** Confirmed 1474/1474 + clippy clean; diff is behavior-preserving for all `Event::Key` and `Event::Resize` cases and strictly improves all other variants (no longer blocks on a second read).
4. **What test would catch a future regression?** A unit test feeding `Event::Resize`, `Event::Mouse(...)`, `Event::FocusGained/Lost` into an extracted `dispatch_event(state, event)` function and asserting: (a) exactly one event is processed per call, (b) no spurious `event::read()` is issued, (c) `DispatchOutcome::Quit` is returned only on the actual quit key. Mocking `event::read()` requires the helper-extract refactor (suggestion 1).

## Acceptance Report