use crate::core::models::DataPath;
use crate::core::platform::{
    ActivityItem, ActivityMetrics as PlatformActivityMetrics, ConnectionStatus,
    DetailedActivities, ErrorContext, ReviewPlatform,
};
use crate::core::unified_config::GitLabConfig;
use async_trait::async_trait;
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// GitLab platform implementation
pub struct GitLabPlatform {
    config: GitLabConfig,
    platform_id: String, // e.g., "gitlab:company", "gitlab:public"
    client: Client,
}

impl GitLabPlatform {
    pub fn new(config: GitLabConfig, instance_id: String, _data_path: &DataPath) -> Self {
        let client = Client::new();
        let platform_id = format!("gitlab:{}", instance_id);
        Self {
            config,
            platform_id,
            client,
        }
    }
}

#[async_trait]
impl ReviewPlatform for GitLabPlatform {
    async fn get_activity_metrics(&self, user: &str, days: u32) -> io::Result<PlatformActivityMetrics> {
        info!("Getting GitLab activity metrics for user: {user} over {days} days from {}", self.config.name);

        // TODO: Implement actual metrics fetching
        Ok(PlatformActivityMetrics {
            total_items: 0,
            items_by_category: HashMap::new(),
            platform_specific: HashMap::new(),
        })
    }

    async fn get_detailed_activities(&self, user: &str, days: u32) -> io::Result<DetailedActivities> {
        info!("Getting detailed GitLab activities for user: {user} over {days} days from {}", self.config.name);

        // TODO: Implement actual activity fetching
        Ok(DetailedActivities {
            items_by_category: HashMap::new(),
        })
    }

    async fn search_items(&self, query: &str, user: &str) -> io::Result<Vec<ActivityItem>> {
        info!("Searching GitLab items with query: '{query}' for user: {user} on {}", self.config.name);

        // TODO: Implement search
        Ok(Vec::new())
    }

    fn get_platform_name(&self) -> &str {
        &self.config.name
    }

    fn get_platform_icon(&self) -> &str {
        "🦊" // GitLab fox emoji
    }

    fn get_platform_id(&self) -> &str {
        &self.platform_id
    }

    fn is_configured(&self) -> bool {
        self.config.is_configured()
    }

    fn get_item_url(&self, item: &ActivityItem) -> String {
        // For now, just return the item URL if it exists
        // TODO: Implement GitLab-specific URL generation if needed
        item.url.clone()
    }

    async fn test_connection(&self) -> io::Result<ConnectionStatus> {
        info!("Testing GitLab connection to: {}", self.config.name);

        let url = format!("{}/projects", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[("simple", "true"), ("per_page", "1")]) // Just get 1 project to test
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "test_connection")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "test_connection")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .log_error();

            return Ok(ConnectionStatus::Error(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        // Try to parse as JSON to ensure we get valid response
        let projects: Result<Vec<serde_json::Value>, _> = response.json().await;

        match projects {
            Ok(_) => {
                info!("GitLab connection successful to {}", self.config.name);
                Ok(ConnectionStatus::Connected)
            }
            Err(e) => {
                ErrorContext::new(&self.platform_id, "test_connection")
                    .with_error("json_parse_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .log_error();

                Ok(ConnectionStatus::Error(format!(
                    "Invalid response from GitLab API: {e}"
                )))
            }
        }
    }
}

// Basic GitLab API response structures for testing
#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
}
