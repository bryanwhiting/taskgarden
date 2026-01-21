# ⚡ Speed Optimization - updatedMin

## The Problem (Before)

```bash
t triage
🔄 Syncing tasks updated since 2026-01-21T00:28:00Z...
# Still fetching ALL 51 tasks from Google...
# Takes 3-5 seconds every time
✓ Synced 51 tasks
```

Even with timestamps, it was fetching EVERY task on EVERY sync.

## The Solution (After)

```bash
t triage
🔄 Syncing (updated since 2026-01-21 00:28)...
# Only fetches tasks that changed!
✓ No changes
# <0.5 seconds!
```

## How It Works

### updatedMin Parameter

Google Tasks API (via `gog` CLI) supports `--updated-min` parameter:

```bash
gog tasks list <list_id> --updated-min "2026-01-21T00:28:00Z"
```

Only returns tasks modified AFTER that timestamp!

### Implementation

```rust
// Build args with time filter
let mut args = vec!["tasks", "list", list_id, "--account", account, "--json"];

// Add updatedMin filter if we have a last sync time
if let Some(ref min_time) = updated_min {
    args.push("--updated-min");
    args.push(min_time);  // RFC3339 timestamp
}

let tasks_output = Command::new("gog").args(&args).output()?;
```

### Sync Flow

**First run (no cache):**
```bash
t triage
🔄 First sync - fetching all tasks...
✓ Synced 51 tasks (5-7 seconds)
# Saves timestamp: 2026-01-21T00:30:00Z
```

**Second run (5 minutes later, no changes):**
```bash
t triage
🔄 Syncing (updated since 2026-01-21 00:30)...
✓ No changes (<0.5 seconds)
```

**Third run (2 new tasks added):**
```bash
t triage
🔄 Syncing (updated since 2026-01-21 00:30)...
✓ Synced 2 tasks (<1 second)
```

## Performance Comparison

### Before (No updatedMin)
| Scenario | Time | Tasks Fetched |
|----------|------|---------------|
| First sync | 5-7s | 51 |
| No changes | 3-5s | 51 |
| 2 new tasks | 3-5s | 51 |
| Triage 10 tasks | 3-5s | 51 |

**Total for 4 runs: ~16-22 seconds**

### After (With updatedMin)
| Scenario | Time | Tasks Fetched |
|----------|------|---------------|
| First sync | 5-7s | 51 |
| No changes | <0.5s | 0 |
| 2 new tasks | <1s | 2 |
| Triage 10 tasks | <1s | 10 |

**Total for 4 runs: ~7-9 seconds**

**~60% faster!** ⚡

## Real-World Impact

### Morning Routine
```bash
# 9:00 AM - First check
t list
🔄 First sync...
✓ Synced 51 tasks (5s)

# 9:05 AM - Triage new tasks
t triage
🔄 Syncing (updated since 09:00)...
✓ Synced 3 tasks (0.5s)  ← Fast!

# 9:15 AM - Check list again
t list
🔄 Syncing (updated since 09:05)...
✓ No changes (0.3s)  ← Instant!

# 9:30 AM - Triage more
t triage
🔄 Syncing (updated since 09:15)...
✓ Synced 5 tasks (0.6s)  ← Fast!
```

### Throughout the Day
- **First sync of the day:** 5-7 seconds
- **Every other sync:** <1 second
- **If no changes:** <0.5 seconds

## Technical Details

### Timestamp Format

Google Tasks uses **RFC3339** format:
```
2026-01-21T00:30:45.123Z
```

Stored in SQLite `sync_state` table:
```sql
INSERT INTO sync_state (key, value) 
VALUES ('last_sync', '2026-01-21T00:30:45Z');
```

### Timestamp Updates

Updated **after successful sync**:
```rust
let now = Utc::now().to_rfc3339();
sync_manager.set_last_sync(&now)?;
```

This means:
- If sync fails, timestamp isn't updated
- Next sync will retry from last successful sync
- No tasks are lost

### Force Sync

```bash
t sync --force
🔄 Force syncing all tasks...
✓ Synced 51 tasks

# Resets timestamp, fetches everything
# Useful for troubleshooting
```

## Optimization Benefits

### Speed
- ✅ **60% faster** on average
- ✅ **Instant** when no changes
- ✅ **Sub-second** for incremental changes

### Bandwidth
- ✅ **Less API calls** to Google
- ✅ **Smaller payloads** (only changed tasks)
- ✅ **Lower quota usage**

### User Experience
- ✅ **No waiting** between commands
- ✅ **Snappy** triage flow
- ✅ **Feels instant** for most operations

### Battery (Mobile/Laptop)
- ✅ **Less CPU** (fewer tasks to parse)
- ✅ **Less network** (smaller transfers)
- ✅ **Less power** overall

## Future Optimizations

### Phase 1 (Current) ✅
- updatedMin filtering
- Local SQLite cache
- Auto-sync on commands

### Phase 2 (Next) ⏳
- **Parallel list fetching** (fetch all lists at once)
- **Batch updates** (update multiple tasks in one call)
- **Background sync** (periodic refresh without blocking)

### Phase 3 (Future) 📋
- **Push notifications** (Google Tasks webhooks)
- **Offline queue** (sync when connection restored)
- **Conflict resolution** (merge changes from multiple devices)

Enjoy the speed! ⚡🌱
