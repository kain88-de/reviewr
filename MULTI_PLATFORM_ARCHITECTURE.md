# Multi-Platform Architecture Documentation

## Overview

Reviewr has been designed with a flexible multi-platform architecture that allows seamless integration of different review and issue tracking systems. The current implementation supports Gerrit (code review) and JIRA (issue tracking), with the architecture designed to easily accommodate additional platforms like GitLab, GitHub, Bitbucket, and others.

## Architecture Components

### 1. Platform Abstraction Layer (`src/core/platform.rs`)

The core of the multi-platform architecture is the `ReviewPlatform` trait, which defines a common interface for all platforms:

```rust
#[async_trait::async_trait]
pub trait ReviewPlatform: Send + Sync {
    // Core data retrieval
    async fn get_activity_metrics(&self, user: &str, days: u32) -> io::Result<ActivityMetrics>;
    async fn get_detailed_activities(&self, user: &str, days: u32) -> io::Result<DetailedActivities>;
    async fn search_items(&self, query: &str, user: &str) -> io::Result<Vec<ActivityItem>>;

    // Platform identification
    fn get_platform_name(&self) -> &str;
    fn get_platform_icon(&self) -> &str;
    fn get_platform_id(&self) -> &str;

    // Configuration and health
    fn is_configured(&self) -> bool;
    async fn test_connection(&self) -> io::Result<ConnectionStatus>;
    fn get_item_url(&self, item: &ActivityItem) -> String;
}
```

### 2. Unified Data Models

All platforms translate their native data into common structures:

#### ActivityItem
Represents any reviewable item (code change, issue ticket, merge request):
```rust
pub struct ActivityItem {
    pub id: String,           // Platform-specific ID
    pub title: String,        // Human-readable title
    pub status: String,       // Current status
    pub created: String,      // Creation timestamp
    pub updated: String,      // Last update timestamp
    pub url: String,         // Direct link to item
    pub platform: String,    // Platform identifier
    pub category: ActivityCategory,
    pub project: String,     // Project/repository name
    pub metadata: HashMap<String, String>, // Platform-specific data
}
```

#### ActivityCategory
Standardized categories across platforms:
- **Code Review**: `ChangesCreated`, `ChangesReviewed`, `ChangesMerged`, `ReviewsGiven`, `ReviewsReceived`
- **Issue Tracking**: `IssuesCreated`, `IssuesAssigned`, `IssuesResolved`, `IssuesCommented`
- **Repository**: `MergeRequestsCreated`, `MergeRequestsReviewed`, `MergeRequestsMerged`, `CommitsPushed`
- **Generic**: `Other(String)` for platform-specific categories

#### ConnectionStatus
Platform health indicators:
- `Connected` - Platform is accessible and working
- `Warning(String)` - Platform accessible but has issues
- `Error(String)` - Platform not accessible or has errors
- `NotConfigured` - Platform not set up

### 3. Platform Registry (`src/core/platform.rs`)

The `PlatformRegistry` manages multiple platform instances:

```rust
pub struct PlatformRegistry {
    platforms: HashMap<String, Box<dyn ReviewPlatform>>,
}

impl PlatformRegistry {
    pub fn new() -> Self;
    pub fn register_platform(&mut self, platform: Box<dyn ReviewPlatform>);
    pub fn get_platform(&self, id: &str) -> Option<&dyn ReviewPlatform>;
    pub fn get_configured_platforms(&self) -> Vec<&dyn ReviewPlatform>;
    pub async fn test_all_connections(&self) -> HashMap<String, ConnectionStatus>;
}
```

### 4. Unified Configuration (`src/core/unified_config.rs`)

Manages configuration for all platforms in a single structure:

```rust
pub struct UnifiedConfig {
    pub platforms: PlatformConfigs,           // Platform-specific configs
    pub cross_references: CrossSystemMapping, // Cross-platform linking
    pub ui_preferences: UiPreferences,        // User interface settings
    pub version: u32,                        // Config format version
}

pub struct PlatformConfigs {
    pub gerrit: Option<GerritConfig>,
    pub jira: Option<JiraConfig>,
    pub gitlab: Option<GitLabConfig>,
}
```

## Platform Implementations

### Gerrit Platform (`src/core/gerrit.rs`)

Integrates with Gerrit code review system:

**Configuration**:
```toml
# gerrit_config.toml
gerrit_url = "https://review.example.com"
username = "your-username"
http_password = "your-http-password"
```

**Features**:
- HTTP Basic Auth authentication
- Change tracking (created, merged, reviewed)
- Comment analysis and review metrics
- Direct links to Gerrit changes
- Connection testing

**Activity Categories**:
- `ChangesCreated` - Changes authored by user
- `ChangesMerged` - Changes successfully merged
- `ReviewsGiven` - Reviews provided by user
- `ReviewsReceived` - Reviews received on user's changes

### JIRA Platform (`src/core/jira.rs`)

Integrates with Atlassian JIRA issue tracking:

**Configuration**:
```toml
# jira_config.toml
jira_url = "https://company.atlassian.net"
username = "user@company.com"
api_token = "your-api-token"
project_filter = ["PROJ", "TEAM"]  # Optional: limit to specific projects
```

**Features**:
- API token authentication
- JQL-based issue queries
- Issue lifecycle tracking
- Project and component filtering
- Custom field support

**Activity Categories**:
- `IssuesCreated` - Issues created by user
- `IssuesResolved` - Issues resolved by user
- `IssuesAssigned` - Issues currently assigned to user
- `IssuesCommented` - Issues commented on by user

## User Interface

### Multi-Platform TUI (`src/tui/multi_platform_browser.rs`)

The TUI provides a unified interface for browsing data from all platforms:

**Navigation Hierarchy**:
1. **Summary View** - Overview of all configured platforms
2. **Platform View** - Categories available within a platform
3. **Category View** - Individual items within a category

**Controls**:
- `Tab/Shift+Tab` - Switch between platforms
- `Enter` - Drill down into selected item
- `Backspace` - Go back to previous level
- `↑/↓` - Navigate within lists
- `h/?` - Show help
- `q/Esc` - Quit

**Features**:
- Real-time platform status indicators
- Item count summaries
- Direct browser integration (opens items in web browser)
- Responsive layout with detail panels
- Error handling for missing data

## Adding New Platforms

To add a new platform (e.g., GitLab), follow this pattern:

### 1. Create Platform Module

Create `src/core/gitlab.rs`:

```rust
use crate::core::platform::{ReviewPlatform, ActivityMetrics, DetailedActivities, ActivityItem, ConnectionStatus};
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitLabConfig {
    pub gitlab_url: String,
    pub access_token: String,
    pub username: String,
    pub group_filter: Vec<String>,
    pub project_filter: Vec<String>,
}

pub struct GitLabPlatform {
    data_path: DataPath,
}

#[async_trait]
impl ReviewPlatform for GitLabPlatform {
    async fn get_activity_metrics(&self, user: &str, days: u32) -> io::Result<ActivityMetrics> {
        // Implementation using GitLab API
    }

    async fn get_detailed_activities(&self, user: &str, days: u32) -> io::Result<DetailedActivities> {
        // Implementation using GitLab API
    }

    // ... implement other trait methods

    fn get_platform_name(&self) -> &str { "GitLab" }
    fn get_platform_icon(&self) -> &str { "🦊" }
    fn get_platform_id(&self) -> &str { "gitlab" }
}
```

### 2. Add to Unified Configuration

Update `src/core/unified_config.rs`:

```rust
pub struct PlatformConfigs {
    pub gerrit: Option<GerritConfig>,
    pub jira: Option<JiraConfig>,
    pub gitlab: Option<GitLabConfig>,  // Add new platform
}
```

### 3. Register in CLI

Update `src/cli/mod.rs`:

```rust
fn create_platform_registry(data_path: &DataPath) -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();

    registry.register_platform(Box::new(GerritPlatform::new(data_path.clone())));
    registry.register_platform(Box::new(JiraPlatform::new(data_path.clone())));
    registry.register_platform(Box::new(GitLabPlatform::new(data_path.clone()))); // Add new platform

    registry
}
```

### 4. Add Module Declaration

Update `src/core/mod.rs`:

```rust
pub mod gitlab;  // Add new module
```

## Configuration Management

### Legacy Migration

The system automatically migrates from legacy single-platform configurations:

1. Detects existing `gerrit_config.toml` or `config.toml`
2. Migrates settings to unified configuration format
3. Preserves existing settings and adds new defaults
4. Logs migration process for user awareness

### Validation

Configuration validation occurs at multiple levels:

- **Syntax Validation**: TOML parsing and structure validation
- **Semantic Validation**: URL format, required field presence
- **Connection Testing**: Live platform connectivity tests
- **Credential Validation**: Authentication verification

### Storage

Configurations are stored in the user's data directory:

```
~/.reviewr/
├── config.toml                 # Unified configuration
├── gerrit_config.toml         # Legacy Gerrit config (migrated)
├── jira_config.toml           # Legacy JIRA config (migrated)
├── employees/                 # Employee data
└── notes/                     # Review notes
```

## Error Handling

### Platform-Level Errors

Each platform handles errors gracefully:

- **Network Errors**: Timeouts, connection failures
- **Authentication Errors**: Invalid credentials, expired tokens
- **API Errors**: Rate limiting, service unavailable
- **Data Parsing Errors**: Invalid JSON/XML responses

### System-Level Error Handling

- **Configuration Errors**: Invalid settings, missing required fields
- **Platform Registration**: Duplicate IDs, invalid implementations
- **Data Consistency**: Cross-platform data validation
- **UI Errors**: Navigation state management, rendering failures

### User Experience

- Non-blocking error handling (one platform failure doesn't stop others)
- Clear error messages with actionable guidance
- Graceful degradation (missing data shown as "unavailable")
- Diagnostic information for troubleshooting

## Performance Considerations

### Async Data Loading

- All platform operations are asynchronous
- Concurrent data fetching from multiple platforms
- Progress indicators during loading
- Timeout handling for slow platforms

### Caching Strategy

- Platform connection status caching
- Activity data caching with TTL
- Configuration validation result caching
- UI state preservation across navigation

### Memory Management

- Streaming data processing for large datasets
- Lazy loading of detailed activity information
- Efficient data structures for UI rendering
- Proper cleanup of async resources

## Security

### Credential Management

- No plaintext password storage
- Support for API tokens and application passwords
- Secure credential validation
- Clear guidance on credential creation

### Network Security

- HTTPS-only communication
- Certificate validation
- Request timeout limits
- Rate limiting compliance

### Data Privacy

- Local-only data storage
- No telemetry or usage tracking
- User-controlled data retention
- Clear data access patterns

## Testing Strategy

### Unit Tests

- Platform implementation correctness
- Data model validation
- Configuration parsing and validation
- Error handling scenarios

### Integration Tests

- End-to-end platform connectivity
- Multi-platform data consistency
- UI navigation and state management
- Configuration migration accuracy

### Performance Tests

- Large dataset handling
- Concurrent platform access
- Memory usage under load
- UI responsiveness

This architecture provides a solid foundation for supporting additional review and issue tracking platforms while maintaining a consistent user experience and robust error handling.
