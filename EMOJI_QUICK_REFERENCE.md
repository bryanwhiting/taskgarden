# TaskGarden Emoji Quick Reference

## Task Properties

| Emoji | Property | Example | Notes |
|-------|----------|---------|-------|
| ⏰ | Duration | `⏰ 1h`, `⏰ 30m`, `⏰ 2h30m` | How long it takes |
| 📅 | Due | `📅 2026-01-28`, `📅 tomorrow` | When it's due |
| 🛫 | Start | `🛫 2026-01-27` | When to start |
| ⏳ | Scheduled | `⏳ 2026-01-28T14:00` | When you plan to work |
| ➕ | Created | `➕ 2026-01-26` | Auto-added |
| ✅ | Done | `✅ 2026-01-28` | Auto-added on completion |
| ❌ | Cancelled | `❌ 2026-01-28` | Auto-added when cancelled |

## Priority

| Emoji | Level | Name | ClickUp |
|-------|-------|------|---------|
| 🔺 | P0 | Highest | 🔴 Urgent |
| ⏫ | P1 | High | ⚠️ High |
| 🔼 | P2 | Medium | 📘 Normal |
| 🔽 | P3 | Low | 📗 Low |
| ⏬ | P5 | Lowest | 📗 Low |

## Organization

| Symbol | Type | Example | Platforms |
|--------|------|---------|-----------|
| # | Tag | `#DeepWork #FollowUp` | ClickUp, Airtable |
| @ | Context | `@work @home @phone` | ClickUp (tags) |
| / | Project | `/silvermine /workday /life` | ClickUp (list/tags) |

## Advanced

| Emoji | Property | Example | Notes |
|-------|----------|---------|-------|
| 🔁 | Recurrence | `🔁 every week` | Repeating tasks |
| 🆔 | ID | `🆔 abc123` | For dependencies |
| ⛔ | Depends On | `⛔ abc123,def456` | Task dependencies |
| 🏁 | On Completion | `🏁 delete`, `🏁 keep` | Post-completion action |

## Examples

### Quick Capture
```
Buy milk 📅 tomorrow @errands
```

### Work Task
```
Review Q4 docs ⏰ 2h 📅 2026-01-30 ⏫ /workday @computer #DeepWork
```

### Personal Recurring
```
Weekly planning ⏰ 30m 🔁 every Sunday /life @home
```

### Team Task
```
Deploy feature ⏰ 1h 📅 2026-02-01 🔺 /silvermine @work #release
```

## Tips

✅ **Order doesn't matter** - Properties can be anywhere after title  
✅ **Copy-paste friendly** - Emojis work in Slack, emails, ClickUp  
✅ **Auto-complete** - Type `:clock:` → ⏰ in most editors  
✅ **Minimal is OK** - Just title is valid: `Task name`  

## Keyboard Shortcuts

**macOS:**
- `⌃⌘Space` - Emoji picker
- Type emoji name: `:clock:` → ⏰

**Windows:**
- `Win + .` - Emoji panel

**Linux:**
- `Ctrl + .` or `Ctrl + ;` (Ubuntu)

## Common Patterns

### Tomorrow's urgent task
```
Task name ⏰ 30m 📅 tomorrow 🔺
```

### This week, medium priority
```
Task name ⏰ 2h 📅 this Friday 🔼
```

### Delegated task
```
Task name 📅 next week ⏬ @team
```

### No deadline, just track effort
```
Task name ⏰ 1h
```
