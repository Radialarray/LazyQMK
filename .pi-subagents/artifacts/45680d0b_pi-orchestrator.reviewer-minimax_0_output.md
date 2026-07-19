### Summary
Fix is correct. `src/tui/mod.rs:936-945` now consumes exactly the event that made `poll()` ready, routes key events unchanged, treats resize as a redraw trigger, and intentionally ignores Mouse, FocusGained, FocusLost, and Paste through the wildcard arm.

### Blockers
- None.

### Suggested
- None. No hidden Crossterm 0.29 event variant is mishandled: the remaining variants are non-actionable for this TUI and `_ => {}` is idiomatic.

### Nitpicks
- None. Removing the `// User quit` comment does not change behavior; `break` remains clear in context.

### Regression assessment
- `src/tui/mod.rs:936-945`: Key behavior is unchanged; Resize still causes the next loop to redraw. Mouse, focus, and paste events remain ignored, but no longer consume or block waiting for a second event. This is the intended behavioral change and fixes both dropped follow-up events and the possible blocking second `event::read()`.