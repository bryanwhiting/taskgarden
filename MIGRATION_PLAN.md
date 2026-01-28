# Migration Plan: Bracket Format → Emoji Format

## Current State (v1)

**Format:**
```
[2026-01-27][P0][SILVERMINE][progress][@work]{2h} Fix bug #DeepWork
```

**Problems:**
- Hard to read at a glance
- Brackets get confused with Markdown
- No standard for optional fields
- Doesn't work well in Slack/emails
- Platform-specific (designed for Google Tasks sync)

## Target State (v2)

**Format:**
```
Fix bug ⏰ 2h 📅 2026-01-27 🔺 /silvermine @work #DeepWork
```

**Benefits:**
- Emoji = universal, readable
- Order-independent properties
- Works in any text field
- Copy-paste friendly
- Platform-agnostic

## Migration Strategy

### Phase 1: Dual Format Support (Week 1)

**Goal:** Support BOTH formats during transition

```rust
// Parse function recognizes both:
"[2026-01-27][P0]..." → old format
"Task ⏰ 1h 📅..." → new format

// Output: Always write new format
```

**Changes:**
1. Update parser to detect format type
2. Add emoji-based parser
3. Keep bracket parser for reading
4. New tasks always use emoji format

**User Impact:** ✅ None - existing tasks still work

### Phase 2: Conversion Tool (Week 1)

**Goal:** Convert existing tasks to new format

```bash
# Convert all tasks in SQLite cache
t migrate --to-emoji --dry-run    # Preview changes
t migrate --to-emoji              # Apply conversion

# Convert Google Tasks in-place
t migrate --to-emoji --sync-google
```

**Implementation:**
```rust
fn convert_task(old: &str) -> String {
    // [2026-01-27][P0][SILVERMINE][@work]{2h} Title #tag
    // → Title ⏰ 2h 📅 2026-01-27 🔺 /silvermine @work #tag
    
    let parsed = parse_bracket_format(old);
    format_emoji_style(&parsed)
}
```

**User Impact:** ⚠️ One-time command to run

### Phase 3: Deprecate Bracket Format (Week 2)

**Goal:** Remove bracket format support

1. Announce deprecation
2. Ensure all tasks migrated
3. Remove bracket parser code
4. Update docs

**User Impact:** ✅ None if migrated

## Conversion Examples

### Simple Task
```
Before: [2026-01-28][--][---] Call mom
After:  Call mom 📅 2026-01-28
```

### Full-Featured Task
```
Before: [2026-01-30][P0][SILVERMINE][progress][@work]{2h} Review docs #DeepWork
After:  Review docs ⏰ 2h 📅 2026-01-30 🔺 /silvermine @work #DeepWork
```

### Task with No Metadata
```
Before: [--][--][---] Buy milk
After:  Buy milk
```

## Database Migration

### SQLite Cache

**Current Schema:**
```sql
CREATE TABLE tasks (
  id TEXT PRIMARY KEY,
  title TEXT,  -- "[date][P0]..." format
  status TEXT,
  ...
);
```

**No Schema Change Needed:**
- `title` field just changes format
- Old format: `[2026-01-27][P0]...`
- New format: `Task ⏰ 1h...`

**Migration SQL:**
```sql
-- Convert all titles to emoji format
UPDATE tasks 
SET title = convert_to_emoji(title)
WHERE title LIKE '[%';
```

### Google Tasks

**Before Sync:**
```
Title: "[2026-01-27][P0][SILVERMINE][@work]{2h} Fix bug"
```

**After Sync:**
```
Title: "🔃 Fix bug ⏰ 2h 📅 2026-01-27 🔺 /silvermine @work"
```

**Process:**
1. Fetch tasks from Google
2. Parse bracket format
3. Push back in emoji format
4. Mark with 🔃

### ClickUp

**Before:**
```
Name: "Fix bug"
Description: "[2026-01-27][P0][SILVERMINE][@work]{2h} Fix bug"
```

**After:**
```
Name: "Fix bug"
Time Estimate: 7200000ms
Due Date: 2026-01-27
Priority: Urgent (1)
Tags: ["silvermine", "work"]
Description: "⏰ 2h 📅 2026-01-27 🔺 /silvermine @work

Original task string for reference and sync."
```

## Breaking Changes

### Config File

**Old:**
```json
{
  "format": "[{date}][{priority}][{project}]{status}{context}{time} {title}",
  "date_format": "%Y-%m-%d"
}
```

**New:**
```json
{
  "format": "emoji",  // or "bracket" for legacy
  "date_format": "%Y-%m-%d",  // Still used for parsing
  "emoji_config": {
    "use_start_date": true,
    "use_scheduled_date": true,
    "auto_add_created": true
  }
}
```

### CLI Commands

**Old:**
```bash
t add "Task" --priority P0 --project SILVERMINE --time 2h
# Creates: [2026-01-27][P0][SILVERMINE]{2h} Task
```

**New:**
```bash
t add "Task" --priority P0 --project SILVERMINE --time 2h
# Creates: Task ⏰ 2h 📅 2026-01-27 🔺 /silvermine

# Or natural input:
t add "Task ⏰ 2h 🔺 /silvermine"
# Parses emoji format directly
```

## Testing Plan

### Unit Tests

```rust
#[test]
fn test_parse_emoji_format() {
    let input = "Task ⏰ 1h 📅 2026-01-28 🔺 @work";
    let parsed = parse_task(input);
    
    assert_eq!(parsed.title, "Task");
    assert_eq!(parsed.duration, Some("1h"));
    assert_eq!(parsed.due_date, Some("2026-01-28"));
    assert_eq!(parsed.priority, Some("P0"));
    assert_eq!(parsed.context, Some("@work"));
}

#[test]
fn test_convert_bracket_to_emoji() {
    let input = "[2026-01-28][P0][SILVERMINE]{1h} Task @work";
    let output = convert_to_emoji(input);
    
    assert_eq!(output, "Task ⏰ 1h 📅 2026-01-28 🔺 /silvermine @work");
}

#[test]
fn test_order_independence() {
    let input1 = "Task ⏰ 1h 📅 2026-01-28 🔺";
    let input2 = "Task 📅 2026-01-28 ⏰ 1h 🔺";
    let input3 = "Task 🔺 ⏰ 1h 📅 2026-01-28";
    
    let p1 = parse_task(input1);
    let p2 = parse_task(input2);
    let p3 = parse_task(input3);
    
    assert_eq!(p1, p2);
    assert_eq!(p2, p3);
}
```

### Integration Tests

```bash
# Test conversion
t migrate --to-emoji --dry-run
# Should show preview of all changes

# Test sync maintains format
t add "Test task ⏰ 1h 🔺"
t sync --clickup
# Verify in ClickUp: time=1h, priority=Urgent

# Test round-trip
# Create in ClickUp → Pull to Google → Verify emoji format
```

## Rollout Timeline

### Week 1: Development + Testing
- **Day 1-2:** Implement emoji parser
- **Day 3-4:** Implement converter
- **Day 5:** Add migration command
- **Day 6-7:** Testing + docs

### Week 2: User Migration
- **Day 1:** Release with dual-format support
- **Day 2-3:** Users test, report issues
- **Day 4:** Run migration command
- **Day 5-7:** Verify, fix edge cases

### Week 3: Cleanup
- **Day 1-2:** Remove bracket parser
- **Day 3-4:** Update all docs
- **Day 5:** Final release

## Communication Plan

### Announcement (Week 1)

**Subject:** 🎉 TaskGarden v2: New Emoji-Based Task Format

**Message:**
```
We're introducing a new, more readable task format inspired by Obsidian Tasks!

OLD: [2026-01-28][P0][PROJECT]{2h} Task name @context
NEW: Task name ⏰ 2h 📅 2026-01-28 🔺 /project @context

Benefits:
✅ More readable
✅ Copy-paste friendly  
✅ Order-independent
✅ Platform-agnostic

Migration:
1. Update: `git pull && cargo build --release`
2. Preview: `t migrate --to-emoji --dry-run`
3. Convert: `t migrate --to-emoji`

Old format will be supported for 2 weeks, then deprecated.
Questions? Open an issue!
```

### Migration Guide (Week 1)

Create `MIGRATION_GUIDE.md` with:
- Format comparison
- Step-by-step instructions
- Troubleshooting
- FAQ

### Deprecation Notice (Week 2)

Update README with deprecation warning:
```
⚠️ DEPRECATION: Bracket format [date][P0]... will be removed in v2.1
Please migrate using: t migrate --to-emoji
```

## Risk Mitigation

### Backup Before Migration

```bash
# Automatic backup before conversion
t migrate --to-emoji
# Creates: ~/.thegarden/backup-2026-01-28.db
```

### Rollback Plan

```bash
# If something goes wrong
t migrate --rollback
# Restores from latest backup
```

### Gradual Migration Option

```bash
# Migrate one list at a time
t migrate --to-emoji --list "Silvermine AI"
t migrate --to-emoji --list "Workday"
# etc.
```

## Success Metrics

- ✅ All tasks converted without data loss
- ✅ Sync still works (Google ↔ ClickUp)
- ✅ No user complaints about readability
- ✅ Emoji format parsing is < 5ms
- ✅ 0 regressions in existing features

## Future Enhancements (Post-Migration)

Once emoji format is stable:

1. **Natural Language Parser**
   ```bash
   t add "Review docs tomorrow at 2pm for 1h high priority"
   # → Review docs ⏰ 1h 📅 tomorrow ⏳ 2pm ⏫
   ```

2. **Smart Emoji Suggestions**
   ```bash
   t add "Task"
   # Interactive prompt:
   # Duration? (⏰) [1h]: 
   # Due date? (📅) [tomorrow]:
   # Priority? (🔺⏫🔼🔽⏬) [medium]:
   ```

3. **Template Support**
   ```bash
   t template save "client-followup" "Follow up with {client} ⏰ 30m ⏫ /silvermine @phone #FollowUp"
   t add --template client-followup --client "Acme Corp"
   # → Follow up with Acme Corp ⏰ 30m ⏫ /silvermine @phone #FollowUp
   ```

4. **Batch Operations**
   ```bash
   t batch "Fix all bugs with 📅 < today to tomorrow"
   # Updates due dates for all overdue bug tasks
   ```
