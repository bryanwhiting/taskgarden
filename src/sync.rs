use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;

pub struct SyncManager {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct CachedTask {
    pub id: String,               // Google Tasks ID or external platform ID
    pub unique_id: String,        // TaskGarden UUID (generated once, never changes)
    pub list_id: String,
    pub title: String,            // Clean title (no emoji properties)
    pub status: String,
    pub updated: String,
    pub created: Option<String>,
    pub links: Option<String>,
    pub dirty: bool,
    
    // Structured properties
    pub priority: Option<String>,      // P0, P1, P2, P3, P5
    pub project: Option<String>,       // silvermine, workday, life
    pub context: Option<String>,       // @work, @home, @phone
    pub duration: Option<String>,      // 1h, 30m, 2h30m
    pub due_date: Option<String>,      // 2026-01-28
    pub start_date: Option<String>,    // 2026-01-27
    pub scheduled_date: Option<String>, // 2026-01-28T14:00
    pub tags: Option<String>,          // Comma-separated: DeepWork,FollowUp
    pub user_description: Option<String>, // User's notes/description
    
    // Derived/computed field (immutable, always regenerated)
    pub taskgarden_description: String, // Emoji format of all properties
}

impl CachedTask {
    /// Generate the taskgarden_description from task properties
    /// This is immutable and always derived from the structured fields
    pub fn generate_taskgarden_description(&self) -> String {
        let mut parts = vec![self.title.clone()];
        
        // Duration
        if let Some(ref duration) = self.duration {
            parts.push(format!("⏰ {}", duration));
        }
        
        // Due date
        if let Some(ref due) = self.due_date {
            parts.push(format!("📅 {}", due));
        }
        
        // Start date
        if let Some(ref start) = self.start_date {
            parts.push(format!("🛫 {}", start));
        }
        
        // Scheduled date
        if let Some(ref scheduled) = self.scheduled_date {
            parts.push(format!("⏳ {}", scheduled));
        }
        
        // Priority emoji
        if let Some(ref priority) = self.priority {
            let emoji = match priority.as_str() {
                "P0" => "🔺",
                "P1" => "⏫",
                "P2" => "🔼",
                "P3" => "🔽",
                "P5" => "⏬",
                _ => "",
            };
            if !emoji.is_empty() {
                parts.push(emoji.to_string());
            }
        }
        
        // Project
        if let Some(ref project) = self.project {
            parts.push(format!("/{}", project.to_lowercase()));
        }
        
        // Context
        if let Some(ref context) = self.context {
            if !context.starts_with('@') {
                parts.push(format!("@{}", context));
            } else {
                parts.push(context.clone());
            }
        }
        
        // Tags
        if let Some(ref tags) = self.tags {
            for tag in tags.split(',') {
                let tag = tag.trim();
                if !tag.is_empty() {
                    parts.push(format!("#{}", tag));
                }
            }
        }
        
        // Created date (if present)
        if let Some(ref created) = self.created {
            parts.push(format!("➕ {}", created));
        }
        
        parts.join(" ")
    }
    
    /// Parse emoji format string into properties
    /// This is for importing tasks that were created externally
    pub fn parse_from_emoji_string(input: &str, id: &str, list_id: &str) -> Self {
        use regex::Regex;
        
        // Extract title (everything before first emoji property)
        let title_regex = Regex::new(r"^([^⏰📅🛫⏳🔺⏫🔼🔽⏬/#@➕✅❌]+)").unwrap();
        let title = title_regex
            .captures(input)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .unwrap_or_else(|| input.to_string());
        
        // Extract properties
        let duration_regex = Regex::new(r"⏰\s*([0-9]+[hm](?:[0-9]+[m])?)").unwrap();
        let due_regex = Regex::new(r"📅\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").unwrap();
        let start_regex = Regex::new(r"🛫\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").unwrap();
        let scheduled_regex = Regex::new(r"⏳\s*([0-9]{4}-[0-9]{2}-[0-9]{2}(?:T[0-9]{2}:[0-9]{2})?)").unwrap();
        let project_regex = Regex::new(r"/([a-zA-Z0-9_-]+)").unwrap();
        let context_regex = Regex::new(r"@([a-zA-Z0-9_-]+)").unwrap();
        let tag_regex = Regex::new(r"#([a-zA-Z0-9_-]+)").unwrap();
        let created_regex = Regex::new(r"➕\s*([0-9]{4}-[0-9]{2}-[0-9]{2})").unwrap();
        
        let duration = duration_regex.captures(input).map(|c| c[1].to_string());
        let due_date = due_regex.captures(input).map(|c| c[1].to_string());
        let start_date = start_regex.captures(input).map(|c| c[1].to_string());
        let scheduled_date = scheduled_regex.captures(input).map(|c| c[1].to_string());
        let project = project_regex.captures(input).map(|c| c[1].to_string());
        let context = context_regex.captures(input).map(|c| c[1].to_string());
        let created = created_regex.captures(input).map(|c| c[1].to_string());
        
        // Extract all tags
        let tags: Vec<String> = tag_regex
            .captures_iter(input)
            .map(|c| c[1].to_string())
            .collect();
        let tags_str = if tags.is_empty() {
            None
        } else {
            Some(tags.join(","))
        };
        
        // Detect priority from emoji
        let priority = if input.contains("🔺") {
            Some("P0".to_string())
        } else if input.contains("⏫") {
            Some("P1".to_string())
        } else if input.contains("🔼") {
            Some("P2".to_string())
        } else if input.contains("🔽") {
            Some("P3".to_string())
        } else if input.contains("⏬") {
            Some("P5".to_string())
        } else {
            None
        };
        
        let unique_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        
        let mut task = CachedTask {
            id: id.to_string(),
            unique_id,
            list_id: list_id.to_string(),
            title,
            status: "needsAction".to_string(),
            updated: now.clone(),
            created,
            links: None,
            dirty: false,
            priority,
            project,
            context,
            duration,
            due_date,
            start_date,
            scheduled_date,
            tags: tags_str,
            user_description: None,
            taskgarden_description: String::new(), // Will be regenerated
        };
        
        // Generate the description from parsed properties
        task.taskgarden_description = task.generate_taskgarden_description();
        
        task
    }
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
                unique_id TEXT NOT NULL UNIQUE,
                list_id TEXT NOT NULL,
                title TEXT NOT NULL,
                status TEXT NOT NULL,
                updated TEXT NOT NULL,
                created TEXT,
                links TEXT,
                last_synced TEXT NOT NULL,
                dirty INTEGER DEFAULT 0,
                
                -- Structured properties
                priority TEXT,
                project TEXT,
                context TEXT,
                duration TEXT,
                due_date TEXT,
                start_date TEXT,
                scheduled_date TEXT,
                tags TEXT,
                user_description TEXT,
                
                -- Derived field (immutable, always regenerated)
                taskgarden_description TEXT NOT NULL
            )",
            [],
        )?;

        // Migration: add new columns if they don't exist
        let migrations = vec![
            "ALTER TABLE tasks ADD COLUMN unique_id TEXT",
            "ALTER TABLE tasks ADD COLUMN dirty INTEGER DEFAULT 0",
            "ALTER TABLE tasks ADD COLUMN created TEXT",
            "ALTER TABLE tasks ADD COLUMN priority TEXT",
            "ALTER TABLE tasks ADD COLUMN project TEXT",
            "ALTER TABLE tasks ADD COLUMN context TEXT",
            "ALTER TABLE tasks ADD COLUMN duration TEXT",
            "ALTER TABLE tasks ADD COLUMN due_date TEXT",
            "ALTER TABLE tasks ADD COLUMN start_date TEXT",
            "ALTER TABLE tasks ADD COLUMN scheduled_date TEXT",
            "ALTER TABLE tasks ADD COLUMN tags TEXT",
            "ALTER TABLE tasks ADD COLUMN user_description TEXT",
            "ALTER TABLE tasks ADD COLUMN taskgarden_description TEXT",
        ];
        
        for migration in migrations {
            let _ = conn.execute(migration, []);
        }
        
        // Generate unique_ids for existing tasks that don't have one
        conn.execute(
            "UPDATE tasks SET unique_id = lower(hex(randomblob(16))) WHERE unique_id IS NULL",
            [],
        )?;
        
        // Generate taskgarden_description for existing tasks
        conn.execute(
            "UPDATE tasks SET taskgarden_description = title WHERE taskgarden_description IS NULL OR taskgarden_description = ''",
            [],
        )?;

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
            "SELECT id, unique_id, list_id, title, status, updated, created, links, dirty,
                    priority, project, context, duration, due_date, start_date, 
                    scheduled_date, tags, user_description, taskgarden_description
             FROM tasks"
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let dirty_val: i32 = row.get(8)?;
                let unique_id: Option<String> = row.get(1)?;
                let taskgarden_desc: Option<String> = row.get(18)?;
                
                Ok(CachedTask {
                    id: row.get(0)?,
                    unique_id: unique_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    list_id: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    updated: row.get(5)?,
                    created: row.get(6)?,
                    links: row.get(7)?,
                    dirty: dirty_val != 0,
                    priority: row.get(9)?,
                    project: row.get(10)?,
                    context: row.get(11)?,
                    duration: row.get(12)?,
                    due_date: row.get(13)?,
                    start_date: row.get(14)?,
                    scheduled_date: row.get(15)?,
                    tags: row.get(16)?,
                    user_description: row.get(17)?,
                    taskgarden_description: taskgarden_desc.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tasks)
    }

    pub fn upsert_task(&self, task: &CachedTask) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        let dirty_val: i32 = if task.dirty { 1 } else { 0 };
        
        // Always regenerate taskgarden_description from properties
        let mut task_to_save = task.clone();
        task_to_save.taskgarden_description = task_to_save.generate_taskgarden_description();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO tasks (
                id, unique_id, list_id, title, status, updated, created, links, 
                last_synced, dirty, priority, project, context, duration, 
                due_date, start_date, scheduled_date, tags, user_description, 
                taskgarden_description
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
            params![
                &task_to_save.id,
                &task_to_save.unique_id,
                &task_to_save.list_id,
                &task_to_save.title,
                &task_to_save.status,
                &task_to_save.updated,
                &task_to_save.created,
                &task_to_save.links,
                &now,
                &dirty_val,
                &task_to_save.priority,
                &task_to_save.project,
                &task_to_save.context,
                &task_to_save.duration,
                &task_to_save.due_date,
                &task_to_save.start_date,
                &task_to_save.scheduled_date,
                &task_to_save.tags,
                &task_to_save.user_description,
                &task_to_save.taskgarden_description,
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
            "SELECT id, unique_id, list_id, title, status, updated, created, links, dirty,
                    priority, project, context, duration, due_date, start_date, 
                    scheduled_date, tags, user_description, taskgarden_description
             FROM tasks WHERE id = ?1",
            params![task_id],
            |row| {
                let dirty_val: i32 = row.get(8)?;
                let unique_id: Option<String> = row.get(1)?;
                let taskgarden_desc: Option<String> = row.get(18)?;
                
                Ok(CachedTask {
                    id: row.get(0)?,
                    unique_id: unique_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    list_id: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    updated: row.get(5)?,
                    created: row.get(6)?,
                    links: row.get(7)?,
                    dirty: dirty_val != 0,
                    priority: row.get(9)?,
                    project: row.get(10)?,
                    context: row.get(11)?,
                    duration: row.get(12)?,
                    due_date: row.get(13)?,
                    start_date: row.get(14)?,
                    scheduled_date: row.get(15)?,
                    tags: row.get(16)?,
                    user_description: row.get(17)?,
                    taskgarden_description: taskgarden_desc.unwrap_or_default(),
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
            "SELECT id, unique_id, list_id, title, status, updated, created, links, dirty,
                    priority, project, context, duration, due_date, start_date, 
                    scheduled_date, tags, user_description, taskgarden_description
             FROM tasks WHERE dirty = 1"
        )?;

        let tasks = stmt
            .query_map([], |row| {
                let dirty_val: i32 = row.get(8)?;
                let unique_id: Option<String> = row.get(1)?;
                let taskgarden_desc: Option<String> = row.get(18)?;
                
                Ok(CachedTask {
                    id: row.get(0)?,
                    unique_id: unique_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    list_id: row.get(2)?,
                    title: row.get(3)?,
                    status: row.get(4)?,
                    updated: row.get(5)?,
                    created: row.get(6)?,
                    links: row.get(7)?,
                    dirty: dirty_val != 0,
                    priority: row.get(9)?,
                    project: row.get(10)?,
                    context: row.get(11)?,
                    duration: row.get(12)?,
                    due_date: row.get(13)?,
                    start_date: row.get(14)?,
                    scheduled_date: row.get(15)?,
                    tags: row.get(16)?,
                    user_description: row.get(17)?,
                    taskgarden_description: taskgarden_desc.unwrap_or_default(),
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
