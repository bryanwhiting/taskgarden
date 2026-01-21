# 🌱 Interactive Triage - How It Works

## Run it

```bash
t triage
```

Or:
```bash
thegarden triage
```

## What happens

### Step 1: Task displayed
```
──── Task 1/51 ────
switch Figma paid accounts
(from: Silvermine AI)
```

### Step 2: Pick priority (arrow keys!)
```
? Priority? ›
❯ P0 (Urgent + Important)
  P1 (Important, not urgent)
  P2 (Urgent, not important)
  P3 (Not important, not urgent)
  Skip for now
```

Use **↑↓ arrows** to move, **Enter** to select!

### Step 3: Pick project (optional)
```
? Project? ›
❯ WORKDAY
  LIFE
  SILVERMINE
  Skip (no project)
```

### Step 4: Formatted!
```
✓ [2026-01-21][P0][SILVERMINE] switch Figma paid accounts
```

### Step 5: Repeat for all 51 tasks

Go through each one, arrow keys to pick, Enter to confirm.

### Step 6: Save
```
? Save changes to Google Tasks? (Y/n) ›
```

## Features

- ✅ **Fully interactive** - no typing, just arrow keys
- ✅ **Skip anytime** - ESC or Ctrl+C to bail
- ✅ **Progress counter** - See "Task 12/51"
- ✅ **Optional fields** - Can skip project if you want
- ✅ **Colored output** - Easy to read
- ✅ **Fast** - Compiled Rust, instant

## Keyboard shortcuts

- **↑↓** - Navigate options
- **Enter** - Select
- **ESC / Ctrl+C** - Cancel/exit
- **Space** - (on confirm prompts) toggle yes/no

## Tips

- Do a few at a time, don't burn out
- Start with obvious P0s
- When in doubt, mark P1 (can adjust later)
- Projects are optional - use for context switching

Enjoy! 🌱🐚
