use crate::core::models::DataPath;
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
}
