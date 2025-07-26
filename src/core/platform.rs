use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io;

/// Core trait that all review platforms must implement
#[async_trait::async_trait]
pub trait ReviewPlatform: Send + Sync {
    /// Get basic activity metrics for a user
    async fn get_activity_metrics(&self, user: &str, days: u32) -> io::Result<ActivityMetrics>;

    /// Get detailed activities with full item information
    async fn get_detailed_activities(&self, user: &str, days: u32) -> io::Result<DetailedActivities>;

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
    pub cross_references: Vec<CrossReference>,
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

/// Cross-reference between items on different platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    pub from_platform: String,
    pub from_id: String,
    pub to_platform: String,
    pub to_id: String,
    pub reference_type: CrossReferenceType,
    pub confidence: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossReferenceType {
    /// Direct mention (e.g., "PROJ-123" in commit message)
    DirectMention,
    /// Shared project/component
    ProjectRelated,
    /// Time-based correlation
    TimeCorrelated,
    /// Manual mapping
    Manual,
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
        io::Error::new(io::ErrorKind::Other, err)
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
                platform.test_connection().await.unwrap_or_else(|e| {
                    ConnectionStatus::Error(e.to_string())
                })
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
