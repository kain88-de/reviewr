use crate::core::models::DataPath;
use crate::core::platform::{
    ActivityCategory, ActivityItem, ActivityMetrics as PlatformActivityMetrics, ConnectionStatus,
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
        let platform_id = format!("gitlab:{instance_id}");
        Self {
            config,
            platform_id,
            client,
        }
    }
}

#[async_trait]
impl ReviewPlatform for GitLabPlatform {
    async fn get_activity_metrics(
        &self,
        user: &str,
        days: u32,
    ) -> io::Result<PlatformActivityMetrics> {
        info!(
            "Getting GitLab activity metrics for user: {user} over {days} days from {}",
            self.config.name
        );

        // TODO: Implement actual metrics fetching
        Ok(PlatformActivityMetrics {
            total_items: 0,
            items_by_category: HashMap::new(),
            platform_specific: HashMap::new(),
        })
    }

    async fn get_detailed_activities(
        &self,
        user: &str,
        days: u32,
    ) -> io::Result<DetailedActivities> {
        info!(
            "Getting detailed GitLab activities for user: {user} over {days} days from {}",
            self.config.name
        );

        let mut items_by_category = HashMap::new();

        // Calculate date range
        let since = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let since_str = since.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();

        // Fetch merge requests authored by user
        if let Ok(authored_mrs) = self
            .fetch_merge_requests_authored_by(user, &since_str)
            .await
        {
            items_by_category.insert(ActivityCategory::MergeRequestsCreated, authored_mrs);
        }

        // Fetch merge requests assigned for review
        if let Ok(review_mrs) = self.fetch_merge_requests_for_review(user, &since_str).await {
            items_by_category.insert(ActivityCategory::MergeRequestsReviewed, review_mrs);
        }

        // Fetch merge requests merged by user
        if let Ok(merged_mrs) = self.fetch_merge_requests_merged_by(user, &since_str).await {
            items_by_category.insert(ActivityCategory::MergeRequestsMerged, merged_mrs);
        }

        // Fetch issues assigned to user
        if let Ok(assigned_issues) = self.fetch_issues_assigned_to(user, &since_str).await {
            items_by_category.insert(ActivityCategory::IssuesAssigned, assigned_issues);
        }

        // Fetch issues created by user
        if let Ok(created_issues) = self.fetch_issues_created_by(user, &since_str).await {
            items_by_category.insert(ActivityCategory::IssuesCreated, created_issues);
        }

        Ok(DetailedActivities { items_by_category })
    }

    async fn search_items(&self, query: &str, user: &str) -> io::Result<Vec<ActivityItem>> {
        info!(
            "Searching GitLab items with query: '{query}' for user: {user} on {}",
            self.config.name
        );

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

impl GitLabPlatform {
    /// Fetch merge requests authored by the user
    async fn fetch_merge_requests_authored_by(
        &self,
        user: &str,
        since: &str,
    ) -> io::Result<Vec<ActivityItem>> {
        // Extract username from email if needed (GitLab uses usernames, not email addresses)
        let username = if user.contains('@') {
            user.split('@').next().unwrap_or(user)
        } else {
            user
        };

        let url = format!("{}/merge_requests", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[
                ("author_username", username),
                ("created_after", since),
                ("state", "all"), // Include open, closed, and merged
                ("order_by", "created_at"),
                ("sort", "desc"),
                ("per_page", "100"),
                ("with_projects_enabled", "true"), // Include project information
            ])
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "fetch_authored_mrs")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .with_metadata("user", user)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "fetch_authored_mrs")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .with_metadata("user", user)
                .log_error();

            return Err(io::Error::other(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        let mrs: Vec<GitLabMergeRequest> = response.json().await.map_err(|e| {
            ErrorContext::new(&self.platform_id, "fetch_authored_mrs")
                .with_error("json_parse_error", &e.to_string())
                .with_request_details(&url, None, None)
                .with_metadata("user", user)
                .log_error();
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(mrs
            .into_iter()
            .map(|mr| self.merge_request_to_activity_item(mr))
            .collect())
    }

    /// Fetch merge requests where user is assigned as reviewer
    async fn fetch_merge_requests_for_review(
        &self,
        user: &str,
        since: &str,
    ) -> io::Result<Vec<ActivityItem>> {
        // Extract username from email if needed (GitLab uses usernames, not email addresses)
        let username = if user.contains('@') {
            user.split('@').next().unwrap_or(user)
        } else {
            user
        };

        let url = format!("{}/merge_requests", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[
                ("reviewer_username", username),
                ("created_after", since),
                ("state", "all"),
                ("order_by", "created_at"),
                ("sort", "desc"),
                ("per_page", "100"),
                ("with_projects_enabled", "true"), // Include project information
            ])
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "fetch_review_mrs")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .with_metadata("user", user)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "fetch_review_mrs")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .with_metadata("user", user)
                .log_error();

            return Err(io::Error::other(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        let mrs: Vec<GitLabMergeRequest> = response.json().await.map_err(|e| {
            ErrorContext::new(&self.platform_id, "fetch_review_mrs")
                .with_error("json_parse_error", &e.to_string())
                .with_request_details(&url, None, None)
                .with_metadata("user", user)
                .log_error();
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(mrs
            .into_iter()
            .map(|mr| self.merge_request_to_activity_item(mr))
            .collect())
    }

    /// Fetch merge requests that were merged by the user
    async fn fetch_merge_requests_merged_by(
        &self,
        user: &str,
        since: &str,
    ) -> io::Result<Vec<ActivityItem>> {
        // Extract username from email if needed (GitLab uses usernames, not email addresses)
        let username = if user.contains('@') {
            user.split('@').next().unwrap_or(user)
        } else {
            user
        };

        // Note: GitLab API doesn't have a direct filter for "merged_by_username"
        // We need to fetch merged MRs and filter client-side
        let url = format!("{}/merge_requests", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[
                ("state", "merged"),
                ("updated_after", since), // Use updated_after for merged MRs
                ("order_by", "updated_at"),
                ("sort", "desc"),
                ("per_page", "100"),
                ("with_projects_enabled", "true"), // Include project information
            ])
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "fetch_merged_mrs")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .with_metadata("user", user)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "fetch_merged_mrs")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .with_metadata("user", user)
                .log_error();

            return Err(io::Error::other(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        let mrs: Vec<GitLabMergeRequest> = response.json().await.map_err(|e| {
            ErrorContext::new(&self.platform_id, "fetch_merged_mrs")
                .with_error("json_parse_error", &e.to_string())
                .with_request_details(&url, None, None)
                .with_metadata("user", user)
                .log_error();
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        // Filter to only MRs merged by this user
        let filtered_mrs: Vec<GitLabMergeRequest> = mrs
            .into_iter()
            .filter(|mr| {
                mr.merged_by
                    .as_ref()
                    .map(|merged_by| merged_by.username == username)
                    .unwrap_or(false)
            })
            .collect();

        Ok(filtered_mrs
            .into_iter()
            .map(|mr| self.merge_request_to_activity_item(mr))
            .collect())
    }

    /// Fetch issues assigned to the user
    async fn fetch_issues_assigned_to(
        &self,
        user: &str,
        since: &str,
    ) -> io::Result<Vec<ActivityItem>> {
        // Extract username from email if needed (GitLab uses usernames, not email addresses)
        let username = if user.contains('@') {
            user.split('@').next().unwrap_or(user)
        } else {
            user
        };

        let url = format!("{}/issues", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[
                ("assignee_username", username),
                ("created_after", since),
                ("state", "all"),
                ("order_by", "created_at"),
                ("sort", "desc"),
                ("per_page", "100"),
                ("with_projects_enabled", "true"), // Include project information
            ])
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "fetch_assigned_issues")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .with_metadata("user", user)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "fetch_assigned_issues")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .with_metadata("user", user)
                .log_error();

            return Err(io::Error::other(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        let issues: Vec<GitLabIssue> = response.json().await.map_err(|e| {
            ErrorContext::new(&self.platform_id, "fetch_assigned_issues")
                .with_error("json_parse_error", &e.to_string())
                .with_request_details(&url, None, None)
                .with_metadata("user", user)
                .log_error();
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(issues
            .into_iter()
            .map(|issue| self.issue_to_activity_item(issue))
            .collect())
    }

    /// Fetch issues created by the user
    async fn fetch_issues_created_by(
        &self,
        user: &str,
        since: &str,
    ) -> io::Result<Vec<ActivityItem>> {
        // Extract username from email if needed (GitLab uses usernames, not email addresses)
        let username = if user.contains('@') {
            user.split('@').next().unwrap_or(user)
        } else {
            user
        };

        let url = format!("{}/issues", self.config.api_base_url());

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.token))
            .header("User-Agent", "reviewr/1.0")
            .query(&[
                ("author_username", username),
                ("created_after", since),
                ("state", "all"),
                ("order_by", "created_at"),
                ("sort", "desc"),
                ("per_page", "100"),
                ("with_projects_enabled", "true"), // Include project information
            ])
            .send()
            .await
            .map_err(|e| {
                ErrorContext::new(&self.platform_id, "fetch_created_issues")
                    .with_error("network_error", &e.to_string())
                    .with_request_details(&url, None, None)
                    .with_metadata("user", user)
                    .log_error();
                io::Error::other(format!("GitLab API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            ErrorContext::new(&self.platform_id, "fetch_created_issues")
                .with_error("api_error", &format!("HTTP {status}"))
                .with_request_details(&url, Some(status.as_u16()), Some(&error_text))
                .with_metadata("user", user)
                .log_error();

            return Err(io::Error::other(format!(
                "GitLab API returned {status}: {error_text}"
            )));
        }

        let issues: Vec<GitLabIssue> = response.json().await.map_err(|e| {
            ErrorContext::new(&self.platform_id, "fetch_created_issues")
                .with_error("json_parse_error", &e.to_string())
                .with_request_details(&url, None, None)
                .with_metadata("user", user)
                .log_error();
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(issues
            .into_iter()
            .map(|issue| self.issue_to_activity_item(issue))
            .collect())
    }

    /// Convert GitLab merge request to ActivityItem
    fn merge_request_to_activity_item(&self, mr: GitLabMergeRequest) -> ActivityItem {
        let status = match mr.state.as_str() {
            "opened" => "Open",
            "merged" => "Merged",
            "closed" => "Closed",
            _ => &mr.state,
        };

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), mr.author.name);
        metadata.insert("item_type".to_string(), "Merge Request".to_string());
        metadata.insert("target_branch".to_string(), mr.target_branch);
        metadata.insert("source_branch".to_string(), mr.source_branch);

        if let Some(assignee) = mr.assignees.first() {
            metadata.insert("assignee".to_string(), assignee.name.clone());
        }

        if let Some(merged_by) = mr.merged_by {
            metadata.insert("merged_by".to_string(), merged_by.name);
        }

        ActivityItem {
            id: format!("mr-{}", mr.iid),
            title: mr.title,
            url: mr.web_url,
            status: status.to_string(),
            created: mr.created_at,
            updated: mr.updated_at,
            platform: self.config.name.clone(),
            category: ActivityCategory::MergeRequestsCreated, // Will be overridden based on context
            project: mr
                .project
                .map(|p| format!("{} ({})", p.name, p.path_with_namespace))
                .unwrap_or_else(|| format!("Project ID: {}", mr.project_id)),
            metadata,
        }
    }

    /// Convert GitLab issue to ActivityItem
    fn issue_to_activity_item(&self, issue: GitLabIssue) -> ActivityItem {
        let status = match issue.state.as_str() {
            "opened" => "Open",
            "closed" => "Closed",
            _ => &issue.state,
        };

        let mut metadata = HashMap::new();
        metadata.insert("author".to_string(), issue.author.name);
        metadata.insert("item_type".to_string(), "Issue".to_string());

        if let Some(assignee) = issue.assignees.first() {
            metadata.insert("assignee".to_string(), assignee.name.clone());
        }

        ActivityItem {
            id: format!("issue-{}", issue.iid),
            title: issue.title,
            url: issue.web_url,
            status: status.to_string(),
            created: issue.created_at,
            updated: issue.updated_at,
            platform: self.config.name.clone(),
            category: ActivityCategory::IssuesCreated, // Will be overridden based on context
            project: issue
                .project
                .map(|p| format!("{} ({})", p.name, p.path_with_namespace))
                .unwrap_or_else(|| format!("Project ID: {}", issue.project_id)),
            metadata,
        }
    }
}

// GitLab API response structures
#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabProject {
    pub id: u64,
    pub name: String,
    pub path_with_namespace: String,
    pub web_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabMergeRequest {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub target_branch: String,
    pub source_branch: String,
    pub author: GitLabUser,
    pub assignees: Vec<GitLabUser>,
    pub reviewers: Vec<GitLabUser>,
    pub merged_by: Option<GitLabUser>,
    pub web_url: String,
    pub project_id: u64,
    pub project: Option<GitLabProject>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabIssue {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    pub closed_at: Option<String>,
    pub author: GitLabUser,
    pub assignees: Vec<GitLabUser>,
    pub web_url: String,
    pub project_id: u64,
    pub project: Option<GitLabProject>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitLabUser {
    pub id: u64,
    pub username: String,
    pub name: String,
    pub email: Option<String>,
}
