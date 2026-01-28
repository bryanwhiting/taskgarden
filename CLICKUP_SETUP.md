# ClickUp Integration Setup

## 1. Get ClickUp API Token

1. Log in to [ClickUp](https://app.clickup.com)
2. Click your avatar (bottom left)
3. Go to **Settings** → **Apps**
4. Click **+ Generate** under "API Token"
5. **Copy the token** (starts with `pk_...`)

## 2. Get List ID

### Option A: Via URL (easiest)

1. Go to your ClickUp workspace
2. Navigate to the **List** where you want tasks to appear
3. Look at the URL: `https://app.clickup.com/12345678/v/li/901234567`
4. The number after `/li/` is your **List ID**: `901234567`

### Option B: Via API Explorer

1. Go to https://clickup.com/api
2. Use the API Explorer to find your workspace/space/folder/list hierarchy
3. Or use this curl command:

```bash
curl -H "Authorization: YOUR_API_TOKEN" \
  https://api.clickup.com/api/v2/team
```

Then navigate down:
- Teams (workspaces) → Spaces → Folders → Lists

## 3. Configure ClickUp List (Recommended)

For best results, set up your ClickUp list with these **statuses**:

1. Go to your List in ClickUp
2. Click the **⚙️** (settings) icon
3. Manage **Statuses**:
   - `TO DO` (default)
   - `IN PROGRESS`
   - `REVIEW`
   - `BLOCKED`
   - `COMPLETE` (default)

These map to taskgarden statuses:
- `needsTriage` → TO DO
- `progress` → IN PROGRESS
- `review` → REVIEW
- `blocked` → BLOCKED
- `done` / completed → COMPLETE

### Priority Mapping

ClickUp priorities map automatically:
- **P0** → 🔴 Urgent (1)
- **P1** → ⚠️ High (2)
- **P2** → 📘 Normal (3)
- **P3** → 📗 Low (4)
- **P5** → 📗 Low (4)

### Custom Fields (Optional)

You can add these custom fields to your list:
- **Project** (Dropdown): WORKDAY, LIFE, SILVERMINE
- **Context** (Dropdown): @work, @home, @anywhere

## 4. Update config.json

```json
{
  "clickup": {
    "enabled": true,
    "api_token": "pk_your_api_token_here",
    "list_id": "901234567"
  }
}
```

## 5. Test the Connection

```bash
cd ~/clawd/taskgarden
cargo build --release

# Test sync
t sync --clickup
```

## What Gets Synced

### Task Fields

| ClickUp Field | Source | Notes |
|---------------|--------|-------|
| **Name** | Task title | Clean title without metadata |
| **Description** | Full details | Includes links, created date |
| **Status** | Task status | needsTriage, progress, review, blocked, done |
| **Priority** | P0-P5 | Maps to ClickUp 1-4 scale |
| **Due Date** | `[date]` field | Parsed from taskgarden format |
| **Time Estimate** | `{time}` field | Converted to milliseconds |
| **Tags** | Hashtags + project | #DeepWork, SILVERMINE, @work |

### Example Conversion

**TaskGarden:**
```
[2026-01-27][P0][SILVERMINE][progress][@work]{2h} Fix editor bug #DeepWork
```

**ClickUp:**
- Name: "Fix editor bug #DeepWork"
- Status: IN PROGRESS
- Priority: 🔴 Urgent
- Due Date: Jan 27, 2026
- Time Estimate: 2h
- Tags: DeepWork, SILVERMINE, @work

## Team Visibility Benefits

Once pushed to ClickUp, your team can:
- ✅ **Assignees** - assign tasks to team members
- ✅ **Comments** - discuss tasks inline
- ✅ **Subtasks** - break down large tasks
- ✅ **Dependencies** - link related tasks
- ✅ **Time tracking** - track actual time spent
- ✅ **Views** - List, Board, Calendar, Gantt, Timeline
- ✅ **Notifications** - get pinged when tasks change
- ✅ **Mobile app** - iOS/Android access

## Usage

### Daily Workflow

```bash
# Morning
t sync              # Pull from Google Tasks
t triage            # Prioritize, categorize, estimate
t sync --clickup    # Push to ClickUp for team

# Add new task
t add "Review PR" --priority P1 --project SILVERMINE
t sync --clickup    # Push to team

# End of day
t sync --clickup    # Update ClickUp with progress
```

### Sync Both Airtable and ClickUp

```bash
# Push to both platforms
t sync --airtable --clickup

# Or create an alias
alias tsync='t sync --airtable --clickup'
```

## Troubleshooting

### "ClickUp API error 401"
- Invalid API token
- Regenerate at Settings → Apps

### "ClickUp API error 404"
- Wrong list_id
- Double-check the URL or use the API to find it

### "ClickUp API error 403"
- API token doesn't have access to that list
- Check workspace permissions

### Tasks not appearing
- Make sure you've run `t sync` first (to fetch from Google)
- Check that the list_id is correct
- Verify the list isn't archived

### Status mapping issues
- ClickUp list might have different status names
- Edit your list statuses to match (TO DO, IN PROGRESS, etc.)
- Or customize the mapping in `src/clickup_sync.rs`

## Advanced: Bidirectional Sync

Currently this is **one-way** (Google Tasks → ClickUp).

For two-way sync:
1. Fetch tasks from ClickUp
2. Compare timestamps
3. Merge changes with conflict resolution
4. Push back to Google Tasks

This is complex - for now, treat ClickUp as **read-only** for team visibility.

## API Rate Limits

ClickUp has generous rate limits:
- **100 requests/minute** per API token
- Most syncs use 1-2 requests per task
- You're fine for <50 tasks

If you hit limits, add throttling or batch updates.

## Resources

- [ClickUp API Docs](https://clickup.com/api)
- [API Explorer](https://clickup.com/api)
- [Status Codes](https://clickup.com/api#section/Status-Codes)

## Support

Questions? Open an issue or ping @bryanwhiting
