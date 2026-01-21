# 🌱 thegarden Quick Start Guide

## 🎯 New Commands

### `t schedule` - Smart Calendar Blocking
```bash
t schedule           # Schedule today's tasks
t schedule --week    # Schedule whole week
t schedule --auto    # Auto-create events (no prompt)
```
Finds free time blocks and schedules P0/P1 tasks into your calendar.

**Example Output:**
```
🗓️ Schedule Suggestions for Today

Free blocks available:
  2:00-4:30 PM (2.5h free)
  5:00-6:00 PM (1h free)

Suggested schedule:
  ✓ 2:00-4:00 PM → 🔴 [P0][SILVERMINE]{2h} Fix bug #DeepWork
  ✓ 4:00-4:30 PM → 🟡 [P1][WORKDAY]{30m} Client prep

Block these on your calendar? (y/n)
```

---

### `t plan` - Your Week at a Glance
```bash
t plan
```
Shows meetings from Google Calendar + tasks for the entire week, grouped by day.

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

### `t focus` - Critical Tasks Only
```bash
t focus
```
Shows ONLY what needs your attention right now (P0 + overdue/due today P1).

**Example Output:**
```
🎯 Focus - Critical Tasks

🔴 P0 (do first):
  1. [SILVERMINE]{2h} Fix editor bug

🟡 P1 (due today):
  2. [WORKDAY]{1h} Client call prep

Total planned: 3h
```

---

### `t list --grouped` - Organized by Date
```bash
t list --grouped
# or
t list -g
```
Groups your tasks by date sections (Overdue → Today → This Week).

**Example Output:**
```
════ OVERDUE ════
[P0][SILVERMINE]{1h} Fix bug

════ TODAY (Monday, Jan 20) ════
[P1][WORKDAY]{30m} Client prep
[P2][LIFE]{15m} Quick admin

════ THIS WEEK ════
Tuesday, Jan 21:
  [P0][WORKDAY]{2h} Presentation
```

---

### `t list --tag` - Filter by Hashtag
```bash
t list --tag FollowUp
t list --tag Plan
t list --tag DeepWork
```
Shows only tasks with specific hashtags.

---

## 🏷️ Smart Hashtags with Auto-Defaults

When you create a task with these hashtags, priority and time are automatically applied:

| Hashtag | Auto Priority | Auto Time | Use Case |
|---------|--------------|-----------|----------|
| `#FollowUp` | P1 | 30m | Quick check-ins, brief responses |
| `#Plan` | P1 | 1h | Strategic planning, roadmapping |
| `#DeepWork` | P0 | 2h | Focused work, uninterrupted time |

**Example:**
```bash
# You create: "Client strategy session #Plan"
# System applies: P1 priority, 1h time estimate
# Result: [2026-01-20][P1][---]{1h} Client strategy session #Plan
```

---

## 📅 Natural Date Parsing

Create tasks with human-friendly dates:

```bash
"Call client due Monday"      → Sets date to next Monday
"Review docs due 1/25"        → Sets date to January 25
"Follow up in 3 days"         → Sets date to +3 days
"Meeting tomorrow"            → Sets date to tomorrow
```

---

## 🔄 Complete Workflow Example

### Morning Routine
```bash
# 1. Sync with Google
t sync

# 2. See the week ahead
t plan

# 3. Focus on today's critical items
t focus

# 4. Schedule tasks into calendar blocks
t schedule

# 5. Check all tasks grouped by date
t list --grouped
```

### Throughout the Day
```bash
# Add a quick follow-up (auto: P1, 30m)
t add "Check in with Sarah #FollowUp"

# Add deep work session (auto: P0, 2h)
t add "Architecture review #DeepWork"

# Filter by task type
t list --tag FollowUp
```

### End of Day
```bash
# Triage new tasks
t triage

# Bump incomplete tasks to tomorrow
t bump

# View tomorrow's plan
t list --days 1
```

---

## 🎨 Visual Legend

### Priority Emojis
- 🔴 **P0** - Urgent + Important (do first)
- 🟡 **P1** - Important, not urgent (plan for)
- 🟢 **P2** - Urgent, not important (delegate if possible)
- 🔵 **P3** - Not important, not urgent (backlog)

### Item Type Indicators
- 📅 **Calendar event** (from Google Calendar)
- 🔴/🟡/🟢/🔵 **Task** (from Google Tasks)

### Date Section Colors
- **Red (OVERDUE)** - Tasks past their date
- **Green (TODAY)** - Tasks for today
- **Cyan (THIS WEEK)** - Tasks for this week

---

## ⚙️ Configuration

Edit `~/.thegarden/config.json` to customize:

```json
{
  "task_types": {
    "FollowUp": { "priority": "P1", "time": "30m" },
    "Plan": { "priority": "P1", "time": "1h" },
    "DeepWork": { "priority": "P0", "time": "2h" },
    
    // Add your own:
    "Bug": { "priority": "P0", "time": "1h" },
    "Meeting": { "priority": "P1", "time": "45m" }
  }
}
```

---

## 🚀 Pro Tips

1. **Use hashtags consistently**
   - `#FollowUp` for all follow-ups
   - `#Plan` for planning sessions
   - `#DeepWork` for focused work

2. **Start your day with `t plan`**
   - See meetings + tasks in one view
   - Plan your day around calendar blocks

3. **Use `t focus` when overwhelmed**
   - Cuts through noise
   - Shows only what matters NOW

4. **Combine filters**
   ```bash
   t list --tag FollowUp --project WORKDAY
   t list --grouped --priority P0,P1
   ```

5. **Natural dates save time**
   ```bash
   "Review design due Friday"
   "Call back in 2 days"
   ```

---

## 📖 All Commands

```bash
t triage          # Interactive triage workflow
t focus           # Show critical tasks only
t plan            # Show this week (meetings + tasks)
t schedule        # Schedule tasks into calendar blocks
t list            # List tasks (add --grouped or --tag)
t add             # Add new task
t bump            # Bump incomplete tasks to tomorrow
t merge           # Find and merge duplicates
t sync            # Manual sync with Google Tasks
t show <id>       # Show task details
t search <term>   # Search tasks
```

---

## 🆘 Need Help?

```bash
t --help              # List all commands
t list --help         # See all list options
t triage --help       # See triage options
```

---

## 🎉 You're Ready!

Start with:
```bash
t plan      # See your week
t focus     # Focus on critical tasks
t schedule  # Block time on calendar
```

Happy task managing! 🌱✨
