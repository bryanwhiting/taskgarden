# 🌱 TaskGarden

**ADHD-friendly task triage CLI** that integrates with Google Tasks.

Built with Rust for speed, designed for humans who get overwhelmed by todo lists.

## What It Does

- **Smart Triage** - Three-pass system: prioritize, categorize, estimate time
- **Instant Save** - Every change saves immediately (never lose progress)
- **Fast Sync** - Local SQLite cache with smart sync throttling
- **Task Types** - Use hashtags: `#FollowUp`, `#Plan`, `#DeepWork`
- **Calendar Integration** - See your week with meetings + tasks
- **Focus Mode** - Show only critical tasks (hide the noise)

## Quick Start

```bash
# Clone
git clone https://github.com/bryanwhiting/taskgarden.git
cd taskgarden

# Build
cargo build --release

# Add to PATH
export PATH="$HOME/clawd/taskgarden/target/release:$PATH"
alias t="taskgarden"

# Configure Google account
# Edit config.json with your account
```

## Usage

```bash
# Interactive triage (prioritize, categorize, estimate)
t triage

# Focus mode (show only P0s + overdue P1s)
t focus

# Weekly plan (tasks + calendar events)
t plan

# Smart schedule (block time for tasks)
t schedule

# List tasks
t list
t list --grouped  # Group by date

# Manual sync
t sync
t sync --force    # Full re-sync
```

## Features

### Priority Matrix
- **P0** - Urgent + Important (do now)
- **P1** - Important, not urgent (schedule it)
- **P2** - Urgent, not important (delegate or quick-hit)
- **P3** - Not important, not urgent (maybe someday)
- **P5** - Delegate

### Task Types
- `#FollowUp` - Auto-assigns P1, 30m
- `#Plan` - Auto-assigns P1, 1h
- `#DeepWork` - Auto-assigns P0, 2h

### Format
Tasks are formatted as:
```
[creation-date][priority][PROJECT]{time} Task title #hashtag
```

Example:
```
[2026-01-21][P0][SILVERMINE]{2h} Fix editor bug #DeepWork
```

## Why TaskGarden?

**Built for ADHD:**
- ✅ One decision type at a time (no context switching)
- ✅ Instant saves (no fear of losing work)
- ✅ Smart filtering (only see what needs triage)
- ✅ Single keystroke input (no typing fatigue)
- ✅ Undo functionality (mistakes are safe)

**Built for speed:**
- ✅ Local SQLite cache
- ✅ Smart sync throttling (10 min default)
- ✅ Incremental Google Tasks sync
- ✅ Sub-second response time

**Built for reality:**
- ✅ Integrates with Google Tasks (works everywhere)
- ✅ Calendar-aware (see your meetings)
- ✅ Time-blocking (schedule your work)
- ✅ Delegate option (you can't do everything)

## Configuration

Edit `config.json`:

```json
{
  "format": "[{date}][{priority}][{project}]{time} {title}",
  "date_format": "%Y-%m-%d",
  "sync_throttle_minutes": 10,
  "projects": {
    "WORK": "Day job",
    "LIFE": "Personal tasks",
    "BUSINESS": "Side business"
  },
  "task_types": {
    "FollowUp": {
      "default_priority": "P1",
      "default_time": "30m"
    }
  },
  "google_account": "you@gmail.com"
}
```

## Requirements

- Rust 1.70+
- [gog CLI](https://github.com/your-gog-link) for Google Tasks integration
- Google account with Tasks API enabled

## License

MIT

## Author

Built with ❤️ and ADHD by [@bryanwhiting](https://github.com/bryanwhiting)

---

*TaskGarden: Because your todos shouldn't make you feel like you're drowning in weeds.* 🌱
