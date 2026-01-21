# 🌱 Three-Pass Triage Flow

## The ADHD-Friendly Way

One decision type at a time = way easier!

## How It Works

### Run it
```bash
t triage
```

### Pass 1: Priorities ONLY
```
═══ Pass 1: Priorities ═══

Task 1/51
switch Figma paid accounts
(from: Silvermine AI)

? Priority? ›
❯ P0 (Urgent + Important)
  P1 (Important, not urgent)
  P2 (Urgent, not important)
  P3 (Not important, not urgent)
  Skip

  → P3
```

Go through ALL tasks, just picking priorities. Skip if unsure.

### Pass 2: Projects ONLY
```
═══ Pass 2: Projects ═══

Task 1/42
switch Figma paid accounts
[P3]

? Project? ›
❯ WORKDAY
  LIFE
  SILVERMINE
  Skip (no project)

  → SILVERMINE
```

Only tasks you prioritized. Just pick project. Skip if it doesn't fit.

### Pass 3: Time Estimates ONLY
```
═══ Pass 3: Time Estimates ═══

Task 1/42
switch Figma paid accounts
[P3][SILVERMINE]

? Estimated time? ›
  15m
  30m
❯ 1h
  2h
  3h
  4h
  8h
  Skip

  → 1h
```

Only tasks you prioritized. Quick estimate. Skip if unsure.

### Summary
```
═══ Summary ═══

[2026-01-21][P3][SILVERMINE]{1h} switch Figma paid accounts
[2026-01-21][P0][WORKDAY]{30m} Fix editor bug
[2026-01-21][P1][LIFE]{2h} Call insurance
...

? Save changes to Google Tasks? (Y/n) ›
```

Review everything, then save!

## Why This Works

**For ADHD:**
- ✅ One decision type at a time (no context switching)
- ✅ Can skip and come back later
- ✅ Progress tracking (Task 12/51)
- ✅ Visual summary at end
- ✅ Arrow keys only (no typing)

**Cognitive load:**
- Pass 1: "How urgent is this?" → Fast gut check
- Pass 2: "What area of life?" → Easy categorization
- Pass 3: "How long?" → Simple estimation

## Format Output

```
[creation date][priority][PROJECT]{time estimate} task title
```

Example:
```
[2026-01-21][P0][SILVERMINE]{2h} Fix editor jumping bug
```

**Variables in config.json:**
- `{date}` - Creation date (YYYY-MM-DD)
- `{priority}` - P0, P1, P2, P3
- `{project}` - WORKDAY, LIFE, SILVERMINE
- `{time}` - 15m, 30m, 1h, 2h, 3h, 4h, 8h
- `{title}` - Task description

## Tips

- **Pass 1:** When in doubt, mark P1 or P2, not P0
- **Pass 2:** Skip if task spans multiple projects
- **Pass 3:** Round up, not down (better to overestimate)
- **ESC anytime** - Can bail and run again later
- **Do it in chunks** - 10-15 tasks at a time, not all 51 at once

Enjoy the flow! 🌱🐚
