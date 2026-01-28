use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClickUpConfig {
    pub api_token: String,
    pub list_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClickUpTask {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<u8>,
    pub due_date: Option<i64>,
    pub start_date: Option<i64>,
    pub time_estimate: Option<i64>, // milliseconds
    pub tags: Vec<String>,
    pub assignees: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fields: Option<Vec<CustomField>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomField {
    pub id: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct ClickUpTaskResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: ClickUpStatus,
    pub priority: Option<ClickUpPriority>,
    pub due_date: Option<String>,
    pub start_date: Option<String>,
    pub time_estimate: Option<i64>,
    pub tags: Vec<ClickUpTag>,
}

#[derive(Debug, Deserialize)]
pub struct ClickUpStatus {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickUpPriority {
    pub id: String,
    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickUpTag {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickUpListResponse {
    pub tasks: Vec<ClickUpTaskResponse>,
}

pub struct ClickUpClient {
    config: ClickUpConfig,
    client: reqwest::blocking::Client,
}

impl ClickUpClient {
    pub fn new(config: ClickUpConfig) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        Ok(Self { config, client })
    }

    fn get_base_url(&self) -> String {
        "https://api.clickup.com/api/v2".to_string()
    }

    /// Fetch all tasks from a ClickUp list
    pub fn fetch_all_tasks(&self) -> Result<Vec<ClickUpTaskResponse>> {
        let url = format!("{}/list/{}/task", self.get_base_url(), self.config.list_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.config.api_token)
            .query(&[
                ("archived", "false"),
                ("include_closed", "true"),
            ])
            .send()
            .context("Failed to fetch tasks from ClickUp")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("ClickUp API error {}: {}", status, body));
        }

        let list_response: ClickUpListResponse = response
            .json()
            .context("Failed to parse ClickUp response")?;

        Ok(list_response.tasks)
    }

    /// Create a new task in ClickUp
    pub fn create_task(&self, list_id: &str, task: &ClickUpTask) -> Result<ClickUpTaskResponse> {
        let url = format!("{}/list/{}/task", self.get_base_url(), list_id);

        let response = self
            .client
            .post(&url)
            .header("Authorization", &self.config.api_token)
            .header("Content-Type", "application/json")
            .json(&task)
            .send()
            .context("Failed to create task in ClickUp")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("ClickUp create error {}: {}", status, body));
        }

        let task_response: ClickUpTaskResponse = response
            .json()
            .context("Failed to parse ClickUp create response")?;

        Ok(task_response)
    }

    /// Update an existing task in ClickUp
    pub fn update_task(&self, task_id: &str, task: &ClickUpTask) -> Result<ClickUpTaskResponse> {
        let url = format!("{}/task/{}", self.get_base_url(), task_id);

        let response = self
            .client
            .put(&url)
            .header("Authorization", &self.config.api_token)
            .header("Content-Type", "application/json")
            .json(&task)
            .send()
            .context("Failed to update task in ClickUp")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("ClickUp update error {}: {}", status, body));
        }

        let task_response: ClickUpTaskResponse = response
            .json()
            .context("Failed to parse ClickUp update response")?;

        Ok(task_response)
    }

    /// Delete a task from ClickUp
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        let url = format!("{}/task/{}", self.get_base_url(), task_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", &self.config.api_token)
            .send()
            .context("Failed to delete task from ClickUp")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("ClickUp delete error {}: {}", status, body));
        }

        Ok(())
    }

    /// Get available statuses for a list
    pub fn get_list_statuses(&self) -> Result<Vec<String>> {
        let url = format!("{}/list/{}", self.get_base_url(), self.config.list_id);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.config.api_token)
            .send()
            .context("Failed to get list info from ClickUp")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("ClickUp API error {}: {}", status, body));
        }

        #[derive(Deserialize)]
        struct ListInfo {
            statuses: Vec<StatusInfo>,
        }

        #[derive(Deserialize)]
        struct StatusInfo {
            status: String,
        }

        let list_info: ListInfo = response
            .json()
            .context("Failed to parse list info")?;

        Ok(list_info.statuses.into_iter().map(|s| s.status).collect())
    }
}
