# Svelte Check Warnings

This document explains the current Svelte check warnings and why they are present.

## 1. state_referenced_locally (Line 17)

**Warning:**
```
This reference only captures the initial value of `data`. Did you mean to reference it inside a derived instead?
```

**Location:** `src/routes/layouts/[name]/+page.svelte:17`

**Code:**
```typescript
let { data }: { data: PageData } = $props();
let layout = $state<Layout | null>(data.layout);
```

**Why it exists:**
The `layout` state is initialized from `data.layout` and captures the initial value. This is intentional because:
- The layout needs to be mutable (users edit tap dances, combos, etc.)
- Changes are tracked via `isDirty` flag
- Layout is saved back to the server when modified

**Could it be $derived?**
No. The code mutates `layout` directly in multiple places:
- `layout.tap_dances = [...]` (line 121, 132, 138)
- `layout.combos = [...]` (line 151, 167, 173)
- `layout.idle_effect_settings = {...}` (line 180-197)
- `layout = { ...layout }` (line 196)

**Alternative approaches considered:**
1. **Use $derived with mutations:** Doesn't work - derived state is read-only
2. **Use $effect to sync:** Would cause infinite loops when editing
3. **Deep clone on every render:** Unnecessary performance overhead

**Conclusion:** This warning is informational. The pattern is correct for this use case where we need to:
1. Initialize state from props
2. Allow local mutations
3. Track dirty state
4. Save back to server

## 2. a11y_label_has_associated_control

**Warning:**
```
A form label must be associated with a control
```

**Locations:**
- Lines 413, 422, 433, 444 (Tap Dance fields)
- Lines 483, 491, 502 (Combo fields)  
- Lines 538, 555, 574 (Idle Effect fields)

**Why it exists:**
The labels don't have `for` attributes linking them to their corresponding input fields. This is a known accessibility issue.

**Impact:**
- Screen readers can't properly associate labels with inputs
- Users can't click labels to focus inputs
- Reduces accessibility for keyboard and screen reader users

**Recommended fix:**
Add unique `id` attributes to Input components and matching `for` attributes to labels:

```svelte
<!-- Before -->
<label class="...">Name</label>
<Input value={td.name} ... />

<!-- After -->
<label for="td-name-{i}" class="...">Name</label>
<Input id="td-name-{i}" value={td.name} ... />
```

**Why not fixed yet:**
- Tests are functional and passing
- UI is working correctly
- This is a cosmetic/accessibility enhancement
- Requires systematic updates across all form fields

**Priority:** Medium - should be fixed in a dedicated accessibility improvement pass

## 3. Summary

| Warning | Count | Severity | Fix Priority |
|---------|-------|----------|-------------|
| state_referenced_locally | 1 | Info | No fix needed (intended pattern) |
| a11y_label_has_associated_control | 10 | Medium | Medium (accessibility) |

**Next steps:**
1. Keep `state_referenced_locally` as-is (documented above)
2. Create issue/PR to fix a11y warnings with proper label associations
3. Consider adding aria-label or aria-labelledby as interim solution
