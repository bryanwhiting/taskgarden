# Airtable Integration Setup

## 1. Create Airtable Base

Go to [airtable.com](https://airtable.com) and create a new base with a table called **Tasks** (or whatever you prefer).

### Required Fields

Create these fields in your Airtable table:

| Field Name | Type | Description |
|------------|------|-------------|
| Title | Single line text | Task title |
| Priority | Single select | P0, P1, P2, P3, P5 |
| Project | Single select | WORKDAY, LIFE, SILVERMINE (match config.json) |
| Status | Single select | needsTriage, progress, review, blocked, done |
| Context | Single line text | @work, @home, @anywhere, etc. |
| TimeEstimate | Single line text | 30m, 1h, 2h, etc. |
| DueDate | Date | When task is due |
| CreatedDate | Date | When task was created |
| Assignee | Single line text | Team member name/email |
| Tags | Single line text | Comma-separated tags |
| Notes | Long text | Additional notes |
| Completed | Checkbox | Whether task is done |

### Single Select Options

**Priority:**
- P0 (Urgent + Important)
- P1 (Important, not urgent)
- P2 (Urgent, not important)
- P3 (Not important, not urgent)
- P5 (Delegate)

**Project:** (match your `config.json` projects)
- WORKDAY
- LIFE
- SILVERMINE

**Status:**
- needsTriage
- progress
- review
- blocked
- done

**Context:**
- @work
- @home
- @anywhere
- @errands
- @calls

## 2. Get API Key

1. Go to https://airtable.com/create/tokens
2. Click "Create new token"
3. Name it "TaskGarden"
4. Add these scopes:
   - `data.records:read`
   - `data.records:write`
   - `schema.bases:read`
5. Add access to your base
6. Click "Create token"
7. **Copy the token** (you won't see it again!)

## 3. Get Base ID

1. Go to your Airtable base
2. Click "Help" in top right
3. Click "API documentation"
4. Your base ID is in the URL or shown in the docs (starts with `app...`)

## 4. Update config.json

```json
{
  "airtable": {
    "enabled": true,
    "api_key": "pat...your_key_here",
    "base_id": "appXXXXXXXXXXXXXX",
    "table_name": "Tasks"
  }
}
```

## 5. Add Sync Command

You'll need to add a sync function that:
1. Fetches all tasks from Airtable
2. Converts them to taskgarden format
3. Syncs with local SQLite cache
4. Pushes local changes back to Airtable

### Example Usage (once implemented)

```bash
# Sync with Airtable
t sync --airtable

# Sync with both Google Tasks and Airtable
t sync --all

# Push a new task to Airtable
t add "Fix bug in editor" --project SILVERMINE --priority P0
```

## Team Visibility Benefits

With Airtable, your team can:
- ✅ See all tasks in real-time
- ✅ Filter by person, priority, status, project
- ✅ Use kanban/calendar/timeline views
- ✅ Comment and collaborate on tasks
- ✅ Get notifications when tasks change
- ✅ Mobile access via Airtable app

## Next Steps

1. Create the sync integration in `src/airtable.rs` ✅ (Done!)
2. Add Airtable sync to commands in `main.rs`
3. Implement bidirectional sync logic
4. Add conflict resolution (last-write-wins or prompt user)
5. Test with team!

## Architecture

```
┌─────────────┐
│  TaskGarden │
│     CLI     │
└──────┬──────┘
       │
       ├──────────┐
       │          │
┌──────▼─────┐ ┌─▼─────────┐
│   SQLite   │ │ Airtable  │
│   Cache    │ │    API    │
└──────┬─────┘ └─┬─────────┘
       │         │
┌──────▼─────┐ ┌─▼─────────┐
│   Google   │ │  Airtable │
│   Tasks    │ │   Base    │
└────────────┘ └───────────┘
     ▲              ▲
     │              │
   ┌─┴──┐      ┌────┴────┐
   │You │      │  Team   │
   └────┘      └─────────┘
```

TaskGarden becomes the sync hub between Google Tasks (your personal view) and Airtable (team visibility).
