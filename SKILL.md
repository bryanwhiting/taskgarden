---
name: thegarden
description: "ADHD-friendly task triage CLI. Integrates with Google Tasks using priority matrix (P0-P3) and project tags. Format: [date][priority][PROJECT] title."
---

# 🌱 The Garden - Task Management Skill

ADHD-optimized task triage system that works with Google Tasks.

## Task Format

Tasks are prefixed with: `[creationdate][priority][PROJECT] title`

Example: `[2026-01-21][P0][SILVERMINE] Fix editor jumping bug`

### Priority Matrix
- **P0** = Urgent + Important (do today, drop everything)
- **P1** = Important, not urgent (schedule it, protect time)
- **P2** = Urgent, not important (delegate or quick-hit)
- **P3** = Not important, not urgent (maybe someday, probably never)

### Projects
- **WORKDAY** - Day job tasks
- **LIFE** - Personal life tasks (family, health, home, finances)
- **SILVERMINE** - Silvermine AI business tasks

## Commands

### Interactive Triage (Primary Workflow)

```bash
thegarden triage
```

Shows all unprioritized tasks from Google Tasks. User responds via Telegram with priorities:

```
1)p0, 2)p1, 3)p3, 4)p0, 5)p2
```

Sand (the assistant) then updates Google Tasks with formatted prefixes.

### List Today's Tasks

```bash
thegarden list
```

Shows only tasks with today's date, sorted P0 → P3.

```bash
thegarden list --all
```

Shows all tasks across all dates, sorted by priority.

### Add New Task

```bash
thegarden add "Task description" --priority P0 --project SILVERMINE
# Short flags:
thegarden add "Task description" -p P0 -j SILVERMINE
```

### Bump Incomplete Tasks

```bash
thegarden bump
```

Moves incomplete tasks from yesterday to today, downgrades priority by one level (P0 → P1, P1 → P2, etc.).

### Sync with Google Tasks

```bash
thegarden sync
```

Pulls from Google Tasks and updates local cache.

## Configuration

Edit `~/clawd/thegarden/config.json`:

```json
{
  "format": "[{date}][{priority}][{project}] {title}",
  "date_format": "%Y-%m-%d",
  "projects": {
    "WORKDAY": "Workday job",
    "LIFE": "Personal life tasks",
    "SILVERMINE": "Silvermine AI business"
  },
  "google_account": "bryan@silvermineai.com"
}
```

### Format Variables
- `{date}` - Creation date (YYYY-MM-DD by default)
- `{priority}` - P0, P1, P2, P3
- `{project}` - WORKDAY, LIFE, SILVERMINE, etc.
- `{title}` - Task description

Change the format string to customize task prefixes. The system will parse both old and new formats.

## Integration with Google Tasks

Uses the `gog` CLI (Google Tasks tool) to:
- Read all task lists
- Parse existing formatted tasks
- Identify unprioritized tasks
- Update task titles with new prefixes

**Google account:** Configured in `config.json` as `google_account`.

## Workflow

**Morning Triage (9am):**
1. Run `thegarden triage`
2. Review unprioritized tasks
3. Respond with priorities via Telegram
4. Sand updates Google Tasks automatically

**During Day:**
1. Run `thegarden list` to see today's P0s
2. Focus on P0 → P1 → ignore the rest
3. Quick-hit P2s between meetings

**Evening (9pm):**
1. Run `thegarden bump` to move incomplete tasks
2. Reflect on what got done vs. planned (automated via Sand's nightly check-in)

## Installation

Requires:
- Rust toolchain (`cargo`)
- `gog` CLI (Google Tasks integration)
- Clawdbot with Telegram integration (for interactive triage)

```bash
cd ~/clawd/thegarden
cargo build --release

# Add to PATH
echo 'export PATH="$HOME/clawd/thegarden/target/release:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Binary Location

`~/clawd/thegarden/target/release/thegarden`

## Dependencies

- **gog** - Google Tasks CLI (install via `npm install -g @gog/cli` or similar)
- Google account with Tasks enabled
- Clawdbot with Telegram (for interactive triage responses)

## Design Philosophy

Built for ADHD:
- **Externalize everything** - Get it out of your head into Google Tasks
- **Daily reset** - Priorities can shift every day, embrace it
- **Visual focus** - Only see today's tasks, sorted by urgency
- **Interactive** - You + AI work together to triage, not alone
- **Dopamine hits** - Quick-win P2s for between-task motivation
- **Forgiveness** - Bump unprioritized tasks instead of guilt

## Notes

- Tasks without prefixes are considered unprioritized
- Creation date is set when task is first formatted (not Google Tasks creation date)
- Priority can be changed by re-running triage
- Project tags help with context switching (Workday mode vs. Life mode vs. Silvermine mode)
- Format is configurable - adjust variables in `config.json` to fit your system
