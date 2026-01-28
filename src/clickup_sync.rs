use anyhow::{Result};
use regex::Regex;
use once_cell::sync::Lazy;
use crate::clickup::{ClickUpClient, ClickUpTask};
use crate::sync::{SyncManager, CachedTask};
use chrono::{DateTime, Utc};

// Parse taskgarden format: [date][priority][project][status][@context]{time} title
static TASK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]\[([^\]]+)\]\[([^\]]+)\](?:\[([^\]]+)\])?(?:\[@([^\]]+)\])?(?:\{([^}]+)\})?\s*(.+)").unwrap()
});

pub struct ClickUpSync {
    sync_manager: SyncManager,
    clickup_client: ClickUpClient,
    list_mappings: std::collections::HashMap<String, String>,  // Google list ID -> ClickUp list ID
}

impl ClickUpSync {
    pub fn new(sync_manager: SyncManager, clickup_client: ClickUpClient, list_mappings: std::collections::HashMap<String, String>) -> Self {
        Self {
            sync_manager,
            clickup_client,
            list_mappings,
        }
    }

    /// Parse a taskgarden formatted title into structured fields
    fn parse_task_title(title: &str) -> ParsedTask {
        if let Some(caps) = TASK_REGEX.captures(title) {
            ParsedTask {
                date: caps.get(1).map(|m| m.as_str().to_string()),
                priority: caps.get(2).map(|m| m.as_str().to_string()),
                project: caps.get(3).map(|m| m.as_str().to_string()),
                status: caps.get(4).map(|m| m.as_str().to_string()),
                context: caps.get(5).map(|m| format!("@{}", m.as_str())),
                time_estimate: caps.get(6).map(|m| m.as_str().to_string()),
                title: caps.get(7).map(|m| m.as_str().to_string()).unwrap_or_default(),
            }
        } else {
            // Unparsed task - just use the raw title
            ParsedTask {
                date: None,
                priority: None,
                project: None,
                status: None,
                context: None,
                time_estimate: None,
                title: title.to_string(),
            }
        }
    }

    /// Convert taskgarden priority to ClickUp priority (1=urgent, 2=high, 3=normal, 4=low)
    fn priority_to_clickup(priority: Option<&str>) -> Option<u8> {
        match priority {
            Some("P0") => Some(1), // Urgent
            Some("P1") => Some(2), // High
            Some("P2") => Some(3), // Normal
            Some("P3") => Some(4), // Low
            Some("P5") => Some(4), // Delegate -> Low
            _ => None,
        }
    }

    /// Convert taskgarden status to ClickUp status string
    fn status_to_clickup(status: Option<&str>, google_status: &str) -> String {
        if google_status == "completed" {
            return "complete".to_string();
        }

        match status {
            Some("needsTriage") => "to do".to_string(),
            Some("progress") => "in progress".to_string(),
            Some("review") => "review".to_string(),
            Some("blocked") => "blocked".to_string(),
            Some("done") => "complete".to_string(),
            _ => "to do".to_string(),
        }
    }

    /// Parse time estimate to milliseconds
    fn time_estimate_to_ms(time_str: Option<&str>) -> Option<i64> {
        time_str.and_then(|s| {
            let s = s.trim().to_lowercase();
            if let Some(h) = s.strip_suffix('h') {
                h.parse::<i64>().ok().map(|h| h * 3600 * 1000)
            } else if let Some(m) = s.strip_suffix('m') {
                m.parse::<i64>().ok().map(|m| m * 60 * 1000)
            } else {
                None
            }
        })
    }

    /// Parse date string to Unix timestamp (milliseconds)
    fn date_to_timestamp(date_str: Option<&str>) -> Option<i64> {
        date_str.and_then(|s| {
            DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", s))
                .ok()
                .map(|dt| dt.timestamp_millis())
        })
    }

    /// Convert a CachedTask to ClickUpTask
    fn to_clickup_task(task: &CachedTask) -> ClickUpTask {
        let parsed = Self::parse_task_title(&task.title);
        
        // Extract tags from title (anything with #)
        let mut tags = extract_hashtags(&parsed.title);
        
        // Add project as a tag if present
        if let Some(ref project) = parsed.project {
            tags.push(project.clone());
        }
        
        // Add context as a tag if present
        if let Some(ref context) = parsed.context {
            tags.push(context.clone());
        }

        // Build description with metadata
        let mut description_parts = vec![parsed.title.clone()];
        if let Some(ref links) = task.links {
            description_parts.push(format!("\n\nLinks: {}", links));
        }
        if let Some(ref created) = task.created {
            description_parts.push(format!("Created: {}", created));
        }

        ClickUpTask {
            id: None,
            name: parsed.title.clone(),
            description: Some(description_parts.join("\n")),
            status: Some(Self::status_to_clickup(parsed.status.as_deref(), &task.status)),
            priority: Self::priority_to_clickup(parsed.priority.as_deref()),
            due_date: Self::date_to_timestamp(parsed.date.as_deref()),
            start_date: None,
            time_estimate: Self::time_estimate_to_ms(parsed.time_estimate.as_deref()),
            tags,
            assignees: vec![], // Can be set based on config or manually in ClickUp
            custom_fields: None,
        }
    }

    /// Push all tasks from SQLite cache to ClickUp
    pub fn push_to_clickup(&self) -> Result<PushStats> {
        let cached_tasks = self.sync_manager.get_all_cached_tasks()?;
        
        // Get existing ClickUp task IDs (stored in sync state)
        let clickup_map = self.get_clickup_id_map()?;
        
        let mut stats = PushStats::default();
        
        for task in cached_tasks {
            // Skip completed tasks
            if task.status == "completed" {
                continue;
            }
            
            // Get the ClickUp list ID for this Google Task list
            let clickup_list_id = match self.list_mappings.get(&task.list_id) {
                Some(id) => id,
                None => {
                    eprintln!("Warning: No ClickUp list mapping for Google list {}, skipping task {}", task.list_id, task.id);
                    stats.errors += 1;
                    continue;
                }
            };
            
            let clickup_task = Self::to_clickup_task(&task);
            
            if let Some(clickup_id) = clickup_map.get(&task.id) {
                // Update existing task
                match self.clickup_client.update_task(clickup_id, &clickup_task) {
                    Ok(_) => stats.updated += 1,
                    Err(e) => {
                        eprintln!("Failed to update task {}: {}", task.id, e);
                        stats.errors += 1;
                    }
                }
            } else {
                // Create new task in the correct list
                match self.clickup_client.create_task(clickup_list_id, &clickup_task) {
                    Ok(clickup_response) => {
                        // Store mapping
                        self.save_clickup_id(&task.id, &clickup_response.id)?;
                        
                        // Mark task as synced in Google Tasks with 🔃
                        self.mark_task_synced(&task)?;
                        
                        stats.created += 1;
                    }
                    Err(e) => {
                        eprintln!("Failed to create task {} in list {}: {}", task.id, clickup_list_id, e);
                        stats.errors += 1;
                    }
                }
            }
        }
        
        Ok(stats)
    }

    /// Get map of Google Task ID → ClickUp Task ID
    fn get_clickup_id_map(&self) -> Result<std::collections::HashMap<String, String>> {
        let json_str = self.sync_manager.get_state("clickup_id_map")?
            .unwrap_or_else(|| "{}".to_string());
        
        let map: std::collections::HashMap<String, String> = serde_json::from_str(&json_str)
            .unwrap_or_default();
        
        Ok(map)
    }

    /// Save mapping of Google Task ID → ClickUp Task ID
    fn save_clickup_id(&self, google_id: &str, clickup_id: &str) -> Result<()> {
        let mut map = self.get_clickup_id_map()?;
        map.insert(google_id.to_string(), clickup_id.to_string());
        
        let json_str = serde_json::to_string(&map)?;
        self.sync_manager.set_state("clickup_id_map", &json_str)?;
        
        Ok(())
    }
    
    /// Mark a task as synced in Google Tasks by prepending 🔃
    fn mark_task_synced(&self, task: &CachedTask) -> Result<()> {
        // Skip if already marked
        if task.title.starts_with("🔃") {
            return Ok(());
        }
        
        // Update task title in Google Tasks with 🔃 prefix
        let new_title = format!("🔃 {}", task.title);
        
        // Use gog CLI to update the task
        // gog tasks update <tasklistId> <taskId> --title "..." --account ...
        let output = std::process::Command::new("gog")
            .args(&[
                "tasks",
                "update",
                &task.list_id,
                &task.id,
                "--title",
                &new_title,
                "--account",
                "bryan@silvermineai.com",
            ])
            .output();
        
        match output {
            Ok(result) if result.status.success() => {
                // Update local cache too
                let mut updated_task = task.clone();
                updated_task.title = new_title;
                self.sync_manager.upsert_task(&updated_task)?;
                Ok(())
            }
            Ok(result) => {
                eprintln!("Warning: Failed to mark task {} as synced: {}", 
                    task.id, 
                    String::from_utf8_lossy(&result.stderr));
                Ok(()) // Don't fail the whole sync
            }
            Err(e) => {
                eprintln!("Warning: Failed to execute gog to mark task {}: {}", task.id, e);
                Ok(()) // Don't fail the whole sync
            }
        }
    }
}

#[derive(Debug)]
struct ParsedTask {
    date: Option<String>,
    priority: Option<String>,
    project: Option<String>,
    status: Option<String>,
    context: Option<String>,
    time_estimate: Option<String>,
    title: String,
}

#[derive(Debug, Default)]
pub struct PushStats {
    pub created: usize,
    pub updated: usize,
    pub errors: usize,
}

/// Extract hashtags from a string
fn extract_hashtags(text: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for word in text.split_whitespace() {
        if let Some(tag) = word.strip_prefix('#') {
            tags.push(tag.to_string());
        }
    }
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_task_title() {
        let title = "[2026-01-27][P0][SILVERMINE][progress][@work]{2h} Fix editor bug #DeepWork";
        let parsed = ClickUpSync::parse_task_title(title);
        
        assert_eq!(parsed.date, Some("2026-01-27".to_string()));
        assert_eq!(parsed.priority, Some("P0".to_string()));
        assert_eq!(parsed.project, Some("SILVERMINE".to_string()));
        assert_eq!(parsed.status, Some("progress".to_string()));
        assert_eq!(parsed.context, Some("@work".to_string()));
        assert_eq!(parsed.time_estimate, Some("2h".to_string()));
        assert_eq!(parsed.title, "Fix editor bug #DeepWork");
    }

    #[test]
    fn test_priority_conversion() {
        assert_eq!(ClickUpSync::priority_to_clickup(Some("P0")), Some(1));
        assert_eq!(ClickUpSync::priority_to_clickup(Some("P1")), Some(2));
        assert_eq!(ClickUpSync::priority_to_clickup(Some("P2")), Some(3));
        assert_eq!(ClickUpSync::priority_to_clickup(Some("P3")), Some(4));
    }

    #[test]
    fn test_time_estimate_conversion() {
        assert_eq!(ClickUpSync::time_estimate_to_ms(Some("2h")), Some(7200000));
        assert_eq!(ClickUpSync::time_estimate_to_ms(Some("30m")), Some(1800000));
    }
}
