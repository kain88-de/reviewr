use crate::core::models::DataPath;
use crate::core::platform::{
    ActivityCategory, ActivityItem, ActivityMetrics, ConnectionStatus, DetailedActivities,
    ReviewPlatform,
};
use crate::core::unified_config::JiraConfig;
use async_trait::async_trait;
use base64::Engine;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;
use std::time::Duration;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub assignee: Option<String>,
    pub created: String,
    pub updated: String,
    pub resolved: Option<String>,
    pub project: String,
    pub issue_type: String,
    pub priority: Option<String>,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JiraActivityMetrics {
    pub tickets_created: u32,
    pub tickets_resolved: u32,
    pub tickets_assigned: u32,
    pub comments_added: u32,
}

#[derive(Debug, Clone)]
pub struct DetailedJiraMetrics {
    pub tickets_created: Vec<IssueInfo>,
    pub tickets_resolved: Vec<IssueInfo>,
    pub tickets_assigned: Vec<IssueInfo>,
    pub tickets_commented: Vec<IssueInfo>,
}

#[derive(Debug, Deserialize)]
struct JiraSearchResponse {
    issues: Vec<JiraIssue>,
    total: i32,
}

#[derive(Debug, Deserialize)]
struct JiraIssue {
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    status: JiraStatus,
    assignee: Option<JiraUser>,
    #[allow(dead_code)]
    reporter: Option<JiraUser>,
    created: String,
    updated: String,
    resolutiondate: Option<String>,
    project: JiraProject,
    issuetype: JiraIssueType,
    priority: Option<JiraPriority>,
    components: Option<Vec<JiraComponent>>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraUser {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    #[allow(dead_code)]
    email_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JiraProject {
    key: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraPriority {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraComponent {
    name: String,
}

pub struct JiraClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub fn new(config: &JiraConfig) -> io::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| io::Error::other(format!("Failed to create HTTP client: {e}")))?;

        let credentials = format!("{}:{}", config.username, config.api_token);
        let auth_header = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(credentials)
        );

        let base_url = config.jira_url.trim_end_matches('/').to_string();

        info!("Created JIRA client for {base_url}");

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    pub async fn get_activity_metrics(
        &self,
        user_email: &str,
        days: u32,
    ) -> io::Result<JiraActivityMetrics> {
        info!("Fetching JIRA activity metrics for {user_email} (last {days} days)");

        let mut metrics = JiraActivityMetrics::default();

        // Get tickets created by user
        let created_count = self.count_tickets_created(user_email, days).await?;
        metrics.tickets_created = created_count;

        // Get tickets resolved by user
        let resolved_count = self.count_tickets_resolved(user_email, days).await?;
        metrics.tickets_resolved = resolved_count;

        // Get tickets currently assigned to user
        let assigned_count = self.count_tickets_assigned(user_email).await?;
        metrics.tickets_assigned = assigned_count;

        // Comments are harder to track efficiently, leaving at 0 for now
        metrics.comments_added = 0;

        info!("JIRA activity metrics for {user_email}: {metrics:?}");
        Ok(metrics)
    }

    async fn count_tickets_created(&self, user_email: &str, days: u32) -> io::Result<u32> {
        let jql = format!("reporter = \"{user_email}\" AND created >= -{days}d");
        self.search_issues_count(&jql).await
    }

    async fn count_tickets_resolved(&self, user_email: &str, days: u32) -> io::Result<u32> {
        let jql = format!("assignee = \"{user_email}\" AND resolved >= -{days}d");
        self.search_issues_count(&jql).await
    }

    async fn count_tickets_assigned(&self, user_email: &str) -> io::Result<u32> {
        let jql = format!("assignee = \"{user_email}\" AND resolution = Unresolved");
        self.search_issues_count(&jql).await
    }

    async fn search_issues_count(&self, jql: &str) -> io::Result<u32> {
        let url = format!(
            "{}/rest/api/3/search?jql={}&maxResults=0",
            self.base_url,
            urlencoding::encode(jql)
        );

        info!("JIRA JQL query: {jql}");

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to query JIRA: {e}");
                io::Error::other(format!("JIRA API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("JIRA API error {status}: {error_text}");
            return Err(io::Error::other(format!(
                "JIRA API returned {status}: {error_text}"
            )));
        }

        let search_response: JiraSearchResponse = response.json().await.map_err(|e| {
            error!("Failed to parse JIRA JSON response: {e}");
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(search_response.total as u32)
    }

    pub async fn get_detailed_activity_metrics(
        &self,
        user_email: &str,
        days: u32,
    ) -> io::Result<DetailedJiraMetrics> {
        info!("Fetching detailed JIRA activity metrics for {user_email} (last {days} days)");

        let tickets_created = self.get_detailed_tickets_created(user_email, days).await?;
        let tickets_resolved = self.get_detailed_tickets_resolved(user_email, days).await?;
        let tickets_assigned = self.get_detailed_tickets_assigned(user_email).await?;
        let tickets_commented = Vec::new(); // Complex to implement efficiently

        Ok(DetailedJiraMetrics {
            tickets_created,
            tickets_resolved,
            tickets_assigned,
            tickets_commented,
        })
    }

    async fn get_detailed_tickets_created(
        &self,
        user_email: &str,
        days: u32,
    ) -> io::Result<Vec<IssueInfo>> {
        let jql =
            format!("reporter = \"{user_email}\" AND created >= -{days}d ORDER BY created DESC");
        self.search_detailed_issues(&jql).await
    }

    async fn get_detailed_tickets_resolved(
        &self,
        user_email: &str,
        days: u32,
    ) -> io::Result<Vec<IssueInfo>> {
        let jql =
            format!("assignee = \"{user_email}\" AND resolved >= -{days}d ORDER BY resolved DESC");
        self.search_detailed_issues(&jql).await
    }

    async fn get_detailed_tickets_assigned(&self, user_email: &str) -> io::Result<Vec<IssueInfo>> {
        let jql = format!(
            "assignee = \"{user_email}\" AND resolution = Unresolved ORDER BY updated DESC"
        );
        self.search_detailed_issues(&jql).await
    }

    async fn search_detailed_issues(&self, jql: &str) -> io::Result<Vec<IssueInfo>> {
        let url = format!(
            "{}/rest/api/3/search?jql={}&maxResults=50&fields=summary,status,assignee,reporter,created,updated,resolutiondate,project,issuetype,priority,components",
            self.base_url,
            urlencoding::encode(jql)
        );

        info!("JIRA detailed query: {jql}");

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                error!("Failed to query JIRA: {e}");
                io::Error::other(format!("JIRA API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("JIRA API error {status}: {error_text}");
            return Err(io::Error::other(format!(
                "JIRA API returned {status}: {error_text}"
            )));
        }

        let search_response: JiraSearchResponse = response.json().await.map_err(|e| {
            error!("Failed to parse JIRA JSON response: {e}");
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(search_response
            .issues
            .into_iter()
            .map(|issue| {
                let components = issue
                    .fields
                    .components
                    .unwrap_or_default()
                    .into_iter()
                    .map(|c| c.name)
                    .collect();

                IssueInfo {
                    key: issue.key,
                    summary: issue.fields.summary,
                    status: issue.fields.status.name,
                    assignee: issue.fields.assignee.and_then(|a| a.display_name),
                    created: issue.fields.created,
                    updated: issue.fields.updated,
                    resolved: issue.fields.resolutiondate,
                    project: issue.fields.project.key,
                    issue_type: issue.fields.issuetype.name,
                    priority: issue.fields.priority.map(|p| p.name),
                    components,
                }
            })
            .collect())
    }

    pub fn get_issue_url(&self, issue_key: &str) -> String {
        format!("{}/browse/{}", self.base_url, issue_key)
    }

    pub async fn test_connection(&self) -> io::Result<()> {
        let url = format!("{}/rest/api/3/myself", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| io::Error::other(format!("Connection test failed: {e}")))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(io::Error::other(format!(
                "JIRA connection test failed with status: {}",
                response.status()
            )))
        }
    }
}

pub struct JiraService;

impl JiraService {
    pub fn load_jira_config(data_path: &DataPath) -> io::Result<Option<JiraConfig>> {
        use crate::core::unified_config::UnifiedConfigService;

        let unified_config = UnifiedConfigService::load_config(data_path)?;
        if let Some(jira_config) = unified_config.platforms.jira {
            info!("Loaded JIRA config from unified config");
            Ok(Some(jira_config))
        } else {
            info!("JIRA config not found in unified config");
            Ok(None)
        }
    }

    pub async fn get_employee_metrics(
        data_path: &DataPath,
        employee_email: &str,
    ) -> io::Result<JiraActivityMetrics> {
        let config = Self::load_jira_config(data_path)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
"JIRA configuration not found. Please configure JIRA in the [platforms.jira] section of config.toml"
                )
            })?;

        let client = JiraClient::new(&config)?;
        client.get_activity_metrics(employee_email, 30).await
    }

    pub async fn get_detailed_employee_metrics(
        data_path: &DataPath,
        employee_email: &str,
    ) -> io::Result<(DetailedJiraMetrics, String)> {
        let config = Self::load_jira_config(data_path)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
"JIRA configuration not found. Please configure JIRA in the [platforms.jira] section of config.toml"
                )
            })?;

        let client = JiraClient::new(&config)?;
        let metrics = client
            .get_detailed_activity_metrics(employee_email, 30)
            .await?;
        let base_url = config.jira_url.trim_end_matches('/').to_string();

        Ok((metrics, base_url))
    }
}

/// Platform wrapper for JIRA that implements the ReviewPlatform trait
pub struct JiraPlatform {
    data_path: DataPath,
}

impl JiraPlatform {
    pub fn new(data_path: DataPath) -> Self {
        Self { data_path }
    }

    /// Convert JIRA's ActivityMetrics to platform ActivityMetrics
    fn convert_metrics(&self, jira_metrics: &JiraActivityMetrics) -> ActivityMetrics {
        let mut metrics = ActivityMetrics {
            total_items: jira_metrics.tickets_created
                + jira_metrics.tickets_resolved
                + jira_metrics.tickets_assigned
                + jira_metrics.comments_added,
            ..Default::default()
        };

        metrics.items_by_category.insert(
            ActivityCategory::IssuesCreated,
            jira_metrics.tickets_created,
        );
        metrics.items_by_category.insert(
            ActivityCategory::IssuesResolved,
            jira_metrics.tickets_resolved,
        );
        metrics.items_by_category.insert(
            ActivityCategory::IssuesAssigned,
            jira_metrics.tickets_assigned,
        );
        metrics.items_by_category.insert(
            ActivityCategory::IssuesCommented,
            jira_metrics.comments_added,
        );

        metrics
            .platform_specific
            .insert("tickets_created".to_string(), jira_metrics.tickets_created);
        metrics.platform_specific.insert(
            "tickets_resolved".to_string(),
            jira_metrics.tickets_resolved,
        );
        metrics.platform_specific.insert(
            "tickets_assigned".to_string(),
            jira_metrics.tickets_assigned,
        );
        metrics
            .platform_specific
            .insert("comments_added".to_string(), jira_metrics.comments_added);

        metrics
    }

    /// Convert JIRA IssueInfo to platform ActivityItem
    fn convert_issue_to_item(
        &self,
        issue: &IssueInfo,
        category: ActivityCategory,
        base_url: &str,
    ) -> ActivityItem {
        let mut metadata = HashMap::new();
        metadata.insert("issue_type".to_string(), issue.issue_type.clone());
        metadata.insert("project".to_string(), issue.project.clone());
        metadata.insert("status".to_string(), issue.status.clone());

        if let Some(assignee) = &issue.assignee {
            metadata.insert("assignee".to_string(), assignee.clone());
        }
        if let Some(priority) = &issue.priority {
            metadata.insert("priority".to_string(), priority.clone());
        }
        if !issue.components.is_empty() {
            metadata.insert("components".to_string(), issue.components.join(", "));
        }

        ActivityItem {
            id: issue.key.clone(),
            title: issue.summary.clone(),
            status: issue.status.clone(),
            created: issue.created.clone(),
            updated: issue.updated.clone(),
            url: format!("{}/browse/{}", base_url, issue.key),
            platform: "jira".to_string(),
            category,
            project: issue.project.clone(),
            metadata,
        }
    }
}

#[async_trait]
impl ReviewPlatform for JiraPlatform {
    async fn get_activity_metrics(&self, user: &str, _days: u32) -> io::Result<ActivityMetrics> {
        let jira_metrics = JiraService::get_employee_metrics(&self.data_path, user).await?;
        Ok(self.convert_metrics(&jira_metrics))
    }

    async fn get_detailed_activities(
        &self,
        user: &str,
        _days: u32,
    ) -> io::Result<DetailedActivities> {
        let (detailed_metrics, base_url) =
            JiraService::get_detailed_employee_metrics(&self.data_path, user).await?;

        let mut activities = DetailedActivities::default();

        // Convert each category of issues to activity items
        let issues_created: Vec<ActivityItem> = detailed_metrics
            .tickets_created
            .iter()
            .map(|issue| {
                self.convert_issue_to_item(issue, ActivityCategory::IssuesCreated, &base_url)
            })
            .collect();

        let issues_resolved: Vec<ActivityItem> = detailed_metrics
            .tickets_resolved
            .iter()
            .map(|issue| {
                self.convert_issue_to_item(issue, ActivityCategory::IssuesResolved, &base_url)
            })
            .collect();

        let issues_assigned: Vec<ActivityItem> = detailed_metrics
            .tickets_assigned
            .iter()
            .map(|issue| {
                self.convert_issue_to_item(issue, ActivityCategory::IssuesAssigned, &base_url)
            })
            .collect();

        let issues_commented: Vec<ActivityItem> = detailed_metrics
            .tickets_commented
            .iter()
            .map(|issue| {
                self.convert_issue_to_item(issue, ActivityCategory::IssuesCommented, &base_url)
            })
            .collect();

        activities
            .items_by_category
            .insert(ActivityCategory::IssuesCreated, issues_created);
        activities
            .items_by_category
            .insert(ActivityCategory::IssuesResolved, issues_resolved);
        activities
            .items_by_category
            .insert(ActivityCategory::IssuesAssigned, issues_assigned);
        activities
            .items_by_category
            .insert(ActivityCategory::IssuesCommented, issues_commented);

        Ok(activities)
    }

    async fn search_items(&self, query: &str, user: &str) -> io::Result<Vec<ActivityItem>> {
        // For now, implement basic search by getting all activities and filtering
        // TODO: Implement proper JIRA JQL search integration
        let activities = self.get_detailed_activities(user, 30).await?;

        let mut results = Vec::new();
        for items in activities.items_by_category.values() {
            for item in items {
                if item.title.to_lowercase().contains(&query.to_lowercase())
                    || item.project.to_lowercase().contains(&query.to_lowercase())
                    || item.id.to_lowercase().contains(&query.to_lowercase())
                {
                    results.push(item.clone());
                }
            }
        }

        Ok(results)
    }

    fn get_platform_name(&self) -> &str {
        "JIRA"
    }

    fn get_platform_icon(&self) -> &str {
        "🎫"
    }

    fn get_platform_id(&self) -> &str {
        "jira"
    }

    fn is_configured(&self) -> bool {
        JiraService::load_jira_config(&self.data_path)
            .map(|config| config.is_some())
            .unwrap_or(false)
    }

    async fn test_connection(&self) -> io::Result<ConnectionStatus> {
        match JiraService::load_jira_config(&self.data_path)? {
            Some(config) => match JiraClient::new(&config) {
                Ok(client) => match client.test_connection().await {
                    Ok(_) => Ok(ConnectionStatus::Connected),
                    Err(e) => {
                        if e.to_string().contains("authentication") || e.to_string().contains("401")
                        {
                            Ok(ConnectionStatus::Error("Authentication failed".to_string()))
                        } else if e.to_string().contains("timeout") {
                            Ok(ConnectionStatus::Warning("Connection timeout".to_string()))
                        } else {
                            Ok(ConnectionStatus::Error(format!("Connection failed: {e}")))
                        }
                    }
                },
                Err(e) => Ok(ConnectionStatus::Error(format!(
                    "Client creation failed: {e}"
                ))),
            },
            None => Ok(ConnectionStatus::NotConfigured),
        }
    }

    fn get_item_url(&self, item: &ActivityItem) -> String {
        item.url.clone()
    }
}
