use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct SyncManager {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct CachedTask {
    pub id: String,
    pub list_id: String,
    pub title: String,
    pub status: String,
    pub updated: String,
    pub created: Option<String>,
    pub links: Option<String>,
    pub dirty: bool,
}

impl SyncManager {
    pub fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(&db_path)
            .context("Failed to open cache database")?;

        // Create tables if they don't exist
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                list_id TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                updated TEXT NOT NULL,
                links TEXT,
                last_synced TEXT NOT NULL,
                dirty INTEGER DEFAULT 0
            )",
            [],
        )?;

        // Migration: add dirty column if it doesn't exist
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN dirty INTEGER DEFAULT 0", []);

        // Migration: add created column if it doesn't exist
        let _ = conn.execute("ALTER TABLE tasks ADD COLUMN created TEXT", []);

        // Create indexes for faster queries
        conn.execute("CREATE INDEX IF NOT EXISTS idx_dirty ON tasks(dirty)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_status ON tasks(status)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_list_id ON tasks(list_id)", [])?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_created ON tasks(created)", [])?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Table for tracking dismissed duplicate pairs
        conn.execute(
            "CREATE TABLE IF NOT EXISTS dismissed_pairs (
                task_id_1 TEXT NOT NULL,
                task_id_2 TEXT NOT NULL,
                dismissed_at TEXT NOT NULL,
                PRIMARY KEY (task_id_1, task_id_2)
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let cache_dir = home.join(".thegarden");
        std::fs::create_dir_all(&cache_dir)?;
        Ok(cache_dir.join("cache.db"))
    }

    pub fn get_last_sync(&self) -> Result<Option<String>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = 'last_sync'",
                [],
                |row| row.get(0),
            )
            .ok();
        Ok(result)
    }

    pub fn set_last_sync(&self, timestamp: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value) VALUES ('last_sync', ?1)",
            params![timestamp],
        )?;
        Ok(())
    }

    /// Generic get state value
    pub fn get_state(&self, key: &str) -> Result<Option<String>> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT value FROM sync_state WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok();
        Ok(result)
    }

    /// Generic set state value
    pub fn set_state(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO sync_state (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_cached_tasks(&self) -> Result<Vec<CachedTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, list_id, title, status, updated, links, dirty, created FROM tasks"
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let dirty_val: i32 = row.get(6)?;
                Ok(CachedTask {
                    id: row.get(0)?,
                    list_id: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    updated: row.get(4)?,
                    links: row.get(5)?,
                    dirty: dirty_val != 0,
                    created: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    pub fn upsert_task(&self, task: &CachedTask) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let dirty_val: i32 = if task.dirty { 1 } else { 0 };
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (id, list_id, title, status, updated, links, last_synced, dirty, created)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                &task.id,
                &task.list_id,
                &task.title,
                &task.status,
                &task.updated,
                &task.links,
                &now,
                &dirty_val,
                &task.created,
            ],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        Ok(())
    }

    /// Get a single task by ID (to preserve links and other metadata)
    pub fn get_task_by_id(&self, task_id: &str) -> Result<Option<CachedTask>> {
        let result = self.conn.query_row(
            "SELECT id, list_id, title, status, updated, links, dirty, created FROM tasks WHERE id = ?1",
            params![task_id],
            |row| {
                let dirty_val: i32 = row.get(6)?;
                Ok(CachedTask {
                    id: row.get(0)?,
                    list_id: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    updated: row.get(4)?,
                    links: row.get(5)?,
                    dirty: dirty_val != 0,
                    created: row.get(7)?,
                })
            },
        );
        match result {
            Ok(task) => Ok(Some(task)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Get all dirty tasks (local changes not yet pushed to Google)
    pub fn get_dirty_tasks(&self) -> Result<Vec<CachedTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, list_id, title, status, updated, links, dirty, created FROM tasks WHERE dirty = 1"
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let dirty_val: i32 = row.get(6)?;
                Ok(CachedTask {
                    id: row.get(0)?,
                    list_id: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    updated: row.get(4)?,
                    links: row.get(5)?,
                    dirty: dirty_val != 0,
                    created: row.get(7)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    /// Mark a task as clean (after successfully pushing to Google)
    pub fn mark_task_clean(&self, task_id: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET dirty = 0 WHERE id = ?1",
            params![task_id],
        )?;
        Ok(())
    }

    /// Upsert a task from remote (Google) - skips if local task is dirty
    pub fn upsert_task_from_remote(&self, task: &CachedTask) -> Result<bool> {
        // Check if local task exists and is dirty
        if let Some(existing) = self.get_task_by_id(&task.id)? {
            if existing.dirty {
                // Don't overwrite dirty local changes
                return Ok(false);
            }
        }

        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (id, list_id, title, status, updated, links, last_synced, dirty, created)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
            params![
                &task.id,
                &task.list_id,
                &task.title,
                &task.status,
                &task.updated,
                &task.links,
                &now,
                &task.created,
            ],
        )?;
        Ok(true)
    }

    /// Upsert a task locally and mark it as dirty
    pub fn upsert_task_locally(&self, task: &CachedTask) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (id, list_id, title, status, updated, links, last_synced, dirty, created)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
            params![
                &task.id,
                &task.list_id,
                &task.title,
                &task.status,
                &task.updated,
                &task.links,
                &now,
                &task.created,
            ],
        )?;
        Ok(())
    }

    pub fn clear_cache(&self) -> Result<()> {
        self.conn.execute("DELETE FROM tasks", [])?;
        self.conn.execute("DELETE FROM sync_state", [])?;
        Ok(())
    }

    /// Check if a pair of tasks has been dismissed as not-duplicates
    pub fn is_pair_dismissed(&self, id1: &str, id2: &str) -> Result<bool> {
        // Normalize order for consistent lookup
        let (a, b) = if id1 < id2 { (id1, id2) } else { (id2, id1) };
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM dismissed_pairs WHERE task_id_1 = ?1 AND task_id_2 = ?2",
            params![a, b],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Mark a pair of tasks as not-duplicates
    pub fn dismiss_pair(&self, id1: &str, id2: &str) -> Result<()> {
        let (a, b) = if id1 < id2 { (id1, id2) } else { (id2, id1) };
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT OR REPLACE INTO dismissed_pairs (task_id_1, task_id_2, dismissed_at) VALUES (?1, ?2, ?3)",
            params![a, b, now],
        )?;
        Ok(())
    }

    /// Reset all dismissed pairs
    pub fn reset_dismissed_pairs(&self) -> Result<()> {
        self.conn.execute("DELETE FROM dismissed_pairs", [])?;
        Ok(())
    }

    /// Delete a task by ID
    pub fn delete_task_by_id(&self, task_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM tasks WHERE id = ?1", params![task_id])?;
        // Also clean up any dismissed pairs involving this task
        self.conn.execute(
            "DELETE FROM dismissed_pairs WHERE task_id_1 = ?1 OR task_id_2 = ?1",
            params![task_id],
        )?;
        Ok(())
    }
}
