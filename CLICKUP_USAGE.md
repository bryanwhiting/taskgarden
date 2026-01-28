# ClickUp Sync - Quick Start

## What This Does

**Flow:** Google Tasks (capture) → TaskGarden (triage) → ClickUp (team collaboration)

- Capture tasks in Google Tasks (via gog CLI or mobile app)
- Triage them in TaskGarden CLI (prioritize, categorize, estimate time)
- **Push to ClickUp** for your team to collaborate

This is a **one-way sync**: Google Tasks → ClickUp

## Setup

### 1. Get ClickUp Credentials

Follow [CLICKUP_SETUP.md](./CLICKUP_SETUP.md) to:
- Get your API token
- Find your list ID
- Configure your ClickUp list statuses

### 2. Update config.json

```json
{
  "clickup": {
    "enabled": true,
    "api_token": "pk_your_api_token_here",
    "list_id": "901234567"
  }
}
```

### 3. Build

```bash
cd ~/clawd/taskgarden
cargo build --release
```

## Usage

### Normal workflow (no ClickUp sync)

```bash
# Capture in Google Tasks (mobile or web)

# Triage in TaskGarden
t triage

# View your tasks
t list
t focus

# This syncs with Google Tasks but NOT ClickUp yet
t sync
```

### Push to ClickUp

```bash
# Sync with Google Tasks AND push to ClickUp
t sync --clickup

# Or separately:
t sync           # sync with Google Tasks
t sync --clickup # push to ClickUp
```

## What Gets Pushed

Every task in your local SQLite cache gets pushed to ClickUp with these fields:

| ClickUp Field | Source |
|---------------|--------|
| Name | Task title (without metadata) |
| Description | Full details + links + created date |
| Status | TO DO, IN PROGRESS, REVIEW, BLOCKED, COMPLETE |
| Priority | 🔴 Urgent (P0), ⚠️ High (P1), 📘 Normal (P2), 📗 Low (P3/P5) |
| Due Date | From your `[date]` field |
| Time Estimate | From `{time}` field (converted to hours) |
| Tags | Hashtags (#DeepWork), Project (SILVERMINE), Context (@work) |

## Team Benefits

Once pushed to ClickUp, your team can:
- ✅ **Assign tasks** - delegate to team members
- ✅ **Add subtasks** - break down large tasks
- ✅ **Track time** - see actual vs estimated time
- ✅ **Link dependencies** - connect related tasks
- ✅ **Comment** - discuss tasks inline
- ✅ **Multiple views** - List, Board, Calendar, Gantt, Timeline
- ✅ **Notifications** - get pinged on changes
- ✅ **Mobile app** - iOS/Android access

## Example Workflow

```bash
# Monday morning
t sync              # pull latest from Google Tasks
t triage            # prioritize, categorize, estimate
t sync --clickup    # push to ClickUp for team

# During the week
t add "Review PR" --priority P1 --project SILVERMINE
t sync --clickup    # push new task to ClickUp

# End of day
t list --status done --days 1   # see what you finished today
t sync --clickup                # update ClickUp with progress
```

## How Sync Works

1. **Fetch** from Google Tasks → SQLite cache
2. **Parse** taskgarden format: `[date][priority][project][status][@context]{time} title #tags`
3. **Convert** to ClickUp fields
4. **Create or Update** in ClickUp
   - If task exists in ClickUp (based on stored mapping), it updates
   - If new task, it creates
5. **Store mapping** of Google Task ID → ClickUp Task ID

## Conflict Resolution

Since this is **one-way sync** (Google Tasks → ClickUp):
- Changes in ClickUp **won't sync back** to Google Tasks
- If you edit a task in both places, TaskGarden wins (overwrites ClickUp)
- **Recommendation:** Use TaskGarden for personal workflow, ClickUp for team collaboration

## ClickUp vs Airtable

| Feature | ClickUp | Airtable |
|---------|---------|----------|
| **Setup** | API token + list ID | API key + base ID |
| **Subtasks** | ✅ Native support | ❌ Manual workaround |
| **Time tracking** | ✅ Built-in | ❌ Via extensions |
| **Dependencies** | ✅ Yes | ❌ Manual links |
| **Views** | List, Board, Calendar, Gantt, Timeline | Grid, Kanban, Calendar, Gallery |
| **Best for** | Full project management | Flexible databases |
| **Use both?** | ✅ Yes! `t sync --airtable --clickup` | ✅ Yes! |

## Sync to Both

You can push to both Airtable and ClickUp:

```bash
# Push to both platforms
t sync --airtable --clickup

# Create an alias
alias tsync='t sync --airtable --clickup'
```

**Why use both?**
- **Airtable** - Your personal dashboard with custom views
- **ClickUp** - Team collaboration with full PM features

## Troubleshooting

### "ClickUp not configured"
- Check `config.json` has `clickup` section
- Make sure `enabled: true`

### "ClickUp API error 401"
- Invalid API token
- Regenerate at Settings → Apps in ClickUp

### "ClickUp API error 404"
- Wrong list_id
- Double-check the URL: `/li/YOUR_LIST_ID`

### Tasks not showing up
- Make sure you've run `t sync` first (to fetch from Google)
- Check that tasks aren't completed (we skip old completed tasks)
- Verify list_id is correct

### Status mapping issues
- Your ClickUp list might have different status names
- Go to List Settings → Statuses
- Add: TO DO, IN PROGRESS, REVIEW, BLOCKED, COMPLETE

### Need help?
Open an issue or ping @bryanwhiting
