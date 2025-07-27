# Integration Testing Plan

## Overview

This document outlines a comprehensive integration testing strategy for the multi-platform reviewr architecture. The testing plan covers platform integrations, user workflows, configuration management, and error handling scenarios.

## Testing Strategy

### 1. Test Categories

#### A. Platform Integration Tests
- **Gerrit Integration**: End-to-end connectivity and data retrieval
- **JIRA Integration**: API authentication and JQL query validation
- **Multi-Platform Coordination**: Cross-platform data consistency and error isolation

#### B. User Workflow Tests
- **CLI Command Testing**: All subcommands with various parameter combinations
- **TUI Navigation Testing**: Multi-platform browser navigation and state management
- **Configuration Management**: Setup, migration, and validation workflows

#### C. Error Handling Tests
- **Network Failure Scenarios**: Timeout handling, connection errors, authentication failures
- **Data Consistency Tests**: Invalid responses, malformed data, empty datasets
- **Configuration Error Tests**: Invalid configs, missing credentials, corrupted files

#### D. Performance Tests
- **Concurrent Platform Access**: Multiple platforms loading simultaneously
- **Large Dataset Handling**: Performance with hundreds of items
- **Memory Usage**: Long-running TUI sessions and memory leaks

## 2. Test Implementation Structure

### Directory Structure
```
tests/
├── integration/
│   ├── platform_tests/
│   │   ├── gerrit_integration_test.rs
│   │   ├── jira_integration_test.rs
│   │   └── multi_platform_test.rs
│   ├── workflow_tests/
│   │   ├── cli_workflow_test.rs
│   │   ├── tui_navigation_test.rs
│   │   └── configuration_test.rs
│   ├── error_handling_tests/
│   │   ├── network_error_test.rs
│   │   ├── data_consistency_test.rs
│   │   └── config_error_test.rs
│   └── performance_tests/
│       ├── concurrent_access_test.rs
│       ├── large_dataset_test.rs
│       └── memory_usage_test.rs
├── fixtures/
│   ├── mock_servers/
│   │   ├── gerrit_mock.rs
│   │   └── jira_mock.rs
│   ├── test_data/
│   │   ├── gerrit_responses.json
│   │   ├── jira_responses.json
│   │   └── sample_configs.toml
│   └── test_environments/
│       ├── isolated_data_dirs/
│       └── mock_credentials.toml
└── common/
    ├── test_helpers.rs
    ├── mock_server_setup.rs
    └── assertion_helpers.rs
```

## 3. Detailed Test Specifications

### A. Platform Integration Tests

#### Gerrit Integration Test (`tests/integration/platform_tests/gerrit_integration_test.rs`)

```rust
#[tokio::test]
async fn test_gerrit_full_workflow() {
    // Setup mock Gerrit server
    let mock_server = setup_gerrit_mock_server().await;

    // Test configuration
    let temp_dir = create_test_environment();
    let config = create_gerrit_test_config(mock_server.url());

    // Test connection
    let platform = GerritPlatform::new(temp_dir.data_path());
    let status = platform.test_connection().await.unwrap();
    assert_eq!(status, ConnectionStatus::Connected);

    // Test activity metrics
    let metrics = platform.get_activity_metrics("test@example.com", 30).await.unwrap();
    assert!(metrics.total_items > 0);

    // Test detailed activities
    let activities = platform.get_detailed_activities("test@example.com", 30).await.unwrap();
    assert!(!activities.items_by_category.is_empty());

    // Test search functionality
    let results = platform.search_items("feature", "test@example.com").await.unwrap();
    assert!(!results.is_empty());
}

#[tokio::test]
async fn test_gerrit_error_handling() {
    // Test authentication failure
    let platform = create_platform_with_invalid_credentials();
    let status = platform.test_connection().await.unwrap();
    assert!(matches!(status, ConnectionStatus::Error(_)));

    // Test network timeout
    let platform = create_platform_with_timeout_server();
    let result = platform.get_activity_metrics("test@example.com", 30).await;
    assert!(result.is_err());
}

#[test]
fn test_gerrit_data_conversion() {
    // Test ChangeInfo to ActivityItem conversion
    let change_info = create_sample_change_info();
    let platform = GerritPlatform::new(test_data_path());
    let activity_item = platform.convert_change_to_item(&change_info, ActivityCategory::ChangesCreated, "https://gerrit.example.com");

    assert_eq!(activity_item.platform, "gerrit");
    assert_eq!(activity_item.category, ActivityCategory::ChangesCreated);
    assert!(activity_item.url.contains("gerrit.example.com"));
}
```

#### JIRA Integration Test (`tests/integration/platform_tests/jira_integration_test.rs`)

```rust
#[tokio::test]
async fn test_jira_full_workflow() {
    // Setup mock JIRA server
    let mock_server = setup_jira_mock_server().await;

    // Test configuration
    let temp_dir = create_test_environment();
    let config = create_jira_test_config(mock_server.url());

    // Test connection
    let platform = JiraPlatform::new(temp_dir.data_path());
    let status = platform.test_connection().await.unwrap();
    assert_eq!(status, ConnectionStatus::Connected);

    // Test activity metrics
    let metrics = platform.get_activity_metrics("test@example.com", 30).await.unwrap();
    assert!(metrics.total_items > 0);

    // Test JQL queries
    let activities = platform.get_detailed_activities("test@example.com", 30).await.unwrap();
    let created_issues = activities.items_by_category.get(&ActivityCategory::IssuesCreated);
    assert!(created_issues.is_some());
}

#[test]
fn test_jql_query_generation() {
    // Test various JQL query scenarios
    let client = create_test_jira_client();

    // Test user email escaping
    let jql = client.generate_created_tickets_query("user+test@example.com", 30);
    assert!(jql.contains("\"user+test@example.com\""));

    // Test days parameter
    assert!(jql.contains("-30d"));
}

#[tokio::test]
async fn test_jira_rate_limiting() {
    // Test API rate limiting behavior
    let mock_server = setup_rate_limited_jira_server().await;
    let platform = create_jira_platform_with_server(mock_server.url());

    // Make multiple rapid requests
    let results = join_all((0..10).map(|_| {
        platform.get_activity_metrics("test@example.com", 30)
    })).await;

    // Should handle rate limiting gracefully
    assert!(results.iter().any(|r| r.is_ok()));
}
```

### B. Multi-Platform Coordination Tests

#### Multi-Platform Test (`tests/integration/platform_tests/multi_platform_test.rs`)

```rust
#[tokio::test]
async fn test_platform_registry_coordination() {
    let temp_dir = create_test_environment();
    let registry = create_test_platform_registry(&temp_dir);

    // Test platform registration
    let platforms = registry.get_all_platforms();
    assert_eq!(platforms.len(), 2); // Gerrit + JIRA

    // Test configured platforms
    let configured = registry.get_configured_platforms();
    assert!(!configured.is_empty());

    // Test connection status for all platforms
    let statuses = registry.test_all_connections().await;
    assert_eq!(statuses.len(), 2);
    assert!(statuses.values().any(|s| s.is_ok()));
}

#[tokio::test]
async fn test_platform_isolation() {
    // Test that failure in one platform doesn't affect others
    let mut registry = PlatformRegistry::new();

    // Add working platform
    let working_platform = create_working_mock_platform();
    registry.register_platform(Box::new(working_platform));

    // Add failing platform
    let failing_platform = create_failing_mock_platform();
    registry.register_platform(Box::new(failing_platform));

    // Test that registry still works
    let statuses = registry.test_all_connections().await;
    assert!(statuses.values().any(|s| matches!(s, ConnectionStatus::Connected)));
    assert!(statuses.values().any(|s| matches!(s, ConnectionStatus::Error(_))));
}

#[tokio::test]
async fn test_cross_platform_data_consistency() {
    // Test that data models are consistent across platforms
    let registry = create_test_registry_with_mock_data();

    for platform in registry.get_configured_platforms() {
        let activities = platform.get_detailed_activities("test@example.com", 30).await.unwrap();

        // Verify all activity items have required fields
        for items in activities.items_by_category.values() {
            for item in items {
                assert!(!item.id.is_empty());
                assert!(!item.title.is_empty());
                assert!(!item.platform.is_empty());
                assert!(item.url.starts_with("http"));
                assert!(!item.project.is_empty());
            }
        }
    }
}
```

### C. User Workflow Tests

#### CLI Workflow Test (`tests/integration/workflow_tests/cli_workflow_test.rs`)

```rust
#[tokio::test]
async fn test_end_to_end_review_workflow() {
    let temp_dir = create_test_environment_with_mock_servers().await;

    // Step 1: Add employee
    let output = Command::cargo_bin("reviewr")
        .unwrap()
        .args(&["--data-path", temp_dir.path(), "add", "John Doe"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Step 2: Update employee with email
    let output = Command::cargo_bin("reviewr")
        .unwrap()
        .args(&["--data-path", temp_dir.path(), "edit", "John Doe"])
        .write_stdin("John Doe\nSenior Engineer\njohn.doe@example.com\n")
        .output()
        .unwrap();
    assert!(output.status.success());

    // Step 3: Generate review (should use mock data)
    let output = Command::cargo_bin("reviewr")
        .unwrap()
        .args(&["--data-path", temp_dir.path(), "review", "John Doe"])
        .timeout(Duration::from_secs(10))
        .output()
        .unwrap();
    assert!(output.status.success());

    // Verify output contains expected data
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Loaded data from"));
}

#[test]
fn test_config_management_workflow() {
    let temp_dir = create_test_environment();

    // Test config creation
    let output = Command::cargo_bin("reviewr")
        .unwrap()
        .args(&["--data-path", temp_dir.path(), "config", "set", "allowed_domains", "example.com"])
        .output()
        .unwrap();
    assert!(output.status.success());

    // Test config retrieval
    let output = Command::cargo_bin("reviewr")
        .unwrap()
        .args(&["--data-path", temp_dir.path(), "config", "get", "allowed_domains"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout).unwrap().contains("example.com"));
}
```

#### TUI Navigation Test (`tests/integration/workflow_tests/tui_navigation_test.rs`)

```rust
#[test]
fn test_multi_platform_browser_state_management() {
    // Test TUI state transitions
    let registry = create_test_registry();
    let mut browser = MultiPlatformBrowser::new(
        "Test Employee".to_string(),
        "test@example.com".to_string(),
        &registry
    );

    // Test initial state
    assert_eq!(browser.current_view, ViewMode::Summary);

    // Test platform navigation
    browser.handle_key_event(KeyEvent::from(KeyCode::Tab)).unwrap();
    // Verify platform selection updated

    // Test entering platform view
    browser.handle_key_event(KeyEvent::from(KeyCode::Enter)).unwrap();
    assert!(matches!(browser.current_view, ViewMode::PlatformView { .. }));

    // Test going back
    browser.handle_key_event(KeyEvent::from(KeyCode::Backspace)).unwrap();
    assert_eq!(browser.current_view, ViewMode::Summary);
}

#[tokio::test]
async fn test_tui_error_handling() {
    // Test TUI behavior with platform errors
    let registry = create_registry_with_failing_platforms();
    let mut browser = MultiPlatformBrowser::new(
        "Test Employee".to_string(),
        "test@example.com".to_string(),
        &registry
    );

    // Load data (should handle errors gracefully)
    let result = browser.load_data(&registry).await;
    assert!(result.is_ok()); // Should not fail completely

    // Verify graceful degradation
    // (empty platforms should show "No data available")
}
```

### D. Configuration Tests

#### Configuration Test (`tests/integration/workflow_tests/configuration_test.rs`)

```rust
#[test]
fn test_unified_config_migration() {
    let temp_dir = create_test_environment();

    // Create legacy Gerrit config
    let gerrit_config = r#"
        gerrit_url = "https://gerrit.example.com"
        username = "testuser"
        http_password = "testpass"
    "#;
    fs::write(temp_dir.path().join("gerrit_config.toml"), gerrit_config).unwrap();

    // Create legacy main config
    let main_config = r#"
        allowed_domains = ["example.com", "test.com"]
    "#;
    fs::write(temp_dir.path().join("config.toml"), main_config).unwrap();

    // Load unified config (should migrate)
    let data_path = DataPath::new(temp_dir.path().to_path_buf());
    let config = UnifiedConfigService::load_config(&data_path).unwrap();

    // Verify migration
    assert!(config.platforms.gerrit.is_some());
    assert_eq!(config.platforms.gerrit.unwrap().gerrit_url, "https://gerrit.example.com");
}

#[test]
fn test_config_validation() {
    // Test various invalid configurations
    let invalid_configs = vec![
        r#"
        [platforms.gerrit]
        gerrit_url = "not-a-url"
        username = ""
        http_password = "test"
        "#,
        r#"
        [platforms.jira]
        jira_url = "ftp://invalid-protocol.com"
        username = "test"
        api_token = ""
        "#,
    ];

    for config_str in invalid_configs {
        let config: Result<UnifiedConfig, _> = toml::from_str(config_str);
        if let Ok(config) = config {
            // Should fail validation
            assert!(UnifiedConfigService::validate_platform_config("gerrit", &serde_json::to_value(&config.platforms.gerrit).unwrap()).is_err());
        }
    }
}
```

### E. Error Handling Tests

#### Network Error Test (`tests/integration/error_handling_tests/network_error_test.rs`)

```rust
#[tokio::test]
async fn test_network_timeout_handling() {
    // Create server that responds slowly
    let slow_server = create_slow_response_server(Duration::from_secs(5)).await;

    let platform = create_platform_with_short_timeout(slow_server.url());

    // Should timeout gracefully
    let start = Instant::now();
    let result = platform.get_activity_metrics("test@example.com", 30).await;
    let elapsed = start.elapsed();

    assert!(result.is_err());
    assert!(elapsed < Duration::from_secs(2)); // Should timeout quickly
    assert!(result.unwrap_err().to_string().contains("timeout"));
}

#[tokio::test]
async fn test_authentication_error_handling() {
    // Test various authentication error scenarios
    let auth_error_server = create_auth_error_server().await;
    let platform = create_platform_with_server(auth_error_server.url());

    let status = platform.test_connection().await.unwrap();
    assert!(matches!(status, ConnectionStatus::Error(_)));
    assert!(status.to_string().contains("authentication"));
}

#[tokio::test]
async fn test_malformed_response_handling() {
    // Test handling of invalid JSON responses
    let invalid_json_server = create_invalid_json_server().await;
    let platform = create_platform_with_server(invalid_json_server.url());

    let result = platform.get_activity_metrics("test@example.com", 30).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid JSON"));
}
```

### F. Performance Tests

#### Performance Test (`tests/integration/performance_tests/concurrent_access_test.rs`)

```rust
#[tokio::test]
async fn test_concurrent_platform_access() {
    let registry = create_test_registry_with_mock_servers().await;

    // Create multiple concurrent requests
    let tasks: Vec<_> = (0..10).map(|i| {
        let registry = &registry;
        async move {
            let platform = registry.get_platform("gerrit").unwrap();
            platform.get_activity_metrics(&format!("user{i}@example.com"), 30).await
        }
    }).collect();

    let start = Instant::now();
    let results = join_all(tasks).await;
    let elapsed = start.elapsed();

    // All requests should complete
    assert!(results.iter().all(|r| r.is_ok()));

    // Should complete within reasonable time
    assert!(elapsed < Duration::from_secs(5));
}

#[tokio::test]
async fn test_large_dataset_handling() {
    // Test with mock server returning large datasets
    let large_dataset_server = create_large_dataset_server(1000).await; // 1000 items
    let platform = create_platform_with_server(large_dataset_server.url());

    let start = Instant::now();
    let activities = platform.get_detailed_activities("test@example.com", 365).await.unwrap();
    let elapsed = start.elapsed();

    // Should handle large datasets efficiently
    assert!(activities.items_by_category.values().map(|v| v.len()).sum::<usize>() >= 1000);
    assert!(elapsed < Duration::from_secs(10));
}

#[test]
fn test_memory_usage() {
    // Test memory usage during TUI operation
    let registry = create_test_registry();

    let initial_memory = get_memory_usage();

    // Create and destroy many browser instances
    for _ in 0..100 {
        let browser = MultiPlatformBrowser::new(
            "Test Employee".to_string(),
            "test@example.com".to_string(),
            &registry
        );
        drop(browser);
    }

    let final_memory = get_memory_usage();

    // Memory usage should not grow significantly
    assert!(final_memory - initial_memory < 10 * 1024 * 1024); // 10MB threshold
}
```

## 4. Test Infrastructure

### Mock Server Implementation

#### Gerrit Mock Server (`tests/fixtures/mock_servers/gerrit_mock.rs`)

```rust
pub struct GerritMockServer {
    server: MockServer,
}

impl GerritMockServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub fn url(&self) -> String {
        self.server.uri()
    }

    pub async fn setup_changes_endpoint(&self, responses: Vec<ChangeInfo>) {
        Mock::given(method("GET"))
            .and(path_regex(r"/a/changes/.*"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(responses)
                .insert_header("content-type", "application/json"))
            .mount(&self.server)
            .await;
    }

    pub async fn setup_auth_failure(&self) {
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(401)
                .set_body_string("Authentication required"))
            .mount(&self.server)
            .await;
    }

    pub async fn setup_timeout_response(&self, delay: Duration) {
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200)
                .set_delay(delay)
                .set_body_json(json!([])))
            .mount(&self.server)
            .await;
    }
}
```

#### JIRA Mock Server (`tests/fixtures/mock_servers/jira_mock.rs`)

```rust
pub struct JiraMockServer {
    server: MockServer,
}

impl JiraMockServer {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub async fn setup_search_endpoint(&self, issues: Vec<JiraIssue>) {
        Mock::given(method("GET"))
            .and(path("/rest/api/3/search"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "issues": issues,
                    "total": issues.len()
                })))
            .mount(&self.server)
            .await;
    }

    pub async fn setup_rate_limiting(&self, requests_per_minute: u32) {
        // Setup rate limiting mock
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429)
                .set_body_string("Rate limit exceeded")
                .insert_header("retry-after", "60"))
            .up_to_n_times(requests_per_minute as u64)
            .mount(&self.server)
            .await;
    }
}
```

### Test Helpers (`tests/common/test_helpers.rs`)

```rust
pub fn create_test_environment() -> TempDir {
    let temp_dir = tempfile::tempdir().unwrap();

    // Create necessary subdirectories
    fs::create_dir_all(temp_dir.path().join("employees")).unwrap();
    fs::create_dir_all(temp_dir.path().join("notes")).unwrap();

    temp_dir
}

pub fn create_test_employee_data() -> EmployeeData {
    EmployeeData {
        name: "Test Employee".to_string(),
        title: "Software Engineer".to_string(),
        committer_email: Some("test@example.com".to_string()),
    }
}

pub async fn create_test_registry_with_mock_servers() -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();

    // Setup mock servers
    let gerrit_server = GerritMockServer::new().await;
    gerrit_server.setup_changes_endpoint(create_sample_changes()).await;

    let jira_server = JiraMockServer::new().await;
    jira_server.setup_search_endpoint(create_sample_issues()).await;

    // Create platforms with mock server URLs
    let gerrit_platform = create_gerrit_platform_with_server(gerrit_server.url());
    let jira_platform = create_jira_platform_with_server(jira_server.url());

    registry.register_platform(Box::new(gerrit_platform));
    registry.register_platform(Box::new(jira_platform));

    registry
}

pub fn assert_activity_item_valid(item: &ActivityItem) {
    assert!(!item.id.is_empty(), "Activity item ID should not be empty");
    assert!(!item.title.is_empty(), "Activity item title should not be empty");
    assert!(!item.platform.is_empty(), "Activity item platform should not be empty");
    assert!(item.url.starts_with("http"), "Activity item URL should be valid HTTP URL");
    assert!(!item.project.is_empty(), "Activity item project should not be empty");
}
```

## 5. Test Execution

### Running Tests

```bash
# Run all integration tests
cargo test --test '*' -- --test-threads=1

# Run specific test categories
cargo test --test platform_tests
cargo test --test workflow_tests
cargo test --test error_handling_tests
cargo test --test performance_tests

# Run with coverage
cargo tarpaulin --out html --output-dir coverage/

# Run with logging
RUST_LOG=debug cargo test --test integration_test
```

### Continuous Integration

```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  integration-tests:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run integration tests
      run: |
        cargo test --test '*' -- --test-threads=1

    - name: Run performance tests
      run: |
        cargo test --test performance_tests --release

    - name: Generate test coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --out xml

    - name: Upload coverage
      uses: codecov/codecov-action@v1
```

## 6. Test Data Management

### Fixture Data

```rust
// tests/fixtures/test_data/sample_data.rs
pub fn create_sample_changes() -> Vec<ChangeInfo> {
    vec![
        ChangeInfo {
            id: "change1".to_string(),
            change_id: "I1234567890".to_string(),
            subject: "Add new feature".to_string(),
            status: "MERGED".to_string(),
            created: "2023-01-01T00:00:00Z".to_string(),
            updated: "2023-01-02T00:00:00Z".to_string(),
            project: "example-project".to_string(),
            number: 12345,
            owner: Owner {
                name: Some("Test User".to_string()),
                email: Some("test@example.com".to_string()),
            },
        },
        // Add more sample data...
    ]
}

pub fn create_sample_issues() -> Vec<JiraIssue> {
    vec![
        JiraIssue {
            key: "PROJ-123".to_string(),
            fields: JiraFields {
                summary: "Fix critical bug".to_string(),
                status: JiraStatus { name: "Done".to_string() },
                assignee: Some(JiraUser {
                    display_name: Some("Test User".to_string()),
                    email_address: Some("test@example.com".to_string()),
                }),
                created: "2023-01-01T00:00:00Z".to_string(),
                updated: "2023-01-02T00:00:00Z".to_string(),
                project: JiraProject {
                    key: "PROJ".to_string(),
                    name: "Example Project".to_string(),
                },
                // Add other required fields...
            },
        },
        // Add more sample data...
    ]
}
```

## 7. Test Success Criteria

### Coverage Targets
- **Unit Test Coverage**: > 90%
- **Integration Test Coverage**: > 80%
- **Critical Path Coverage**: 100%

### Performance Benchmarks
- **Platform Response Time**: < 2 seconds for 30-day queries
- **TUI Responsiveness**: < 100ms for navigation actions
- **Memory Usage**: < 50MB for typical workflows
- **Concurrent Requests**: Handle 10+ simultaneous platform requests

### Reliability Targets
- **Error Handling**: 100% of error scenarios have graceful handling
- **Data Consistency**: 0% data corruption in any test scenario
- **Platform Isolation**: 100% isolation (one platform failure doesn't affect others)

This comprehensive integration testing plan ensures the multi-platform architecture is robust, performant, and user-friendly across all supported scenarios.
