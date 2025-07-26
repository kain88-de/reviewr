use crate::core::models::DataPath;
use crate::core::platform::{
    ActivityCategory, ActivityItem, ActivityMetrics as PlatformActivityMetrics,
    ConnectionStatus, DetailedActivities, ReviewPlatform
};
use async_trait::async_trait;
use base64::Engine;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct GerritConfig {
    pub gerrit_url: String,
    pub username: String,
    pub http_password: String,
}

#[derive(Debug, Clone, Default)]
pub struct ActivityMetrics {
    pub commits_merged: u32,
    pub changes_created: u32,
    pub reviews_given: u32,
    pub reviews_received: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeInfo {
    pub id: String,
    pub change_id: String,
    pub subject: String,
    pub status: String,
    pub created: String,
    pub updated: String,
    pub project: String,
    #[serde(rename = "_number")]
    pub number: u32,
    pub owner: Owner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DetailedActivityMetrics {
    pub commits_merged: Vec<ChangeInfo>,
    pub changes_created: Vec<ChangeInfo>,
    pub reviews_given: Vec<ChangeInfo>,
    pub reviews_received: Vec<ChangeInfo>,
}

pub struct GerritClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl GerritClient {
    pub fn new(config: &GerritConfig) -> io::Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| io::Error::other(format!("Failed to create HTTP client: {e}")))?;

        let credentials = format!("{}:{}", config.username, config.http_password);
        let auth_header = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode(credentials)
        );

        let base_url = config.gerrit_url.trim_end_matches('/').to_string();

        info!("Created Gerrit client for {base_url}");

        Ok(Self {
            client,
            base_url,
            auth_header,
        })
    }

    pub async fn get_activity_metrics(
        &self,
        email: &str,
        days: u32,
    ) -> io::Result<ActivityMetrics> {
        info!("Fetching activity metrics for {email} (last {days} days)");

        let mut metrics = ActivityMetrics::default();

        // Get changes created by this user in the last N days
        let created_changes = self.get_changes_created(email, days).await?;
        metrics.changes_created = created_changes;

        // Get merged changes (commits that landed)
        let merged_changes = self.get_changes_merged(email, days).await?;
        metrics.commits_merged = merged_changes;

        // Get reviews given by this user
        let reviews_given = self.get_reviews_given(email, days).await?;
        metrics.reviews_given = reviews_given;

        // Get reviews received on this user's changes
        let reviews_received = self.get_reviews_received(email, days).await?;
        metrics.reviews_received = reviews_received;

        info!("Activity metrics for {email}: {metrics:?}");
        Ok(metrics)
    }

    async fn get_changes_created(&self, email: &str, days: u32) -> io::Result<u32> {
        let query = format!("owner:{email} -age:{days}d");
        self.query_changes(&query).await
    }

    async fn get_changes_merged(&self, email: &str, days: u32) -> io::Result<u32> {
        let query = format!("owner:{email} status:merged -age:{days}d");
        self.query_changes(&query).await
    }

    async fn get_reviews_given(&self, email: &str, days: u32) -> io::Result<u32> {
        let query = format!("reviewer:{email} -age:{days}d");
        self.query_changes(&query).await
    }

    async fn get_reviews_received(&self, email: &str, days: u32) -> io::Result<u32> {
        // Get changes by this user - we'll count all their changes as potentially reviewed
        let query = format!("owner:{email} -age:{days}d");
        self.query_changes(&query).await
    }

    async fn query_changes(&self, query: &str) -> io::Result<u32> {
        let url = format!(
            "{}/a/changes/?q={}",
            self.base_url,
            urlencoding::encode(query)
        );

        info!("Querying Gerrit: {query}");

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to query Gerrit: {e}");
                io::Error::other(format!("Gerrit API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Gerrit API error {status}: {error_text}");
            return Err(io::Error::other(format!(
                "Gerrit API returned {status}: {error_text}"
            )));
        }

        let text = response.text().await.map_err(|e| {
            error!("Failed to read Gerrit response: {e}");
            io::Error::other(format!("Failed to read response: {e}"))
        })?;

        // Gerrit API responses start with ")]}'" to prevent JSON hijacking
        let json_text = text.strip_prefix(")]}'").unwrap_or(&text);

        let changes: Vec<serde_json::Value> = serde_json::from_str(json_text).map_err(|e| {
            error!("Failed to parse Gerrit JSON response: {e}");
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(changes.len() as u32)
    }

    pub async fn get_detailed_activity_metrics(
        &self,
        email: &str,
        days: u32,
    ) -> io::Result<DetailedActivityMetrics> {
        info!("Fetching detailed activity metrics for {email} (last {days} days)");

        let commits_merged = self.get_detailed_changes_merged(email, days).await?;
        let changes_created = self.get_detailed_changes_created(email, days).await?;
        let reviews_given = self.get_detailed_reviews_given(email, days).await?;
        let reviews_received = self.get_detailed_reviews_received(email, days).await?;

        Ok(DetailedActivityMetrics {
            commits_merged,
            changes_created,
            reviews_given,
            reviews_received,
        })
    }

    async fn get_detailed_changes_created(&self, email: &str, days: u32) -> io::Result<Vec<ChangeInfo>> {
        let query = format!("owner:{email} -age:{days}d");
        self.query_detailed_changes(&query).await
    }

    async fn get_detailed_changes_merged(&self, email: &str, days: u32) -> io::Result<Vec<ChangeInfo>> {
        let query = format!("owner:{email} status:merged -age:{days}d");
        self.query_detailed_changes(&query).await
    }

    async fn get_detailed_reviews_given(&self, email: &str, days: u32) -> io::Result<Vec<ChangeInfo>> {
        let query = format!("reviewer:{email} -age:{days}d");
        self.query_detailed_changes(&query).await
    }

    async fn get_detailed_reviews_received(&self, email: &str, days: u32) -> io::Result<Vec<ChangeInfo>> {
        let query = format!("owner:{email} -age:{days}d");
        self.query_detailed_changes(&query).await
    }

    async fn query_detailed_changes(&self, query: &str) -> io::Result<Vec<ChangeInfo>> {
        let url = format!(
            "{}/a/changes/?q={}",
            self.base_url,
            urlencoding::encode(query)
        );

        info!("Querying Gerrit for detailed changes: {query}");

        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.auth_header)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to query Gerrit: {e}");
                io::Error::other(format!("Gerrit API request failed: {e}"))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Gerrit API error {status}: {error_text}");
            return Err(io::Error::other(format!(
                "Gerrit API returned {status}: {error_text}"
            )));
        }

        let text = response.text().await.map_err(|e| {
            error!("Failed to read Gerrit response: {e}");
            io::Error::other(format!("Failed to read response: {e}"))
        })?;

        // Gerrit API responses start with ")]}'" to prevent JSON hijacking
        let json_text = text.strip_prefix(")]}'").unwrap_or(&text);

        let changes: Vec<ChangeInfo> = serde_json::from_str(json_text).map_err(|e| {
            error!("Failed to parse Gerrit JSON response: {e}");
            io::Error::new(io::ErrorKind::InvalidData, format!("Invalid JSON: {e}"))
        })?;

        Ok(changes)
    }

    pub fn get_change_url(&self, project: &str, change_number: u32) -> String {
        format!("{}/c/{}/+/{}", self.base_url, project, change_number)
    }
}

pub struct GerritService;

impl GerritService {
    pub fn load_gerrit_config(data_path: &DataPath) -> io::Result<Option<GerritConfig>> {
        let config_path = data_path.root.join("gerrit_config.toml");

        if !config_path.exists() {
            info!("Gerrit config not found at {}", config_path.display());
            return Ok(None);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: GerritConfig = toml::from_str(&content).map_err(|e| {
            error!("Failed to parse Gerrit config: {e}");
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid Gerrit config: {e}"),
            )
        })?;

        info!("Loaded Gerrit config from {}", config_path.display());
        Ok(Some(config))
    }

    pub async fn get_employee_metrics(
        data_path: &DataPath,
        employee_email: &str,
    ) -> io::Result<ActivityMetrics> {
        let config = Self::load_gerrit_config(data_path)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Gerrit configuration not found. Please create gerrit_config.toml in your data directory"
                )
            })?;

        let client = GerritClient::new(&config)?;
        client.get_activity_metrics(employee_email, 30).await
    }

    pub async fn get_detailed_employee_metrics(
        data_path: &DataPath,
        employee_email: &str,
    ) -> io::Result<(DetailedActivityMetrics, String)> {
        let config = Self::load_gerrit_config(data_path)?
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "Gerrit configuration not found. Please create gerrit_config.toml in your data directory"
                )
            })?;

        let client = GerritClient::new(&config)?;
        let metrics = client.get_detailed_activity_metrics(employee_email, 30).await?;
        let base_url = config.gerrit_url.trim_end_matches('/').to_string();

        Ok((metrics, base_url))
    }
}

/// Platform wrapper for Gerrit that implements the ReviewPlatform trait
pub struct GerritPlatform {
    data_path: DataPath,
}

impl GerritPlatform {
    pub fn new(data_path: DataPath) -> Self {
        Self { data_path }
    }

    /// Convert Gerrit's ActivityMetrics to platform ActivityMetrics
    fn convert_metrics(&self, gerrit_metrics: &ActivityMetrics) -> PlatformActivityMetrics {
        let mut metrics = PlatformActivityMetrics::default();
        metrics.total_items = gerrit_metrics.commits_merged +
                             gerrit_metrics.changes_created +
                             gerrit_metrics.reviews_given +
                             gerrit_metrics.reviews_received;

        metrics.items_by_category.insert(ActivityCategory::ChangesMerged, gerrit_metrics.commits_merged);
        metrics.items_by_category.insert(ActivityCategory::ChangesCreated, gerrit_metrics.changes_created);
        metrics.items_by_category.insert(ActivityCategory::ReviewsGiven, gerrit_metrics.reviews_given);
        metrics.items_by_category.insert(ActivityCategory::ReviewsReceived, gerrit_metrics.reviews_received);

        metrics.platform_specific.insert("commits_merged".to_string(), gerrit_metrics.commits_merged);
        metrics.platform_specific.insert("changes_created".to_string(), gerrit_metrics.changes_created);
        metrics.platform_specific.insert("reviews_given".to_string(), gerrit_metrics.reviews_given);
        metrics.platform_specific.insert("reviews_received".to_string(), gerrit_metrics.reviews_received);

        metrics
    }

    /// Convert Gerrit ChangeInfo to platform ActivityItem
    fn convert_change_to_item(&self, change: &ChangeInfo, category: ActivityCategory, base_url: &str) -> ActivityItem {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("change_id".to_string(), change.change_id.clone());
        metadata.insert("project".to_string(), change.project.clone());
        metadata.insert("status".to_string(), change.status.clone());

        if let Some(owner_name) = &change.owner.name {
            metadata.insert("owner_name".to_string(), owner_name.clone());
        }
        if let Some(owner_email) = &change.owner.email {
            metadata.insert("owner_email".to_string(), owner_email.clone());
        }

        ActivityItem {
            id: change.number.to_string(),
            title: change.subject.clone(),
            status: change.status.clone(),
            created: change.created.clone(),
            updated: change.updated.clone(),
            url: format!("{}/c/{}/+/{}", base_url, change.project, change.number),
            platform: "gerrit".to_string(),
            category,
            project: change.project.clone(),
            metadata,
        }
    }
}

#[async_trait]
impl ReviewPlatform for GerritPlatform {
    async fn get_activity_metrics(&self, user: &str, _days: u32) -> std::io::Result<PlatformActivityMetrics> {
        let gerrit_metrics = GerritService::get_employee_metrics(&self.data_path, user).await?;
        Ok(self.convert_metrics(&gerrit_metrics))
    }

    async fn get_detailed_activities(&self, user: &str, _days: u32) -> std::io::Result<DetailedActivities> {
        let (detailed_metrics, base_url) = GerritService::get_detailed_employee_metrics(&self.data_path, user).await?;

        let mut activities = DetailedActivities::default();

        // Convert each category of changes to activity items
        let changes_created: Vec<ActivityItem> = detailed_metrics.changes_created
            .iter()
            .map(|change| self.convert_change_to_item(change, ActivityCategory::ChangesCreated, &base_url))
            .collect();

        let changes_merged: Vec<ActivityItem> = detailed_metrics.commits_merged
            .iter()
            .map(|change| self.convert_change_to_item(change, ActivityCategory::ChangesMerged, &base_url))
            .collect();

        let reviews_given: Vec<ActivityItem> = detailed_metrics.reviews_given
            .iter()
            .map(|change| self.convert_change_to_item(change, ActivityCategory::ReviewsGiven, &base_url))
            .collect();

        let reviews_received: Vec<ActivityItem> = detailed_metrics.reviews_received
            .iter()
            .map(|change| self.convert_change_to_item(change, ActivityCategory::ReviewsReceived, &base_url))
            .collect();

        activities.items_by_category.insert(ActivityCategory::ChangesCreated, changes_created);
        activities.items_by_category.insert(ActivityCategory::ChangesMerged, changes_merged);
        activities.items_by_category.insert(ActivityCategory::ReviewsGiven, reviews_given);
        activities.items_by_category.insert(ActivityCategory::ReviewsReceived, reviews_received);

        Ok(activities)
    }

    async fn search_items(&self, query: &str, user: &str) -> std::io::Result<Vec<ActivityItem>> {
        // For now, implement basic search by getting all activities and filtering
        // TODO: Implement proper Gerrit search API integration
        let activities = self.get_detailed_activities(user, 30).await?;

        let mut results = Vec::new();
        for items in activities.items_by_category.values() {
            for item in items {
                if item.title.to_lowercase().contains(&query.to_lowercase()) ||
                   item.project.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(item.clone());
                }
            }
        }

        Ok(results)
    }

    fn get_platform_name(&self) -> &str {
        "Gerrit"
    }

    fn get_platform_icon(&self) -> &str {
        "🔧"
    }

    fn get_platform_id(&self) -> &str {
        "gerrit"
    }

    fn is_configured(&self) -> bool {
        GerritService::load_gerrit_config(&self.data_path)
            .map(|config| config.is_some())
            .unwrap_or(false)
    }

    async fn test_connection(&self) -> std::io::Result<ConnectionStatus> {
        match GerritService::load_gerrit_config(&self.data_path)? {
            Some(config) => {
                match GerritClient::new(&config) {
                    Ok(client) => {
                        // Try a simple query to test the connection
                        match client.query_changes("limit:1").await {
                            Ok(_) => Ok(ConnectionStatus::Connected),
                            Err(e) => {
                                if e.to_string().contains("authentication") {
                                    Ok(ConnectionStatus::Error("Authentication failed".to_string()))
                                } else if e.to_string().contains("timeout") {
                                    Ok(ConnectionStatus::Warning("Connection timeout".to_string()))
                                } else {
                                    Ok(ConnectionStatus::Error(format!("Connection failed: {}", e)))
                                }
                            }
                        }
                    }
                    Err(e) => Ok(ConnectionStatus::Error(format!("Client creation failed: {}", e))),
                }
            }
            None => Ok(ConnectionStatus::NotConfigured),
        }
    }

    fn get_item_url(&self, item: &ActivityItem) -> String {
        item.url.clone()
    }
}
