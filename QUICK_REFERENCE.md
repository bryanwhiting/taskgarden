# 🌱 thegarden - Quick Reference Card

## 🎯 Essential Commands

```bash
t plan              # Week overview (meetings + tasks)
t focus             # Critical tasks only (P0 + overdue P1)
t schedule          # Auto-block calendar time
t list --grouped    # Tasks grouped by date
t list --tag Tag    # Filter by hashtag
t triage            # Interactive prioritization
t sync              # Force sync with Google Tasks
```

## 📝 Quick Task Entry

```bash
# With smart hashtag defaults:
t add "Client call #FollowUp"      # → P1, 30m
t add "Strategy meeting #Plan"     # → P1, 1h
t add "Deep coding session #DeepWork" # → P0, 2h

# With natural dates:
t add "Review docs due Monday"     # → date: next Monday
t add "Follow up in 3 days"        # → date: +3 days
t add "Call tomorrow"              # → date: tomorrow
```

## ⚡ Smart Hashtags

| Hashtag | Priority | Time | Use Case |
|---------|----------|------|----------|
| `#FollowUp` | P1 | 30m | Quick check-ins |
| `#Plan` | P1 | 1h | Strategic planning |
| `#DeepWork` | P0 | 2h | Focused work |

*Add your own in `~/.thegarden/config.json`*

## 🗓️ Schedule Commands

```bash
t schedule          # Schedule today's P0/P1 tasks
t schedule --week   # Schedule entire week
t schedule --auto   # Auto-create events (no prompt)
```

## 📋 List Filters

```bash
t list                        # Today's tasks
t list --all                  # All tasks
t list --grouped              # Group by date
t list --tag FollowUp         # Filter by hashtag
t list --priority P0,P1       # Filter by priority
t list --project WORKDAY      # Filter by project
t list --days 7               # Last 7 days
t list --grouped --tag Plan   # Combine filters!
```

## 🎨 Priority Colors

- 🔴 **P0** - Urgent + Important (do first)
- 🟡 **P1** - Important, not urgent
- 🟢 **P2** - Urgent, not important
- 🔵 **P3** - Not important, not urgent

## ⚡ Sync Behavior

```bash
# First command: Syncs
$ t list
🔄 Syncing...
✓ Synced 3 tasks

# Next commands: Instant (uses cache)
$ t focus
⚡ Using cache (synced 2 min ago)

$ t plan
⚡ Using cache (synced 3 min ago)

# Force sync anytime
$ t sync
🔄 Force syncing...
```

**Default throttle:** 10 minutes  
**Configure:** Edit `sync_throttle_minutes` in config.json

## 🔄 Daily Workflow

### Morning
```bash
t plan      # See your week
t focus     # See critical items  
t schedule  # Block calendar time
```

### During Day
```bash
t add "Task #FollowUp"    # Quick entry
t list --tag Plan         # Filter tasks
```

### End of Day
```bash
t triage    # Prioritize new tasks
t bump      # Bump incomplete tasks
```

## 📅 Date Formats

Natural language parsing:
- `"due Monday"` → next Monday
- `"due Tuesday"`, `"due Wed"` → weekdays
- `"tomorrow"` → next day
- `"in 3 days"` → +3 days
- `"due 1/25"` → January 25

## 🎛️ Configuration

**Location:** `~/.thegarden/config.json`

**Key settings:**
```json
{
  "sync_throttle_minutes": 10,   // Cache window
  "task_types": {
    "YourTag": {
      "priority": "P1",
      "time": "1h"
    }
  }
}
```

## 🔍 Search & Filter

```bash
t search "keyword"              # Search titles
t search --project WORKDAY      # Search in project
t search --status progress      # Search by status
t show <task-id>                # Show task details
```

## 🔧 Maintenance

```bash
t sync              # Force sync
t sync --force      # Full sync (all tasks)
t merge             # Find duplicates
t bump              # Bump to tomorrow
t bump --week       # Bump to next week
```

## 💡 Pro Tips

1. **Use hashtags consistently** for auto-defaults
2. **Start day with `t plan`** to see everything
3. **Use `t focus`** when overwhelmed
4. **Combine filters:** `t list --grouped --tag Plan`
5. **Natural dates save time:** "due Monday" vs manual entry
6. **Let throttling work:** Instant commands after first sync!

## 🆘 Troubleshooting

**Slow syncs?**
- Throttling is working! First command syncs, rest use cache
- Force sync: `t sync`

**Missing tasks?**
- Force sync: `t sync`
- Check Google Tasks web/app

**Calendar not blocking?**
- Install `gog` CLI
- Authenticate: `gog auth login`
- Test: `gog calendar events`

## 📖 Full Documentation

- `QUICK_START.md` - Getting started guide
- `SCHEDULE_FEATURE.md` - Schedule command details
- `SYNC_THROTTLE.md` - Throttling explained
- `COMPLETE_BUILD_REPORT.md` - All features

## 🎯 Most Common Commands

```bash
# Daily essentials
t plan              # Morning overview
t focus             # What matters now
t schedule          # Block time
t add "Task #Tag"   # Quick entry

# Quick views
t list --grouped    # Date view
t list --tag Tag    # Category view

# Maintenance
t triage            # Prioritize
t sync              # Refresh
```

---

**Happy task managing!** 🌱✨

*For help: `t --help` or `t <command> --help`*
