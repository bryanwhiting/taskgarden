use anyhow::{Context, Result};
use regex::Regex;
use once_cell::sync::Lazy;
use crate::airtable::{AirtableClient, AirtableFields, AirtableTask};
use crate::sync::{SyncManager, CachedTask};

// Parse taskgarden format: [date][priority][project][status][@context]{time} title
static TASK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]\[([^\]]+)\]\[([^\]]+)\](?:\[([^\]]+)\])?(?:\[@([^\]]+)\])?(?:\{([^}]+)\})?\s*(.+)").unwrap()
});

pub struct AirtableSync {
    sync_manager: SyncManager,
    airtable_client: AirtableClient,
}

impl AirtableSync {
    pub fn new(sync_manager: SyncManager, airtable_client: AirtableClient) -> Self {
        Self {
            sync_manager,
            airtable_client,
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

    /// Convert a CachedTask to AirtableFields
    fn to_airtable_fields(task: &CachedTask) -> AirtableFields {
        let parsed = Self::parse_task_title(&task.title);
        
        // Extract tags from title (anything with #)
        let tags = extract_hashtags(&parsed.title);
        
        AirtableFields {
            title: parsed.title.clone(),
            priority: parsed.priority,
            project: parsed.project,
            status: parsed.status.or_else(|| {
                // Map Google Tasks status to our status
                match task.status.as_str() {
                    "needsAction" => Some("needsTriage".to_string()),
                    "completed" => Some("done".to_string()),
                    _ => None,
                }
            }),
            context: parsed.context,
            time_estimate: parsed.time_estimate,
            due_date: parsed.date.clone(),
            created_date: task.created.clone(),
            assignee: None, // Can be set manually in Airtable or via config
            tags: if tags.is_empty() { None } else { Some(tags.join(", ")) },
            notes: task.links.clone(),
            completed: Some(task.status == "completed"),
        }
    }

    /// Push all tasks from SQLite cache to Airtable
    pub fn push_to_airtable(&self) -> Result<PushStats> {
        let cached_tasks = self.sync_manager.get_all_cached_tasks()?;
        
        // Get existing Airtable task IDs (stored in sync state)
        let airtable_map = self.get_airtable_id_map()?;
        
        let mut stats = PushStats::default();
        
        for task in cached_tasks {
            // Skip completed tasks older than 7 days (optional filter)
            if task.status == "completed" {
                // Could add date filtering here
                continue;
            }
            
            let airtable_fields = Self::to_airtable_fields(&task);
            
            if let Some(airtable_id) = airtable_map.get(&task.id) {
                // Update existing record
                match self.airtable_client.update_task(airtable_id, airtable_fields) {
                    Ok(_) => stats.updated += 1,
                    Err(e) => {
                        eprintln!("Failed to update task {}: {}", task.id, e);
                        stats.errors += 1;
                    }
                }
            } else {
                // Create new record
                match self.airtable_client.create_task(airtable_fields) {
                    Ok(airtable_task) => {
                        if let Some(id) = airtable_task.id {
                            // Store mapping
                            self.save_airtable_id(&task.id, &id)?;
                            stats.created += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to create task {}: {}", task.id, e);
                        stats.errors += 1;
                    }
                }
            }
        }
        
        Ok(stats)
    }

    /// Get map of Google Task ID → Airtable Record ID
    fn get_airtable_id_map(&self) -> Result<std::collections::HashMap<String, String>> {
        let json_str = self.sync_manager.get_state("airtable_id_map")?
            .unwrap_or_else(|| "{}".to_string());
        
        let map: std::collections::HashMap<String, String> = serde_json::from_str(&json_str)
            .unwrap_or_default();
        
        Ok(map)
    }

    /// Save mapping of Google Task ID → Airtable Record ID
    fn save_airtable_id(&self, google_id: &str, airtable_id: &str) -> Result<()> {
        let mut map = self.get_airtable_id_map()?;
        map.insert(google_id.to_string(), airtable_id.to_string());
        
        let json_str = serde_json::to_string(&map)?;
        self.sync_manager.set_state("airtable_id_map", &json_str)?;
        
        Ok(())
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
        let parsed = AirtableSync::parse_task_title(title);
        
        assert_eq!(parsed.date, Some("2026-01-27".to_string()));
        assert_eq!(parsed.priority, Some("P0".to_string()));
        assert_eq!(parsed.project, Some("SILVERMINE".to_string()));
        assert_eq!(parsed.status, Some("progress".to_string()));
        assert_eq!(parsed.context, Some("@work".to_string()));
        assert_eq!(parsed.time_estimate, Some("2h".to_string()));
        assert_eq!(parsed.title, "Fix editor bug #DeepWork");
    }

    #[test]
    fn test_extract_hashtags() {
        let text = "Fix bug #DeepWork #Urgent test #tag";
        let tags = extract_hashtags(text);
        assert_eq!(tags, vec!["DeepWork", "Urgent", "tag"]);
    }
}
