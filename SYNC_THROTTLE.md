# ⚡ Smart Sync Throttling

Avoids unnecessary syncs when running multiple commands in quick succession.

## How It Works

**Problem:** Running multiple commands back-to-back triggers a full Google Tasks sync each time, which adds latency.

**Solution:** Track the last query time and skip syncing if it happened within N minutes (default: 10).

---

## Configuration

Add to `~/.thegarden/config.json`:

```json
{
  "sync_throttle_minutes": 10
}
```

**Default:** 10 minutes  
**Customizable:** Set to any value (in minutes)

---

## Behavior

### First Command (Cache is stale)
```bash
$ t list
🔄 Syncing (updated since 2026-01-21 05:35)...
✓ Synced 3 tasks
[Lists tasks...]
```

### Subsequent Commands (Within throttle window)
```bash
$ t triage
⚡ Using cache (synced 2 min ago)
🌱 The Garden - Interactive Triage
[Instant! No sync delay]
```

### After Throttle Window Expires
```bash
# 15 minutes later
$ t list
🔄 Syncing (updated since 2026-01-21 05:47)...
✓ Synced 1 task
[Lists tasks...]
```

### Force Sync Anytime
```bash
$ t sync
🔄 Force syncing all tasks...
✓ Synced 51 tasks
```

Or use `--force` on sync command:
```bash
$ t sync --force
```

---

## Implementation Details

### 1. State Tracking

Uses SQLite `sync_state` table (already exists):

```sql
CREATE TABLE IF NOT EXISTS sync_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

Stores:
- `last_query`: Timestamp of last command execution (RFC3339 format)
- `last_sync`: Timestamp of last actual sync (for incremental sync)

### 2. Smart Sync Logic

```rust
fn should_sync(config: &Config, force: bool) -> Result<bool> {
    if force {
        return Ok(true);  // Always sync if forced
    }
    
    // Check last query time
    let last_query = sync_manager.get_state("last_query")?;
    
    if let Some(last) = last_query {
        let elapsed_minutes = (now - last_time).num_minutes();
        
        if elapsed_minutes < config.sync_throttle_minutes {
            // Skip sync, use cache
            println!("⚡ Using cache (synced {} min ago)", elapsed_minutes);
            return Ok(false);
        }
    }
    
    Ok(true)  // Sync needed
}
```

### 3. Command Flow

```rust
match &cli.command {
    Commands::Sync { force } => {
        // Always sync when explicitly called
        sync_with_google(&config.google_account, *force)?;
        update_last_query()?;
    }
    Commands::Triage { .. } | Commands::List { .. } | ... => {
        // Smart sync (check throttle)
        if should_sync(&config, false)? {
            sync_with_google(&config.google_account, false)?;
        }
        // Update last_query timestamp (even if we didn't sync)
        update_last_query()?;
    }
}
```

### 4. New Methods in SyncManager

```rust
// Generic state get/set
pub fn get_state(&self, key: &str) -> Result<Option<String>>
pub fn set_state(&self, key: &str, value: &str) -> Result<()>
```

---

## Benefits

### Performance
- **Instant commands** when cache is fresh
- **No redundant API calls** to Google Tasks
- **Faster workflow** for rapid command sequences

### User Experience
- **Transparent caching** - shows when using cache
- **Manual override** - `t sync` forces sync anytime
- **Configurable** - adjust throttle window to preference

### Example Workflow

```bash
# Morning routine (all instant after first sync)
$ t plan           # Syncs (first command)
$ t focus          # ⚡ Cache (0 min ago)
$ t schedule       # ⚡ Cache (0 min ago)
$ t list --grouped # ⚡ Cache (1 min ago)

# Later in the day
$ t triage         # Syncs (15 min since last)
$ t list           # ⚡ Cache (0 min ago)

# Force sync if needed
$ t sync           # Always syncs
```

---

## Configuration Examples

### More Aggressive Caching (5 minutes)
```json
{
  "sync_throttle_minutes": 5
}
```
Best for: Frequent command use, fast workflows

### Conservative (15 minutes)
```json
{
  "sync_throttle_minutes": 15
}
```
Best for: Infrequent updates, prefer fresh data

### Always Sync (Disable throttling)
```json
{
  "sync_throttle_minutes": 0
}
```
Best for: Multi-device workflows, prefer always-fresh data

---

## Edge Cases

### Multiple Devices
If you use thegarden on multiple devices, you may want a lower throttle (or 0) to ensure you see updates from other devices.

**Recommendation:** Use `t sync` manually when switching devices.

### After External Changes
If you update tasks via Google Tasks app/web:

```bash
# Force sync to pull latest
$ t sync

# Then use normally
$ t list
```

### Race Conditions
The throttle tracks *last query*, not *last sync*. This means:
- Running commands updates the throttle window
- Even if sync was skipped, the window extends
- This prevents sync after throttle expires if you've been actively using the tool

---

## Troubleshooting

### "Using cache" but tasks are stale
```bash
# Force a sync
$ t sync

# Or adjust throttle to be more aggressive
# Edit ~/.thegarden/config.json:
"sync_throttle_minutes": 5
```

### Want to disable throttling
```bash
# Edit ~/.thegarden/config.json:
"sync_throttle_minutes": 0
```

### Check last sync time
```bash
# The sync message shows the incremental sync timestamp
$ t sync
🔄 Syncing (updated since 2026-01-21 05:47)...
```

---

## Technical Notes

### Timestamp Storage
- Stored in RFC3339 format: `2026-01-21T05:45:00Z`
- Uses UTC timezone
- Parsed with `chrono::DateTime::parse_from_rfc3339()`

### Duration Calculation
```rust
let elapsed_duration = now.signed_duration_since(last_time);
let elapsed_minutes = elapsed_duration.num_minutes();
```

### State Isolation
- `last_query`: When you last ran ANY command
- `last_sync`: When Google Tasks was last synced
- Independent tracking allows smart throttling

---

## Future Enhancements

Possible improvements:
- Per-command throttle settings
- Smart invalidation on write operations
- Multi-device sync detection
- Background sync daemon
- Configurable cache TTL per task

---

## Summary

Smart sync throttling:
- ✅ Speeds up rapid command sequences
- ✅ Reduces API calls
- ✅ Transparent to user
- ✅ Manual override available
- ✅ Fully configurable

**Default behavior:** Sync once per 10 minutes unless forced.

**Best practice:** Run `t sync` when:
1. Switching devices
2. After external changes (web/app)
3. When you need absolute latest data

Otherwise, let the throttle handle it automatically! ⚡
