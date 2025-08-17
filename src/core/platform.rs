use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// Core trait that all review platforms must implement
#[async_trait::async_trait]
pub trait ReviewPlatform: Send + Sync {
    /// Get basic activity metrics for a user
    async fn get_activity_metrics(&self, user: &str, days: u32) -> io::Result<ActivityMetrics>;

    /// Get detailed activities with full item information
    async fn get_detailed_activities(
        &self,
        user: &str,
        days: u32,
    ) -> io::Result<DetailedActivities>;

    /// Search for items matching a query
    async fn search_items(&self, query: &str, user: &str) -> io::Result<Vec<ActivityItem>>;

    /// Platform identification
    fn get_platform_name(&self) -> &str;
    fn get_platform_icon(&self) -> &str;
    fn get_platform_id(&self) -> &str;

    /// Configuration and connection status
    fn is_configured(&self) -> bool;
    async fn test_connection(&self) -> io::Result<ConnectionStatus>;

    /// URL generation for items
    fn get_item_url(&self, item: &ActivityItem) -> String;
}

/// Basic activity metrics summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityMetrics {
    pub total_items: u32,
    pub items_by_category: HashMap<ActivityCategory, u32>,
    pub platform_specific: HashMap<String, u32>,
}

/// Detailed activities with full item lists
#[derive(Debug, Clone, Default)]
pub struct DetailedActivities {
    pub items_by_category: HashMap<ActivityCategory, Vec<ActivityItem>>,
}

/// Individual activity item (change, ticket, MR, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created: String,
    pub updated: String,
    pub url: String,
    pub platform: String,
    pub category: ActivityCategory,
    pub project: String,
    pub metadata: HashMap<String, String>,
}

/// Categories of activities across platforms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActivityCategory {
    // Code review activities
    ChangesCreated,
    ChangesReviewed,
    ChangesMerged,
    ReviewsGiven,
    ReviewsReceived,

    // Issue tracking
    IssuesCreated,
    IssuesAssigned,
    IssuesResolved,
    IssuesCommented,

    // Repository activities
    MergeRequestsCreated,
    MergeRequestsReviewed,
    MergeRequestsMerged,
    CommitsPushed,

    // Generic/Other
    Other(String),
}

impl ActivityCategory {
    pub fn display_name(&self) -> &str {
        match self {
            ActivityCategory::ChangesCreated => "Changes Created",
            ActivityCategory::ChangesReviewed => "Changes Reviewed",
            ActivityCategory::ChangesMerged => "Changes Merged",
            ActivityCategory::ReviewsGiven => "Reviews Given",
            ActivityCategory::ReviewsReceived => "Reviews Received",
            ActivityCategory::IssuesCreated => "Issues Created",
            ActivityCategory::IssuesAssigned => "Issues Assigned",
            ActivityCategory::IssuesResolved => "Issues Resolved",
            ActivityCategory::IssuesCommented => "Issues Commented",
            ActivityCategory::MergeRequestsCreated => "Merge Requests Created",
            ActivityCategory::MergeRequestsReviewed => "Merge Requests Reviewed",
            ActivityCategory::MergeRequestsMerged => "Merge Requests Merged",
            ActivityCategory::CommitsPushed => "Commits Pushed",
            ActivityCategory::Other(name) => name,
        }
    }

    pub fn short_key(&self) -> char {
        match self {
            ActivityCategory::ChangesCreated => 'c',
            ActivityCategory::ChangesMerged => 'm',
            ActivityCategory::ReviewsGiven => 'g',
            ActivityCategory::ReviewsReceived => 'r',
            ActivityCategory::IssuesCreated => 'c',
            ActivityCategory::IssuesAssigned => 'a',
            ActivityCategory::IssuesResolved => 'r',
            ActivityCategory::MergeRequestsCreated => 'c',
            ActivityCategory::MergeRequestsMerged => 'm',
            _ => 'o',
        }
    }
}

/// Connection status for platform health checks
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Warning(String),
    Error(String),
    NotConfigured,
}

impl ConnectionStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, ConnectionStatus::Connected)
    }

    pub fn status_icon(&self) -> &str {
        match self {
            ConnectionStatus::Connected => "✅",
            ConnectionStatus::Warning(_) => "⚠️",
            ConnectionStatus::Error(_) => "❌",
            ConnectionStatus::NotConfigured => "⚪",
        }
    }
}

/// Error types for platform operations
#[derive(Debug)]
pub enum PlatformError {
    ConnectionError(String),
    AuthenticationError(String),
    ConfigurationError(String),
    ApiError(String),
    DataParseError(String),
}

/// Structured error context for detailed error reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub platform_id: String,
    pub operation: String,
    pub user: Option<String>,
    pub timestamp: String,
    pub error_type: String,
    pub error_message: String,
    pub request_url: Option<String>,
    pub status_code: Option<u16>,
    pub response_body: Option<String>,
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    pub fn new(platform_id: &str, operation: &str) -> Self {
        Self {
            platform_id: platform_id.to_string(),
            operation: operation.to_string(),
            user: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            error_type: String::new(),
            error_message: String::new(),
            request_url: None,
            status_code: None,
            response_body: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }

    pub fn with_error(mut self, error_type: &str, message: &str) -> Self {
        self.error_type = error_type.to_string();
        self.error_message = message.to_string();
        self
    }

    pub fn with_request_details(
        mut self,
        url: &str,
        status_code: Option<u16>,
        response_body: Option<&str>,
    ) -> Self {
        self.request_url = Some(url.to_string());
        self.status_code = status_code;
        self.response_body = response_body.map(|s| s.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn log_error(&self) {
        log::error!(
            target: "platform_errors",
            "Platform error: {} | Operation: {} | Type: {} | Message: {} | User: {:?} | URL: {:?} | Status: {:?} | Context: {:?}",
            self.platform_id,
            self.operation,
            self.error_type,
            self.error_message,
            self.user,
            self.request_url,
            self.status_code,
            self.metadata
        );

        // Write detailed error to error.log file
        if let Err(e) = self.write_to_error_log() {
            log::warn!("Failed to write to error log file: {e}");
        }
    }

    fn write_to_error_log(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::OpenOptions;
        use std::io::Write;

        // Get the data directory (where config.toml lives)
        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".reviewr");

        // Ensure directory exists
        std::fs::create_dir_all(&data_dir)?;

        let error_log_path = data_dir.join("error.log");

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(error_log_path)?;

        // Write structured error as JSON line
        let error_json = serde_json::to_string(self)?;
        writeln!(file, "{error_json}")?;

        Ok(())
    }
}

/// Utility functions for reading error logs
pub struct ErrorLogReader;

impl ErrorLogReader {
    /// Read recent errors from the error.log file
    pub fn read_recent_errors(
        limit: usize,
        platform_filter: Option<&str>,
    ) -> Result<Vec<ErrorContext>, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".reviewr");

        let error_log_path = data_dir.join("error.log");

        if !error_log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(error_log_path)?;
        let reader = BufReader::new(file);

        let mut errors = Vec::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(error) = serde_json::from_str::<ErrorContext>(&line) {
                if let Some(platform) = platform_filter {
                    if error.platform_id != platform {
                        continue;
                    }
                }
                errors.push(error);
            }
        }

        // Return most recent errors first
        errors.reverse();
        errors.truncate(limit);

        Ok(errors)
    }

    /// Get error statistics by reading the log file
    pub fn get_error_stats()
    -> Result<HashMap<String, ErrorStats>, Box<dyn std::error::Error + Send + Sync>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let data_dir = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".reviewr");

        let error_log_path = data_dir.join("error.log");

        if !error_log_path.exists() {
            return Ok(HashMap::new());
        }

        let file = File::open(error_log_path)?;
        let reader = BufReader::new(file);

        let mut stats: HashMap<String, ErrorStats> = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            if let Ok(error) = serde_json::from_str::<ErrorContext>(&line) {
                let entry = stats
                    .entry(error.platform_id.clone())
                    .or_insert_with(ErrorStats::new);
                entry.total_errors += 1;

                let count = entry
                    .error_types
                    .entry(error.error_type.clone())
                    .or_insert(0);
                *count += 1;

                entry.last_error_time = Some(error.timestamp.clone());
            }
        }

        Ok(stats)
    }
}

/// Statistics about errors for a platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    pub total_errors: usize,
    pub error_types: HashMap<String, usize>,
    pub last_error_time: Option<String>,
}

impl ErrorStats {
    fn new() -> Self {
        Self {
            total_errors: 0,
            error_types: HashMap::new(),
            last_error_time: None,
        }
    }
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlatformError::ConnectionError(msg) => write!(f, "Connection error: {msg}"),
            PlatformError::AuthenticationError(msg) => write!(f, "Authentication error: {msg}"),
            PlatformError::ConfigurationError(msg) => write!(f, "Configuration error: {msg}"),
            PlatformError::ApiError(msg) => write!(f, "API error: {msg}"),
            PlatformError::DataParseError(msg) => write!(f, "Data parse error: {msg}"),
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<PlatformError> for io::Error {
    fn from(err: PlatformError) -> Self {
        io::Error::other(err)
    }
}

/// Platform registry for managing multiple review platforms
pub struct PlatformRegistry {
    platforms: HashMap<String, Box<dyn ReviewPlatform>>,
}

impl PlatformRegistry {
    pub fn new() -> Self {
        Self {
            platforms: HashMap::new(),
        }
    }

    pub fn register_platform(&mut self, platform: Box<dyn ReviewPlatform>) {
        let id = platform.get_platform_id().to_string();
        self.platforms.insert(id, platform);
    }

    pub fn get_platform(&self, id: &str) -> Option<&dyn ReviewPlatform> {
        self.platforms.get(id).map(|p| p.as_ref())
    }

    pub fn get_configured_platforms(&self) -> Vec<&dyn ReviewPlatform> {
        self.platforms
            .values()
            .filter(|p| p.is_configured())
            .map(|p| p.as_ref())
            .collect()
    }

    pub fn get_all_platforms(&self) -> Vec<&dyn ReviewPlatform> {
        self.platforms.values().map(|p| p.as_ref()).collect()
    }

    pub async fn test_all_connections(&self) -> HashMap<String, ConnectionStatus> {
        let mut results = HashMap::new();
        for platform in self.platforms.values() {
            let status = if platform.is_configured() {
                platform
                    .test_connection()
                    .await
                    .unwrap_or_else(|e| ConnectionStatus::Error(e.to_string()))
            } else {
                ConnectionStatus::NotConfigured
            };
            results.insert(platform.get_platform_id().to_string(), status);
        }
        results
    }
}

impl Default for PlatformRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_error_context_creation() {
        let error = ErrorContext::new("test_platform", "test_operation");

        assert_eq!(error.platform_id, "test_platform");
        assert_eq!(error.operation, "test_operation");
        assert!(error.user.is_none());
        assert!(!error.timestamp.is_empty());
        assert!(error.error_type.is_empty());
        assert!(error.error_message.is_empty());
    }

    #[test]
    fn test_error_context_builder_pattern() {
        let error = ErrorContext::new("gerrit", "query_changes")
            .with_user("test@example.com")
            .with_error("network_error", "Connection timeout")
            .with_request_details(
                "https://gerrit.example.com/api",
                Some(500),
                Some("Internal Server Error"),
            )
            .with_metadata("query", "owner:test@example.com");

        assert_eq!(error.platform_id, "gerrit");
        assert_eq!(error.operation, "query_changes");
        assert_eq!(error.user, Some("test@example.com".to_string()));
        assert_eq!(error.error_type, "network_error");
        assert_eq!(error.error_message, "Connection timeout");
        assert_eq!(
            error.request_url,
            Some("https://gerrit.example.com/api".to_string())
        );
        assert_eq!(error.status_code, Some(500));
        assert_eq!(
            error.response_body,
            Some("Internal Server Error".to_string())
        );
        assert_eq!(
            error.metadata.get("query"),
            Some(&"owner:test@example.com".to_string())
        );
    }

    #[test]
    fn test_error_context_serialization() {
        let error = ErrorContext::new("jira", "search_issues")
            .with_user("user@test.com")
            .with_error("json_parse_error", "Invalid JSON response");

        let json = serde_json::to_string(&error).expect("Should serialize to JSON");
        let deserialized: ErrorContext =
            serde_json::from_str(&json).expect("Should deserialize from JSON");

        assert_eq!(error.platform_id, deserialized.platform_id);
        assert_eq!(error.operation, deserialized.operation);
        assert_eq!(error.user, deserialized.user);
        assert_eq!(error.error_type, deserialized.error_type);
        assert_eq!(error.error_message, deserialized.error_message);
    }

    #[test]
    fn test_error_stats_creation() {
        let stats = ErrorStats::new();

        assert_eq!(stats.total_errors, 0);
        assert!(stats.error_types.is_empty());
        assert!(stats.last_error_time.is_none());
    }

    #[test]
    fn test_error_log_reader_empty_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        let errors =
            ErrorLogReader::read_recent_errors(10, None).expect("Should read empty errors");
        assert!(errors.is_empty());

        let stats = ErrorLogReader::get_error_stats().expect("Should get empty stats");
        assert!(stats.is_empty());

        // Restore original HOME if it existed
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn test_error_log_reader_with_data() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let original_home = std::env::var("HOME").ok();
        unsafe {
            std::env::set_var("HOME", temp_dir.path());
        }

        // Create .reviewr directory and error.log
        let reviewr_dir = temp_dir.path().join(".reviewr");
        fs::create_dir_all(&reviewr_dir).expect("Should create reviewr dir");

        let error_log_path = reviewr_dir.join("error.log");

        // Write test error data
        let error1 = ErrorContext::new("gerrit", "query_changes")
            .with_user("user1@test.com")
            .with_error("network_error", "Connection failed");

        let error2 = ErrorContext::new("jira", "search_issues")
            .with_user("user2@test.com")
            .with_error("api_error", "HTTP 401");

        let json1 = serde_json::to_string(&error1).expect("Should serialize error1");
        let json2 = serde_json::to_string(&error2).expect("Should serialize error2");

        fs::write(&error_log_path, format!("{}\n{}\n", json1, json2))
            .expect("Should write errors to file");

        // Test reading errors
        let errors = ErrorLogReader::read_recent_errors(10, None).expect("Should read errors");
        assert_eq!(errors.len(), 2);

        // Should be in reverse order (most recent first)
        assert_eq!(errors[0].platform_id, "jira");
        assert_eq!(errors[1].platform_id, "gerrit");

        // Test filtering by platform
        let gerrit_errors = ErrorLogReader::read_recent_errors(10, Some("gerrit"))
            .expect("Should read gerrit errors");
        assert_eq!(gerrit_errors.len(), 1);
        assert_eq!(gerrit_errors[0].platform_id, "gerrit");

        // Test statistics
        let stats = ErrorLogReader::get_error_stats().expect("Should get stats");
        assert_eq!(stats.len(), 2);

        let gerrit_stats = stats.get("gerrit").expect("Should have gerrit stats");
        assert_eq!(gerrit_stats.total_errors, 1);
        assert_eq!(gerrit_stats.error_types.get("network_error"), Some(&1));

        let jira_stats = stats.get("jira").expect("Should have jira stats");
        assert_eq!(jira_stats.total_errors, 1);
        assert_eq!(jira_stats.error_types.get("api_error"), Some(&1));

        // Restore original HOME if it existed
        if let Some(home) = original_home {
            unsafe {
                std::env::set_var("HOME", home);
            }
        } else {
            unsafe {
                std::env::remove_var("HOME");
            }
        }
    }
}
