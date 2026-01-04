# Spec-to-Beads Migration Process

**Purpose:** Document the process of converting spec documents to closed beads issues for historical tracking.

## Overview

LazyQMK previously tracked work using spec documents in the `specs/` directory. We're migrating this history to beads (bd) issue tracker to have a single source of truth for all work - past, present, and future.

## Migration Steps

### 1. Extract Completion Dates

Use git history to find when each spec was completed:

```bash
# For archived specs
git log --all --format="%ai" -- "specs/archived/[spec-folder]/"

# For active specs
git log --all --format="%ai" -- "specs/[spec-folder]/"
```

See `.beads/spec-completion-dates.md` for extracted dates.

### 2. Create Epic for Each Spec

Each spec becomes a closed epic in beads:

```bash
bd create \
  --title="Epic: [Spec Name]" \
  --type=feature \
  --priority=2 \
  --description="[Summary from spec plan.md]"

# Mark as closed with completion date
bd close [issue-id] --reason="Completed: [summary of work done]"
```

### 3. Extract Tasks from plan.md

If the spec has a detailed `plan.md` with task breakdown:

1. Read the plan.md file
2. Identify distinct tasks/phases
3. Create sub-tasks for the epic:

```bash
bd create \
  --title="[Task name from plan]" \
  --type=task \
  --priority=2 \
  --description="[Task details]"

# Add dependency to epic
bd dep add [task-id] [epic-id]

# Close task
bd close [task-id] --reason="Part of completed spec [spec-number]"
```

### 4. Link Related Specs

Some specs reference or depend on others. Use dependencies:

```bash
# If spec B built on spec A
bd dep add [spec-b-id] [spec-a-id]
```

## Mapping Guide

### Spec → Beads Mapping

| Spec Element | Beads Equivalent |
|--------------|------------------|
| Spec folder (e.g., `024-tap-dance/`) | Epic (feature type) |
| Plan.md sections | Tasks (task type) |
| Completion date from git | `closed_at` timestamp |
| Spec number + name | Epic title |
| Summary/Goals | Epic description |
| Implementation details | Task descriptions |

### Example: Spec 024 (Tap Dance)

**Spec structure:**
```
specs/024-tap-dance/
├── plan.md         # Main plan with phases
└── tasks.md        # Optional detailed tasks
```

**Beads migration:**
```bash
# 1. Create epic
bd create --title="Epic: Tap Dance Support" \
  --type=feature --priority=2 \
  --description="Add tap dance configuration to LazyQMK..."

# 2. Create tasks from plan.md phases
bd create --title="Data model: Add TapDanceAction struct" \
  --type=task --priority=2
  
bd create --title="TUI: Tap Dance Editor component" \
  --type=task --priority=2

bd create --title="Firmware: Generate tap_dance_actions array" \
  --type=task --priority=2

# 3. Link tasks to epic
bd dep add LazyQMK-xxx LazyQMK-epic

# 4. Close everything with completion date
bd close LazyQMK-epic --reason="Completed 2025-12-13: Full tap dance support..."
bd close LazyQMK-task1 --reason="Part of tap dance epic"
...
```

## Guidelines

### What to Migrate

✅ **Do migrate:**
- All specs in `specs/archived/` (001-021)
- Completed specs in `specs/` root (022-026)
- Major tasks/phases from plan.md files
- Key decisions and rationale
- Links between related specs

❌ **Don't migrate:**
- Detailed implementation notes (keep in git history)
- Code snippets (keep in commits)
- Temporary exploration docs
- Duplicate information already in code comments

### Descriptions

Keep descriptions concise but informative:

**Good epic description:**
```
Add idle effect screensaver for RGB lighting with configurable 
timeout and duration. Implements 3-state system: normal → 
animation → off. Settings stored per-layout in markdown frontmatter.
```

**Good task description:**
```
Implement TUI component for configuring idle effect settings:
timeout, duration, and effect selection. Integrates with existing
Settings Manager (Shift+S).
```

### Closure Reasons

Provide context when closing:

**Good closure reason:**
```
Completed 2025-12-13: Full tap dance support implemented with
TUI editor, firmware generation, and 20 integration tests. 
Feature documented in FEATURES.md and help system.
```

## Automation Opportunities

For bulk migration, consider scripting:

```bash
#!/bin/bash
# migrate-specs.sh - Bulk migrate specs to beads

for spec_dir in specs/archived/*; do
  spec_name=$(basename "$spec_dir")
  spec_number=$(echo "$spec_name" | grep -oE '^[0-9]+')
  
  # Extract title from plan.md
  title=$(grep "^# " "$spec_dir/plan.md" | head -1 | sed 's/^# //')
  
  # Create epic
  issue_id=$(bd create --title="Epic: $title" \
    --type=feature --priority=2 | grep -oE 'LazyQMK-[a-z0-9]+')
  
  # Close with date from spec-completion-dates.md
  bd close "$issue_id" --reason="Migrated from spec $spec_number"
done
```

## Verification

After migration:

1. **Check counts:**
   ```bash
   bd stats  # Should show 24+ closed issues
   ```

2. **Verify dates:**
   ```bash
   bd list --status=closed | grep "2025-12-"
   ```

3. **Check dependencies:**
   ```bash
   bd show [epic-id]  # Should show sub-tasks if created
   ```

4. **Test queries:**
   ```bash
   bd list --status=closed --priority=2  # All migrated specs
   ```

## Benefits of Migration

1. **Single source of truth** - All work tracked in one system
2. **Better searchability** - Query by status, priority, dates
3. **Dependency tracking** - See how features built on each other
4. **Project metrics** - Accurate lead time, velocity, completion rates
5. **Clean repository** - Remove spec directories after migration

## After Migration

Once all specs are migrated and verified:

1. Move template files from `specs/` to appropriate location
2. Remove entire `specs/` directory
3. Update documentation references to use beads commands
4. Add note to README about historical tracking in beads

See related tasks:
- LazyQMK-jj2: Migrate archived specs (001-021)
- LazyQMK-73u: Migrate active specs (022-026)
- LazyQMK-db4: Move template files
- LazyQMK-l3f: Remove specs directory
- LazyQMK-c6c: Update documentation references
