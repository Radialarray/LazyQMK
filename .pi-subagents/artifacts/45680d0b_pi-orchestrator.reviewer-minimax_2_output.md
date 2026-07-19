# Simplicity Review: `src/tui/mod.rs` `run_tui` event dispatch

## Summary

The diff replaces a flawed `if-let … else-if-let …` chain with an idiomatic `match` over `Event`. The new code is shorter, clearer, and — crucially — **fixes a real correctness bug** in the version it replaced: the original `else if let Event::Resize(_, _) = event::read()?` called `event::read()` a second time inside the `else` branch, so any non-key event would consume a *second* event from the queue before rendering. The `match` form reads exactly one event per loop iteration, which is the intended behavior.

Overall: clear improvement. The match should land as-is.

---

## Findings

### Blockers 🔴

- **None remaining after the fix.** The pre-fix code called `event::read()?` twice in the worst case (once in the `if let`, again in `else if let`). The rewritten match eliminates the double-read and the related risk of the else branch blocking forever if no further event arrived.

### Suggested ⚠️

1. **`src/tui/mod.rs:946` — `_ => {}` could carry a one-line intent comment.** Crossterm's `Event` has 4 ignorable variants in the default build (`FocusGained`, `FocusLost`, `Mouse`, `Paste`, plus `Resize` which is handled above). A brief comment like
   ```rust
   _ => { /* ignore focus/mouse/paste — not used by this app */ }
   ```
   makes the catch-all self-documenting and prevents a future reader from wondering whether the omission was intentional. Low cost, modest upside.

2. **`src/tui/mod.rs:937-947` — consider extracting a small dispatcher.** Not required, but if the event handling grows (mouse support, paste handler, focus-aware redraw), hoisting the match into a helper such as
   ```rust
   fn dispatch_event(state: &mut AppState, event: Event) -> Result<bool>
   ```
   that returns `true` to quit keeps `run_tui` readable. With only two real arms today, inlining is fine; flagging so the next reviewer knows where to add the seam.

### Nitpicks 💡

3. **`src/tui/mod.rs:941` — `Event::Resize(_, _) =>` discards dimensions.** If the app ever needs to know its size (e.g. for min-size layout guards or HiDPI tweaks), capture them now. Currently fine — just a note.

4. **No tests.** Hard to exercise a TUI event loop without integration harness; not a blocker, but the bug above would not have shipped if a basic "process one event per poll tick" assertion existed. Out of scope for this diff.

---

## Answers to the requested checks

1. **Is the match arm cleaner than the if-let-else-if it replaced?** Yes. It is one match expression, one `event::read()`, and the intent ("here are the events we react to") is immediately obvious. The old form was both uglier and buggy.
2. **Is `_ => {}` reasonable?** Yes. Crossterm emits several non-reactive variants; commenting every one is noise. (See suggestion 1 if you want a single line of intent.)
3. **Are docs/comments needed?** Not for the behavior itself; the surrounding `run_tui` function is reasonably self-describing. The optional `_ => {}` intent comment in suggestion 1 is the only one I'd add.
4. **Could this be simpler with helpers?** Yes — extracting a `dispatch_event` returning `quit: bool` would shrink `run_tui` further, but with only two real arms the extraction is premature. Worth doing the moment a third variant becomes interesting.

## Positive notes ✅

- Real bug fixed in passing (double `event::read()`).
- Idiomatic Rust, exhaustiveness-friendly (adding `Event::Paste` etc. won't require touching call sites).
- Diff is minimal — narrowly scoped, no incidental changes.
- The pre-existing `// Terminal resized, will re-render on next loop` comment was preserved and is genuinely useful; good signal for the next reader.

## Residual risks

- None identified in the diff. The catch-all is intentional and the bug it fixes is the only material risk, which is now closed.