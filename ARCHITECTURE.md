# TaskGarden Architecture

## Core Principle: Local DB as Source of Truth

**Local SQLite DB** → **taskgarden_description** (derived) → **Remote platforms** (synced)

## Data Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    LOCAL SQLITE DATABASE                     │
│                   (Source of Truth)                         │
│                                                              │
│  ┌─────────────┐  ┌──────────────────────────────────────┐ │
│  │  unique_id  │  │     Structured Properties            │ │
│  │  (UUID)     │  │  - title                             │ │
│  └─────────────┘  │  - priority (P0-P5)                  │ │
│                   │  - project (silvermine, workday...)  │ │
│                   │  - context (@work, @home...)         │ │
│                   │  - duration (1h, 30m...)             │ │
│                   │  - due_date                          │ │
│                   │  - start_date                        │ │
│                   │  - scheduled_date                    │ │
│                   │  - tags (DeepWork, FollowUp...)      │ │
│                   │  - user_description                  │ │
│                   └──────────────────────────────────────┘ │
│                                ▼                            │
│                   ┌──────────────────────────────────────┐ │
│                   │  taskgarden_description              │ │
│                   │  (DERIVED/IMMUTABLE)                 │ │
│                   │                                      │ │
│                   │  Generated from properties:          │ │
│                   │  "Task ⏰ 1h 📅 2026-01-28 🔺        │ │
│                   │   /silvermine @work #DeepWork"       │ │
│                   └──────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                                ▼
                    SYNC TO REMOTE PLATFORMS
                                │
         ┌──────────────────────┼──────────────────────┐
         ▼                      ▼                      ▼
    ┌─────────┐          ┌──────────┐          ┌──────────┐
    │  Google │          │  ClickUp │          │ Airtable │
    │  Tasks  │          │          │          │          │
    └─────────┘          └──────────┘          └──────────┘
```

## Database Schema

### tasks table

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PRIMARY KEY | External platform ID (Google Tasks, ClickUp, etc.) |
| `unique_id` | TEXT UNIQUE | TaskGarden UUID (generated once, never changes) |
| `list_id` | TEXT | Which list/project task belongs to |
| `title` | TEXT | Clean task title (no emoji properties) |
| `status` | TEXT | needsAction, completed, cancelled |
| `updated` | TEXT | Last update timestamp |
| `created` | TEXT | Creation timestamp |
| `links` | TEXT | Related URLs/links |
| `last_synced` | TEXT | Last sync with remote |
| `dirty` | INTEGER | 1 if local changes not synced yet |
| **Structured Properties** | | |
| `priority` | TEXT | P0, P1, P2, P3, P5 |
| `project` | TEXT | silvermine, workday, life |
| `context` | TEXT | @work, @home, @phone |
| `duration` | TEXT | 1h, 30m, 2h30m |
| `due_date` | TEXT | 2026-01-28 (ISO date) |
| `start_date` | TEXT | 2026-01-27 (ISO date) |
| `scheduled_date` | TEXT | 2026-01-28T14:00 (ISO datetime) |
| `tags` | TEXT | Comma-separated: DeepWork,FollowUp |
| `user_description` | TEXT | User's notes/description |
| **Derived Field** | | |
| `taskgarden_description` | TEXT | **IMMUTABLE** - Generated from properties |

## Key Concepts

### 1. unique_id

- **Generated once** when task is first created
- **Never changes** even if task moves between platforms
- **UUID format** (e.g., `550e8400-e29b-41d4-a716-446655440000`)
- **Use for**: Dependencies, tracking across platforms

### 2. taskgarden_description

**IMMUTABLE** - Always derived from structured properties, never edited directly.

**Generation Function:**
```rust
impl CachedTask {
    pub fn generate_taskgarden_description(&self) -> String {
        // Combines properties into emoji format:
        // "Task ⏰ 1h 📅 2026-01-28 🔺 /silvermine @work #DeepWork"
    }
}
```

**When regenerated:**
- ✅ On `upsert_task()` - any time task is saved
- ✅ On property change
- ✅ Before sync to remote

**Never:**
- ❌ Edited by user
- ❌ Modified by external sync
- ❌ Changed directly in DB

### 3. Separation of Concerns

| Field | Purpose | Who Writes |
|-------|---------|------------|
| `title` | Clean task name | User input |
| `priority`, `project`, etc. | Structured metadata | User input via flags/triage |
| `user_description` | User's notes | User freeform text |
| `taskgarden_description` | Serialized format | **Auto-generated** |

## Workflow

### Creating a Task

```bash
t add "Review docs" --priority P1 --project silvermine --time 1h
```

**What happens:**

1. **Create CachedTask** with properties:
   ```rust
   CachedTask {
       id: "...",  // From ClickUp/Google
       unique_id: uuid::new_v4(),  // Generated
       title: "Review docs",
       priority: Some("P1"),
       project: Some("silvermine"),
       duration: Some("1h"),
       ...
   }
   ```

2. **Generate taskgarden_description:**
   ```
   "Review docs ⏰ 1h ⏫ /silvermine"
   ```

3. **Save to local DB** (both properties and description)

4. **Sync to ClickUp:**
   - Name: "Review docs"
   - Time Estimate: 3600000ms (1h)
   - Priority: High (2)
   - Tags: ["silvermine"]
   - **Description:** "⏰ 1h ⏫ /silvermine\n\nTaskGarden format for sync reference."

### Updating a Task

```bash
t triage  # Change priority P1 → P0
```

**What happens:**

1. **Update property** in local DB:
   ```rust
   task.priority = Some("P0");
   ```

2. **Auto-regenerate taskgarden_description:**
   ```
   "Review docs ⏰ 1h 🔺 /silvermine"  // ⏫ → 🔺
   ```

3. **Save to DB** (description updated automatically)

4. **Sync updates** priority everywhere

### Syncing from External Platform

**Scenario:** User edits task name in ClickUp

**What happens:**

1. **Fetch from ClickUp:**
   - Name changed to "Review Q4 docs"
   - Description still has: "⏰ 1h 🔺 /silvermine"

2. **Update local DB:**
   ```rust
   task.title = "Review Q4 docs";
   ```

3. **Auto-regenerate taskgarden_description:**
   ```
   "Review Q4 docs ⏰ 1h 🔺 /silvermine"
   ```

4. **Properties stay intact** (parsed from description if needed)

## Platform Sync Mappings

### ClickUp (Full Support)

| TaskGarden | ClickUp Field | Notes |
|------------|---------------|-------|
| `title` | Name | Clean title |
| `duration` | Time Estimate | Converted to milliseconds |
| `due_date` | Due Date | Native support |
| `start_date` | Start Date | Native support |
| `priority` | Priority | P0→Urgent, P1→High, P2→Normal, P3/P5→Low |
| `project` | Tags | As tag |
| `context` | Tags | As tag |
| `tags` | Tags | Native support |
| `user_description` | Description (top) | User's notes |
| `taskgarden_description` | Description (bottom) | Full emoji format backup |

### Google Tasks (Limited Support)

| TaskGarden | Google Field | Notes |
|------------|--------------|-------|
| `title` | Title | Either clean or full emoji format |
| `due_date` | Due | Native support (if API allows) |
| `user_description` | Notes | User's notes |
| All properties | Title/Notes | Stored as emoji format string |

### Airtable (Flexible Support)

| TaskGarden | Airtable Field | Notes |
|------------|----------------|-------|
| `title` | Title | Clean title |
| `duration` | Duration (text) | "1h", "30m", etc. |
| `due_date` | Due Date (date) | Native date field |
| `priority` | Priority (select) | Dropdown: P0, P1, P2, P3, P5 |
| `project` | Project (select) | Dropdown |
| `tags` | Tags (multi-select) | Native tags |
| `taskgarden_description` | TaskGarden Format (text) | Full emoji backup |

## Benefits of This Architecture

### 1. Consistency

✅ **taskgarden_description** is always accurate  
✅ Never gets out of sync with properties  
✅ Can't be manually corrupted  

### 2. Debuggability

✅ Can see full task state in description field  
✅ Easy to verify what was synced  
✅ Human-readable backup  

### 3. Platform Independence

✅ Works even if ClickUp loses a field  
✅ Can reconstruct from description alone  
✅ Easy to add new platforms  

### 4. Conflict Resolution

✅ Always know which properties changed  
✅ Description acts as canonical format  
✅ Can detect external edits  

### 5. Migration Safety

✅ Old tasks can be parsed from description  
✅ New properties can be added without breaking  
✅ Rollback is possible  

## Implementation Rules

### DO ✅

- Always call `generate_taskgarden_description()` before saving
- Use `unique_id` for cross-platform tracking
- Parse properties from external description when syncing
- Keep `title` clean (no emoji properties)
- Store user's notes in `user_description`

### DON'T ❌

- Never edit `taskgarden_description` directly
- Don't store emoji format in `title`
- Don't rely on external platform's description
- Don't skip regeneration on property change
- Don't use `id` for cross-platform tracking (use `unique_id`)

## Future Enhancements

### 1. Bidirectional Sync

Currently: Local → Remote (one-way)

Future: Compare `taskgarden_description` from remote with local properties to detect changes.

```rust
if remote.description != local.taskgarden_description {
    // Something changed remotely
    // Parse and merge changes
}
```

### 2. Conflict Resolution

When both local and remote change:

```rust
fn resolve_conflict(local: &CachedTask, remote: &str) -> CachedTask {
    let remote_task = CachedTask::parse_from_emoji_string(remote);
    // Merge based on timestamps or user preference
}
```

### 3. Task Dependencies

Using `unique_id`:

```rust
task.depends_on = vec![
    "550e8400-e29b-41d4-a716-446655440000",  // unique_id of parent
];
```

### 4. Version History

Track changes to `taskgarden_description`:

```sql
CREATE TABLE task_history (
    task_unique_id TEXT,
    timestamp TEXT,
    taskgarden_description TEXT,
    changed_properties TEXT  -- JSON of what changed
);
```

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_generate_description() {
    let task = CachedTask {
        title: "Test task",
        duration: Some("1h"),
        priority: Some("P0"),
        project: Some("test"),
        ...
    };
    
    let desc = task.generate_taskgarden_description();
    assert_eq!(desc, "Test task ⏰ 1h 🔺 /test");
}

#[test]
fn test_parse_from_description() {
    let desc = "Test task ⏰ 1h 🔺 /test @work #tag";
    let task = CachedTask::parse_from_emoji_string(desc, "id", "list_id");
    
    assert_eq!(task.title, "Test task");
    assert_eq!(task.duration, Some("1h".to_string()));
    assert_eq!(task.priority, Some("P0".to_string()));
}
```

### Integration Tests

```bash
# Test property changes regenerate description
t add "Task" --priority P1
# Verify: taskgarden_description contains "⏫"

t triage --priority P0
# Verify: taskgarden_description updated to "🔺"

# Test sync maintains consistency
t sync --clickup
# Verify: ClickUp description matches local taskgarden_description
```

## Migration from Old Format

See [MIGRATION_PLAN.md](./MIGRATION_PLAN.md) for full details.

**TL;DR:**
1. Add new columns with migrations
2. Generate `unique_id` for existing tasks
3. Parse old bracket format `[date][P0]...` into properties
4. Generate `taskgarden_description` from properties
5. Update all code to use new fields
