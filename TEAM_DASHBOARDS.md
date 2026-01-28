# Team Dashboards - Overview

TaskGarden now supports pushing your triaged tasks to **Airtable** or **ClickUp** for team visibility!

## Quick Comparison

| Feature | Airtable | ClickUp |
|---------|----------|---------|
| **Setup Difficulty** | Easy | Easy |
| **Free Tier** | ✅ Generous | ✅ Generous |
| **Custom Fields** | ✅✅✅ Excellent | ✅ Good |
| **Project Management** | ✅ Basic | ✅✅✅ Full-featured |
| **Subtasks** | ❌ Manual | ✅ Native |
| **Time Tracking** | ❌ Extensions | ✅ Built-in |
| **Dependencies** | ❌ Manual | ✅ Native |
| **Views** | Grid, Kanban, Calendar, Gallery, Timeline | List, Board, Calendar, Gantt, Timeline, Mind Map |
| **API Quality** | ✅✅ Excellent | ✅✅ Excellent |
| **Mobile App** | ✅ Great | ✅ Great |
| **Best For** | Flexible databases, custom workflows | Full project management, team collaboration |

## Setup Guides

- **Airtable:** [AIRTABLE_SETUP.md](./AIRTABLE_SETUP.md)
- **ClickUp:** [CLICKUP_SETUP.md](./CLICKUP_SETUP.md)

## Usage Guides

- **Airtable:** [AIRTABLE_USAGE.md](./AIRTABLE_USAGE.md)
- **ClickUp:** [CLICKUP_USAGE.md](./CLICKUP_USAGE.md)

## Architecture

```
┌─────────────────┐
│  Google Tasks   │  ← Capture system (mobile/web)
│   (via gog CLI) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   TaskGarden    │  ← Triage system (CLI)
│   SQLite Cache  │     - Prioritize
└────────┬────────┘     - Categorize
         │              - Estimate time
         │
         ├──────────────┬──────────────┐
         ▼              ▼              ▼
┌─────────────┐  ┌──────────┐  ┌──────────┐
│   Airtable  │  │ ClickUp  │  │  Future  │
│             │  │          │  │  (Notion,│
│ Database +  │  │ Full PM  │  │  Linear, │
│ Views       │  │ Features │  │  etc.)   │
└─────────────┘  └──────────┘  └──────────┘
      ▲               ▲
      │               │
┌─────┴────┐    ┌────┴─────┐
│   Team   │    │   Team   │
│  Member  │    │  Member  │
└──────────┘    └──────────┘
```

## Workflow

### 1. Capture
```bash
# Use Google Tasks (mobile app, web, or gog CLI)
# Just brain dump - don't worry about organization yet
```

### 2. Triage
```bash
# Pull from Google Tasks
t sync

# Organize your thoughts
t triage
# → Prioritize (P0-P5)
# → Categorize by project (WORKDAY, LIFE, SILVERMINE)
# → Estimate time (30m, 1h, 2h, etc.)
# → Add status, context, tags
```

### 3. Share
```bash
# Push to team dashboard(s)
t sync --airtable    # Team can view/filter/comment
# or
t sync --clickup     # Team can assign/track/collaborate
# or both!
t sync --airtable --clickup
```

## Use Cases

### Solo Freelancer
- **Capture:** Google Tasks (quick inbox)
- **Triage:** TaskGarden (organize your day)
- **Dashboard:** Airtable (client visibility into projects)

### Small Team (2-5 people)
- **Capture:** Google Tasks (personal inbox)
- **Triage:** TaskGarden (personal workflow)
- **Collaborate:** ClickUp (team sees priorities, can assign/comment)

### Department in Larger Org
- **Capture:** Google Tasks (personal tasks)
- **Triage:** TaskGarden (personal workflow)
- **Team Dashboard:** ClickUp (full PM features)
- **Executive Dashboard:** Airtable (custom views for leadership)

### Personal + Side Business
- **Capture:** Google Tasks (everything in one place)
- **Triage:** TaskGarden (separate LIFE vs BUSINESS)
- **Share:** Airtable for business partners, keep LIFE private

## Commands

### Basic
```bash
# Sync with Google Tasks only
t sync

# Push to Airtable
t sync --airtable

# Push to ClickUp
t sync --clickup

# Push to both
t sync --airtable --clickup
```

### Alias for Daily Use
```bash
# Add to your ~/.bashrc or ~/.zshrc
alias tsync='t sync --airtable --clickup'

# Then just:
tsync
```

## Configuration

Edit `config.json`:

```json
{
  "google_account": "you@gmail.com",
  
  "airtable": {
    "enabled": true,
    "api_key": "patXXXXXXXXXXXXXX",
    "base_id": "appXXXXXXXXXXXXXX",
    "table_name": "Tasks"
  },
  
  "clickup": {
    "enabled": true,
    "api_token": "pk_XXXXXXXXXXXXXXXX",
    "list_id": "901234567"
  }
}
```

## Data Flow

### What Gets Synced

Every task is parsed from this format:
```
[2026-01-27][P0][SILVERMINE][progress][@work]{2h} Fix editor bug #DeepWork
```

And converted to structured fields:

| Field | Airtable | ClickUp |
|-------|----------|---------|
| **Title** | "Fix editor bug #DeepWork" | "Fix editor bug #DeepWork" |
| **Priority** | "P0" | 🔴 Urgent (1) |
| **Project** | "SILVERMINE" | Tag: SILVERMINE |
| **Status** | "progress" | IN PROGRESS |
| **Context** | "@work" | Tag: @work |
| **Time Estimate** | "2h" | 2h (7200000ms) |
| **Due Date** | 2026-01-27 | Jan 27, 2026 |
| **Tags** | "DeepWork" | Tag: DeepWork |
| **Completed** | ☐ false | Open |

## Sync Behavior

### Current (v1): One-Way Push
- Google Tasks → TaskGarden → Airtable/ClickUp
- Changes in Airtable/ClickUp **do NOT sync back** to Google Tasks
- Last-write-wins: TaskGarden overwrites Airtable/ClickUp on next sync

**Recommendation:**
- Use TaskGarden as your **source of truth**
- Use Airtable/ClickUp as **read-only team dashboards**
- Team can comment/view, but updates should happen in TaskGarden

### Future (v2): Two-Way Sync
- Track modification timestamps
- Merge changes intelligently
- Conflict resolution UI
- Enable with `--bidirectional` flag

## Privacy & Security

### What's Synced
- Task titles, descriptions, metadata
- Priorities, projects, statuses, tags
- Due dates, time estimates
- Google Task IDs (stored as mapping)

### What's NOT Synced
- Your Google account credentials (never shared)
- Private notes (unless in task title)
- Completed tasks >7 days old (filtered out)

### API Tokens
- Stored in `config.json` (keep this file private!)
- Never committed to git (add to `.gitignore`)
- Airtable/ClickUp tokens are scoped to specific workspaces

## Troubleshooting

### "No changes to push"
- You haven't synced from Google Tasks yet: `t sync` first
- All your tasks are completed (we skip old completed tasks)

### Both syncs failing
- Check internet connection
- Verify API credentials in `config.json`
- Check API token scopes/permissions

### One platform works, other doesn't
- Check enabled flag: `"enabled": true`
- Verify credentials for that platform
- See platform-specific troubleshooting docs

## Performance

### How Fast?
- **Google Tasks sync:** ~1-2 seconds
- **Airtable push:** ~0.5s per task
- **ClickUp push:** ~0.5s per task

For 50 tasks: ~30 seconds total

### Rate Limits
- **Airtable:** 5 requests/second
- **ClickUp:** 100 requests/minute

You're unlikely to hit these with normal usage.

### Optimization
- We skip completed tasks >7 days old
- Incremental updates (only changed tasks)
- Cached ID mappings (no duplicate creates)

## Future Integrations

Want to add more platforms? Open an issue or PR!

Candidates:
- **Notion** - Knowledge base + tasks
- **Linear** - Issue tracking for dev teams
- **Asana** - Enterprise PM
- **Trello** - Simple kanban boards
- **Monday.com** - Visual project tracking
- **Todoist** - Personal task management

## Support

- **Issues:** GitHub Issues
- **Questions:** @bryanwhiting
- **Docs:** This repo's markdown files

## License

MIT - use however you want!
