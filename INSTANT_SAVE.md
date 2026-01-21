# 🌱 Instant Save & Smart Filtering

## NEW: Tasks Save Immediately!

### Before (Old Behavior)
```bash
t triage
# Triage 50 tasks...
# [30 minutes later]
# Ctrl+C to cancel
# ❌ Lost all your work! Have to start over
```

### After (NEW Behavior)
```bash
t triage
# Task 1: P0 ↵
  💾 Saving... ✓
# Task 2: P1 ↵
  💾 Saving... ✓
# Task 3: Ctrl+C
⚠️  Triage cancelled (progress saved)

# ✅ Tasks 1 & 2 are saved! Can continue later
```

## How It Works

Every time you prioritize/categorize a task, it **saves immediately** to:
1. ✅ **Google Tasks** (via gog CLI)
2. ✅ **Local cache** (SQLite)

**No more "Save at the end" prompt!** Everything is instant and safe.

### Example Flow
```
Task 1/51
Fix editor bug (email)

Priority:
  0. P0 - Urgent + Important
  ...

> 0 ↵
  ✓ P0
  💾 Saving... ✓    ← Saved to Google Tasks instantly!
```

## Benefits

### For ADHD
- ✅ **No fear of losing work** - Everything saves immediately
- ✅ **Can quit anytime** - Ctrl+C is safe now
- ✅ **Take breaks** - Come back later, pick up where you left off
- ✅ **No commitment anxiety** - Don't have to finish all 51 tasks at once

### For Workflow
- ✅ **Incremental progress** - Triage 5 tasks, come back later for more
- ✅ **Interruption-safe** - Phone call? Meeting? No problem!
- ✅ **Real-time sync** - Changes appear in Google Tasks immediately
- ✅ **Multi-device** - Start on laptop, continue on phone

## Smart Filtering

### NEW: Already Triaged Tasks Don't Appear

**Before:**
```bash
t triage
# Shows all 51 tasks, including ones you already triaged yesterday
```

**After:**
```bash
t triage
Found 12 tasks needing triage  ← Only untriaged tasks!

# Tasks with P0/P1/P2/P3 already set? Skipped!
# Tasks with projects already set? Skipped!
# Tasks with time estimates? Skipped!
```

### Filter Logic

A task needs triage if it's **missing any property**:
- No priority? → Appears in Pass 1
- Has priority but no project? → Skips Pass 1, appears in Pass 2
- Has priority & project but no time? → Skips Pass 1 & 2, appears in Pass 3

### Example
```
Task A: [blank]                → Needs all 3 passes
Task B: [2026-01-21][P0]       → Needs Pass 2 & 3 only
Task C: [2026-01-21][P1][LIFE] → Needs Pass 3 only
Task D: [2026-01-21][P0][WORK]{2h} → ✅ Fully triaged, skipped!
```

## Force Re-Triage

### Re-Triage Everything
```bash
t triage --force
```

Forces ALL tasks to appear, even if already triaged. Useful when:
- You want to re-evaluate priorities
- Circumstances changed (P1 → P0)
- You made mistakes and want to fix them

### Re-Triage Specific Properties

**Just re-do priorities:**
```bash
t triage --force --priority
🔄 Force mode - re-triaging all tasks
📊 Priority-only mode

# Only shows Pass 1 (priorities)
# Skips Pass 2 (projects) and Pass 3 (time)
```

**Just re-do projects:**
```bash
t triage --force --project
🔄 Force mode - re-triaging all tasks
🏷️  Project-only mode

# Skips Pass 1, only shows Pass 2
```

**Just re-do time estimates:**
```bash
t triage --force --time
🔄 Force mode - re-triaging all tasks
⏱️  Time-only mode

# Skips Pass 1 & 2, only shows Pass 3
```

## Use Cases

### Morning Routine
```bash
# Monday 9am
t triage
Found 8 tasks needing triage  ← New tasks from weekend

# Quickly triage just the new ones
# Everything else already has priorities
```

### Mid-Week Adjustment
```bash
# Wednesday - priorities shifted!
t triage --force --priority

# Re-evaluate all priorities
# Keep existing projects & time estimates
```

### End of Week Cleanup
```bash
# Friday 5pm
t triage

Found 0 tasks needing triage
✅ All tasks are triaged!

# You're done! Weekend starts now
```

## Tips

### Incremental Triage
```bash
# 9am - triage 10 tasks
t triage
# ... triage 10 tasks, then Ctrl+C

# 11am - triage 10 more
t triage
# Picks up where you left off!

# 2pm - finish the rest
t triage
```

### Quick Priority Sweep
```bash
# Just set priorities on everything first
t triage --priority

# Come back later to add projects/time
t triage
# Only shows Pass 2 & 3 now
```

### Emergency Re-Prioritization
```bash
# Something urgent came up!
t triage --force --priority

# Re-evaluate all priorities
# Mark new urgent things as P0
```

## Safety Features

- ✅ **Auto-save on every change** - Never lose work
- ✅ **Cancel anytime** - Ctrl+C saves progress
- ✅ **Sync to Google Tasks** - Changes appear immediately
- ✅ **Cache updated** - Next run sees your changes
- ✅ **No data loss** - Everything is durable

## Error Handling

If a save fails:
```
> 0 ↵
  ✓ P0
  💾 Saving... ❌ Failed: Network error

# Your local cache is updated
# Will retry sync on next `t triage`
```

Don't worry - your change is saved locally and will sync later!

Enjoy the peace of mind! 🌱💾
