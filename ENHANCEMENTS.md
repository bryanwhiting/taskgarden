# thegarden Enhancements

## New Features

### 1. Hashtag Support (#task-types) with Defaults

Tasks can now include hashtags in their titles, which are automatically parsed and stored:

```bash
# Add a task with hashtags
t add "Review PR #FollowUp #CodeReview"

# Filter by tag
t list --tag FollowUp
t list --tag Bug
```

**Features:**
- Hashtags are extracted from task titles automatically
- Display in list views with dimmed styling
- Filter by specific tag using `--tag` flag
- Tags are preserved when merging tasks

**Pre-configured Task Types with Defaults:**

The following task types automatically apply default priority and time estimates:

1. **#FollowUp** - Default: P1, 30m
   - Quick check-ins, status updates, brief responses
   
2. **#Plan** - Default: P1, 1h
   - Strategic planning, roadmap sessions, project scoping
   
3. **#DeepWork** - Default: P0, 2h
   - Focused work requiring uninterrupted time

**Example:**
```bash
# When you create a task with #FollowUp:
"Check in with client #FollowUp"
# → Automatically gets P1 priority and 30m time estimate

# When you create a task with #DeepWork:
"Write architecture doc #DeepWork"
# → Automatically gets P0 priority and 2h time estimate
```

**Configuration:**

Task types are defined in `~/.thegarden/config.json`:
```json
"task_types": {
  "FollowUp": {
    "priority": "P1",
    "time": "30m"
  },
  "Plan": {
    "priority": "P1",
    "time": "1h"
  },
  "DeepWork": {
    "priority": "P0",
    "time": "2h"
  }
}
```

Add your own task types by editing this section!

### 2. `t schedule` Command - Smart Calendar Blocking

Automatically finds free time blocks and schedules tasks into your calendar:

```bash
t schedule           # Schedule today's tasks
t schedule --week    # Schedule whole week
t schedule --auto    # Auto-create calendar events (no prompt)
```

**Output example:**
```
🗓️ Schedule Suggestions for Today (Monday, Jan 20)

Free blocks available:
  2:00-4:30 PM (2.5h free)
  5:00-6:00 PM (1h free)

Suggested schedule:
  ✓ 2:00-4:00 PM → 🔴 [P0][SILVERMINE]{2h} Fix editor bug #DeepWork
  ✓ 4:00-4:30 PM → 🟡 [P1][WORKDAY]{30m} Client prep #Plan
  ✓ 5:00-5:30 PM → 🟡 [P1][LIFE]{30m} Insurance call #FollowUp

Block these on your calendar? (y/n)
```

**How it works:**
1. Fetches your Google Calendar events for the day/week
2. Calculates free time blocks between meetings (8 AM - 6 PM)
3. Gets P0 and P1 tasks with time estimates
4. Matches tasks to available blocks (P0 first, longest tasks first)
5. Prompts to create calendar events (unless `--auto`)

**Smart scheduling algorithm:**
- Prioritizes P0 tasks over P1
- Schedules longer tasks first to ensure they fit
- Only considers blocks of 15+ minutes
- Updates Google Calendar via `gog calendar events create`

**Features:**
- 📅 Respects existing calendar commitments
- 🎯 Focuses on P0/P1 tasks only
- ⏰ Requires time estimates on tasks
- 🔄 Works with task type hashtags (#DeepWork → 2h)
- ✅ Optional auto-creation of calendar blocks

### 3. `t plan` Command

Shows the complete week plan: upcoming meetings from Google Calendar + tasks:

```bash
t plan
```

**Output example:**
```
🌱 This Week Plan

Monday, Jan 20:
  📅 09:00 AM-09:45 AM: Tracy Sync
  📅 11:00 AM-12:00 PM: Website Review
  🔴 [P0][SILVERMINE]{2h} Fix editor bug
  
Tuesday, Jan 21:
  🟡 [P1][WORKDAY]{1h} #Plan Client strategy
  
Wednesday, Jan 22:
  📅 02:00 PM-03:00 PM: Standup
  🟢 [P2][LIFE]{30m} #FollowUp John
```

**Features:**
- Fetches meetings from Google Calendar using `gog calendar events`
- Combines calendar events with tasks for the current week
- Groups by day (Monday - Sunday)
- Shows calendar emoji 📅 for meetings
- Shows priority colors for tasks (🔴 P0, 🟡 P1, 🟢 P2, 🔵 P3)
- Tasks sorted by priority within each day
- Events sorted by time within each day
- Highlights today's date in green

### 3. `t focus` Command

Shows ONLY critical tasks that need immediate attention:

```bash
t focus
```

**Output example:**
```
🎯 Focus - Critical Tasks

🔴 P0 (do first):
  1. [SILVERMINE]{1h} Fix editor bug
  2. [WORKDAY]{30m} Client call prep

🟡 P1 (due today):
  3. [LIFE]{2h} DMARC setup

Total planned: 3.5h
```

**What it shows:**
- All P0 tasks (regardless of date)
- P1 tasks that are overdue or due today
- Hides everything else (P2, P3, future tasks)
- Shows total time estimate

### 4. Better Viewer - Grouped by Date

Enhanced list view with date grouping:

```bash
t list --grouped
# or
t list -g
```

**Output example:**
```
════ OVERDUE ════
[P0][SILVERMINE]{1h} Fix editor bug

════ TODAY (Monday, Jan 20) ════
[P1][WORKDAY]{30m} Client prep
[P2][LIFE]{15m} Quick admin

════ THIS WEEK ════
Tuesday, Jan 21:
  [P0][WORKDAY]{2h} Presentation

Wednesday, Jan 22:
  [P1][SILVERMINE]{1h} Code review
```

**Features:**
- Groups tasks by date category (Overdue, Today, This Week, Future)
- Color-coded section headers
- Shows weekday names for this week
- Works with all existing filters

### 5. Date Parsing

Natural language date parsing in task titles:

```bash
t add "Call client due Monday"           # → sets date to next Monday
t add "Review docs due 1/25"             # → sets date to Jan 25
t add "Follow up in 3 days"              # → sets date to +3 days
t add "Meeting tomorrow"                 # → sets date to tomorrow
```

**Supported formats:**
- Day of week: `Monday`, `Tue`, `Wednesday`, etc.
- Relative: `tomorrow`, `in 3 days`, `in 5 days`
- Date format: `1/25`, `12/5` (M/D or M/DD)
- Works with "due" or "next" prefixes

## Testing

All features have been implemented and build successfully. Test with:

```bash
# Test task type defaults with hashtags
# Create a task with #FollowUp - should auto-apply P1 and 30m
# Create a task with #DeepWork - should auto-apply P0 and 2h
t add "Client check-in #FollowUp"
t list  # Should show with P1 and 30m defaults

# Test hashtag filtering
t list --tag FollowUp

# Test plan command (shows this week's meetings + tasks)
t plan

# Test focus command
t focus

# Test grouped view
t list --grouped

# Test date parsing (requires adding a new task via Google Tasks UI or triage)
# The date will be automatically parsed when the task is synced
```

## Implementation Details

**Files modified:**

1. **`~/.thegarden/config.json`** - Configuration
   - Added `task_types` section with defaults for #FollowUp, #Plan, #DeepWork

2. **`src/main.rs`** - All new features
   - Added `tags: Vec<String>` to Task struct
   - Added `TaskTypeDefaults` struct for config
   - Added `task_types: HashMap<String, TaskTypeDefaults>` to Config struct
   - Added `Focus` and `Plan` command enums
   - Added `--grouped` and `--tag` flags to List command
   - Implemented `cmd_plan()` function - integrates Google Calendar + tasks
   - Implemented `cmd_focus()` function - shows critical tasks only
   - Implemented `parse_date_from_text()` for natural language dates
   - Updated `Task::parse()` to extract hashtags and dates
   - Added `Task::parse_with_config()` to apply task type defaults
   - Updated `Task::format()` to display hashtags
   - Updated `get_tasks_from_cache()` to use parse_with_config
   - Updated `cmd_list()` to support tag filtering and grouped view
   - Updated `cmd_merge()` to merge tags when combining tasks
   - Updated `create_default_config()` to include task_types

**Existing features preserved:**
- ✅ Sync with Google Tasks
- ✅ Triage workflow (priority, project, time, status, context)
- ✅ All existing filters and sorting
- ✅ Instant save to cache
- ✅ Merge duplicates

**New dependencies:**
- Uses `gog calendar events` command for fetching meetings
- Requires `gog` CLI to be installed and authenticated

## Build

```bash
cd /Users/bryanwhiting/clawd/thegarden
cargo build --release
```

Binary location: `target/release/thegarden`
