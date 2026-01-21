# thegarden Release Notes - v0.2.0

## 🎉 New Features

### 1. Smart Hashtag Task Types with Auto-Defaults

**Pre-configured task types:**
- `#FollowUp` → P1, 30m (quick check-ins)
- `#Plan` → P1, 1h (strategic planning)
- `#DeepWork` → P0, 2h (focused work)

**How it works:**
When you create a task with one of these hashtags, thegarden automatically applies the default priority and time estimate:

```bash
# Before triage:
"Client check-in #FollowUp"

# After sync (auto-applied):
[2026-01-20][P1][---]{30m} Client check-in #FollowUp
```

**Customizable:**
Edit `~/.thegarden/config.json` to add your own task types with custom defaults!

---

### 2. `t plan` - Weekly Calendar Integration

View your entire week at a glance: meetings + tasks combined!

```bash
t plan
```

**Features:**
- 📅 Fetches meetings from Google Calendar
- 🎯 Shows all tasks for the current week
- 📊 Groups by day with color-coded headers
- ⏰ Events sorted by time, tasks by priority
- ✨ Highlights today's date

**Example Output:**
```
🌱 This Week Plan

Monday, Jan 20:
  📅 09:00 AM-09:45 AM: Tracy Sync
  🔴 [P0][SILVERMINE]{2h} Fix editor bug
  🟡 [P1][WORKDAY]{1h} #Plan Client strategy
  
Tuesday, Jan 21:
  📅 02:00 PM-03:00 PM: Standup
  🟢 [P2][LIFE]{30m} #FollowUp John
```

---

### 3. `t focus` - Critical Tasks Only

Cut through the noise - see ONLY what matters right now:

```bash
t focus
```

**Shows:**
- All P0 tasks (any date)
- P1 tasks that are overdue or due today
- Total planned time

**Hides:**
- P2, P3 tasks
- Future P1 tasks

**Example Output:**
```
🎯 Focus - Critical Tasks

🔴 P0 (do first):
  1. [SILVERMINE]{2h} Fix editor bug

🟡 P1 (due today):
  2. [WORKDAY]{1h} Client call prep ⚠️

Total planned: 3h
```

---

### 4. Grouped View for Better Task Overview

See your tasks organized by date sections:

```bash
t list --grouped
# or
t list -g
```

**Features:**
- Groups: OVERDUE → TODAY → THIS WEEK → FUTURE
- Color-coded headers (red for overdue, green for today)
- Shows weekday names
- Works with all existing filters

**Example Output:**
```
════ OVERDUE ════
[P0][SILVERMINE]{1h} Fix bug

════ TODAY (Monday, Jan 20) ════
[P1][WORKDAY]{30m} Client prep

════ THIS WEEK ════
Tuesday, Jan 21:
  [P0][WORKDAY]{2h} Presentation
```

---

### 5. Enhanced Hashtag Support

**Filter by tag:**
```bash
t list --tag FollowUp
t list --tag Plan
t list --tag Bug
```

**Display:**
- Hashtags appear in dimmed text after task title
- Tags are preserved when merging duplicates
- Combine multiple tags per task

---

### 6. Natural Language Date Parsing

Add tasks with human-friendly dates:

```bash
"Call client due Monday"      → next Monday
"Review docs due 1/25"        → January 25
"Follow up in 3 days"         → 3 days from now
"Meeting tomorrow"            → tomorrow's date
```

**Supported formats:**
- Day names: Monday, Tue, Wednesday, etc.
- Relative: tomorrow, in X days
- Dates: M/D or M/DD format

---

## 🔧 Technical Updates

**Modified Files:**
- `src/main.rs` - All feature implementations
- `~/.thegarden/config.json` - Added task_types section

**New Config Structure:**
```json
"task_types": {
  "FollowUp": { "priority": "P1", "time": "30m" },
  "Plan": { "priority": "P1", "time": "1h" },
  "DeepWork": { "priority": "P0", "time": "2h" }
}
```

**Dependencies:**
- Requires `gog` CLI for calendar integration
- Uses `gog calendar events` command

---

## ✅ All Existing Features Preserved

- ✅ Google Tasks sync
- ✅ Interactive triage workflow
- ✅ All filters (priority, project, status, context)
- ✅ Sorting options
- ✅ Merge duplicates
- ✅ Bump tasks
- ✅ Search functionality

---

## 📦 Installation

```bash
cd /Users/bryanwhiting/clawd/thegarden
cargo build --release
cp target/release/thegarden ~/.local/bin/t  # or your preferred location
```

**Binary size:** 5.2 MB
**Build time:** ~3-4 seconds

---

## 🚀 Quick Start

```bash
# Sync and view this week's plan
t plan

# Focus on critical tasks
t focus

# Create a task with auto-defaults
t add "Strategy session #Plan"  # Auto: P1, 1h

# View tasks grouped by date
t list --grouped

# Filter by hashtag
t list --tag FollowUp

# Triage new tasks (same as before)
t triage
```

---

## 📝 Notes

1. **Task type defaults** are only applied to NEW unprioritized tasks. Existing tasks retain their settings.

2. **Calendar integration** requires `gog` CLI to be installed and authenticated with your Google account.

3. **Hashtags** can be added to any task title and will be preserved across syncs and merges.

4. **Date parsing** works for both new tasks and when updating existing tasks.

---

## 🐛 Known Issues / Warnings

- Some unused legacy functions generate compiler warnings (non-breaking)
- Calendar events require successful `gog calendar events` call
- Task type defaults only apply on first parse (not retroactive)

---

## 🎯 Next Steps

1. Test with real task data
2. Customize task_types in config.json for your workflow
3. Try `t plan` to see your week overview
4. Use `t focus` for daily prioritization

Enjoy your enhanced task management! 🌱
