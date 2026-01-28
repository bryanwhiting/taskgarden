# Airtable Sync - Quick Start

## What This Does

**Flow:** Google Tasks (capture) → TaskGarden (triage) → Airtable (team visibility)

- Capture tasks in Google Tasks (via gog CLI or mobile app)
- Triage them in TaskGarden CLI (prioritize, categorize, estimate time)
- **Push to Airtable** for your team to see

This is a **one-way sync**: Google Tasks → Airtable

## Setup

### 1. Create Airtable Base

Follow [AIRTABLE_SETUP.md](./AIRTABLE_SETUP.md) to:
- Create the base with proper fields
- Get your API key
- Get your base ID

### 2. Update config.json

```json
{
  "airtable": {
    "enabled": true,
    "api_key": "pat...your_api_key_here",
    "base_id": "appXXXXXXXXXXXXXX",
    "table_name": "Tasks"
  }
}
```

### 3. Build

```bash
cd ~/clawd/taskgarden
cargo build --release
```

## Usage

### Normal workflow (no Airtable sync)

```bash
# Capture in Google Tasks (mobile or web)

# Triage in TaskGarden
t triage

# View your tasks
t list
t focus

# This syncs with Google Tasks but NOT Airtable yet
t sync
```

### Push to Airtable

```bash
# Sync with Google Tasks AND push to Airtable
t sync --airtable

# Or separately:
t sync           # sync with Google Tasks
t sync --airtable  # push to Airtable
```

## What Gets Pushed

Every task in your local SQLite cache gets pushed to Airtable with these fields:

| Airtable Field | Source |
|----------------|--------|
| Title | Task title (without metadata) |
| Priority | P0, P1, P2, P3, P5 |
| Project | WORKDAY, LIFE, SILVERMINE |
| Status | needsTriage, progress, review, blocked, done |
| Context | @work, @home, @anywhere |
| TimeEstimate | 30m, 1h, 2h, etc. |
| DueDate | From your `[date]` field |
| CreatedDate | When task was created in Google |
| Tags | Hashtags from title (#DeepWork, etc.) |
| Notes | Any links from Google Tasks |
| Completed | Checkbox (true if task is done) |

## Team Benefits

Once pushed to Airtable, your team can:
- ✅ **Filter by person** - see who's working on what
- ✅ **Filter by date** - what's due this week
- ✅ **Filter by priority** - see all P0s
- ✅ **Kanban view** - group by status
- ✅ **Calendar view** - see tasks by due date
- ✅ **Comment** - discuss tasks
- ✅ **Mobile app** - check tasks on the go

## Example Workflow

```bash
# Monday morning
t sync              # pull latest from Google Tasks
t triage            # prioritize, categorize, estimate
t sync --airtable   # push to Airtable for team

# During the week
t add "Fix bug in editor" --priority P0 --project SILVERMINE
t sync --airtable   # push new task to Airtable

# End of day
t list --status done --days 1   # see what you finished today
t sync --airtable               # update Airtable
```

## How Sync Works

1. **Fetch** from Google Tasks → SQLite cache
2. **Parse** taskgarden format: `[date][priority][project][status][@context]{time} title #tags`
3. **Convert** to Airtable fields
4. **Create or Update** in Airtable
   - If task exists in Airtable (based on stored mapping), it updates
   - If new task, it creates
5. **Store mapping** of Google Task ID → Airtable Record ID

## Conflict Resolution

Since this is **one-way sync** (Google Tasks → Airtable):
- Changes in Airtable **won't sync back** to Google Tasks
- If you edit a task in both places, TaskGarden wins (overwrites Airtable)
- For team collaboration, **treat Airtable as read-only** for now

## Future: Two-Way Sync

If you want changes in Airtable to sync back:
1. Track "last modified" timestamps
2. Implement conflict resolution
3. Add `--bidirectional` flag to sync command

But for now, keep it simple: **capture in Google, triage in TaskGarden, share via Airtable**.

## Troubleshooting

### "Airtable not configured"
- Check `config.json` has `airtable` section
- Make sure `enabled: true`

### "Airtable API error 401"
- Invalid API key
- Regenerate at https://airtable.com/create/tokens

### "Airtable API error 404"
- Wrong base_id or table_name
- Double check your base ID in Airtable API docs

### Tasks not showing up
- Make sure you've run `t sync` first (to fetch from Google)
- Check that tasks aren't completed (we skip old completed tasks)
- Verify field names in Airtable match AIRTABLE_SETUP.md

### Need help?
Open an issue or ping @bryanwhiting
