# 🌱 Attachment Detection

## What It Does

When you run `t triage`, tasks with attachments/links will show indicators:

```
Task 5/51
VERAFAST- New payment request (email)
(from: Silvermine AI)
```

## Attachment Types Detected

- **(email)** - Gmail message linked to task
- **(doc)** - Google Doc linked to task  
- **(drive)** - Google Drive file linked to task

## How It Works

The Garden reads the `links` metadata from Google Tasks:
- Checks if `type: "email"` → **(email)**
- Checks if URL contains `docs.google.com` → **(doc)**
- Checks if URL contains `drive.google.com` → **(drive)**
- Checks if URL contains `mail.google.com` → **(email)**

## Why This Matters

**Context at a glance:**
- See if a task came from email (probably external request)
- See if there's a doc (probably needs writing/collaboration)
- See if there's a Drive file (probably needs review)

**Better prioritization:**
- Email tasks might be more urgent (someone waiting)
- Doc tasks might need collaboration time
- Drive tasks might be reference material

## Example Triage Flow

```
═══ Pass 1: Priorities ═══

Task 3/51
Client feedback on website (email)
(from: Silvermine AI)

? Priority? ›
❯ P0 (Urgent + Important)    ← Someone's waiting!
```

vs.

```
Task 12/51
Research competitor analysis (doc)
(from: Silvermine AI)

? Priority? ›
❯ P1 (Important, not urgent)  ← Can schedule this
```

## Notes

- Indicator appears in **all three passes** (priority, project, time)
- Shows during **triage** - not in formatted output
- Helps you make faster decisions without opening links
- Only shows first attachment if multiple exist

Enjoy! 🌱🐚
