use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct AirtableConfig {
    pub api_key: String,
    pub base_id: String,
    pub table_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AirtableTask {
    pub id: Option<String>,
    pub fields: AirtableFields,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AirtableFields {
    #[serde(rename = "Title")]
    pub title: String,
    
    #[serde(rename = "Priority")]
    pub priority: Option<String>,
    
    #[serde(rename = "Project")]
    pub project: Option<String>,
    
    #[serde(rename = "Status")]
    pub status: Option<String>,
    
    #[serde(rename = "Context")]
    pub context: Option<String>,
    
    #[serde(rename = "TimeEstimate")]
    pub time_estimate: Option<String>,
    
    #[serde(rename = "DueDate")]
    pub due_date: Option<String>,
    
    #[serde(rename = "CreatedDate")]
    pub created_date: Option<String>,
    
    #[serde(rename = "Assignee")]
    pub assignee: Option<String>,
    
    #[serde(rename = "Tags")]
    pub tags: Option<String>,
    
    #[serde(rename = "Notes")]
    pub notes: Option<String>,
    
    #[serde(rename = "Completed")]
    pub completed: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AirtableResponse {
    records: Vec<AirtableRecord>,
    offset: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AirtableRecord {
    id: String,
    fields: AirtableFields,
    #[serde(rename = "createdTime")]
    created_time: String,
}

pub struct AirtableClient {
    config: AirtableConfig,
    client: reqwest::blocking::Client,
}

impl AirtableClient {
    pub fn new(config: AirtableConfig) -> Result<Self> {
        let client = reqwest::blocking::Client::new();
        Ok(Self { config, client })
    }

    fn get_base_url(&self) -> String {
        format!(
            "https://api.airtable.com/v0/{}/{}",
            self.config.base_id, self.config.table_name
        )
    }

    /// Fetch all tasks from Airtable
    pub fn fetch_all_tasks(&self) -> Result<Vec<AirtableTask>> {
        let mut all_records = Vec::new();
        let mut offset: Option<String> = None;

        loop {
            let mut url = self.get_base_url();
            if let Some(ref offset_val) = offset {
                url = format!("{}?offset={}", url, offset_val);
            }

            let response = self
                .client
                .get(&url)
                .header("Authorization", format!("Bearer {}", self.config.api_key))
                .send()
                .context("Failed to fetch tasks from Airtable")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().unwrap_or_default();
                return Err(anyhow!("Airtable API error {}: {}", status, body));
            }

            let airtable_response: AirtableResponse = response
                .json()
                .context("Failed to parse Airtable response")?;

            for record in airtable_response.records {
                all_records.push(AirtableTask {
                    id: Some(record.id),
                    fields: record.fields,
                });
            }

            offset = airtable_response.offset;
            if offset.is_none() {
                break;
            }
        }

        Ok(all_records)
    }

    /// Create a new task in Airtable
    pub fn create_task(&self, fields: AirtableFields) -> Result<AirtableTask> {
        let url = self.get_base_url();

        let payload = serde_json::json!({
            "fields": fields
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .context("Failed to create task in Airtable")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Airtable create error {}: {}", status, body));
        }

        let record: AirtableRecord = response
            .json()
            .context("Failed to parse Airtable create response")?;

        Ok(AirtableTask {
            id: Some(record.id),
            fields: record.fields,
        })
    }

    /// Update an existing task in Airtable
    pub fn update_task(&self, task_id: &str, fields: AirtableFields) -> Result<AirtableTask> {
        let url = format!("{}/{}", self.get_base_url(), task_id);

        let payload = serde_json::json!({
            "fields": fields
        });

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .context("Failed to update task in Airtable")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Airtable update error {}: {}", status, body));
        }

        let record: AirtableRecord = response
            .json()
            .context("Failed to parse Airtable update response")?;

        Ok(AirtableTask {
            id: Some(record.id),
            fields: record.fields,
        })
    }

    /// Delete a task from Airtable
    pub fn delete_task(&self, task_id: &str) -> Result<()> {
        let url = format!("{}/{}", self.get_base_url(), task_id);

        let response = self
            .client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .context("Failed to delete task from Airtable")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(anyhow!("Airtable delete error {}: {}", status, body));
        }

        Ok(())
    }
}
