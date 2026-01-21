# 🌱 Local Sync & Cache

## What Changed

The Garden now uses **local SQLite cache** for blazing-fast performance!

### Before (Slow)
```bash
t triage
# Fetches all 51 tasks from Google Tasks... 5-10 seconds ⏱️
```

### After (Fast)
```bash
t triage
🔄 Syncing tasks updated since 2026-01-21T00:28:00Z...
✓ Synced 3 tasks
🌱 The Garden - Interactive Triage
# Instant from cache! ⚡
```

## How It Works

### 1. Auto-Sync on Every Command

```bash
t triage    # Auto-syncs, then shows triage
t list      # Auto-syncs, then shows list
t sync      # Manual sync
```

Every time you run `t`, it:
1. Checks last sync timestamp
2. Fetches only NEW/UPDATED tasks since last sync
3. Updates local cache
4. Runs your command using cached data

### 2. Two-Way Sync (Coming Soon)

Current implementation:
- ✅ **Google → Local** (fetch changes)
- ⏳ **Local → Google** (push changes) - coming soon

When you update tasks during triage, changes will be:
1. Saved to local cache immediately
2. Pushed to Google Tasks on next sync

### 3. Cache Location

```
~/.thegarden/cache.db
```

SQLite database stores:
- Task ID, title, status, updated timestamp
- List ID (which list it belongs to)
- Links/attachments metadata
- Last sync timestamp

## Commands

### Auto-Sync (Automatic)
```bash
t triage    # Syncs automatically
t list      # Syncs automatically
```

### Manual Sync
```bash
t sync              # Incremental sync (only fetch changes)
t sync --force      # Full sync (re-fetch everything)
```

### Force Sync (Full Refresh)
```bash
t sync --force
```

Use when:
- Cache seems out of sync
- You made changes directly in Google Tasks
- Something broke

## Performance

### Before (No Cache)
- **First run:** 5-10 seconds (fetch 51 tasks)
- **Every run:** 5-10 seconds (re-fetch everything)

### After (With Cache)
- **First run:** 5-10 seconds (initial sync)
- **Subsequent runs:** <1 second (cache + incremental sync)
- **Sync overhead:** ~1-2 seconds (only fetch changes)

### Example Timeline
```
9:00 AM - t triage          (5s - initial sync)
9:05 AM - Add 2 new tasks in Google Tasks
9:10 AM - t triage          (1s - syncs 2 new tasks)
9:15 AM - Complete 5 tasks in triage
9:16 AM - t list            (<1s - instant from cache)
```

## Sync Strategy

### Incremental Sync (Default)
- Fetches tasks modified since last sync
- Fast and efficient
- Runs automatically on every command

### Full Sync (Force)
- Re-fetches all tasks
- Slower but guaranteed fresh data
- Use when troubleshooting

## Future Optimizations

### Phase 1 (Current) ✅
- Local SQLite cache
- Auto-sync on commands
- Incremental fetch (metadata only)

### Phase 2 (Next) ⏳
- Two-way sync (push local changes to Google)
- Conflict resolution (if task changed in both places)
- Background sync (periodic refresh)

### Phase 3 (Future) 📋
- Offline mode (work without internet)
- Sync queue (batch updates)
- List name caching (currently shows "Tasks" for all)

## Troubleshooting

### Cache out of sync?
```bash
t sync --force    # Re-fetch everything
```

### Clear cache entirely?
```bash
rm ~/.thegarden/cache.db
t sync            # Re-sync from scratch
```

### Sync taking too long?
- Check your internet connection
- Verify `gog` CLI is working: `gog tasks lists list --account yourname@gmail.com`
- Run force sync once: `t sync --force`

## Technical Details

### Database Schema
```sql
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,           -- Google Task ID
    list_id TEXT NOT NULL,         -- Which list it belongs to
    title TEXT NOT NULL,           -- Task title
    status TEXT NOT NULL,          -- needsAction, completed
    updated TEXT NOT NULL,         -- Last modified (RFC3339)
    links TEXT,                    -- JSON array of attachments
    last_synced TEXT NOT NULL      -- When we cached it
);

CREATE TABLE sync_state (
    key TEXT PRIMARY KEY,          -- 'last_sync'
    value TEXT NOT NULL            -- Timestamp (RFC3339)
);
```

### Why SQLite?
- ✅ **Fast** - Blazing local queries
- ✅ **Embedded** - No server needed
- ✅ **Portable** - Single file database
- ✅ **ACID** - Safe concurrent access
- ✅ **Small** - ~60KB for 50 tasks

Enjoy the speed! 🌱⚡
