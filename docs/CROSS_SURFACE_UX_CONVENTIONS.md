# Cross-Surface UX Conventions

Status: shared definition doc for TUI and web refactor work.

Related bd issues:
- `LazyQMK-aee5` terminology and plain-language vocabulary
- `LazyQMK-b5k5` progressive disclosure for advanced keyboard concepts
- `LazyQMK-8zk2` state-feedback model
- `LazyQMK-3je5` destructive-action safety
- `LazyQMK-p0de` shared editing mental model

Related references:
- `docs/BRANDING.md` for canonical product naming
- `docs/AGENT_GUIDE.md` for user-facing setup flow
- `docs/ARCHITECTURE.md` for visual-position mental model and data flow

## 1. Purpose

This document defines shared UX conventions that both LazyQMK surfaces must follow:

- terminal UI (TUI)
- web UI

Goal: make switching surfaces feel like changing tools, not learning a new product.

## 2. Product and vocabulary rules

### 2.1 Canonical product naming

- Product name in prose: **LazyQMK**
- Binary/command name: `lazyqmk`
- TUI surface name: **terminal UI** or **TUI**
- Web surface name: **web UI**, **web editor**, or **browser editor**

Do not invent new names like:

- configurator
- studio
- workspace
- designer

Those terms can be used descriptively in copy, but not as alternate product names.

### 2.2 Plain-language vocabulary

Prefer everyday terms first. Introduce QMK terms only where needed.

| Prefer | Avoid by default | Notes |
|---|---|---|
| keyboard | board, device | Use keyboard unless hardware distinction matters |
| layout | keymap file, config blob | Use layout for editable user artifact |
| layer | layer stack, logical layer | Keep simple |
| key | switch position, matrix slot | Use matrix terms only in advanced contexts |
| keycode | code, symbol | Keycode is standard and concrete |
| behavior | quantum behavior, handler | Use when describing what key does |
| lighting | RGB subsystem, LED pipeline | Use RGB only when feature specifically requires it |
| build firmware | compile artifact | Use compile in secondary/support text |
| save | persist, serialize | Save in primary actions |
| discard changes | reset, revert, abandon | Reset/revert only when technically precise |
| remove | delete, destroy | Default safer verb for reversible removal |
| permanently delete | remove forever | Reserve for irreversible actions |
| advanced | expert, power-user | Expert/power-user can feel exclusionary |
| keyboard details | metadata, info.json data | Explain metadata in secondary text |

### 2.3 Copy rules

- Prefer verb-first labels: **Save layout**, **Build firmware**, **Remove layer**.
- Prefer concrete outcomes over implementation terms.
- Keep first-use explanations short: one sentence or less.
- Avoid unexplained abbreviations on first mention except QMK, RGB, TUI.
- If term is uncommon to general keyboard users, pair with hint or learn-more affordance.

### 2.4 Advanced concept translation table

| Technical concept | User-facing default label | Secondary explanation |
|---|---|---|
| matrix coordinates | switch matrix | Electrical row/column position used by firmware |
| visual coordinates | visual position | Where key appears in editor |
| tap dance | tap dance | One key does different actions based on tap count |
| combo | combo | Press multiple keys together for one action |
| mod-tap | mod-tap | Tap for one action, hold for modifier |
| layer-tap | layer-tap | Tap for key, hold to access layer |
| one-shot | one-shot modifier/layer | Applies once to next key press |
| RGB effect | lighting effect | Animation used for keyboard lighting |
| idle effect | idle lighting effect | Temporary effect shown after inactivity |

Rule: default labels stay plain; deeper QMK vocabulary appears in help, details, docs, and advanced panels.

## 3. Shared editing mental model

## 3.1 Core model

Both surfaces must present same model:

1. **Select context** — choose keyboard, layout, layer, or target key.
2. **Edit draft** — changes apply immediately to working draft.
3. **Review feedback** — user always sees dirty state, validation, and resulting selection.
4. **Save explicitly** — layout changes are not silently committed as final until user saves.
5. **Build/export separately** — saving layout and building firmware are distinct actions.

### 3.2 Shared object model

- **Saved layout**: last saved version on disk.
- **Working draft**: current in-memory editable state.
- **Unsaved changes**: any difference between draft and saved layout.
- **Selection**: current layer/key/item in focus.
- **Preview**: temporary inspection state that does not commit change.

Both surfaces should use these concepts in implementation tickets and user-facing copy.

### 3.3 Required behavior

- Editing a key, layer, combo, or lighting setting updates draft immediately.
- Draft changes must set dirty state immediately.
- Save action writes draft to disk and clears dirty state.
- Cancel inside a picker/modal exits current flow, not entire edit session.
- Discard action returns draft to last saved state or removes unsaved item, depending on context.
- Build should use current draft only after explicit save, unless flow clearly says **Build from unsaved draft**.

### 3.4 Selection and focus conventions

- One primary selection at time per editor scope.
- Selection change should be visible before edit begins.
- Hover may preview, but hover must not commit.
- Keyboard navigation and pointer navigation should land on same underlying selection model.
- Opening inspector/details panel should reflect current selection, not create separate hidden state.

### 3.5 Surface-specific implementation guidance

#### TUI

- Treat cursor/focus as selection.
- Use status bar or inline panel to show dirty state, current layer, current key, and pending action.
- Do not bind destructive changes directly to navigation keys.
- Modal/popup editing should preserve background selection context.

#### Web

- Treat selected key/layer/card as same conceptual selection as TUI focus.
- Hover states can preview details, but click or keyboard confirm commits selection.
- Side panels should bind to current selection and update live.
- Drag interactions must still preserve explicit save/discard model.

## 4. Progressive disclosure rules

### 4.1 Principle

Show common tasks first. Reveal complexity only when user asks for it, needs it, or enters advanced mode.

### 4.2 Tier model

Use same three disclosure tiers on both surfaces:

1. **Primary** — common actions needed by most users
2. **Secondary** — useful but not required for first success
3. **Advanced** — QMK-specific or high-risk concepts requiring extra explanation

### 4.3 What belongs in each tier

| Tier | Include | Hide by default |
|---|---|---|
| Primary | keyboard selection, layout selection, layer switching, key assignment, save, build | matrix terms, timing internals, rare QMK behaviors |
| Secondary | layer naming, color coding, previews, history/logs, import/export details | low-level firmware fields |
| Advanced | combos, tap dance, mod-tap/layer-tap, one-shot, raw metadata, matrix mapping, generated artifacts internals | nothing beyond this tier |

### 4.4 Disclosure rules

- Advanced concepts must never block basic keymap editing.
- If advanced item appears in primary flow, provide one-line explanation plus link/details toggle.
- Use **Advanced** as consistent affordance label. Avoid mixing with **Expert**, **More options**, and **Power user** unless context truly differs.
- Keep advanced sections collapsed by default unless user previously opted in.
- Remember advanced-mode preference per surface if technically cheap, but content structure must remain same across surfaces.

### 4.5 Teaching pattern for advanced keyboard concepts

When introducing advanced concepts, use this order:

1. plain-language label
2. one-line outcome explanation
3. optional example
4. technical/QMK name if needed

Example:

- **Tap dance**
- One key can do different actions based on how many times you tap it.
- Example: tap once for `Esc`, tap twice for `` ` ``.
- Advanced detail: implemented as QMK tap dance.

### 4.6 Surface-specific implementation guidance

#### TUI

- Put advanced concepts behind dedicated screens, expandable help, or explicit advanced sections.
- Use concise inline helper text; route longer explanations to help panel or popup.
- Avoid dense multi-column terminology walls in primary views.

#### Web

- Use disclosure panels, drawers, accordions, info popovers, or progressive forms.
- Keep default forms short; reveal advanced fields only after enabling advanced behavior.
- Avoid showing disabled advanced controls without explanation.

## 5. State-feedback model

### 5.1 States every surface must represent

At minimum, both surfaces must clearly represent:

- idle
- focused/selected
- editing
- unsaved/dirty
- saving
- saved
- loading
- success
- warning
- error
- blocked/disabled
- destructive/danger

### 5.2 Feedback priorities

Feedback should answer, in order:

1. What is selected?
2. What changed?
3. Is it saved?
4. Is action still running?
5. Did action succeed, fail, or need attention?

### 5.3 Required feedback rules

- Dirty state must be persistent until saved or discarded.
- Success feedback should be visible but lightweight, then clear automatically when appropriate.
- Errors must be actionable: what failed, what user can do next.
- Warnings should explain risk without blocking unless necessary.
- Disabled actions must explain why they are disabled.
- Long-running work must show progress or activity indicator plus cancelability where possible.

### 5.4 Shared feedback patterns

| Situation | Required feedback |
|---|---|
| Key changed | selection remains visible + dirty indicator + optional short confirmation |
| Save started | saving indicator, save action disabled while in flight |
| Save success | dirty clears, saved confirmation appears |
| Save failure | dirty remains, error shown with retry path |
| Build started | running status, logs/progress view, build action changes to running/cancel state |
| Validation issue | inline explanation near field/item plus summary if action blocked |
| Destructive action armed | danger styling + explicit consequence text |

### 5.5 Surface-specific implementation guidance

#### TUI

- Use persistent status region for global state.
- Use inline styling/icons/text for selected, dirty, warning, and error states.
- Avoid relying on transient flash messages alone; status bar must remain source of truth.

#### Web

- Use persistent save/status affordance in top bar or editor frame.
- Use inline validation near fields and toast/banner for global results.
- Toasts must not be only place where failure or dirty state is communicated.

## 6. Destructive-action safety conventions

### 6.1 Principle

Danger increases with irreversibility. Friction must increase with danger.

### 6.2 Risk tiers

| Tier | Example | Required guardrail |
|---|---|---|
| Low | clear search, close panel, cancel picker | no confirmation |
| Medium | remove unsaved item, overwrite generated preview, discard local edits in one view | confirm or obvious undo |
| High | delete saved layout, remove layer with assigned keys, overwrite exported file, destructive bulk change | explicit confirmation with consequence text |
| Critical | permanent delete without recovery, reset all user data, destructive filesystem action | typed confirmation or equivalent high-friction confirmation |

### 6.3 Required destructive-action rules

- Use **Remove** for reversible or locally scoped deletion.
- Use **Delete permanently** only for irreversible actions.
- Confirmation text must name object and consequence.
- Default focus in confirmations must be safe action.
- Dangerous actions must be visually distinct from primary actions.
- Never place destructive action adjacent to common primary action without spacing or hierarchy.
- Keyboard shortcuts for destructive actions must require deliberate chord or confirmation.
- If undo is available and reliable, say so explicitly.

### 6.4 Confirmation copy pattern

Use this structure:

- title: clear action + object
- body: consequence + recovery statement
- primary dangerous button: exact action
- secondary safe button: cancel/keep/discard no changes

Example:

- Title: **Delete layout “Corne travel” permanently?**
- Body: This removes saved layout from disk. This cannot be undone from LazyQMK.
- Danger button: **Delete permanently**
- Safe button: **Keep layout**

### 6.5 Surface-specific implementation guidance

#### TUI

- Use confirmation modal for medium+ risk destructive actions.
- Require second confirm keystroke or explicit button selection for high/critical actions.
- Do not overload single-key shortcuts like `d` for irreversible deletion without confirm step.

#### Web

- Use modal/dialog for medium+ risk destructive actions.
- Reserve inline destructive buttons for low-risk removals or cases with strong undo.
- Danger color alone is insufficient; include clear text label.

## 7. Cross-surface action naming

Use same action names where feature parity exists.

| Shared action | Notes |
|---|---|
| Save layout | commit draft to disk |
| Discard changes | revert draft to last save |
| Build firmware | generate/compile firmware output |
| Cancel build | stop build process |
| Add layer | create layer |
| Remove layer | use until deletion is irreversible |
| Rename layer | update label only |
| Assign keycode | apply selected key behavior |
| Open advanced settings | reveal advanced options |
| Show details | reveal more explanation, not edit |

Avoid mixing synonyms across surfaces for same action, such as Save vs Apply, Remove vs Delete, or Details vs Inspector, unless difference is intentional and documented.

## 8. Implementation checklist for tickets

Every TUI or web implementation ticket derived from this doc should answer:

1. Which shared vocabulary terms appear in UI copy?
2. What is primary selection model?
3. When does dirty state turn on and off?
4. What state feedback appears for idle/loading/success/error?
5. What disclosure tier does this feature belong to?
6. What destructive risk tier applies?
7. Does action naming match cross-surface convention?

If ticket cannot answer these, definition is incomplete.

## 9. Recommended follow-up work

- Create surface-specific implementation tickets for TUI and web using this doc as acceptance criteria.
- Audit existing labels, dialogs, and shortcuts against terminology and destructive-action tables.
- Add shared copy snippets/help content to central sources where possible.
- Add UI review checklist to PR template or refactor tracking docs.
