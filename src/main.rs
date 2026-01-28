mod sync;
mod airtable;
mod airtable_sync;
mod clickup;
mod clickup_sync;

use anyhow::{Context, Result};
use chrono::{Datelike, Local, Utc, TimeZone};
use clap::{Parser, Subcommand};
use colored::*;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use sync::{SyncManager, CachedTask};
use airtable::AirtableClient;
use airtable_sync::AirtableSync;
use clickup::ClickUpClient;
use clickup_sync::ClickUpSync;

// Pre-compiled regex for parsing task titles
// Format: [date][priority][project][status][@context]{time} title
// Status and context are optional
static TASK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\[([^\]]+)\]\[([^\]]+)\]\[([^\]]+)\](?:\[([^\]]+)\])?(?:\[(@[^\]]+)\])?(?:\{([^}]+)\})?\s*(.+)").unwrap()
});

#[derive(Parser)]
#[command(name = "thegarden")]
#[command(about = "Task management CLI for ADHD-friendly triage", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive task triage (prioritize unprioritized tasks)
    Triage {
        /// Force re-triage of all tasks (including already triaged)
        #[arg(short, long)]
        force: bool,
        /// Only do priority pass
        #[arg(long)]
        priority: bool,
        /// Only do project pass
        #[arg(long)]
        project: bool,
        /// Only do time estimate pass
        #[arg(long)]
        time: bool,
        /// Only do status pass
        #[arg(long)]
        status: bool,
        /// Only do context pass
        #[arg(long)]
        context: bool,
    },
    /// Show only critical tasks for today (P0 + overdue/due today P1)
    Focus,
    /// Show this week's plan (meetings + tasks)
    Plan,
    /// Schedule tasks into calendar blocks
    Schedule {
        /// Schedule for the whole week instead of just today
        #[arg(short, long)]
        week: bool,
        /// Automatically create calendar events without prompting
        #[arg(short, long)]
        auto: bool,
    },
    /// List today's tasks
    List {
        /// Show all tasks (not just today)
        #[arg(short, long)]
        all: bool,
        /// Sort by: priority, date, project, time, status (default: priority)
        #[arg(short, long, default_value = "priority")]
        sort: String,
        /// Reverse sort order
        #[arg(short, long)]
        reverse: bool,
        /// Filter by status (e.g., progress, review, blocked)
        #[arg(long)]
        status: Option<String>,
        /// Filter by context (e.g., @work, @home)
        #[arg(short = 'c', long)]
        context: Option<String>,
        /// Filter by priority (e.g., P0, P1, or "P0,P1" for multiple)
        #[arg(short = 'p', long)]
        priority: Option<String>,
        /// Filter by project
        #[arg(short = 'j', long)]
        project: Option<String>,
        /// Filter by tag
        #[arg(short = 't', long)]
        tag: Option<String>,
        /// Show tasks from last N days
        #[arg(short = 'd', long)]
        days: Option<i64>,
        /// Limit number of results
        #[arg(short = 'n', long)]
        limit: Option<usize>,
        /// Group tasks by date
        #[arg(short = 'g', long)]
        grouped: bool,
    },
    /// Add a new task
    Add {
        /// Task title
        title: String,
        /// Priority (P0, P1, P2, P3)
        #[arg(short, long)]
        priority: Option<String>,
        /// Project (WORKDAY, LIFE, SILVERMINE)
        #[arg(short = 'j', long)]
        project: Option<String>,
    },
    /// Bump incomplete tasks to tomorrow
    Bump {
        /// Number of days to bump (default: 1)
        #[arg(short, long, default_value = "1")]
        days: i64,
        /// Bump to next Monday
        #[arg(short, long)]
        week: bool,
    },
    /// Find and merge duplicate tasks
    Merge {
        /// Similarity threshold as percentage (0-100, default 80)
        #[arg(short = 't', long = "thresh", default_value = "80")]
        threshold: f64,
        /// Reset dismissed pairs (show all potential duplicates again)
        #[arg(long)]
        reset: bool,
    },
    /// Sync with Google Tasks (auto-runs on every command)
    Sync {
        #[arg(short, long)]
        force: bool,
        /// Push tasks to Airtable for team visibility
        #[arg(short, long)]
        airtable: bool,
        /// Push tasks to ClickUp for team visibility
        #[arg(short, long)]
        clickup: bool,
    },
    /// Show details for a specific task
    Show {
        /// Task ID (or partial ID)
        id: String,
    },
    /// Search tasks by title
    Search {
        /// Search query (substring match)
        query: String,
        /// Filter by project
        #[arg(short = 'j', long)]
        project: Option<String>,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by context
        #[arg(short = 'c', long)]
        context: Option<String>,
        /// Filter by priority
        #[arg(short = 'p', long)]
        priority: Option<String>,
    },
    /// Summarize tasks with flexible slicing and grouping
    Summary {
        /// Group by: date, priority, project, status, context, created (default: date)
        #[arg(short, long, default_value = "date")]
        group: String,
        /// Show tasks created in the last N days
        #[arg(long)]
        created_days: Option<i64>,
        /// Show tasks due in the next N days
        #[arg(long)]
        due_days: Option<i64>,
        /// Include completed tasks
        #[arg(long)]
        include_done: bool,
        /// Sort groups by: count, time, name (default: name)
        #[arg(short, long, default_value = "name")]
        sort: String,
        /// Show detailed task list for each group
        #[arg(short, long)]
        detailed: bool,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Priority {
    name: String,
    key: char,
    alt_key: Option<char>,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TimeOption {
    label: String,
    key: char,
    #[serde(default)]
    alt_key: Option<char>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct StatusOption {
    name: String,
    key: char,
    #[serde(default)]
    alt_key: Option<char>,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskTypeDefaults {
    priority: String,
    time: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    format: String,
    date_format: String,
    projects: std::collections::HashMap<String, String>,
    google_account: String,
    #[serde(default = "default_priorities")]
    priorities: Vec<Priority>,
    #[serde(default = "default_time_options")]
    time_options: Vec<TimeOption>,
    #[serde(default = "default_statuses")]
    statuses: Vec<StatusOption>,
    #[serde(default)]
    contexts: Vec<String>,
    #[serde(default)]
    task_types: std::collections::HashMap<String, TaskTypeDefaults>,
    #[serde(default = "default_sync_throttle")]
    sync_throttle_minutes: i64,
    #[serde(default)]
    airtable: Option<AirtableConfig>,
    #[serde(default)]
    clickup: Option<ClickUpConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AirtableConfig {
    #[serde(default)]
    enabled: bool,
    api_key: String,
    base_id: String,
    table_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ClickUpConfig {
    #[serde(default)]
    enabled: bool,
    api_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    list_id: Option<String>,  // Default list for new tasks
    #[serde(skip_serializing_if = "Option::is_none")]
    list_mappings: Option<std::collections::HashMap<String, String>>,  // Google list ID -> ClickUp list ID
}

fn default_sync_throttle() -> i64 {
    10
}

fn default_priorities() -> Vec<Priority> {
    vec![
        Priority { name: "P0".into(), key: '0', alt_key: Some('j'), description: "Urgent + Important".into() },
        Priority { name: "P1".into(), key: '1', alt_key: Some('k'), description: "Important, not urgent".into() },
        Priority { name: "P2".into(), key: '2', alt_key: Some('l'), description: "Urgent, not important".into() },
        Priority { name: "P3".into(), key: '3', alt_key: Some(';'), description: "Not important, not urgent".into() },
        Priority { name: "P5".into(), key: 'd', alt_key: Some('D'), description: "Delegate".into() },
    ]
}

fn default_time_options() -> Vec<TimeOption> {
    vec![
        TimeOption { label: "15m".into(), key: '0', alt_key: Some('j') },
        TimeOption { label: "30m".into(), key: '1', alt_key: Some('k') },
        TimeOption { label: "45m".into(), key: '2', alt_key: Some('l') },
        TimeOption { label: "1h".into(), key: '3', alt_key: Some(';') },
        TimeOption { label: "3h".into(), key: '4', alt_key: Some('i') },
        TimeOption { label: "8h".into(), key: '5', alt_key: Some('o') },
    ]
}

fn default_statuses() -> Vec<StatusOption> {
    vec![
        StatusOption { name: "todo".into(), key: 't', alt_key: Some('j'), description: "Not started".into() },
        StatusOption { name: "progress".into(), key: 'p', alt_key: Some('k'), description: "In progress".into() },
        StatusOption { name: "review".into(), key: 'r', alt_key: Some('l'), description: "In review".into() },
        StatusOption { name: "blocked".into(), key: 'b', alt_key: Some(';'), description: "Blocked".into() },
    ]
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Task {
    id: Option<String>,
    list_id: Option<String>,
    title: String,
    date: String,
    priority: Option<String>,
    project: Option<String>,
    status: Option<String>,
    context: Option<String>,
    time: Option<String>,
    list: String,
    attachment_type: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskLink {
    #[serde(rename = "type")]
    link_type: String,
    link: String,
    description: Option<String>,
}

/// Parse natural language date from text (e.g., "due Monday", "in 3 days", "due 1/25")
fn parse_date_from_text(text: &str) -> Option<String> {
    let text_lower = text.to_lowercase();
    let today = Local::now();

    // Check for "in X days"
    if let Some(caps) = Regex::new(r"in (\d+) days?").unwrap().captures(&text_lower) {
        if let Ok(days) = caps.get(1).unwrap().as_str().parse::<i64>() {
            let target = today + chrono::Duration::days(days);
            return Some(target.format("%Y-%m-%d").to_string());
        }
    }

    // Check for "tomorrow"
    if text_lower.contains("tomorrow") {
        let target = today + chrono::Duration::days(1);
        return Some(target.format("%Y-%m-%d").to_string());
    }

    // Check for day of week (e.g., "due Monday", "next Tuesday")
    let weekdays = [
        ("monday", 0), ("mon", 0),
        ("tuesday", 1), ("tue", 1),
        ("wednesday", 2), ("wed", 2),
        ("thursday", 3), ("thu", 3),
        ("friday", 4), ("fri", 4),
        ("saturday", 5), ("sat", 5),
        ("sunday", 6), ("sun", 6),
    ];

    for (day_name, target_weekday) in &weekdays {
        if text_lower.contains(day_name) {
            let current_weekday = today.weekday().num_days_from_monday();
            let days_ahead = if *target_weekday >= current_weekday {
                (*target_weekday - current_weekday) as i64
            } else {
                (7 - current_weekday + *target_weekday) as i64
            };
            
            // If it's the same day, assume next week
            let days_to_add = if days_ahead == 0 { 7 } else { days_ahead };
            
            let target = today + chrono::Duration::days(days_to_add);
            return Some(target.format("%Y-%m-%d").to_string());
        }
    }

    // Check for M/D or M/DD format (e.g., "1/25", "12/5")
    if let Some(caps) = Regex::new(r"(\d{1,2})/(\d{1,2})").unwrap().captures(&text_lower) {
        if let (Ok(month), Ok(day)) = (
            caps.get(1).unwrap().as_str().parse::<u32>(),
            caps.get(2).unwrap().as_str().parse::<u32>(),
        ) {
            let year = today.year();
            if let Some(target) = chrono::NaiveDate::from_ymd_opt(year, month, day) {
                // If the date is in the past this year, assume next year
                if target < today.naive_local().date() {
                    if let Some(next_year_target) = chrono::NaiveDate::from_ymd_opt(year + 1, month, day) {
                        return Some(next_year_target.format("%Y-%m-%d").to_string());
                    }
                } else {
                    return Some(target.format("%Y-%m-%d").to_string());
                }
            }
        }
    }

    None
}

impl Task {
    /// Get a short ID for display (first 8 chars)
    fn short_id(&self) -> String {
        self.id.as_ref()
            .map(|id| id.chars().take(8).collect())
            .unwrap_or_else(|| "--------".to_string())
    }

    fn format(&self, config: &Config) -> String {
        let mut formatted = config.format.clone();
        formatted = formatted.replace("{date}", &self.date);
        formatted = formatted.replace("{priority}", self.priority.as_deref().unwrap_or("--"));
        formatted = formatted.replace("{project}", self.project.as_deref().unwrap_or("---"));

        // Status is optional in format
        let status_str = self.status.as_ref().map(|s| format!("[{}]", s)).unwrap_or_default();
        formatted = formatted.replace("{status}", &status_str);

        // Context is optional in format
        let context_str = self.context.as_ref().map(|c| format!("[{}]", c)).unwrap_or_default();
        formatted = formatted.replace("{context}", &context_str);

        formatted = formatted.replace("{time}", self.time.as_deref().unwrap_or(""));
        
        // Add hashtags to title if present
        let title_with_tags = if self.tags.is_empty() {
            self.title.clone()
        } else {
            let tags_str = self.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" ");
            format!("{} {}", self.title, tags_str.dimmed().to_string())
        };
        formatted = formatted.replace("{title}", &title_with_tags);
        formatted
    }

    fn parse(title: &str, list: &str) -> Task {
        Task::parse_with_config(title, list, None)
    }

    fn parse_with_config(title: &str, list: &str, config: Option<&Config>) -> Task {
        // Extract hashtags from title
        let hashtag_regex = Regex::new(r"#(\w+)").unwrap();
        let tags: Vec<String> = hashtag_regex
            .captures_iter(title)
            .map(|cap| cap.get(1).unwrap().as_str().to_string())
            .collect();

        // Try to parse date from title text
        let parsed_date = parse_date_from_text(title);

        // Parse format: [date][priority][project][status][@context]{time} actual title
        if let Some(caps) = TASK_REGEX.captures(title) {
            let mut task = Task {
                id: None,
                list_id: None,
                date: caps.get(1).unwrap().as_str().to_string(),
                priority: Some(caps.get(2).unwrap().as_str().to_string()),
                project: Some(caps.get(3).unwrap().as_str().to_string()),
                status: caps.get(4).map(|m| m.as_str().to_string()),
                context: caps.get(5).map(|m| m.as_str().to_string()),
                time: caps.get(6).map(|m| m.as_str().to_string()),
                title: caps.get(7).unwrap().as_str().to_string(),
                list: list.to_string(),
                attachment_type: None,
                tags,
            };
            
            // Override date if we found one in the title text
            if let Some(date) = parsed_date {
                task.date = date;
            }
            
            task
        } else {
            // Unprioritized task - apply defaults based on hashtags
            let mut priority = None;
            let mut time = None;

            // Apply task type defaults if config is provided
            if let Some(cfg) = config {
                for tag in &tags {
                    if let Some(defaults) = cfg.task_types.get(tag) {
                        priority = Some(defaults.priority.clone());
                        time = Some(defaults.time.clone());
                        break; // Use first matching task type
                    }
                }
            }

            Task {
                id: None,
                list_id: None,
                date: parsed_date.unwrap_or_else(|| Local::now().format("%Y-%m-%d").to_string()),
                priority,
                project: None,
                status: None,
                context: None,
                time,
                title: title.to_string(),
                list: list.to_string(),
                attachment_type: None,
                tags,
            }
        }
    }
    
    fn needs_triage(&self, force: bool, priority_only: bool, project_only: bool, time_only: bool, status_only: bool, context_only: bool) -> bool {
        if force {
            return true;
        }

        // If specific pass requested, check only that property
        if priority_only {
            return self.priority.is_none();
        }
        if project_only {
            return self.project.is_none();
        }
        if time_only {
            return self.time.is_none();
        }
        if status_only {
            return self.status.is_none();
        }
        if context_only {
            return self.context.is_none();
        }

        // Otherwise, needs triage if ANY of the core properties is missing
        // (status and context are optional extras, not required for basic triage)
        self.priority.is_none() || self.project.is_none() || self.time.is_none()
    }
}

fn get_config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not find home directory")?;
    Ok(home.join(".thegarden").join("config.json"))
}

fn create_default_config() -> Config {
    let mut projects = std::collections::HashMap::new();
    projects.insert("WORK".into(), "Work tasks".into());
    projects.insert("LIFE".into(), "Personal life tasks".into());
    projects.insert("SIDE".into(), "Side projects".into());

    let mut task_types = std::collections::HashMap::new();
    task_types.insert("FollowUp".into(), TaskTypeDefaults {
        priority: "P1".into(),
        time: "30m".into(),
    });
    task_types.insert("Plan".into(), TaskTypeDefaults {
        priority: "P1".into(),
        time: "1h".into(),
    });
    task_types.insert("DeepWork".into(), TaskTypeDefaults {
        priority: "P0".into(),
        time: "2h".into(),
    });

    Config {
        format: "[{date}][{priority}][{project}]{status}{context}{time} {title}".into(),
        date_format: "%Y-%m-%d".into(),
        projects,
        google_account: "your-email@gmail.com".into(),
        priorities: default_priorities(),
        time_options: default_time_options(),
        statuses: default_statuses(),
        contexts: vec!["@work".into(), "@home".into(), "@phone".into(), "@errands".into()],
        task_types,
        sync_throttle_minutes: 10,
        airtable: None,
        clickup: None,
    }
}

fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Create default config
        let config = create_default_config();
        let config_dir = config_path.parent().unwrap();
        fs::create_dir_all(config_dir)?;
        let contents = serde_json::to_string_pretty(&config)?;
        fs::write(&config_path, &contents)?;
        println!("{}", format!("Created default config at {}", config_path.display()).yellow());
        println!("{}", "Please edit it to set your google_account and customize settings.\n".yellow());
        return Ok(config);
    }

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to read {}", config_path.display()))?;
    let config: Config = serde_json::from_str(&contents)
        .context(format!("Failed to parse {}", config_path.display()))?;
    Ok(config)
}

fn read_single_key() -> Result<char> {
    enable_raw_mode()?;
    let key = loop {
        if let Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char(c) => break c,
                KeyCode::Enter => break '\n',
                KeyCode::Esc => break '\x1b',
                _ => continue,
            }
        }
    };
    disable_raw_mode()?;
    Ok(key)
}

fn should_sync(config: &Config, force: bool) -> Result<bool> {
    if force {
        return Ok(true);
    }
    
    let sync_manager = SyncManager::new()?;
    let last_query = sync_manager.get_state("last_query")?;
    
    if let Some(last) = last_query {
        if let Ok(last_time) = chrono::DateTime::parse_from_rfc3339(&last) {
            let now = Utc::now();
            let elapsed_duration = now.signed_duration_since(last_time);
            let elapsed_minutes = elapsed_duration.num_minutes();
            
            if elapsed_minutes < config.sync_throttle_minutes {
                println!("{}", format!("⚡ Using cache (synced {} min ago)", elapsed_minutes).dimmed());
                return Ok(false);
            }
        }
    }
    
    Ok(true)
}

fn update_last_query() -> Result<()> {
    let sync_manager = SyncManager::new()?;
    sync_manager.set_state("last_query", &Utc::now().to_rfc3339())?;
    Ok(())
}

fn sync_with_google(account: &str, force: bool) -> Result<()> {
    let sync_manager = SyncManager::new()?;
    let last_sync = sync_manager.get_last_sync()?;

    // If force or no last sync, do full sync
    let updated_min = if force {
        None
    } else {
        last_sync.clone()
    };

    if force {
        println!("{}", "🔄 Force syncing all tasks...".cyan());
    } else if let Some(ref last) = last_sync {
        println!("{}", format!("🔄 Syncing (updated since {})...", &last[..19]).dimmed());
    } else {
        println!("{}", "🔄 First sync - fetching all tasks...".cyan());
    }

    // Fetch lists
    let lists_output = Command::new("gog")
        .args(&["tasks", "lists", "list", "--account", account, "--json"])
        .output()
        .context("Failed to run gog command")?;

    if !lists_output.status.success() {
        anyhow::bail!("gog command failed");
    }

    let lists_json: serde_json::Value = serde_json::from_slice(&lists_output.stdout)?;
    let mut synced_count = 0;

    if let Some(tasklists) = lists_json["tasklists"].as_array() {
        for list in tasklists {
            let list_id = list["id"].as_str().unwrap_or("");
            
            // Build args with optional updatedMin filter
            let mut args = vec!["tasks", "list", list_id, "--account", account, "--json"];
            
            // Add updatedMin filter if we have a last sync time
            if let Some(ref min_time) = updated_min {
                args.push("--updated-min");
                args.push(min_time);
            }

            let tasks_output = Command::new("gog")
                .args(&args)
                .output()
                .context("Failed to get tasks")?;

            if tasks_output.status.success() {
                let tasks_json: serde_json::Value = serde_json::from_slice(&tasks_output.stdout)?;
                if let Some(tasks) = tasks_json["tasks"].as_array() {
                    for task in tasks {
                        let task_id = task["id"].as_str().unwrap_or("");
                        let title = task["title"].as_str().unwrap_or("");
                        let status = task["status"].as_str().unwrap_or("needsAction");
                        let updated = task["updated"].as_str().unwrap_or("");
                        let created = task["created"].as_str().map(|s| s.to_string());
                        let links = task["links"].as_array().map(|l| serde_json::to_string(l).ok()).flatten();

                        let cached = CachedTask {
                            id: task_id.to_string(),
                            unique_id: uuid::Uuid::new_v4().to_string(),
                            list_id: list_id.to_string(),
                            title: title.to_string(),
                            status: status.to_string(),
                            updated: updated.to_string(),
                            created,
                            links,
                            dirty: false, // From remote, not dirty
                            priority: None,
                            project: None,
                            context: None,
                            duration: None,
                            due_date: None,
                            start_date: None,
                            scheduled_date: None,
                            tags: None,
                            user_description: None,
                            taskgarden_description: String::new(), // Will be regenerated
                        };

                        // Use upsert_task_from_remote to skip dirty tasks
                        if sync_manager.upsert_task_from_remote(&cached)? {
                            synced_count += 1;
                        }
                    }
                }
            }
        }
    }

    // Update last sync timestamp
    let now = Utc::now().to_rfc3339();
    sync_manager.set_last_sync(&now)?;

    if synced_count == 0 {
        println!("{}", "✓ No changes".dimmed());
    } else {
        println!("{}", format!("✓ Synced {} tasks", synced_count).green());
    }
    
    Ok(())
}

fn sync_to_airtable(config: &Config) -> Result<()> {
    // Check if Airtable is enabled
    let airtable_config = match &config.airtable {
        Some(cfg) if cfg.enabled => cfg,
        Some(_) => {
            println!("{}", "ℹ️  Airtable sync disabled in config".yellow());
            return Ok(());
        }
        None => {
            println!("{}", "ℹ️  Airtable not configured. Add 'airtable' section to config.json".yellow());
            return Ok(());
        }
    };

    println!("{}", "📤 Pushing tasks to Airtable...".cyan());

    // Create Airtable client
    let client = AirtableClient::new(airtable::AirtableConfig {
        api_key: airtable_config.api_key.clone(),
        base_id: airtable_config.base_id.clone(),
        table_name: airtable_config.table_name.clone(),
    })?;

    // Create sync manager
    let sync_manager = SyncManager::new()?;
    let airtable_sync = AirtableSync::new(sync_manager, client);

    // Push to Airtable
    let stats = airtable_sync.push_to_airtable()?;

    if stats.created > 0 || stats.updated > 0 {
        println!(
            "{}",
            format!(
                "✓ Created: {}, Updated: {}, Errors: {}",
                stats.created, stats.updated, stats.errors
            )
            .green()
        );
    } else {
        println!("{}", "✓ No changes to push".dimmed());
    }

    if stats.errors > 0 {
        println!(
            "{}",
            format!("⚠️  {} tasks failed to sync", stats.errors).yellow()
        );
    }

    Ok(())
}

fn sync_to_clickup(config: &Config) -> Result<()> {
    // Check if ClickUp is enabled
    let clickup_config = match &config.clickup {
        Some(cfg) if cfg.enabled => cfg,
        Some(_) => {
            println!("{}", "ℹ️  ClickUp sync disabled in config".yellow());
            return Ok(());
        }
        None => {
            println!("{}", "ℹ️  ClickUp not configured. Add 'clickup' section to config.json".yellow());
            return Ok(());
        }
    };

    println!("{}", "📤 Pushing tasks to ClickUp...".cyan());

    // Get list mappings (or use single list_id for all tasks)
    let list_mappings = if let Some(ref mappings) = clickup_config.list_mappings {
        mappings.clone()
    } else if let Some(ref list_id) = clickup_config.list_id {
        // Single list mode: all tasks go to one ClickUp list
        // We'll create a universal mapping by getting all Google list IDs from cache
        let sync_manager = SyncManager::new()?;
        let all_tasks = sync_manager.get_all_cached_tasks()?;
        let mut map = std::collections::HashMap::new();
        
        // Map every Google list ID to the single ClickUp list
        for task in all_tasks {
            if !map.contains_key(&task.list_id) {
                map.insert(task.list_id.clone(), list_id.clone());
            }
        }
        
        map
    } else {
        anyhow::bail!("ClickUp config must have either list_mappings or list_id");
    };

    // Create ClickUp client
    let client = ClickUpClient::new(clickup::ClickUpConfig {
        api_token: clickup_config.api_token.clone(),
        list_id: clickup_config.list_id.clone().unwrap_or_default(),  // Not used anymore
    })?;

    // Create sync manager
    let sync_manager = SyncManager::new()?;
    let clickup_sync = ClickUpSync::new(sync_manager, client, list_mappings);

    // Push to ClickUp
    let stats = clickup_sync.push_to_clickup()?;

    if stats.created > 0 || stats.updated > 0 {
        println!(
            "{}",
            format!(
                "✓ Created: {}, Updated: {}, Errors: {}",
                stats.created, stats.updated, stats.errors
            )
            .green()
        );
    } else {
        println!("{}", "✓ No changes to push".dimmed());
    }

    if stats.errors > 0 {
        println!(
            "{}",
            format!("⚠️  {} tasks failed to sync", stats.errors).yellow()
        );
    }

    Ok(())
}

fn get_tasks_from_cache() -> Result<Vec<Task>> {
    let config = load_config()?;
    let sync_manager = SyncManager::new()?;
    let cached_tasks = sync_manager.get_all_cached_tasks()?;

    let mut tasks = Vec::new();
    for cached in cached_tasks {
        // Map list_id to list title (simplified for now)
        let list_title = "Tasks"; // TODO: Cache list names too

        let mut task = Task::parse_with_config(&cached.title, list_title, Some(&config));

        // Add ID and list_id from cache
        task.id = Some(cached.id);
        task.list_id = Some(cached.list_id);

        // Use Google's creation date as the task date (override parsed date)
        if let Some(ref created) = cached.created {
            // Parse RFC3339 timestamp and extract just the date part
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(created) {
                task.date = dt.format("%Y-%m-%d").to_string();
            }
        }

        // Detect attachment type from cached links
        if let Some(ref links_json) = cached.links {
            if let Ok(links) = serde_json::from_str::<Vec<serde_json::Value>>(links_json) {
                if let Some(first_link) = links.first() {
                    let link_type = first_link["type"].as_str().unwrap_or("");
                    let link_url = first_link["link"].as_str().unwrap_or("");

                    if link_type == "email" || link_url.contains("mail.google.com") {
                        task.attachment_type = Some("email".to_string());
                    } else if link_url.contains("docs.google.com") {
                        task.attachment_type = Some("doc".to_string());
                    } else if link_url.contains("drive.google.com") {
                        task.attachment_type = Some("drive".to_string());
                    }
                }
            }
        }

        tasks.push(task);
    }

    Ok(tasks)
}

fn update_task_in_google(account: &str, task: &Task) -> Result<()> {
    if task.id.is_none() || task.list_id.is_none() {
        anyhow::bail!("Task missing ID or list_id");
    }

    let formatted_title = task.format(&load_config()?);
    let task_id = task.id.as_ref().unwrap();
    let list_id = task.list_id.as_ref().unwrap();

    // Update via gog CLI
    let output = Command::new("gog")
        .args(&[
            "tasks", "update",
            list_id,
            task_id,
            "--title", &formatted_title,
            "--account", account,
        ])
        .output()
        .context("Failed to update task in Google Tasks")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to update task: {}", stderr);
    }

    // Update cache - preserve existing links and created date from cache
    let sync_manager = SyncManager::new()?;
    let existing = sync_manager.get_task_by_id(task_id)?;
    let existing_links = existing.as_ref().and_then(|t| t.links.clone());
    let existing_created = existing.as_ref().and_then(|t| t.created.clone());

    let cached = CachedTask {
        id: task_id.clone(),
        unique_id: existing.as_ref().map(|t| t.unique_id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        list_id: list_id.clone(),
        title: formatted_title,
        status: "needsAction".to_string(),
        updated: Utc::now().to_rfc3339(),
        created: existing_created,
        links: existing_links,
        dirty: false, // Just pushed to Google, so not dirty
        priority: existing.as_ref().and_then(|t| t.priority.clone()),
        project: existing.as_ref().and_then(|t| t.project.clone()),
        context: existing.as_ref().and_then(|t| t.context.clone()),
        duration: existing.as_ref().and_then(|t| t.duration.clone()),
        due_date: existing.as_ref().and_then(|t| t.due_date.clone()),
        start_date: existing.as_ref().and_then(|t| t.start_date.clone()),
        scheduled_date: existing.as_ref().and_then(|t| t.scheduled_date.clone()),
        tags: existing.as_ref().and_then(|t| t.tags.clone()),
        user_description: existing.as_ref().and_then(|t| t.user_description.clone()),
        taskgarden_description: String::new(), // Will be regenerated
    };
    sync_manager.upsert_task(&cached)?;

    Ok(())
}

/// Update task in local cache only (marks as dirty for later push)
fn update_task_locally(task: &Task) -> Result<()> {
    if task.id.is_none() || task.list_id.is_none() {
        anyhow::bail!("Task missing ID or list_id");
    }

    let formatted_title = task.format(&load_config()?);
    let task_id = task.id.as_ref().unwrap();
    let list_id = task.list_id.as_ref().unwrap();

    let sync_manager = SyncManager::new()?;

    // Preserve existing properties from cache
    let existing = sync_manager.get_task_by_id(task_id)?;
    let existing_links = existing.as_ref().and_then(|t| t.links.clone());
    let existing_created = existing.as_ref().and_then(|t| t.created.clone());

    let cached = CachedTask {
        id: task_id.clone(),
        unique_id: existing.as_ref().map(|t| t.unique_id.clone()).unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        list_id: list_id.clone(),
        title: formatted_title,
        status: "needsAction".to_string(),
        updated: Utc::now().to_rfc3339(),
        created: existing_created,
        links: existing_links,
        dirty: true, // Mark for later push
        priority: existing.as_ref().and_then(|t| t.priority.clone()),
        project: existing.as_ref().and_then(|t| t.project.clone()),
        context: existing.as_ref().and_then(|t| t.context.clone()),
        duration: existing.as_ref().and_then(|t| t.duration.clone()),
        due_date: existing.as_ref().and_then(|t| t.due_date.clone()),
        start_date: existing.as_ref().and_then(|t| t.start_date.clone()),
        scheduled_date: existing.as_ref().and_then(|t| t.scheduled_date.clone()),
        tags: existing.as_ref().and_then(|t| t.tags.clone()),
        user_description: existing.as_ref().and_then(|t| t.user_description.clone()),
        taskgarden_description: String::new(), // Will be regenerated
    };
    sync_manager.upsert_task_locally(&cached)?;

    Ok(())
}

/// Push all dirty tasks to Google
fn push_dirty_tasks_to_google(account: &str) -> Result<()> {
    let sync_manager = SyncManager::new()?;
    let dirty_tasks = sync_manager.get_dirty_tasks()?;

    if dirty_tasks.is_empty() {
        return Ok(());
    }

    let mut success_count = 0;
    let mut fail_count = 0;

    for task in &dirty_tasks {
        // Check if task is marked DONE - need to complete it
        let is_done = task.title.contains("[DONE]");

        // Build args
        let mut args = vec![
            "tasks".to_string(), "update".to_string(),
            task.list_id.clone(),
            task.id.clone(),
            "--title".to_string(), task.title.clone(),
            "--account".to_string(), account.to_string(),
        ];

        // If marked DONE, also set status to completed
        if is_done {
            args.push("--status".to_string());
            args.push("completed".to_string());
        }

        // Update via gog CLI
        let output = Command::new("gog")
            .args(&args)
            .output();

        match output {
            Ok(o) if o.status.success() => {
                // Mark as clean after successful push
                sync_manager.mark_task_clean(&task.id)?;
                success_count += 1;
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr);
                eprintln!("{}", format!("  ⚠ Failed to push task {}: {}", &task.id[..8], stderr).yellow());
                fail_count += 1;
            }
            Err(e) => {
                eprintln!("{}", format!("  ⚠ Failed to push task {}: {}", &task.id[..8], e).yellow());
                fail_count += 1;
            }
        }
    }

    if success_count > 0 {
        println!("{}", format!("✓ Pushed {} tasks to Google", success_count).green());
    }
    if fail_count > 0 {
        println!("{}", format!("⚠ {} tasks failed to push (will retry next time)", fail_count).yellow());
    }

    Ok(())
}

fn get_tasks_from_google(account: &str) -> Result<Vec<Task>> {
    let output = Command::new("gog")
        .args(&["tasks", "lists", "list", "--account", account, "--json"])
        .output()
        .context("Failed to run gog command")?;

    if !output.status.success() {
        anyhow::bail!("gog command failed");
    }

    let lists_json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    let mut all_tasks = Vec::new();

    if let Some(tasklists) = lists_json["tasklists"].as_array() {
        for list in tasklists {
            let list_id = list["id"].as_str().unwrap_or("");
            let list_title = list["title"].as_str().unwrap_or("");

            let tasks_output = Command::new("gog")
                .args(&["tasks", "list", list_id, "--account", account, "--json"])
                .output()
                .context("Failed to get tasks")?;

            if tasks_output.status.success() {
                let tasks_json: serde_json::Value = serde_json::from_slice(&tasks_output.stdout)?;
                if let Some(tasks) = tasks_json["tasks"].as_array() {
                    for task in tasks {
                        if let Some(title) = task["title"].as_str() {
                            let mut parsed_task = Task::parse(title, list_title);
                            
                            // Check for attachments/links
                            if let Some(links) = task["links"].as_array() {
                                if let Some(first_link) = links.first() {
                                    let link_type = first_link["type"].as_str().unwrap_or("");
                                    let link_url = first_link["link"].as_str().unwrap_or("");
                                    
                                    if link_type == "email" || link_url.contains("mail.google.com") {
                                        parsed_task.attachment_type = Some("email".to_string());
                                    } else if link_url.contains("docs.google.com") {
                                        parsed_task.attachment_type = Some("doc".to_string());
                                    } else if link_url.contains("drive.google.com") {
                                        parsed_task.attachment_type = Some("drive".to_string());
                                    }
                                }
                            }
                            
                            all_tasks.push(parsed_task);
                        }
                    }
                }
            }
        }
    }

    Ok(all_tasks)
}

fn cmd_triage(config: &Config, force: bool, priority_only: bool, project_only: bool, time_only: bool, status_only: bool, context_only: bool) -> Result<()> {
    println!("{}", "🌱 The Garden - Interactive Triage\n".green().bold());

    if force {
        println!("{}", "🔄 Force mode - re-triaging all tasks\n".yellow());
    }
    if priority_only {
        println!("{}", "📊 Priority-only mode\n".cyan());
    }
    if project_only {
        println!("{}", "🏷️  Project-only mode\n".cyan());
    }
    if time_only {
        println!("{}", "⏱️  Time-only mode\n".cyan());
    }
    if status_only {
        println!("{}", "📋 Status-only mode\n".cyan());
    }
    if context_only {
        println!("{}", "📍 Context-only mode\n".cyan());
    }

    let tasks = get_tasks_from_cache()?;
    let mut to_triage: Vec<Task> = tasks.into_iter()
        .filter(|t| t.needs_triage(force, priority_only, project_only, time_only, status_only, context_only))
        .collect();

    if to_triage.is_empty() {
        println!("{}", "✅ All tasks are triaged!".green());
        return Ok(());
    }

    let total = to_triage.len();
    println!("Found {} tasks needing triage\n", total.to_string().yellow().bold());

    // ═══ PASS 1: PRIORITIES ═══
    let should_do_priority = !project_only && !time_only && !status_only && !context_only;
    
    if should_do_priority {
        println!("\n{}", "═══ Pass 1: Priorities ═══\n".cyan().bold());
        
        // Undo history: stores (index, old_task)
        let mut undo_history: Vec<(usize, Task)> = Vec::new();
        let mut i = 0;

        while i < to_triage.len() {
            // Skip tasks that already have priority (unless force mode)
            if !force && to_triage[i].priority.is_some() {
                i += 1;
                continue;
            }

            // Display current task
            {
                let task = &to_triage[i];

                println!("\n{}", format!("Task {}/{}", i + 1, total).cyan());
                println!("{}", format!("ID: {}", task.short_id()).dimmed());

                // Display title with attachment indicator
                let display_title = if let Some(ref att_type) = task.attachment_type {
                    format!("{} ({})", task.title, att_type).yellow()
                } else {
                    task.title.yellow()
                };
                println!("{}", display_title);
                println!("{}", format!("(from: {})", task.list).dimmed());

                // Show options
                println!("\n{}", "Priority:".dimmed());
                for p in &config.priorities {
                    let key_str = if let Some(alt) = p.alt_key {
                        format!("{}/{}", p.key, alt)
                    } else {
                        p.key.to_string()
                    };
                    println!("  {} {} - {}", format!("{}.", key_str).cyan(), p.name, p.description);
                }
                println!("  {} Done (mark complete)", "x.".cyan());
                println!("  {} Skip", "s.".dimmed());
                if !undo_history.is_empty() {
                    println!("  {} Undo", "u.".yellow());
                }
                println!("  {} Quit (save progress)", "q.".red());
            }

            print!("\n> ");
            stdout().flush()?;

            let choice = read_single_key()?;
            println!("{}", choice); // Echo the keystroke

            let mut go_back = false;

            // Check if choice matches any priority key
            let matched_priority = config.priorities.iter().find(|p| {
                let c_lower = choice.to_ascii_lowercase();
                p.key.to_ascii_lowercase() == c_lower ||
                p.alt_key.map(|a| a.to_ascii_lowercase() == c_lower).unwrap_or(false)
            });

            if let Some(priority) = matched_priority {
                let old_task = to_triage[i].clone();
                to_triage[i].priority = Some(priority.name.clone());
                println!("{}", format!("  ✓ {}", priority.name).green());
                undo_history.push((i, old_task));

                print!("{}", "  💾 Saving...".dimmed());
                if let Err(e) = update_task_locally(&to_triage[i]) {
                    println!(" {}", format!("❌ Failed: {}", e).red());
                } else {
                    println!(" {}", "✓".green());
                }
            } else if choice == 'x' || choice == 'X' {
                let old_task = to_triage[i].clone();
                to_triage[i].priority = Some("DONE".to_string());
                println!("{}", "  ✓ Marked as Done".green());
                undo_history.push((i, old_task));

                print!("{}", "  💾 Saving...".dimmed());
                if let Err(e) = update_task_locally(&to_triage[i]) {
                    println!(" {}", format!("❌ Failed: {}", e).red());
                } else {
                    println!(" {}", "✓".green());
                }
            } else if choice == 's' || choice == 'S' || choice == ' ' {
                println!("{}", "  → Skipped".dimmed());
            } else if choice == 'u' || choice == 'U' {
                if let Some((prev_idx, prev_task)) = undo_history.pop() {
                    println!("{}", "  ↶ Undoing...".yellow());

                    // Restore previous task
                    to_triage[prev_idx] = prev_task.clone();

                    // Save restored state locally
                    print!("{}", "  💾 Restoring...".dimmed());
                    if let Err(e) = update_task_locally(&prev_task) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }

                    // Go back to that task
                    i = prev_idx;
                    go_back = true;
                } else {
                    println!("{}", "  ⚠ Nothing to undo".yellow());
                    continue; // Stay on current task
                }
            } else if choice == 'q' || choice == 'Q' || choice == '\x1b' {
                println!("\n{}", "Saving and exiting...".yellow());
                // Push dirty tasks before exiting
                println!("{}", "Pushing changes to Google...".dimmed());
                push_dirty_tasks_to_google(&config.google_account)?;
                return Ok(());
            } else {
                println!("{}", "  ⚠ Invalid input, skipping".yellow());
            }

            if !go_back {
                i += 1;
            }
        }
    }

    // Filter for pass 2: only tasks with priorities (exclude DONE and skipped)
    let mut for_project_pass: Vec<Task> = to_triage.into_iter()
        .filter(|t| t.priority.is_some() && t.priority.as_ref().unwrap() != "DONE")
        .collect();

    let should_do_project = !priority_only && !time_only && !status_only && !context_only;
    
    if should_do_project && !for_project_pass.is_empty() {
        let project_count = for_project_pass.len();
        
        // ═══ PASS 2: PROJECTS ═══
        println!("\n{}", "═══ Pass 2: Projects ═══\n".cyan().bold());
        let project_keys: Vec<String> = config.projects.keys().cloned().collect();
        
        let mut undo_history: Vec<(usize, Task)> = Vec::new();
        let mut i = 0;

        while i < for_project_pass.len() {
            // Skip tasks that already have a real project (not "---")
            if !force {
                if let Some(ref proj) = for_project_pass[i].project {
                    if proj != "---" {
                        i += 1;
                        continue;
                    }
                }
            }

            // Display current task
            {
                let task = &for_project_pass[i];

                println!("\n{}", format!("Task {}/{}", i + 1, project_count).cyan());
                println!("{}", format!("ID: {}", task.short_id()).dimmed());

                // Display title with attachment indicator
                let display_title = if let Some(ref att_type) = task.attachment_type {
                    format!("{} ({})", task.title, att_type).yellow()
                } else {
                    task.title.yellow()
                };
                println!("{}", display_title);
                println!("{}", format!("[{}]", task.priority.as_ref().unwrap()).green());

                // Show options
                let alt_keys = ['j', 'k', 'l', ';', 'a', 'f', 'g', 'h'];
                println!("\n{}", "Project:".dimmed());
                for (idx, proj) in project_keys.iter().enumerate() {
                    let alt = alt_keys.get(idx).map(|c| format!("/{}", c)).unwrap_or_default();
                    println!("  {} {}", format!("{}{}.", idx, alt).cyan(), proj);
                }
                println!("  {} Skip (no project)", "s.".dimmed());
                if !undo_history.is_empty() {
                    println!("  {} Undo", "u.".yellow());
                }
                println!("  {} Quit (save progress)", "q.".red());
            }

            print!("\n> ");
            stdout().flush()?;

            let choice = read_single_key()?;
            println!("{}", choice);

            let mut go_back = false;

            // Map alt keys to indices
            let alt_keys = ['j', 'k', 'l', ';', 'a', 'f', 'g', 'h'];
            let alt_index = alt_keys.iter().position(|&c| c == choice.to_ascii_lowercase());

            if choice == 'u' || choice == 'U' {
                if let Some((prev_idx, prev_task)) = undo_history.pop() {
                    println!("{}", "  ↶ Undoing...".yellow());

                    for_project_pass[prev_idx] = prev_task.clone();

                    print!("{}", "  💾 Restoring...".dimmed());
                    if let Err(e) = update_task_locally(&prev_task) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }

                    i = prev_idx;
                    go_back = true;
                } else {
                    println!("{}", "  ⚠ Nothing to undo".yellow());
                    continue;
                }
            } else if choice == 's' || choice == 'S' || choice == ' ' {
                println!("{}", "  → No project".dimmed());
            } else if choice == 'q' || choice == 'Q' || choice == '\x1b' {
                println!("\n{}", "Saving and exiting...".yellow());
                // Push dirty tasks before exiting
                println!("{}", "Pushing changes to Google...".dimmed());
                push_dirty_tasks_to_google(&config.google_account)?;
                return Ok(());
            } else {
                // Check for digit or alt key
                let num: Option<usize> = if choice.is_ascii_digit() {
                    choice.to_string().parse().ok()
                } else {
                    alt_index
                };

                if let Some(idx) = num {
                    if idx < project_keys.len() {
                        let old_task = for_project_pass[i].clone();
                        let proj = &project_keys[idx];

                        for_project_pass[i].project = Some(proj.clone());
                        println!("{}", format!("  ✓ {}", proj).green());

                        undo_history.push((i, old_task));

                        print!("{}", "  💾 Saving...".dimmed());
                        if let Err(e) = update_task_locally(&for_project_pass[i]) {
                            println!(" {}", format!("❌ Failed: {}", e).red());
                        } else {
                            println!(" {}", "✓".green());
                        }
                    } else {
                        println!("{}", "  ⚠ Invalid number, skipping".yellow());
                    }
                } else {
                    println!("{}", "  ⚠ Invalid input, skipping".yellow());
                }
            }

            if !go_back {
                i += 1;
            }
        }
    }

    let should_do_time = !priority_only && !project_only && !status_only && !context_only;
    
    if should_do_time && !for_project_pass.is_empty() {
        let time_count = for_project_pass.len();
        
        // ═══ PASS 3: TIME ESTIMATES ═══
        println!("\n{}", "═══ Pass 3: Time Estimates ═══\n".cyan().bold());

        let mut undo_history: Vec<(usize, Task)> = Vec::new();
        let mut i = 0;

        while i < for_project_pass.len() {
            // Skip tasks that already have a time estimate
            if !force && for_project_pass[i].time.is_some() {
                i += 1;
                continue;
            }

            // Display current task
            {
                let task = &for_project_pass[i];

                println!("\n{}", format!("Task {}/{}", i + 1, time_count).cyan());
                println!("{}", format!("ID: {}", task.short_id()).dimmed());

                // Display title with attachment indicator
                let display_title = if let Some(ref att_type) = task.attachment_type {
                    format!("{} ({})", task.title, att_type).yellow()
                } else {
                    task.title.yellow()
                };
                println!("{}", display_title);
                println!("{}", format!("[{}][{}]",
                    task.priority.as_ref().unwrap(),
                    task.project.as_ref().unwrap_or(&"---".to_string())
                ).green());

                // Show options
                println!("\n{}", "Time estimate:".dimmed());
                for time_opt in &config.time_options {
                    let key_str = if let Some(alt) = time_opt.alt_key {
                        format!("{}/{}", time_opt.key, alt)
                    } else {
                        time_opt.key.to_string()
                    };
                    println!("  {} {}", format!("{}.", key_str).cyan(), time_opt.label);
                }
                println!("  {} Skip", "s.".dimmed());
                if !undo_history.is_empty() {
                    println!("  {} Undo", "u.".yellow());
                }
                println!("  {} Quit (save progress)", "q.".red());
            }

            print!("\n> ");
            stdout().flush()?;

            let choice = read_single_key()?;
            println!("{}", choice);

            let mut go_back = false;

            if choice == 'u' || choice == 'U' {
                if let Some((prev_idx, prev_task)) = undo_history.pop() {
                    println!("{}", "  ↶ Undoing...".yellow());

                    for_project_pass[prev_idx] = prev_task.clone();

                    print!("{}", "  💾 Restoring...".dimmed());
                    if let Err(e) = update_task_locally(&prev_task) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }

                    i = prev_idx;
                    go_back = true;
                } else {
                    println!("{}", "  ⚠ Nothing to undo".yellow());
                    continue;
                }
            } else if choice == 's' || choice == 'S' || choice == ' ' {
                println!("{}", "  → No estimate".dimmed());
            } else if choice == 'q' || choice == 'Q' || choice == '\x1b' {
                println!("\n{}", "Saving and exiting...".yellow());
                // Push dirty tasks before exiting
                println!("{}", "Pushing changes to Google...".dimmed());
                push_dirty_tasks_to_google(&config.google_account)?;
                return Ok(());
            } else {
                // Check if choice matches any time option key or alt_key
                let matched_time = config.time_options.iter().find(|t| {
                    let c_lower = choice.to_ascii_lowercase();
                    t.key.to_ascii_lowercase() == c_lower ||
                    t.alt_key.map(|a| a.to_ascii_lowercase() == c_lower).unwrap_or(false)
                });

                if let Some(time_opt) = matched_time {
                    let old_task = for_project_pass[i].clone();
                    for_project_pass[i].time = Some(time_opt.label.clone());
                    println!("{}", format!("  ✓ {}", time_opt.label).green());

                    undo_history.push((i, old_task));

                    print!("{}", "  💾 Saving...".dimmed());
                    if let Err(e) = update_task_locally(&for_project_pass[i]) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }
                } else {
                    println!("{}", "  ⚠ Invalid input, skipping".yellow());
                }
            }

            if !go_back {
                i += 1;
            }
        }
    }

    // ═══ PASS 4: STATUS ═══
    let should_do_status = status_only || (!priority_only && !project_only && !time_only && !context_only);

    if should_do_status && !for_project_pass.is_empty() && !config.statuses.is_empty() {
        let status_count = for_project_pass.len();

        println!("\n{}", "═══ Pass 4: Status ═══\n".cyan().bold());

        let mut undo_history: Vec<(usize, Task)> = Vec::new();
        let mut i = 0;

        while i < for_project_pass.len() {
            // Skip tasks that already have status (unless force or status_only mode)
            if !force && !status_only && for_project_pass[i].status.is_some() {
                i += 1;
                continue;
            }

            // Display current task
            {
                let task = &for_project_pass[i];

                println!("\n{}", format!("Task {}/{}", i + 1, status_count).cyan());
                println!("{}", format!("ID: {}", task.short_id()).dimmed());

                let display_title = if let Some(ref att_type) = task.attachment_type {
                    format!("{} ({})", task.title, att_type).yellow()
                } else {
                    task.title.yellow()
                };
                println!("{}", display_title);
                println!("{}", format!("[{}][{}]{}",
                    task.priority.as_ref().unwrap_or(&"--".to_string()),
                    task.project.as_ref().unwrap_or(&"---".to_string()),
                    task.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default()
                ).green());

                // Show options
                println!("\n{}", "Status:".dimmed());
                for status_opt in &config.statuses {
                    let key_str = if let Some(alt) = status_opt.alt_key {
                        format!("{}/{}", status_opt.key, alt)
                    } else {
                        status_opt.key.to_string()
                    };
                    println!("  {} {} - {}", format!("{}.", key_str).cyan(), status_opt.name, status_opt.description);
                }
                println!("  {} Skip", "s.".dimmed());
                if !undo_history.is_empty() {
                    println!("  {} Undo", "u.".yellow());
                }
                println!("  {} Quit (save progress)", "q.".red());
            }

            print!("\n> ");
            stdout().flush()?;

            let choice = read_single_key()?;
            println!("{}", choice);

            let mut go_back = false;

            if choice == 'u' || choice == 'U' {
                if let Some((prev_idx, prev_task)) = undo_history.pop() {
                    println!("{}", "  ↶ Undoing...".yellow());
                    for_project_pass[prev_idx] = prev_task.clone();

                    print!("{}", "  💾 Restoring...".dimmed());
                    if let Err(e) = update_task_locally(&prev_task) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }

                    i = prev_idx;
                    go_back = true;
                } else {
                    println!("{}", "  ⚠ Nothing to undo".yellow());
                    continue;
                }
            } else if choice == 's' || choice == 'S' || choice == ' ' {
                println!("{}", "  → No status".dimmed());
            } else if choice == 'q' || choice == 'Q' || choice == '\x1b' {
                println!("\n{}", "Saving and exiting...".yellow());
                println!("{}", "Pushing changes to Google...".dimmed());
                push_dirty_tasks_to_google(&config.google_account)?;
                return Ok(());
            } else {
                // Check if choice matches any status key or alt_key
                let matched_status = config.statuses.iter().find(|s| {
                    let c_lower = choice.to_ascii_lowercase();
                    s.key.to_ascii_lowercase() == c_lower ||
                    s.alt_key.map(|a| a.to_ascii_lowercase() == c_lower).unwrap_or(false)
                });

                if let Some(status_opt) = matched_status {
                    let old_task = for_project_pass[i].clone();
                    for_project_pass[i].status = Some(status_opt.name.clone());
                    println!("{}", format!("  ✓ {}", status_opt.name).green());

                    undo_history.push((i, old_task));

                    print!("{}", "  💾 Saving...".dimmed());
                    if let Err(e) = update_task_locally(&for_project_pass[i]) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }
                } else {
                    println!("{}", "  ⚠ Invalid input, skipping".yellow());
                }
            }

            if !go_back {
                i += 1;
            }
        }
    }

    // ═══ PASS 5: CONTEXT ═══
    let should_do_context = context_only || (!priority_only && !project_only && !time_only && !status_only);

    if should_do_context && !for_project_pass.is_empty() && !config.contexts.is_empty() {
        let context_count = for_project_pass.len();

        println!("\n{}", "═══ Pass 5: Context ═══\n".cyan().bold());

        let alt_keys = ['j', 'k', 'l', ';', 'a', 'f', 'g', 'h'];
        let mut undo_history: Vec<(usize, Task)> = Vec::new();
        let mut i = 0;

        while i < for_project_pass.len() {
            // Skip tasks that already have context (unless force or context_only mode)
            if !force && !context_only && for_project_pass[i].context.is_some() {
                i += 1;
                continue;
            }

            // Display current task
            {
                let task = &for_project_pass[i];

                println!("\n{}", format!("Task {}/{}", i + 1, context_count).cyan());
                println!("{}", format!("ID: {}", task.short_id()).dimmed());

                let display_title = if let Some(ref att_type) = task.attachment_type {
                    format!("{} ({})", task.title, att_type).yellow()
                } else {
                    task.title.yellow()
                };
                println!("{}", display_title);
                println!("{}", format!("[{}][{}]{}{}",
                    task.priority.as_ref().unwrap_or(&"--".to_string()),
                    task.project.as_ref().unwrap_or(&"---".to_string()),
                    task.status.as_ref().map(|s| format!("[{}]", s)).unwrap_or_default(),
                    task.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default()
                ).green());

                // Show options
                println!("\n{}", "Context:".dimmed());
                for (idx, ctx) in config.contexts.iter().enumerate() {
                    let alt = alt_keys.get(idx).map(|c| format!("/{}", c)).unwrap_or_default();
                    println!("  {} {}", format!("{}{}.", idx, alt).cyan(), ctx);
                }
                println!("  {} Skip (no context)", "s.".dimmed());
                if !undo_history.is_empty() {
                    println!("  {} Undo", "u.".yellow());
                }
                println!("  {} Quit (save progress)", "q.".red());
            }

            print!("\n> ");
            stdout().flush()?;

            let choice = read_single_key()?;
            println!("{}", choice);

            let mut go_back = false;

            // Map alt keys to indices
            let alt_index = alt_keys.iter().position(|&c| c == choice.to_ascii_lowercase());

            if choice == 'u' || choice == 'U' {
                if let Some((prev_idx, prev_task)) = undo_history.pop() {
                    println!("{}", "  ↶ Undoing...".yellow());
                    for_project_pass[prev_idx] = prev_task.clone();

                    print!("{}", "  💾 Restoring...".dimmed());
                    if let Err(e) = update_task_locally(&prev_task) {
                        println!(" {}", format!("❌ Failed: {}", e).red());
                    } else {
                        println!(" {}", "✓".green());
                    }

                    i = prev_idx;
                    go_back = true;
                } else {
                    println!("{}", "  ⚠ Nothing to undo".yellow());
                    continue;
                }
            } else if choice == 's' || choice == 'S' || choice == ' ' {
                println!("{}", "  → No context".dimmed());
            } else if choice == 'q' || choice == 'Q' || choice == '\x1b' {
                println!("\n{}", "Saving and exiting...".yellow());
                println!("{}", "Pushing changes to Google...".dimmed());
                push_dirty_tasks_to_google(&config.google_account)?;
                return Ok(());
            } else {
                // Check for digit or alt key
                let num: Option<usize> = if choice.is_ascii_digit() {
                    choice.to_string().parse().ok()
                } else {
                    alt_index
                };

                if let Some(idx) = num {
                    if idx < config.contexts.len() {
                        let old_task = for_project_pass[i].clone();
                        let ctx = &config.contexts[idx];

                        for_project_pass[i].context = Some(ctx.clone());
                        println!("{}", format!("  ✓ {}", ctx).green());

                        undo_history.push((i, old_task));

                        print!("{}", "  💾 Saving...".dimmed());
                        if let Err(e) = update_task_locally(&for_project_pass[i]) {
                            println!(" {}", format!("❌ Failed: {}", e).red());
                        } else {
                            println!(" {}", "✓".green());
                        }
                    } else {
                        println!("{}", "  ⚠ Invalid number, skipping".yellow());
                    }
                } else {
                    println!("{}", "  ⚠ Invalid input, skipping".yellow());
                }
            }

            if !go_back {
                i += 1;
            }
        }
    }

    // ═══ SUMMARY ═══
    // Push all dirty tasks to Google at the end
    println!("\n{}", "Pushing changes to Google...".dimmed());
    push_dirty_tasks_to_google(&config.google_account)?;

    println!("\n{}", "═══ Triage Complete! ═══\n".green().bold());
    println!("{}", "All changes have been saved to Google Tasks.".green());

    Ok(())
}

#[derive(Debug, Clone)]
struct TimeBlock {
    start: chrono::DateTime<Local>,
    end: chrono::DateTime<Local>,
    duration_minutes: i64,
}

impl TimeBlock {
    fn format_time_range(&self) -> String {
        format!("{}-{}", 
            self.start.format("%I:%M %p").to_string().trim_start_matches('0'),
            self.end.format("%I:%M %p").to_string().trim_start_matches('0')
        )
    }
    
    fn duration_hours(&self) -> f64 {
        self.duration_minutes as f64 / 60.0
    }
}

#[derive(Debug)]
struct ScheduleSuggestion {
    task: Task,
    block: TimeBlock,
}

fn cmd_schedule(config: &Config, week: bool, auto: bool) -> Result<()> {
    let today = Local::now();
    
    // Determine date range
    let (start_date, end_date, range_label) = if week {
        let days_since_monday = today.weekday().num_days_from_monday();
        let start_of_week = (today - chrono::Duration::days(days_since_monday as i64)).date_naive();
        let end_of_week = start_of_week + chrono::Duration::days(6);
        (start_of_week, end_of_week, "This Week".to_string())
    } else {
        let today_date = today.date_naive();
        (today_date, today_date, format!("Today ({})", today.format("%A, %b %d")))
    };
    
    let start_str = start_date.format("%Y-%m-%d").to_string();
    let end_str = end_date.format("%Y-%m-%d").to_string();
    
    println!("{}", format!("🗓️  Schedule Suggestions for {}\n", range_label).cyan().bold());
    
    // Fetch calendar events
    let events_output = Command::new("gog")
        .args(&[
            "calendar", "events",
            "--from", &start_str,
            "--to", &end_str,
            "--account", &config.google_account,
            "--json",
        ])
        .output();
    
    if events_output.is_err() {
        println!("{}", "⚠️  Could not fetch calendar events. Make sure 'gog' CLI is installed.".yellow());
        return Ok(());
    }
    
    let output = events_output?;
    if !output.status.success() {
        println!("{}", "⚠️  Failed to fetch calendar events.".yellow());
        return Ok(());
    }
    
    // Parse events
    let mut events_by_day: std::collections::BTreeMap<String, Vec<(chrono::DateTime<Local>, chrono::DateTime<Local>, String)>> = std::collections::BTreeMap::new();
    
    if let Ok(events_json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
        if let Some(events) = events_json["events"].as_array() {
            for event in events {
                if let (Some(summary), Some(start_str), Some(end_str)) = (
                    event["summary"].as_str(),
                    event["start"].as_str(),
                    event["end"].as_str(),
                ) {
                    if let (Ok(start_dt), Ok(end_dt)) = (
                        chrono::DateTime::parse_from_rfc3339(start_str),
                        chrono::DateTime::parse_from_rfc3339(end_str),
                    ) {
                        let start_local: chrono::DateTime<Local> = start_dt.into();
                        let end_local: chrono::DateTime<Local> = end_dt.into();
                        let date_key = start_local.format("%Y-%m-%d").to_string();
                        
                        events_by_day.entry(date_key).or_insert_with(Vec::new)
                            .push((start_local, end_local, summary.to_string()));
                    }
                }
            }
        }
    }
    
    // Get tasks that need scheduling (P0 and P1 with time estimates)
    let tasks = get_tasks_from_cache()?;
    let schedulable_tasks: Vec<Task> = tasks.into_iter()
        .filter(|t| {
            // Must have time estimate
            if t.time.is_none() {
                return false;
            }
            
            // Must be P0 or P1
            if let Some(ref priority) = t.priority {
                if priority != "P0" && priority != "P1" {
                    return false;
                }
            } else {
                return false;
            }
            
            // Must be in date range
            t.date >= start_str && t.date <= end_str
        })
        .collect();
    
    if schedulable_tasks.is_empty() {
        println!("{}", "✓ No tasks with time estimates to schedule!".green());
        return Ok(());
    }
    
    // Process each day
    let mut all_suggestions: Vec<ScheduleSuggestion> = Vec::new();
    
    let mut current_date = start_date;
    while current_date <= end_date {
        let date_str = current_date.format("%Y-%m-%d").to_string();
        let date_label = current_date.format("%A, %b %d").to_string();
        
        // Skip past days
        if current_date < today.date_naive() {
            current_date = current_date + chrono::Duration::days(1);
            continue;
        }
        
        // Get events for this day
        let day_events = events_by_day.get(&date_str).cloned().unwrap_or_default();
        
        // Calculate free blocks for this day
        let free_blocks = calculate_free_blocks(&current_date, &day_events);
        
        // Get tasks for this day
        let day_tasks: Vec<&Task> = schedulable_tasks.iter()
            .filter(|t| t.date == date_str)
            .collect();
        
        if !free_blocks.is_empty() || !day_tasks.is_empty() {
            // Show day header
            if current_date == today.date_naive() {
                println!("{}", format!("{}:", date_label).green().bold());
            } else {
                println!("{}", format!("{}:", date_label).cyan());
            }
            
            // Show free blocks
            if !free_blocks.is_empty() {
                println!("\n{}", "  Free blocks available:".dimmed());
                for block in &free_blocks {
                    println!("    {} ({:.1}h free)", 
                        block.format_time_range().cyan(), 
                        block.duration_hours()
                    );
                }
            }
            
            // Schedule tasks into free blocks
            let suggestions = schedule_tasks_into_blocks(day_tasks.clone(), &free_blocks);
            
            if !suggestions.is_empty() {
                println!("\n{}", "  Suggested schedule:".dimmed());
                for suggestion in &suggestions {
                    let priority_emoji = match suggestion.task.priority.as_deref() {
                        Some("P0") => "🔴",
                        Some("P1") => "🟡",
                        _ => "⚪",
                    };
                    
                    let project_str = suggestion.task.project.as_ref()
                        .map(|p| format!("[{}]", p))
                        .unwrap_or_else(|| "[---]".to_string());
                    
                    let tags_str = if suggestion.task.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", suggestion.task.tags.iter()
                            .map(|t| format!("#{}", t))
                            .collect::<Vec<_>>()
                            .join(" "))
                    };
                    
                    println!("    {} {} → {} [{}]{}{}{} {}{}", 
                        "✓".green(),
                        suggestion.block.format_time_range().cyan(),
                        priority_emoji,
                        suggestion.task.priority.as_deref().unwrap_or("--"),
                        project_str.yellow(),
                        format!("{{{}}}", suggestion.task.time.as_ref().unwrap()).cyan(),
                        tags_str.dimmed(),
                        suggestion.task.title,
                        if suggestion.task.date < today.format("%Y-%m-%d").to_string() {
                            " ⚠️".to_string()
                        } else {
                            String::new()
                        }
                    );
                }
                
                all_suggestions.extend(suggestions);
            } else if !day_tasks.is_empty() {
                println!("\n{}", format!("  ⚠️  {} tasks but no free blocks available", day_tasks.len()).yellow());
                for task in day_tasks {
                    println!("    - [{}]{} {} ({})", 
                        task.priority.as_deref().unwrap_or("--"),
                        task.project.as_ref().map(|p| format!("[{}]", p)).unwrap_or_default(),
                        task.title,
                        task.time.as_ref().unwrap()
                    );
                }
            }
            
            println!();
        }
        
        current_date = current_date + chrono::Duration::days(1);
    }
    
    // Offer to create calendar events
    if !all_suggestions.is_empty() && !auto {
        print!("\n{}", "Block these on your calendar? (y/n): ".bold());
        stdout().flush()?;
        
        let choice = read_single_key()?;
        println!("{}", choice);
        
        if choice == 'y' || choice == 'Y' {
            create_calendar_blocks(config, &all_suggestions)?;
        } else {
            println!("{}", "  Skipped calendar blocking.".dimmed());
        }
    } else if !all_suggestions.is_empty() && auto {
        create_calendar_blocks(config, &all_suggestions)?;
    }
    
    Ok(())
}

fn calculate_free_blocks(date: &chrono::NaiveDate, events: &[(chrono::DateTime<Local>, chrono::DateTime<Local>, String)]) -> Vec<TimeBlock> {
    let mut free_blocks = Vec::new();
    
    // Define working hours: 8 AM to 6 PM
    let work_start = date.and_hms_opt(8, 0, 0).unwrap();
    let work_end = date.and_hms_opt(18, 0, 0).unwrap();
    
    let work_start_dt = Local.from_local_datetime(&work_start).unwrap();
    let work_end_dt = Local.from_local_datetime(&work_end).unwrap();
    
    // Sort events by start time
    let mut sorted_events = events.to_vec();
    sorted_events.sort_by_key(|e| e.0);
    
    let mut current_time = work_start_dt;
    
    for (event_start, event_end, _) in sorted_events {
        // If there's a gap before this event
        if event_start > current_time {
            let gap_duration = event_start.signed_duration_since(current_time);
            let gap_minutes = gap_duration.num_minutes();
            
            // Only consider gaps of 15+ minutes
            if gap_minutes >= 15 {
                free_blocks.push(TimeBlock {
                    start: current_time,
                    end: event_start,
                    duration_minutes: gap_minutes,
                });
            }
        }
        
        // Move current time to end of event
        current_time = event_end.max(current_time);
    }
    
    // Check for free time after last event
    if current_time < work_end_dt {
        let gap_duration = work_end_dt.signed_duration_since(current_time);
        let gap_minutes = gap_duration.num_minutes();
        if gap_minutes >= 15 {
            free_blocks.push(TimeBlock {
                start: current_time,
                end: work_end_dt,
                duration_minutes: gap_minutes,
            });
        }
    }
    
    free_blocks
}

fn schedule_tasks_into_blocks(tasks: Vec<&Task>, free_blocks: &[TimeBlock]) -> Vec<ScheduleSuggestion> {
    let mut suggestions = Vec::new();
    let mut remaining_blocks = free_blocks.to_vec();
    
    // Sort tasks by priority (P0 first) then by duration (longest first)
    let mut sorted_tasks = tasks.to_vec();
    sorted_tasks.sort_by(|a, b| {
        let a_pri = a.priority.as_deref().unwrap_or("P9");
        let b_pri = b.priority.as_deref().unwrap_or("P9");
        
        if a_pri != b_pri {
            a_pri.cmp(b_pri)
        } else {
            // Longer tasks first
            let a_mins = parse_time_to_minutes(a.time.as_ref().unwrap());
            let b_mins = parse_time_to_minutes(b.time.as_ref().unwrap());
            b_mins.cmp(&a_mins)
        }
    });
    
    // Greedy scheduling: try to fit each task into available blocks
    for task in sorted_tasks {
        let task_minutes = parse_time_to_minutes(task.time.as_ref().unwrap());
        
        // Find first block that can fit this task
        if let Some((block_idx, block)) = remaining_blocks.iter()
            .enumerate()
            .find(|(_, b)| b.duration_minutes >= task_minutes as i64)
        {
            // Create suggestion
            let scheduled_block = TimeBlock {
                start: block.start,
                end: block.start + chrono::Duration::minutes(task_minutes as i64),
                duration_minutes: task_minutes as i64,
            };
            
            suggestions.push(ScheduleSuggestion {
                task: (*task).clone(),
                block: scheduled_block.clone(),
            });
            
            // Update remaining block
            let new_block_start = scheduled_block.end;
            let new_duration = (block.end - new_block_start).num_minutes();
            
            if new_duration >= 15 {
                // Replace with smaller block
                remaining_blocks[block_idx] = TimeBlock {
                    start: new_block_start,
                    end: block.end,
                    duration_minutes: new_duration,
                };
            } else {
                // Remove block entirely
                remaining_blocks.remove(block_idx);
            }
        }
    }
    
    suggestions
}

fn create_calendar_blocks(config: &Config, suggestions: &[ScheduleSuggestion]) -> Result<()> {
    println!("\n{}", "  Creating calendar events...".dimmed());
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    for suggestion in suggestions {
        let title = format!("[{}] {}", 
            suggestion.task.priority.as_deref().unwrap_or("--"),
            suggestion.task.title
        );
        
        let start_str = suggestion.block.start.to_rfc3339();
        let end_str = suggestion.block.end.to_rfc3339();
        
        // Create calendar event via gog
        let output = Command::new("gog")
            .args(&[
                "calendar", "events", "create",
                "--title", &title,
                "--start", &start_str,
                "--end", &end_str,
                "--account", &config.google_account,
            ])
            .output();
        
        match output {
            Ok(o) if o.status.success() => {
                success_count += 1;
                println!("    {} {}", "✓".green(), title.dimmed());
            }
            _ => {
                fail_count += 1;
                println!("    {} {}", "✗".red(), title.dimmed());
            }
        }
    }
    
    println!();
    if success_count > 0 {
        println!("{}", format!("  ✓ Created {} calendar events", success_count).green());
    }
    if fail_count > 0 {
        println!("{}", format!("  ⚠️  {} events failed to create", fail_count).yellow());
    }
    
    Ok(())
}

fn cmd_plan(config: &Config) -> Result<()> {
    use std::collections::BTreeMap;
    
    let today = Local::now();
    
    // Get start of week (Monday)
    let days_since_monday = today.weekday().num_days_from_monday();
    let start_of_week = (today - chrono::Duration::days(days_since_monday as i64)).date_naive();
    
    // Get end of week (Sunday)
    let end_of_week = start_of_week + chrono::Duration::days(6);
    
    let start_str = start_of_week.format("%Y-%m-%d").to_string();
    let end_str = end_of_week.format("%Y-%m-%d").to_string();
    
    println!("{}", "🌱 This Week Plan\n".cyan().bold());
    
    // Fetch calendar events
    let events_output = Command::new("gog")
        .args(&[
            "calendar", "events",
            "--from", &start_str,
            "--to", &end_str,
            "--account", &config.google_account,
            "--json",
        ])
        .output();
    
    let mut events_by_day: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    
    if let Ok(output) = events_output {
        if output.status.success() {
            if let Ok(events_json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                if let Some(events) = events_json["events"].as_array() {
                    for event in events {
                        if let (Some(summary), Some(start)) = (
                            event["summary"].as_str(),
                            event["start"].as_str(),
                        ) {
                            // Parse start time and extract date and time
                            if let Ok(start_dt) = chrono::DateTime::parse_from_rfc3339(start) {
                                let date_str = start_dt.format("%Y-%m-%d").to_string();
                                let time_str = start_dt.format("%I:%M %p").to_string();
                                
                                // Try to get end time too
                                let end_str = if let Some(end) = event["end"].as_str() {
                                    if let Ok(end_dt) = chrono::DateTime::parse_from_rfc3339(end) {
                                        format!("-{}", end_dt.format("%I:%M %p"))
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                };
                                
                                let event_str = format!("📅 {}{}: {}", time_str, end_str, summary);
                                events_by_day.entry(date_str).or_insert_with(Vec::new).push((time_str, event_str));
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Get tasks for the week
    let tasks = get_tasks_from_cache()?;
    let week_tasks: Vec<&Task> = tasks.iter()
        .filter(|t| t.date >= start_str && t.date <= end_str)
        .collect();
    
    // Group tasks by date
    let mut tasks_by_day: BTreeMap<String, Vec<&Task>> = BTreeMap::new();
    for task in week_tasks {
        tasks_by_day.entry(task.date.clone()).or_insert_with(Vec::new).push(task);
    }
    
    // Combine and display
    let mut all_dates: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for date in events_by_day.keys() {
        all_dates.insert(date.clone());
    }
    for date in tasks_by_day.keys() {
        all_dates.insert(date.clone());
    }
    
    for date in all_dates {
        // Format date as "Monday, Jan 20"
        if let Ok(date_parsed) = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d") {
            let date_label = date_parsed.format("%A, %b %d").to_string();
            
            // Highlight today
            if date == today.format("%Y-%m-%d").to_string() {
                println!("{}", format!("{}:", date_label).green().bold());
            } else {
                println!("{}", format!("{}:", date_label).cyan());
            }
            
            // Show events first (sorted by time)
            if let Some(events) = events_by_day.get(&date) {
                let mut sorted_events = events.clone();
                sorted_events.sort_by(|a, b| a.0.cmp(&b.0));
                for (_, event) in sorted_events {
                    println!("  {}", event);
                }
            }
            
            // Show tasks (sorted by priority)
            if let Some(tasks) = tasks_by_day.get(&date) {
                let mut sorted_tasks = tasks.to_vec();
                sorted_tasks.sort_by(|a, b| {
                    let a_pri = a.priority.as_deref().unwrap_or("P9");
                    let b_pri = b.priority.as_deref().unwrap_or("P9");
                    a_pri.cmp(b_pri)
                });
                
                for task in sorted_tasks {
                    let priority_emoji = match task.priority.as_deref() {
                        Some("P0") => "🔴",
                        Some("P1") => "🟡",
                        Some("P2") => "🟢",
                        Some("P3") => "🔵",
                        _ => "⚪",
                    };
                    
                    let time_str = task.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default();
                    let project_str = task.project.as_ref().map(|p| format!("[{}]", p)).unwrap_or_else(|| "[---]".to_string());
                    let tags_str = if task.tags.is_empty() {
                        String::new()
                    } else {
                        format!(" {}", task.tags.iter().map(|t| format!("#{}", t)).collect::<Vec<_>>().join(" "))
                    };
                    
                    println!("  {} [{}]{}{}{}{}", 
                        priority_emoji,
                        task.priority.as_deref().unwrap_or("--"),
                        project_str.yellow(),
                        time_str.cyan(),
                        tags_str.dimmed(),
                        format!(" {}", task.title)
                    );
                }
            }
            
            println!(); // Blank line between days
        }
    }
    
    Ok(())
}

fn cmd_focus(_config: &Config) -> Result<()> {
    let tasks = get_tasks_from_cache()?;
    let today = Local::now();
    let today_str = today.format("%Y-%m-%d").to_string();

    // Filter for critical tasks:
    // - P0 (any date)
    // - P1 that are overdue or due today
    let critical: Vec<&Task> = tasks.iter()
        .filter(|t| {
            if let Some(ref priority) = t.priority {
                if priority == "P0" {
                    return true;
                }
                if priority == "P1" {
                    return t.date <= today_str;
                }
            }
            false
        })
        .collect();

    if critical.is_empty() {
        println!("{}", "🎯 Focus - No critical tasks! You're clear! ✨".green().bold());
        return Ok(());
    }

    // Separate P0 and P1 tasks
    let p0_tasks: Vec<&&Task> = critical.iter().filter(|t| t.priority.as_deref() == Some("P0")).collect();
    let p1_tasks: Vec<&&Task> = critical.iter().filter(|t| t.priority.as_deref() == Some("P1")).collect();

    println!("{}", "🎯 Focus - Critical Tasks\n".cyan().bold());

    let mut total_minutes = 0;

    if !p0_tasks.is_empty() {
        println!("{}", "🔴 P0 (do first):".red().bold());
        for (idx, task) in p0_tasks.iter().enumerate() {
            let time_str = task.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default();
            let project_str = task.project.as_ref().map(|p| format!("[{}]", p)).unwrap_or_default();
            println!("  {}. {}{} {}", idx + 1, project_str.yellow(), time_str.cyan(), task.title);
            
            // Add to total
            if let Some(ref time) = task.time {
                total_minutes += parse_time_to_minutes(time);
            }
        }
        println!();
    }

    if !p1_tasks.is_empty() {
        println!("{}", "🟡 P1 (due today):".yellow().bold());
        for (idx, task) in p1_tasks.iter().enumerate() {
            let time_str = task.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default();
            let project_str = task.project.as_ref().map(|p| format!("[{}]", p)).unwrap_or_default();
            let overdue_mark = if task.date < today_str { " ⚠️" } else { "" };
            println!("  {}. {}{} {}{}", p0_tasks.len() + idx + 1, project_str.yellow(), time_str.cyan(), task.title, overdue_mark);
            
            // Add to total
            if let Some(ref time) = task.time {
                total_minutes += parse_time_to_minutes(time);
            }
        }
        println!();
    }

    // Show total planned time
    let hours = total_minutes / 60;
    let mins = total_minutes % 60;
    if hours > 0 && mins > 0 {
        println!("{}", format!("Total planned: {}h {}m", hours, mins).dimmed());
    } else if hours > 0 {
        println!("{}", format!("Total planned: {}h", hours).dimmed());
    } else if mins > 0 {
        println!("{}", format!("Total planned: {}m", mins).dimmed());
    }

    Ok(())
}

fn parse_time_to_minutes(time_str: &str) -> i32 {
    if time_str.ends_with('h') {
        time_str.trim_end_matches('h').parse::<i32>().unwrap_or(0) * 60
    } else if time_str.ends_with('m') {
        time_str.trim_end_matches('m').parse::<i32>().unwrap_or(0)
    } else {
        0
    }
}

fn cmd_list(config: &Config, all: bool, sort: &str, reverse: bool, status_filter: Option<&str>, context_filter: Option<&str>, priority_filter: Option<&str>, project_filter: Option<&str>, tag_filter: Option<&str>, days: Option<i64>, limit: Option<usize>, grouped: bool) -> Result<()> {
    let tasks = get_tasks_from_cache()?;
    let today = Local::now();
    let today_str = today.format("%Y-%m-%d").to_string();

    let mut filtered: Vec<&Task> = if let Some(d) = days {
        // Filter by last N days
        let cutoff = (today - chrono::Duration::days(d)).format("%Y-%m-%d").to_string();
        tasks.iter().filter(|t| t.date >= cutoff).collect()
    } else if all {
        tasks.iter().collect()
    } else {
        tasks.iter().filter(|t| t.date == today_str).collect()
    };

    // Apply status filter
    if let Some(status) = status_filter {
        let status_lower = status.to_lowercase();
        filtered.retain(|t| {
            t.status.as_ref()
                .map(|s| s.to_lowercase().contains(&status_lower))
                .unwrap_or(false)
        });
    }

    // Apply context filter
    if let Some(context) = context_filter {
        let context_lower = context.to_lowercase();
        filtered.retain(|t| {
            t.context.as_ref()
                .map(|c| c.to_lowercase().contains(&context_lower))
                .unwrap_or(false)
        });
    }

    // Apply priority filter (supports comma-separated values like "P0,P1")
    if let Some(priority) = priority_filter {
        let priorities: Vec<String> = priority.split(',')
            .map(|p| p.trim().to_uppercase())
            .collect();
        filtered.retain(|t| {
            t.priority.as_ref()
                .map(|p| priorities.iter().any(|pf| p.to_uppercase().contains(pf)))
                .unwrap_or(false)
        });
    }

    // Apply project filter
    if let Some(project) = project_filter {
        let project_lower = project.to_lowercase();
        filtered.retain(|t| {
            t.project.as_ref()
                .map(|p| p.to_lowercase().contains(&project_lower))
                .unwrap_or(false)
        });
    }

    // Apply tag filter
    if let Some(tag) = tag_filter {
        let tag_lower = tag.to_lowercase();
        filtered.retain(|t| {
            t.tags.iter().any(|tag_item| tag_item.to_lowercase().contains(&tag_lower))
        });
    }

    // Sort based on the sort parameter
    match sort {
        "priority" | "p" => {
            filtered.sort_by(|a, b| {
                let a_pri = a.priority.as_deref().unwrap_or("P9");
                let b_pri = b.priority.as_deref().unwrap_or("P9");
                a_pri.cmp(b_pri)
            });
        }
        "date" | "d" => {
            filtered.sort_by(|a, b| a.date.cmp(&b.date));
        }
        "project" | "j" => {
            filtered.sort_by(|a, b| {
                let a_proj = a.project.as_deref().unwrap_or("zzz");
                let b_proj = b.project.as_deref().unwrap_or("zzz");
                a_proj.cmp(b_proj)
            });
        }
        "time" | "t" => {
            fn time_to_minutes(t: Option<&str>) -> i32 {
                match t {
                    Some(s) if s.ends_with('h') => s.trim_end_matches('h').parse::<i32>().unwrap_or(0) * 60,
                    Some(s) if s.ends_with('m') => s.trim_end_matches('m').parse::<i32>().unwrap_or(0),
                    _ => 9999, // No time = sort last
                }
            }
            filtered.sort_by(|a, b| {
                time_to_minutes(a.time.as_deref()).cmp(&time_to_minutes(b.time.as_deref()))
            });
        }
        "title" | "n" => {
            filtered.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
        }
        "status" | "s" => {
            filtered.sort_by(|a, b| {
                let a_status = a.status.as_deref().unwrap_or("zzz");
                let b_status = b.status.as_deref().unwrap_or("zzz");
                a_status.cmp(b_status)
            });
        }
        "context" | "c" => {
            filtered.sort_by(|a, b| {
                let a_ctx = a.context.as_deref().unwrap_or("zzz");
                let b_ctx = b.context.as_deref().unwrap_or("zzz");
                a_ctx.cmp(b_ctx)
            });
        }
        _ => {
            // Default to priority
            filtered.sort_by(|a, b| {
                let a_pri = a.priority.as_deref().unwrap_or("P9");
                let b_pri = b.priority.as_deref().unwrap_or("P9");
                a_pri.cmp(b_pri)
            });
        }
    }

    if reverse {
        filtered.reverse();
    }

    let sort_label = match sort {
        "priority" | "p" => "by priority",
        "date" | "d" => "by date",
        "project" | "j" => "by project",
        "time" | "t" => "by time",
        "title" | "n" => "by title",
        "status" | "s" => "by status",
        "context" | "c" => "by context",
        _ => "by priority",
    };

    // Apply limit
    if let Some(n) = limit {
        filtered.truncate(n);
    }

    let date_desc = if let Some(d) = days {
        format!("Last {} days", d)
    } else if all {
        "All Tasks".to_string()
    } else {
        "Today's Tasks".to_string()
    };

    println!("🌱 The Garden - {} ({} tasks, sorted {}{})\n",
        date_desc,
        filtered.len(),
        sort_label,
        if reverse { ", reversed" } else { "" }
    );

    if grouped {
        // Group by date
        use std::collections::BTreeMap;
        let mut grouped_tasks: BTreeMap<String, Vec<&Task>> = BTreeMap::new();
        
        for task in &filtered {
            grouped_tasks.entry(task.date.clone()).or_insert_with(Vec::new).push(task);
        }

        for (date, tasks) in grouped_tasks {
            // Determine section header
            let date_label = if date < today_str {
                "OVERDUE".to_string().red().bold().to_string()
            } else if date == today_str {
                let weekday = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                    .map(|d| d.format("%A, %b %d").to_string())
                    .unwrap_or_else(|_| "TODAY".to_string());
                format!("TODAY ({})", weekday).green().bold().to_string()
            } else {
                // Check if it's this week
                let task_date = chrono::NaiveDate::parse_from_str(&date, "%Y-%m-%d").ok();
                let is_this_week = task_date.map(|d| {
                    let days_diff = (d - today.naive_local().date()).num_days();
                    days_diff >= 0 && days_diff <= 7
                }).unwrap_or(false);

                if is_this_week {
                    task_date.map(|d| d.format("%A, %b %d").to_string())
                        .unwrap_or_else(|| date.clone())
                } else {
                    date.clone()
                }
            };

            println!("{}", format!("════ {} ════", date_label).cyan());
            for task in tasks {
                let short_id = task.short_id();
                println!("{} {}", short_id.dimmed(), task.format(config));
            }
            println!();
        }
    } else {
        // Regular list view
        for task in filtered {
            // Show task ID on the left for easy reference
            let short_id = task.short_id();
            println!("{} {}", short_id.dimmed(), task.format(config));
        }
    }

    Ok(())
}

fn cmd_add(config: &Config, title: String, priority: Option<String>, project: Option<String>) -> Result<()> {
    // Extract hashtags from title
    let hashtag_regex = Regex::new(r"#(\w+)").unwrap();
    let mut tags: Vec<String> = hashtag_regex
        .captures_iter(&title)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect();

    let task = Task {
        id: None,
        list_id: None,
        date: Local::now().format(&config.date_format).to_string(),
        priority: priority.clone(),
        project: project.clone(),
        status: None,
        context: None,
        time: None,
        title: title.clone(),
        list: "My Tasks".to_string(), // Default list
        attachment_type: None,
        tags: tags.clone(),
    };

    let formatted_title = task.format(config);

    println!("Adding task: {}", formatted_title.cyan());

    // Check if ClickUp is configured with a default list
    if let Some(ref clickup_config) = config.clickup {
        if clickup_config.enabled {
            if let Some(ref default_list_id) = clickup_config.list_id {
                println!("{}", "📤 Adding directly to ClickUp...".dimmed());
                
                // Create ClickUp client
                let client = ClickUpClient::new(clickup::ClickUpConfig {
                    api_token: clickup_config.api_token.clone(),
                    list_id: default_list_id.clone(),
                })?;
                
                // Add project as tag if present
                if let Some(ref proj) = project {
                    tags.push(proj.clone());
                }
                
                // Map priority to ClickUp priority
                let clickup_priority = match priority.as_deref() {
                    Some("P0") => Some(1u8),
                    Some("P1") => Some(2u8),
                    Some("P2") => Some(3u8),
                    Some("P3") | Some("P5") => Some(4u8),
                    _ => None,
                };
                
                // Create task in ClickUp
                let clickup_task = clickup::ClickUpTask {
                    id: None,
                    name: title.clone(),
                    description: None,
                    status: Some("to do".to_string()),
                    priority: clickup_priority,
                    due_date: None,
                    start_date: None,
                    time_estimate: None,
                    tags,
                    assignees: vec![],
                    custom_fields: None,
                };
                
                match client.create_task(default_list_id, &clickup_task) {
                    Ok(response) => {
                        println!("{}", format!("✓ Created in ClickUp: {}", response.id).green());
                        return Ok(());
                    }
                    Err(e) => {
                        eprintln!("{}", format!("⚠️  Failed to create in ClickUp: {}", e).yellow());
                        println!("{}", "Falling back to manual Google Tasks entry...".dimmed());
                    }
                }
            }
        }
    }
    
    // Fallback: print instructions for Google Tasks
    println!("{}", "(Add this to Google Tasks manually)".dimmed());

    Ok(())
}

/// Calculate similarity between two strings using bigrams (Dice coefficient)
fn string_similarity(s1: &str, s2: &str) -> f64 {
    let s1_lower = s1.to_lowercase();
    let s2_lower = s2.to_lowercase();

    if s1_lower == s2_lower {
        return 1.0;
    }

    let bigrams1: std::collections::HashSet<(char, char)> = s1_lower
        .chars()
        .zip(s1_lower.chars().skip(1))
        .collect();
    let bigrams2: std::collections::HashSet<(char, char)> = s2_lower
        .chars()
        .zip(s2_lower.chars().skip(1))
        .collect();

    if bigrams1.is_empty() || bigrams2.is_empty() {
        return 0.0;
    }

    let intersection = bigrams1.intersection(&bigrams2).count();
    (2.0 * intersection as f64) / (bigrams1.len() + bigrams2.len()) as f64
}

/// Compare priorities - returns the "higher" (more urgent) priority
fn higher_priority(p1: Option<&str>, p2: Option<&str>) -> Option<String> {
    let priority_order = ["P0", "P1", "P2", "P3", "P5", "DONE"];

    match (p1, p2) {
        (Some(a), Some(b)) => {
            let idx1 = priority_order.iter().position(|&x| x == a).unwrap_or(99);
            let idx2 = priority_order.iter().position(|&x| x == b).unwrap_or(99);
            Some(if idx1 <= idx2 { a } else { b }.to_string())
        }
        (Some(a), None) => Some(a.to_string()),
        (None, Some(b)) => Some(b.to_string()),
        (None, None) => None,
    }
}

/// Compare times - returns the longer time estimate
fn longer_time(t1: Option<&str>, t2: Option<&str>) -> Option<String> {
    fn time_to_minutes(t: &str) -> i32 {
        if t.ends_with('h') {
            t.trim_end_matches('h').parse::<i32>().unwrap_or(0) * 60
        } else if t.ends_with('m') {
            t.trim_end_matches('m').parse::<i32>().unwrap_or(0)
        } else {
            0
        }
    }

    match (t1, t2) {
        (Some(a), Some(b)) => {
            if time_to_minutes(a) >= time_to_minutes(b) {
                Some(a.to_string())
            } else {
                Some(b.to_string())
            }
        }
        (Some(a), None) => Some(a.to_string()),
        (None, Some(b)) => Some(b.to_string()),
        (None, None) => None,
    }
}

/// Compare dates - returns the earlier date
fn earlier_date(d1: &str, d2: &str) -> String {
    if d1 <= d2 { d1.to_string() } else { d2.to_string() }
}

fn cmd_bump(config: &Config, days: i64, week: bool) -> Result<()> {
    let tasks = get_tasks_from_cache()?;
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Calculate target date
    let target_date = if week {
        // Find next Monday
        let now = Local::now();
        let days_until_monday = (8 - now.weekday().num_days_from_monday()) % 7;
        let days_to_add = if days_until_monday == 0 { 7 } else { days_until_monday };
        (now + chrono::Duration::days(days_to_add as i64)).format("%Y-%m-%d").to_string()
    } else {
        (Local::now() + chrono::Duration::days(days)).format("%Y-%m-%d").to_string()
    };

    // Find today's incomplete tasks (not marked DONE)
    let to_bump: Vec<Task> = tasks.into_iter()
        .filter(|t| {
            t.date == today &&
            t.priority.as_ref().map(|p| p != "DONE").unwrap_or(true)
        })
        .collect();

    if to_bump.is_empty() {
        println!("{}", "No incomplete tasks from today to bump.".green());
        return Ok(());
    }

    println!("{}", format!("📅 Bumping {} tasks to {}\n", to_bump.len(), target_date).cyan().bold());

    let mut bumped_count = 0;

    for mut task in to_bump {
        println!("  {} {}", task.short_id().dimmed(), task.title);

        task.date = target_date.clone();

        if let Err(e) = update_task_locally(&task) {
            println!("    {}", format!("❌ Failed: {}", e).red());
        } else {
            bumped_count += 1;
        }
    }

    // Push changes
    println!("\n{}", "Pushing changes to Google...".dimmed());
    push_dirty_tasks_to_google(&config.google_account)?;

    println!("\n{}", format!("✓ Bumped {} tasks to {}", bumped_count, target_date).green());

    Ok(())
}

fn cmd_search(config: &Config, query: &str, project: Option<&str>, status: Option<&str>, context: Option<&str>, priority: Option<&str>) -> Result<()> {
    let tasks = get_tasks_from_cache()?;
    let query_lower = query.to_lowercase();

    let mut results: Vec<&Task> = tasks.iter()
        .filter(|t| t.title.to_lowercase().contains(&query_lower))
        .collect();

    // Apply filters
    if let Some(proj) = project {
        let proj_lower = proj.to_lowercase();
        results.retain(|t| {
            t.project.as_ref()
                .map(|p| p.to_lowercase().contains(&proj_lower))
                .unwrap_or(false)
        });
    }

    if let Some(stat) = status {
        let stat_lower = stat.to_lowercase();
        results.retain(|t| {
            t.status.as_ref()
                .map(|s| s.to_lowercase().contains(&stat_lower))
                .unwrap_or(false)
        });
    }

    if let Some(ctx) = context {
        let ctx_lower = ctx.to_lowercase();
        results.retain(|t| {
            t.context.as_ref()
                .map(|c| c.to_lowercase().contains(&ctx_lower))
                .unwrap_or(false)
        });
    }

    if let Some(pri) = priority {
        let pri_lower = pri.to_lowercase();
        results.retain(|t| {
            t.priority.as_ref()
                .map(|p| p.to_lowercase().contains(&pri_lower))
                .unwrap_or(false)
        });
    }

    if results.is_empty() {
        println!("{}", format!("No tasks found matching '{}'", query).yellow());
        return Ok(());
    }

    println!("{}", format!("🔍 Found {} tasks matching '{}'\n", results.len(), query).cyan().bold());

    // Sort by priority
    results.sort_by(|a, b| {
        let a_pri = a.priority.as_deref().unwrap_or("P9");
        let b_pri = b.priority.as_deref().unwrap_or("P9");
        a_pri.cmp(b_pri)
    });

    for task in results {
        println!("{} {}", task.short_id().dimmed(), task.format(config));
    }

    Ok(())
}

fn cmd_show(config: &Config, id: &str) -> Result<()> {
    let tasks = get_tasks_from_cache()?;

    // Find task by ID (full or partial match)
    let id_lower = id.to_lowercase();
    let matching_tasks: Vec<&Task> = tasks.iter()
        .filter(|t| {
            t.id.as_ref()
                .map(|tid| tid.to_lowercase().starts_with(&id_lower))
                .unwrap_or(false)
        })
        .collect();

    if matching_tasks.is_empty() {
        println!("{}", format!("No task found with ID starting with '{}'", id).red());
        return Ok(());
    }

    if matching_tasks.len() > 1 {
        println!("{}", format!("Multiple tasks match '{}'. Please be more specific:", id).yellow());
        for task in matching_tasks {
            println!("  {} {}", task.short_id().dimmed(), task.title);
        }
        return Ok(());
    }

    let task = matching_tasks[0];

    println!("{}", "═══ Task Details ═══".cyan().bold());
    println!();
    println!("{}: {}", "ID".dimmed(), task.id.as_deref().unwrap_or("N/A"));
    println!("{}: {}", "Title".dimmed(), task.title.yellow().bold());
    println!("{}: {}", "Date".dimmed(), task.date);
    println!("{}: {}", "Priority".dimmed(), task.priority.as_deref().unwrap_or("--"));
    println!("{}: {}", "Project".dimmed(), task.project.as_deref().unwrap_or("---"));
    println!("{}: {}", "Time".dimmed(), task.time.as_deref().unwrap_or("none"));
    println!("{}: {}", "List".dimmed(), task.list);

    if let Some(ref att_type) = task.attachment_type {
        println!("{}: {}", "Attachment".dimmed(), att_type);
    }

    // Show formatted version
    println!();
    println!("{}: {}", "Formatted".dimmed(), task.format(config));

    Ok(())
}

fn cmd_merge(config: &Config, threshold: f64, reset: bool) -> Result<()> {
    // Convert percentage to decimal (e.g., 80 -> 0.8)
    let threshold = if threshold > 1.0 { threshold / 100.0 } else { threshold };

    println!("{}", format!("🔍 Finding duplicates ({}% similar)...\n", (threshold * 100.0) as i32).cyan().bold());

    let sync_manager = SyncManager::new()?;

    if reset {
        sync_manager.reset_dismissed_pairs()?;
        println!("{}", "✓ Reset all dismissed pairs\n".green());
    }

    let tasks = get_tasks_from_cache()?;

    // Find potential duplicate pairs
    let mut pairs: Vec<(usize, usize, f64)> = Vec::new();

    for i in 0..tasks.len() {
        for j in (i + 1)..tasks.len() {
            // Skip if already dismissed
            if let (Some(id1), Some(id2)) = (&tasks[i].id, &tasks[j].id) {
                if sync_manager.is_pair_dismissed(id1, id2)? {
                    continue;
                }
            }

            let sim = string_similarity(&tasks[i].title, &tasks[j].title);
            if sim >= threshold {
                pairs.push((i, j, sim));
            }
        }
    }

    // Sort by similarity (highest first)
    pairs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());

    if pairs.is_empty() {
        println!("{}", "✅ No potential duplicates found!".green());
        return Ok(());
    }

    println!("Found {} potential duplicate pairs\n", pairs.len().to_string().yellow().bold());

    let mut merged_count = 0;
    let mut dismissed_count = 0;

    for (idx, (i, j, sim)) in pairs.iter().enumerate() {
        let task1 = &tasks[*i];
        let task2 = &tasks[*j];

        // Skip if either task was already merged (deleted)
        if task1.id.is_none() || task2.id.is_none() {
            continue;
        }

        println!("\n{}", format!("Pair {}/{} ({}% similar)", idx + 1, pairs.len(), (sim * 100.0) as i32).cyan());
        println!("{}", "─".repeat(50).dimmed());

        println!("{} {} {}", "1.".cyan(), task1.short_id().dimmed(), task1.title.yellow());
        println!("   {} [{}][{}]{}",
            task1.date.dimmed(),
            task1.priority.as_deref().unwrap_or("--"),
            task1.project.as_deref().unwrap_or("---"),
            task1.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default()
        );

        println!("{} {} {}", "2.".cyan(), task2.short_id().dimmed(), task2.title.yellow());
        println!("   {} [{}][{}]{}",
            task2.date.dimmed(),
            task2.priority.as_deref().unwrap_or("--"),
            task2.project.as_deref().unwrap_or("---"),
            task2.time.as_ref().map(|t| format!("{{{}}}", t)).unwrap_or_default()
        );

        println!("\n{}", "Options:".dimmed());
        println!("  {} Merge (keep task 1's title)", "1/j.".cyan());
        println!("  {} Merge (keep task 2's title)", "2/k.".cyan());
        println!("  {} Merge (join both titles)", "3/l.".cyan());
        println!("  {} Not duplicates (don't ask again)", "n.".yellow());
        println!("  {} Skip for now", "s.".dimmed());
        println!("  {} Quit", "q.".red());

        print!("\n> ");
        stdout().flush()?;

        let choice = read_single_key()?;
        println!("{}", choice);

        match choice {
            '1' | 'j' | 'J' => {
                // Merge keeping task 1's title
                // Combine tags from both tasks
                let mut merged_tags = task1.tags.clone();
                for tag in &task2.tags {
                    if !merged_tags.contains(tag) {
                        merged_tags.push(tag.clone());
                    }
                }
                
                let merged = Task {
                    id: task1.id.clone(),
                    list_id: task1.list_id.clone(),
                    title: task1.title.clone(),
                    date: earlier_date(&task1.date, &task2.date),
                    priority: higher_priority(task1.priority.as_deref(), task2.priority.as_deref()),
                    project: task1.project.clone().or(task2.project.clone()),
                    status: task1.status.clone().or(task2.status.clone()),
                    context: task1.context.clone().or(task2.context.clone()),
                    time: longer_time(task1.time.as_deref(), task2.time.as_deref()),
                    list: task1.list.clone(),
                    attachment_type: task1.attachment_type.clone().or(task2.attachment_type.clone()),
                    tags: merged_tags,
                };

                // Update task 1 with merged data
                update_task_locally(&merged)?;

                // Delete task 2 from cache and mark for deletion in Google
                if let Some(ref id2) = task2.id {
                    if let Some(ref list_id) = task2.list_id {
                        // Delete from Google
                        let _ = Command::new("gog")
                            .args(&["tasks", "delete", list_id, id2, "--account", &config.google_account])
                            .output();
                    }
                    sync_manager.delete_task_by_id(id2)?;
                }

                println!("{}", "  ✓ Merged (kept task 1)".green());
                merged_count += 1;
            }
            '2' | 'k' | 'K' => {
                // Merge keeping task 2's title
                // Combine tags from both tasks
                let mut merged_tags = task2.tags.clone();
                for tag in &task1.tags {
                    if !merged_tags.contains(tag) {
                        merged_tags.push(tag.clone());
                    }
                }
                
                let merged = Task {
                    id: task2.id.clone(),
                    list_id: task2.list_id.clone(),
                    title: task2.title.clone(),
                    date: earlier_date(&task1.date, &task2.date),
                    priority: higher_priority(task1.priority.as_deref(), task2.priority.as_deref()),
                    project: task2.project.clone().or(task1.project.clone()),
                    status: task2.status.clone().or(task1.status.clone()),
                    context: task2.context.clone().or(task1.context.clone()),
                    time: longer_time(task1.time.as_deref(), task2.time.as_deref()),
                    list: task2.list.clone(),
                    attachment_type: task2.attachment_type.clone().or(task1.attachment_type.clone()),
                    tags: merged_tags,
                };

                // Update task 2 with merged data
                update_task_locally(&merged)?;

                // Delete task 1 from cache and Google
                if let Some(ref id1) = task1.id {
                    if let Some(ref list_id) = task1.list_id {
                        let _ = Command::new("gog")
                            .args(&["tasks", "delete", list_id, id1, "--account", &config.google_account])
                            .output();
                    }
                    sync_manager.delete_task_by_id(id1)?;
                }

                println!("{}", "  ✓ Merged (kept task 2)".green());
                merged_count += 1;
            }
            '3' | 'l' | 'L' => {
                // Merge joining both titles
                let joined_title = format!("{} / {}", task1.title, task2.title);
                
                // Combine tags from both tasks
                let mut merged_tags = task1.tags.clone();
                for tag in &task2.tags {
                    if !merged_tags.contains(tag) {
                        merged_tags.push(tag.clone());
                    }
                }
                
                let merged = Task {
                    id: task1.id.clone(),
                    list_id: task1.list_id.clone(),
                    title: joined_title.clone(),
                    date: earlier_date(&task1.date, &task2.date),
                    priority: higher_priority(task1.priority.as_deref(), task2.priority.as_deref()),
                    project: task1.project.clone().or(task2.project.clone()),
                    status: task1.status.clone().or(task2.status.clone()),
                    context: task1.context.clone().or(task2.context.clone()),
                    time: longer_time(task1.time.as_deref(), task2.time.as_deref()),
                    list: task1.list.clone(),
                    attachment_type: task1.attachment_type.clone().or(task2.attachment_type.clone()),
                    tags: merged_tags,
                };

                // Update task 1 with merged data
                update_task_locally(&merged)?;

                // Delete task 2 from cache and Google
                if let Some(ref id2) = task2.id {
                    if let Some(ref list_id) = task2.list_id {
                        let _ = Command::new("gog")
                            .args(&["tasks", "delete", list_id, id2, "--account", &config.google_account])
                            .output();
                    }
                    sync_manager.delete_task_by_id(id2)?;
                }

                println!("{}", format!("  ✓ Merged: {}", joined_title).green());
                merged_count += 1;
            }
            'n' | 'N' => {
                // Mark as not duplicates
                if let (Some(id1), Some(id2)) = (&task1.id, &task2.id) {
                    sync_manager.dismiss_pair(id1, id2)?;
                }
                println!("{}", "  ✓ Marked as not duplicates".yellow());
                dismissed_count += 1;
            }
            's' | 'S' | ' ' => {
                println!("{}", "  → Skipped".dimmed());
            }
            'q' | 'Q' | '\x1b' => {
                println!("\n{}", "Exiting merge...".yellow());
                break;
            }
            _ => {
                println!("{}", "  ⚠ Invalid input, skipping".yellow());
            }
        }
    }

    // Push any changes
    if merged_count > 0 {
        println!("\n{}", "Pushing changes to Google...".dimmed());
        push_dirty_tasks_to_google(&config.google_account)?;
    }

    println!("\n{}", "═══ Merge Complete! ═══".green().bold());
    println!("Merged: {}, Dismissed: {}", merged_count.to_string().green(), dismissed_count.to_string().yellow());

    Ok(())
}

fn cmd_summary(_config: &Config, group_by: &str, created_days: Option<i64>, due_days: Option<i64>, include_done: bool, sort_by: &str, detailed: bool) -> Result<()> {
    use std::collections::HashMap;
    
    println!("{}", format!("📊 Task Summary - Grouped by {}\n", group_by).cyan().bold());
    
    let tasks = get_tasks_from_cache()?;
    let today = Local::now().date_naive();
    
    // Filter tasks based on parameters
    let filtered_tasks: Vec<Task> = tasks.into_iter().filter(|task| {
        // Exclude done tasks unless requested
        if !include_done && task.priority.as_ref().map(|p| p == "DONE").unwrap_or(false) {
            return false;
        }
        
        // Filter by creation date if specified
        if let Some(_days) = created_days {
            // For now, we'll skip this filter as we'd need to add created_date tracking
            // This would require schema changes
        }
        
        // Filter by due date if specified
        if let Some(days) = due_days {
            if let Ok(task_date) = chrono::NaiveDate::parse_from_str(&task.date, "%Y-%m-%d") {
                let days_until = (task_date - today).num_days();
                if days_until > days {
                    return false;
                }
            }
        }
        
        true
    }).collect();
    
    // Group tasks based on the grouping parameter
    let mut groups: HashMap<String, Vec<Task>> = HashMap::new();
    
    for task in filtered_tasks {
        let group_key = match group_by {
            "date" | "due" => task.date.clone(),
            "priority" => task.priority.clone().unwrap_or_else(|| "--".to_string()),
            "project" => task.project.clone().unwrap_or_else(|| "No Project".to_string()),
            "status" => task.status.clone().unwrap_or_else(|| "todo".to_string()),
            "context" => task.context.clone().unwrap_or_else(|| "No Context".to_string()),
            "created" => {
                // For now, use date as proxy (would need schema changes for real created date)
                task.date.clone()
            }
            _ => "Unknown".to_string(),
        };
        
        groups.entry(group_key).or_insert_with(Vec::new).push(task);
    }
    
    // Calculate time estimates and counts for each group
    struct GroupStats {
        name: String,
        count: usize,
        total_minutes: i32,
    }
    
    let mut group_stats: Vec<GroupStats> = groups.iter().map(|(name, tasks)| {
        let total_minutes = tasks.iter().map(|t| t.time.as_ref().map(|time| parse_time_to_minutes(time)).unwrap_or(0)).sum();
        GroupStats {
            name: name.clone(),
            count: tasks.len(),
            total_minutes,
        }
    }).collect();
    
    // Sort groups based on sort parameter
    match sort_by {
        "count" => group_stats.sort_by(|a, b| b.count.cmp(&a.count)),
        "time" => group_stats.sort_by(|a, b| b.total_minutes.cmp(&a.total_minutes)),
        _ => {
            // Sort by name, handling dates specially
            if group_by == "date" || group_by == "due" {
                group_stats.sort_by(|a, b| a.name.cmp(&b.name));
            } else {
                group_stats.sort_by(|a, b| {
                    // Special sorting for priorities
                    if group_by == "priority" {
                        let priority_order = ["P0", "P1", "P2", "P3", "--", "DONE"];
                        let a_idx = priority_order.iter().position(|&p| p == a.name).unwrap_or(999);
                        let b_idx = priority_order.iter().position(|&p| p == b.name).unwrap_or(999);
                        a_idx.cmp(&b_idx)
                    } else {
                        a.name.cmp(&b.name)
                    }
                });
            }
        }
    }
    
    // Display results
    let total_tasks: usize = group_stats.iter().map(|g| g.count).sum();
    let total_minutes: i32 = group_stats.iter().map(|g| g.total_minutes).sum();
    
    for stats in &group_stats {
        let time_str = format_time_from_minutes(stats.total_minutes);
        
        // Format the group name nicely for dates
        let display_name = if group_by == "date" || group_by == "due" {
            if let Ok(date) = chrono::NaiveDate::parse_from_str(&stats.name, "%Y-%m-%d") {
                let days_diff = (date - today).num_days();
                let day_str = match days_diff {
                    0 => " (today)".cyan().to_string(),
                    1 => " (tomorrow)".yellow().to_string(),
                    -1 => " (yesterday)".red().to_string(),
                    d if d < 0 => format!(" ({} days ago)", -d).red().to_string(),
                    d => format!(" (in {} days)", d).green().to_string(),
                };
                format!("{}{}", date.format("%a %b %d"), day_str)
            } else {
                stats.name.clone()
            }
        } else {
            stats.name.clone()
        };
        
        println!("🏷️  {} {}", display_name.bold(), format!("({} tasks, {})", stats.count, time_str).dimmed());
        
        if detailed {
            if let Some(tasks) = groups.get(&stats.name) {
                for task in tasks {
                    let time_str = task.time.as_ref().map(|t| format!(" {{{}}}", t)).unwrap_or_default();
                    let status_str = task.status.as_ref().map(|s| format!(" [{}]", s)).unwrap_or_default();
                    println!("   • {}{}{}", task.title, time_str.dimmed(), status_str.cyan());
                }
            }
            println!();
        }
    }
    
    // Display totals
    println!("{}", "─".repeat(50).dimmed());
    println!("📈 {} Total: {} tasks, {}", 
        "Summary".bold(), 
        total_tasks.to_string().green(),
        format_time_from_minutes(total_minutes).yellow()
    );
    
    // Add insights based on grouping
    match group_by {
        "priority" => {
            if let Some(p0_stats) = group_stats.iter().find(|s| s.name == "P0") {
                if p0_stats.count > 5 {
                    println!("\n⚠️  {} You have {} P0 tasks - consider re-prioritizing", "Warning:".red(), p0_stats.count);
                }
            }
        }
        "date" | "due" => {
            let overdue_count: usize = group_stats.iter()
                .filter(|s| {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(&s.name, "%Y-%m-%d") {
                        date < today
                    } else {
                        false
                    }
                })
                .map(|s| s.count)
                .sum();
                
            if overdue_count > 0 {
                println!("\n⚠️  {} You have {} overdue tasks", "Alert:".red(), overdue_count);
            }
            
            let today_stats = group_stats.iter().find(|s| s.name == today.format("%Y-%m-%d").to_string());
            if let Some(stats) = today_stats {
                if stats.total_minutes > 480 {
                    println!("\n⚠️  {} Today has {} of work scheduled (> 8 hours)", 
                        "Warning:".yellow(), 
                        format_time_from_minutes(stats.total_minutes)
                    );
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

// Helper function to format minutes to readable time
fn format_time_from_minutes(minutes: i32) -> String {
    if minutes == 0 {
        "no estimate".to_string()
    } else if minutes < 60 {
        format!("{}m", minutes)
    } else if minutes % 60 == 0 {
        format!("{}h", minutes / 60)
    } else {
        format!("{}h {}m", minutes / 60, minutes % 60)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config()?;

    // Auto-sync before most commands (unless it's an explicit sync command)
    match &cli.command {
        Commands::Sync { force, airtable, clickup } => {
            // Always sync when explicitly called
            sync_with_google(&config.google_account, *force)?;
            update_last_query()?;
            
            // Push to Airtable if requested
            if *airtable {
                sync_to_airtable(&config)?;
            }
            
            // Push to ClickUp if requested
            if *clickup {
                sync_to_clickup(&config)?;
            }
        }
        Commands::Triage { .. } | Commands::Focus | Commands::Plan | Commands::Schedule { .. } | Commands::List { .. } | Commands::Merge { .. } | Commands::Show { .. } | Commands::Search { .. } | Commands::Bump { .. } | Commands::Summary { .. } => {
            // Smart sync (check throttle)
            if should_sync(&config, false)? {
                sync_with_google(&config.google_account, false)?;
            }
            // Always update last_query timestamp (even if we didn't sync)
            update_last_query()?;
        }
        _ => {}
    }

    match cli.command {
        Commands::Triage { force, priority, project, time, status, context } => cmd_triage(&config, force, priority, project, time, status, context)?,
        Commands::Focus => cmd_focus(&config)?,
        Commands::Plan => cmd_plan(&config)?,
        Commands::Schedule { week, auto } => cmd_schedule(&config, week, auto)?,
        Commands::List { all, sort, reverse, status, context, priority, project, tag, days, limit, grouped } => {
            cmd_list(&config, all, &sort, reverse, status.as_deref(), context.as_deref(), priority.as_deref(), project.as_deref(), tag.as_deref(), days, limit, grouped)?
        }
        Commands::Add { title, priority, project } => cmd_add(&config, title, priority, project)?,
        Commands::Merge { threshold, reset } => cmd_merge(&config, threshold, reset)?,
        Commands::Show { id } => cmd_show(&config, &id)?,
        Commands::Search { query, project, status, context, priority } => {
            cmd_search(&config, &query, project.as_deref(), status.as_deref(), context.as_deref(), priority.as_deref())?
        }
        Commands::Bump { days, week } => cmd_bump(&config, days, week)?,
        Commands::Sync { force: _, airtable: _, clickup: _ } => {
            // Already handled above
            println!("{}", "✓ Sync complete!".green());
        }
        Commands::Summary { group, created_days, due_days, include_done, sort, detailed } => {
            cmd_summary(&config, &group, created_days, due_days, include_done, &sort, detailed)?
        }
    }

    Ok(())
}
