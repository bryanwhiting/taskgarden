# TaskGarden Universal Task Format Design

**Inspired by:** [Obsidian Tasks Plugin](https://publish.obsidian.md/tasks/Introduction)  
**Goal:** One task format that works across platforms (Google Tasks, ClickUp, Airtable, etc.)

## Philosophy

- **Emoji-based** - More readable than brackets, universal across platforms
- **Human-readable** - Easy to type and understand at a glance
- **Platform-aware** - Map to native features when available, store in description when not
- **One-line friendly** - Entire task fits on one line
- **Copy-paste safe** - No special formatting that breaks when copied

## Core Task Format

```
Task title ⏰ 1h 📅 2026-01-28 🛫 2026-01-27 ⏳ 2026-01-28 ⏫ #project @context +tag
```

### Breakdown

| Emoji | Property | Description | Example | Platforms |
|-------|----------|-------------|---------|-----------|
| ⏰ | Duration | Time estimate to complete | `⏰ 1h`, `⏰ 30m`, `⏰ 2h30m` | ClickUp (time estimate), Google (description) |
| 📅 | Due | When task is due | `📅 2026-01-28` | ClickUp, Google Tasks, Airtable |
| 🛫 | Start | When to start working on it | `🛫 2026-01-27` | ClickUp (start date), Google (description) |
| ⏳ | Scheduled | When you plan to work on it | `⏳ 2026-01-28` | ClickUp (custom field), Google (description) |
| ➕ | Created | When task was created | `➕ 2026-01-26` | Auto-added, most platforms support |
| ✅ | Done | When task was completed | `✅ 2026-01-28` | Auto-added on completion |
| ❌ | Cancelled | When task was cancelled | `❌ 2026-01-28` | Status change |

### Priority

| Emoji | Priority | Meaning | ClickUp | Google |
|-------|----------|---------|---------|--------|
| 🔺 | P0 | Highest - Urgent + Important | 🔴 Urgent (1) | Description |
| ⏫ | P1 | High - Important, not urgent | ⚠️ High (2) | Description |
| 🔼 | P2 | Medium - Urgent, not important | 📘 Normal (3) | Description |
| 🔽 | P3 | Low - Neither urgent nor important | 📗 Low (4) | Description |
| ⏬ | P5 | Lowest - Delegate/Maybe | 📗 Low (4) | Description |

### Organization

| Symbol | Property | Description | Example | Platforms |
|--------|----------|-------------|---------|-----------|
| # | Tags | Categorize/filter | `#DeepWork #FollowUp` | ClickUp (tags), Google (description) |
| @ | Context | Where/when you can do it | `@work @home @phone` | ClickUp (tags/custom), Google (description) |
| / | Project | What project it belongs to | `/silvermine /personal /workday` | ClickUp (tags/list), Google (list) |

### Advanced (Obsidian-inspired)

| Emoji | Property | Description | Example | Platforms |
|-------|----------|-------------|---------|-----------|
| 🔁 | Recurrence | Repeating tasks | `🔁 every week` | ClickUp, Google (description) |
| 🆔 | ID | Unique identifier for dependencies | `🆔 abc123` | ClickUp (custom), Google (description) |
| ⛔ | Depends On | Task dependencies | `⛔ abc123,def456` | ClickUp (dependencies), Google (description) |
| 🏁 | On Completion | What to do when done | `🏁 delete`, `🏁 keep` | Metadata |

## Platform Mapping

### ClickUp (Full Feature Support)

```
Task title ⏰ 1h 📅 2026-01-28 🛫 2026-01-27 ⏫ #silvermine @work

Maps to:
- Name: "Task title"
- Time Estimate: 3600000ms (1h)
- Due Date: Jan 28, 2026
- Start Date: Jan 27, 2026
- Priority: ⚠️ High (2)
- Tags: ["silvermine", "work"]
- Description: Full task string for reference
```

### Google Tasks (Limited Features)

```
Task title ⏰ 1h 📅 2026-01-28 🛫 2026-01-27 ⏫ #silvermine @work

Maps to:
- Title: Full task string (everything)
- Due Date: Jan 28, 2026 (if API supports)
- Notes: Breakdown of properties
```

### Airtable (Flexible Fields)

```
Task title ⏰ 1h 📅 2026-01-28 🛫 2026-01-27 ⏫ #silvermine @work

Maps to:
- Title: "Task title"
- Duration: "1h"
- Due Date: 2026-01-28
- Start Date: 2026-01-27
- Priority: "P1"
- Tags: "silvermine, work"
- Full String: (stored for reference)
```

## Examples

### Simple Task
```
Call mom ⏰ 15m 📅 2026-01-28 @phone
```

### Work Task with Everything
```
Review Q4 performance docs ⏰ 2h 📅 2026-01-30 🛫 2026-01-28 ⏳ 2026-01-29 ⏫ /workday @computer #DeepWork
```

### Personal Recurring Task
```
Weekly planning ⏰ 30m 🔁 every Sunday ⏳ 2026-02-02 🔼 /personal @home #planning
```

### Team Task with Dependencies
```
Deploy new feature 🆔 feat-123 ⏰ 1h 📅 2026-02-01 ⛔ test-789 ⏫ /silvermine @work #release
```

### Quick Capture (Minimal)
```
Buy groceries 📅 tomorrow @errands
```

## Date Formats

### Absolute Dates
```
📅 2026-01-28          # ISO format (preferred)
📅 2026-01-28T14:00    # With time
📅 Jan 28              # Month + day (current year assumed)
📅 01/28               # MM/DD (current year assumed)
```

### Relative Dates (Parsed on creation)
```
📅 today
📅 tomorrow
📅 next week
📅 +3d                 # 3 days from now
📅 next Monday
```

### Smart Dates
```
⏳ weekdays            # Scheduled for next weekday
🛫 next sprint         # Context-specific
```

## Duration Formats

```
⏰ 15m                 # Minutes
⏰ 1h                  # Hours
⏰ 1h30m               # Hours + minutes
⏰ 2.5h                # Decimal hours
⏰ 1d                  # Days (8 hours)
```

## Recurrence Patterns (Obsidian-inspired)

```
🔁 every day
🔁 every week
🔁 every 2 weeks
🔁 every month
🔁 every Monday
🔁 every weekday
🔁 every 1st of month
🔁 when done          # Interval starts after completion
```

## Task Status Symbols

```
- [ ]   To do (needsTriage, todo)
- [x]   Done (completed)
- [-]   Cancelled
- [>]   Forwarded/Delegated
- [<]   Scheduled (waiting)
- [!]   Important
- [?]   Question/Needs info
- [/]   In progress
```

## Parsing Rules

### Order Independence
Properties can appear in any order after the title:
```
✅ Task ⏰ 1h 📅 tomorrow @work
✅ Task @work 📅 tomorrow ⏰ 1h
```

### Title Extraction
Everything before the first emoji property = title:
```
"Review docs and send feedback ⏰ 1h" → Title: "Review docs and send feedback"
```

### Smart Hashtag Detection
```
Task about #hashtag ⏰ 1h     → Title includes "#hashtag", no tag extracted
Task ⏰ 1h #tag               → Tag: "tag", title: "Task"
```

### Context vs Tag
```
@work     → Context (where/when)
#work     → Tag (category/label)
```

## Implementation Strategy

### Phase 1: Core Format (Current)
- [x] Duration (⏰)
- [x] Due date (📅)
- [x] Priority (🔺⏫🔼🔽⏬)
- [x] Tags (#)
- [x] Project (/)
- [x] Context (@)

### Phase 2: Advanced Dates
- [ ] Start date (🛫)
- [ ] Scheduled date (⏳)
- [ ] Created date (➕) - auto
- [ ] Done date (✅) - auto

### Phase 3: Dependencies & Recurrence
- [ ] Task ID (🆔)
- [ ] Depends on (⛔)
- [ ] Recurrence (🔁)
- [ ] On completion (🏁)

### Phase 4: Platform-Specific Enhancements
- [ ] ClickUp subtasks
- [ ] Airtable relations
- [ ] Google Tasks notes formatting

## Sync Behavior

### Google Tasks → ClickUp
```
Input (Google):
"Review docs ⏰ 2h 📅 2026-01-28 ⏫ #silvermine @work"

Output (ClickUp):
- Name: "Review docs"
- Time Estimate: 7200000ms
- Due Date: 2026-01-28
- Priority: High (2)
- Tags: ["silvermine", "work"]
- Description: "⏰ 2h 📅 2026-01-28 ⏫ #silvermine @work\n\nOriginal: Review docs..."
```

### ClickUp → Google Tasks (Future)
```
Input (ClickUp):
- Name: "Review docs"
- Time Estimate: 2h
- Due: 2026-01-28
- Tags: ["silvermine"]

Output (Google):
"Review docs ⏰ 2h 📅 2026-01-28 #silvermine"
```

## Migration from Current Format

### Current Format
```
[2026-01-27][P0][SILVERMINE][progress][@work]{2h} Fix bug
```

### New Format
```
Fix bug ⏰ 2h 📅 2026-01-27 🔺 /silvermine @work [progress]
```

### Migration Script
Auto-convert bracket format to emoji format:
- `[date]` → `📅 date`
- `[P0]` → `🔺`
- `[PROJECT]` → `/project`
- `{time}` → `⏰ time`
- `[@context]` → `@context`
- `[status]` → Keep as-is for now

## Benefits

### For Users
✅ **Readable** - "Fix bug ⏰ 1h 📅 tomorrow 🔺" is clearer than `[date][P0][PROJ]{1h}`  
✅ **Fast to type** - Emoji autocomplete in most editors  
✅ **Platform agnostic** - Same format everywhere  
✅ **Copy-paste friendly** - Looks good in Slack, emails, notes  

### For System
✅ **Easy to parse** - Regex-friendly emoji markers  
✅ **Extensible** - Add new properties without breaking existing  
✅ **Reversible** - Can reconstruct task from any platform  
✅ **Debuggable** - Human can read the description and know what happened  

## Future Considerations

### Natural Language Parsing
```
"Call mom tomorrow at 2pm for 15min"
→ "Call mom ⏰ 15m 📅 2026-01-29 ⏳ 2026-01-29T14:00"
```

### AI Integration
Description field in ClickUp can include:
- Full emoji task string
- AI-generated subtasks
- Related links/context
- Previous notes/updates

### Multi-platform Consistency
Same task across 3 platforms:
```
Google Tasks: Full emoji string in title
ClickUp: Parsed into fields + description backup
Airtable: Each field in proper column
```

All three stay in sync via the canonical emoji format.

## References

- [Obsidian Tasks Plugin](https://publish.obsidian.md/tasks/Introduction)
- [Obsidian Tasks Emoji Format](https://publish.obsidian.md/tasks/Reference/Task+Formats/Tasks+Emoji+Format)
- [Obsidian Tasks Quick Reference](https://publish.obsidian.md/tasks/Quick+Reference)
