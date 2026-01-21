# 🗓️ `t schedule` - Smart Calendar Blocking

Automatically finds free time blocks in your Google Calendar and schedules tasks into them.

## Usage

```bash
t schedule           # Schedule today's P0/P1 tasks
t schedule --week    # Schedule whole week
t schedule --auto    # Auto-create calendar events (no prompt)
```

## Example Output

```
🗓️ Schedule Suggestions for Today (Monday, Jan 20)

Monday, Jan 20:

  Free blocks available:
    2:00-4:30 PM (2.5h free)
    5:00-6:00 PM (1h free)

  Suggested schedule:
    ✓ 2:00-4:00 PM → 🔴 [P0][SILVERMINE]{2h} Fix editor bug #DeepWork
    ✓ 4:00-4:30 PM → 🟡 [P1][WORKDAY]{30m} Client prep #Plan
    ✓ 5:00-5:30 PM → 🟡 [P1][LIFE]{30m} Insurance call #FollowUp

Block these on your calendar? (y/n) 
```

## How It Works

### 1. Fetch Calendar Events
Uses `gog calendar events --from [date] --to [date] --json` to get your Google Calendar

### 2. Calculate Free Blocks
- Working hours: 8 AM to 6 PM
- Finds gaps between existing meetings
- Minimum block size: 15 minutes

### 3. Get Schedulable Tasks
Filters for:
- P0 and P1 priorities only
- Tasks with time estimates
- Tasks within date range

### 4. Smart Matching Algorithm
- **Priority first:** P0 tasks before P1
- **Longest first:** Schedules longer tasks first to ensure they fit
- **Greedy allocation:** Fills blocks sequentially

### 5. Optional Calendar Blocking
- Prompts to create calendar events (unless `--auto`)
- Creates events via `gog calendar events create`
- Event titles: `[P0] Task title` format

## Features

✅ **Respects existing commitments** - Won't double-book you  
✅ **Focus on what matters** - Only P0/P1 tasks  
✅ **Works with hashtags** - `#DeepWork` → auto 2h blocks  
✅ **Week-ahead planning** - Schedule whole week with `--week`  
✅ **Automation ready** - Use `--auto` for instant blocking  

## Requirements

- Tasks must have:
  - Priority: P0 or P1
  - Time estimate: e.g., `{30m}`, `{1h}`, `{2h}`
  - Date: within scheduling range

- System requirements:
  - `gog` CLI installed and authenticated
  - Google Calendar access

## Examples

### Daily Workflow
```bash
# Morning: Schedule today's critical tasks
t schedule

# Auto-block without prompting
t schedule --auto
```

### Weekly Planning
```bash
# Schedule whole week on Monday morning
t schedule --week

# Review suggestions, create blocks manually
t schedule --week
# → (Review, press 'n' to skip auto-creation)
# → Manually create calendar events from suggestions
```

### Combined Workflow
```bash
# 1. Add tasks with hashtags (auto-applies time estimates)
t add "Architecture review #DeepWork"     # Auto: P0, 2h
t add "Client strategy call #Plan"        # Auto: P1, 1h
t add "Follow up with Sarah #FollowUp"    # Auto: P1, 30m

# 2. Triage to set priorities/projects
t triage

# 3. Schedule into calendar
t schedule
# → Sees: 2h DeepWork, 1h Plan, 30m FollowUp
# → Suggests time blocks matching your calendar
```

## Algorithm Details

### Free Block Calculation
```
For each day:
  Start: 8:00 AM
  End: 6:00 PM
  
  For each calendar event:
    If gap exists before event >= 15min:
      Add free block
    
    Move cursor to event end time
  
  If time remaining until 6 PM >= 15min:
    Add final free block
```

### Task Matching
```
Sort tasks:
  1. By priority (P0 before P1)
  2. By duration (longest first)

For each task:
  Find first free block that fits
  Schedule task at start of block
  Update block (shrink or remove)
```

### Example Matching
```
Free blocks: 2:00-4:30 PM (2.5h), 5:00-6:00 PM (1h)
Tasks: [P0]{2h}, [P1]{1h}, [P1]{30m}

Match 1: [P0]{2h} → 2:00-4:00 PM
  Remaining: 4:00-4:30 PM (30m), 5:00-6:00 PM (1h)

Match 2: [P1]{1h} → 5:00-6:00 PM
  Remaining: 4:00-4:30 PM (30m)

Match 3: [P1]{30m} → 4:00-4:30 PM
  Remaining: (none)
```

## Tips

1. **Use hashtag defaults** to auto-apply time estimates:
   - `#FollowUp` → 30m
   - `#Plan` → 1h
   - `#DeepWork` → 2h

2. **Schedule in the morning** when you know your day:
   ```bash
   t plan      # See your day
   t focus     # See critical items
   t schedule  # Block time for them
   ```

3. **Week-ahead planning** on Monday mornings:
   ```bash
   t schedule --week
   ```

4. **Combine with `t plan`** to see complete picture:
   ```bash
   t plan      # Meetings + tasks overview
   t schedule  # Block time for tasks
   ```

5. **Adjust working hours** by editing the source:
   - Default: 8 AM - 6 PM
   - Modify `calculate_free_blocks()` for custom hours

## Limitations

- Only schedules during working hours (8 AM - 6 PM)
- Doesn't account for task dependencies
- Doesn't consider task energy levels (focus vs. admin)
- Requires tasks to have time estimates
- Requires Google Calendar access via `gog` CLI

## Future Enhancements

Potential features for future versions:
- Custom working hours per day
- Energy-based scheduling (morning focus time vs. afternoon admin)
- Respect for buffer time between tasks
- Task dependencies and ordering
- Multi-day task splitting
- Calendar preference (minimize context switching)

## Troubleshooting

**"Could not fetch calendar events"**
- Install `gog` CLI: follow gog installation docs
- Authenticate: `gog auth login`
- Test: `gog calendar events --from today --to today`

**"No tasks with time estimates to schedule"**
- Add time estimates during triage
- Or use hashtags: `#FollowUp`, `#Plan`, `#DeepWork`
- Check task priorities (must be P0 or P1)

**"No free blocks available"**
- Your calendar is fully booked
- Try scheduling future days: `t schedule --week`
- Or adjust working hours in code

## Integration with Other Commands

```bash
# Full workflow
t sync              # Sync Google Tasks
t triage            # Set priorities/times
t plan              # View week overview
t focus             # See critical items
t schedule          # Block calendar time
```

---

**Part of thegarden v0.3.0** - ADHD-friendly task management with smart calendar integration
