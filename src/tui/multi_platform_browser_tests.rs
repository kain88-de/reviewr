use crate::core::platform::{
    ActivityCategory, ActivityItem, ActivityMetrics, ConnectionStatus, DetailedActivities,
    PlatformRegistry, ReviewPlatform,
};
use crate::tui::{MultiPlatformBrowser, multi_platform_browser::ViewMode};
use async_trait::async_trait;
use std::collections::HashMap;
use std::io;

/// Mock platform for testing TUI components
pub struct MockPlatform {
    platform_id: String,
    platform_name: String,
    platform_icon: String,
    configured: bool,
    activities: DetailedActivities,
    metrics: ActivityMetrics,
}

impl MockPlatform {
    pub fn new_gerrit() -> Self {
        Self {
            platform_id: "gerrit".to_string(),
            platform_name: "Gerrit".to_string(),
            platform_icon: "🔧".to_string(),
            configured: true,
            activities: Self::create_gerrit_test_data(),
            metrics: Self::create_gerrit_metrics(),
        }
    }

    pub fn new_jira() -> Self {
        Self {
            platform_id: "jira".to_string(),
            platform_name: "JIRA".to_string(),
            platform_icon: "🎫".to_string(),
            configured: true,
            activities: Self::create_jira_test_data(),
            metrics: Self::create_jira_metrics(),
        }
    }

    pub fn new_unconfigured(platform_id: String) -> Self {
        Self {
            platform_id: platform_id.clone(),
            platform_name: platform_id.clone(),
            platform_icon: "❌".to_string(),
            configured: false,
            activities: DetailedActivities::default(),
            metrics: ActivityMetrics::default(),
        }
    }

    fn create_gerrit_test_data() -> DetailedActivities {
        let mut activities = DetailedActivities::default();

        let merged_change = ActivityItem {
            id: "12345".to_string(),
            title: "Fix critical bug in authentication module".to_string(),
            status: "MERGED".to_string(),
            created: "2024-01-15T10:30:00Z".to_string(),
            updated: "2024-01-15T16:45:00Z".to_string(),
            url: "https://gerrit.example.com/c/project/+/12345".to_string(),
            platform: "gerrit".to_string(),
            category: ActivityCategory::ChangesMerged,
            project: "auth-service".to_string(),
            metadata: HashMap::new(),
        };

        let created_change = ActivityItem {
            id: "12346".to_string(),
            title: "Add comprehensive unit tests for user service".to_string(),
            status: "NEW".to_string(),
            created: "2024-01-16T09:15:00Z".to_string(),
            updated: "2024-01-16T14:20:00Z".to_string(),
            url: "https://gerrit.example.com/c/project/+/12346".to_string(),
            platform: "gerrit".to_string(),
            category: ActivityCategory::ChangesCreated,
            project: "user-service".to_string(),
            metadata: HashMap::new(),
        };

        activities
            .items_by_category
            .insert(ActivityCategory::ChangesMerged, vec![merged_change]);
        activities
            .items_by_category
            .insert(ActivityCategory::ChangesCreated, vec![created_change]);

        activities
    }

    fn create_jira_test_data() -> DetailedActivities {
        let mut activities = DetailedActivities::default();

        let resolved_issue = ActivityItem {
            id: "PROJ-123".to_string(),
            title: "Implement OAuth2 integration for third-party services".to_string(),
            status: "Done".to_string(),
            created: "2024-01-10T08:00:00Z".to_string(),
            updated: "2024-01-14T17:30:00Z".to_string(),
            url: "https://jira.example.com/browse/PROJ-123".to_string(),
            platform: "jira".to_string(),
            category: ActivityCategory::IssuesResolved,
            project: "PROJ".to_string(),
            metadata: HashMap::new(),
        };

        let assigned_issue = ActivityItem {
            id: "PROJ-124".to_string(),
            title: "Investigate performance issues in search functionality".to_string(),
            status: "In Progress".to_string(),
            created: "2024-01-12T11:30:00Z".to_string(),
            updated: "2024-01-16T15:45:00Z".to_string(),
            url: "https://jira.example.com/browse/PROJ-124".to_string(),
            platform: "jira".to_string(),
            category: ActivityCategory::IssuesAssigned,
            project: "PROJ".to_string(),
            metadata: HashMap::new(),
        };

        activities
            .items_by_category
            .insert(ActivityCategory::IssuesResolved, vec![resolved_issue]);
        activities
            .items_by_category
            .insert(ActivityCategory::IssuesAssigned, vec![assigned_issue]);

        activities
    }

    fn create_gerrit_metrics() -> ActivityMetrics {
        let mut metrics = ActivityMetrics::default();
        metrics.total_items = 8;
        metrics
            .items_by_category
            .insert(ActivityCategory::ChangesCreated, 3);
        metrics
            .items_by_category
            .insert(ActivityCategory::ChangesMerged, 2);
        metrics
            .items_by_category
            .insert(ActivityCategory::ReviewsGiven, 2);
        metrics
            .items_by_category
            .insert(ActivityCategory::ReviewsReceived, 1);
        metrics
    }

    fn create_jira_metrics() -> ActivityMetrics {
        let mut metrics = ActivityMetrics::default();
        metrics.total_items = 6;
        metrics
            .items_by_category
            .insert(ActivityCategory::IssuesCreated, 2);
        metrics
            .items_by_category
            .insert(ActivityCategory::IssuesResolved, 2);
        metrics
            .items_by_category
            .insert(ActivityCategory::IssuesAssigned, 1);
        metrics
            .items_by_category
            .insert(ActivityCategory::IssuesCommented, 1);
        metrics
    }
}

#[async_trait]
impl ReviewPlatform for MockPlatform {
    async fn get_activity_metrics(&self, _user: &str, _days: u32) -> io::Result<ActivityMetrics> {
        Ok(self.metrics.clone())
    }

    async fn get_detailed_activities(
        &self,
        _user: &str,
        _days: u32,
    ) -> io::Result<DetailedActivities> {
        Ok(self.activities.clone())
    }

    async fn search_items(&self, query: &str, _user: &str) -> io::Result<Vec<ActivityItem>> {
        let mut results = Vec::new();
        for items in self.activities.items_by_category.values() {
            for item in items {
                if item.title.to_lowercase().contains(&query.to_lowercase()) {
                    results.push(item.clone());
                }
            }
        }
        Ok(results)
    }

    fn get_platform_name(&self) -> &str {
        &self.platform_name
    }

    fn get_platform_icon(&self) -> &str {
        &self.platform_icon
    }

    fn get_platform_id(&self) -> &str {
        &self.platform_id
    }

    fn is_configured(&self) -> bool {
        self.configured
    }

    async fn test_connection(&self) -> io::Result<ConnectionStatus> {
        if self.configured {
            Ok(ConnectionStatus::Connected)
        } else {
            Ok(ConnectionStatus::NotConfigured)
        }
    }

    fn get_item_url(&self, item: &ActivityItem) -> String {
        item.url.clone()
    }
}

/// Create a test registry with mock platforms
pub fn create_test_registry() -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();
    registry.register_platform(Box::new(MockPlatform::new_gerrit()));
    registry.register_platform(Box::new(MockPlatform::new_jira()));
    registry
}

/// Create a registry with mixed configured/unconfigured platforms
pub fn create_mixed_registry() -> PlatformRegistry {
    let mut registry = PlatformRegistry::new();
    registry.register_platform(Box::new(MockPlatform::new_gerrit()));
    registry.register_platform(Box::new(MockPlatform::new_unconfigured("jira".to_string())));
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_platform_browser_creation() {
        let registry = create_test_registry();
        let browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Browser should be created with correct employee info
        assert_eq!(browser.employee_name(), "John Doe");
        assert_eq!(browser.employee_email(), "john.doe@example.com");

        // Should have platform metadata from registry
        assert!(browser.platform_names().contains_key("gerrit"));
        assert!(browser.platform_names().contains_key("jira"));
        assert_eq!(
            browser.platform_names().get("gerrit"),
            Some(&"Gerrit".to_string())
        );
        assert_eq!(
            browser.platform_names().get("jira"),
            Some(&"JIRA".to_string())
        );

        // Should have platform icons
        assert_eq!(
            browser.platform_icons().get("gerrit"),
            Some(&"🔧".to_string())
        );
        assert_eq!(
            browser.platform_icons().get("jira"),
            Some(&"🎫".to_string())
        );

        // Should start in Summary view
        assert!(matches!(browser.current_view(), ViewMode::Summary));
    }

    #[tokio::test]
    async fn test_data_loading() {
        let registry = create_test_registry();
        let mut browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Load data from platforms
        browser.load_data(&registry).await.unwrap();

        // Should have data from both platforms
        assert!(browser.platform_activities().contains_key("gerrit"));
        assert!(browser.platform_activities().contains_key("jira"));

        // Verify Gerrit data structure
        let gerrit_activities = browser.platform_activities().get("gerrit").unwrap();
        assert!(
            gerrit_activities
                .items_by_category
                .contains_key(&ActivityCategory::ChangesMerged)
        );
        assert!(
            gerrit_activities
                .items_by_category
                .contains_key(&ActivityCategory::ChangesCreated)
        );

        // Verify JIRA data structure
        let jira_activities = browser.platform_activities().get("jira").unwrap();
        assert!(
            jira_activities
                .items_by_category
                .contains_key(&ActivityCategory::IssuesResolved)
        );
        assert!(
            jira_activities
                .items_by_category
                .contains_key(&ActivityCategory::IssuesAssigned)
        );
    }

    #[test]
    fn test_platform_navigation() {
        let registry = create_test_registry();
        let mut browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Should start with first platform selected
        assert_eq!(browser.selected_platform_index(), 0);

        // Test next platform navigation
        browser.next_platform();
        assert_eq!(browser.selected_platform_index(), 1);

        // Should wrap around
        browser.next_platform();
        assert_eq!(browser.selected_platform_index(), 0);

        // Test previous platform navigation
        browser.prev_platform();
        assert_eq!(browser.selected_platform_index(), 1);

        browser.prev_platform();
        assert_eq!(browser.selected_platform_index(), 0);
    }

    #[test]
    fn test_view_mode_transitions() {
        let registry = create_test_registry();
        let mut browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Load some test data
        let gerrit_activities = MockPlatform::create_gerrit_test_data();
        browser
            .platform_activities_mut()
            .insert("gerrit".to_string(), gerrit_activities);

        // Start in Summary view
        assert!(matches!(browser.current_view(), ViewMode::Summary));

        // Simulate entering platform view
        let platform_id = browser.platform_order()[0].clone();
        browser.set_current_view(ViewMode::PlatformView {
            platform_id: platform_id.clone(),
        });

        match browser.current_view() {
            ViewMode::PlatformView { platform_id } => {
                // Should be one of the configured platforms (order may vary)
                assert!(platform_id == "gerrit" || platform_id == "jira");
            }
            _ => panic!("Should be in PlatformView"),
        }

        // Simulate entering category view
        browser.set_current_view(ViewMode::CategoryView {
            platform_id: platform_id.clone(),
            category: ActivityCategory::ChangesMerged,
        });

        match browser.current_view() {
            ViewMode::CategoryView {
                platform_id,
                category,
            } => {
                // Should be the same platform_id we used above
                assert!(platform_id == "gerrit" || platform_id == "jira");
                assert_eq!(category, &ActivityCategory::ChangesMerged);
            }
            _ => panic!("Should be in CategoryView"),
        }
    }

    #[test]
    fn test_available_categories() {
        let registry = create_test_registry();
        let mut browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Add test data
        let gerrit_activities = MockPlatform::create_gerrit_test_data();
        browser
            .platform_activities_mut()
            .insert("gerrit".to_string(), gerrit_activities);

        let categories = browser.get_available_categories("gerrit");

        // Should have the categories from our test data
        assert!(categories.contains(&ActivityCategory::ChangesMerged));
        assert!(categories.contains(&ActivityCategory::ChangesCreated));
        assert!(categories.len() >= 2);

        // Non-existent platform should return empty
        let empty_categories = browser.get_available_categories("nonexistent");
        assert!(empty_categories.is_empty());
    }

    #[test]
    fn test_category_items() {
        let registry = create_test_registry();
        let mut browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Add test data
        let gerrit_activities = MockPlatform::create_gerrit_test_data();
        browser
            .platform_activities_mut()
            .insert("gerrit".to_string(), gerrit_activities);

        let items = browser.get_category_items("gerrit", &ActivityCategory::ChangesMerged);

        // Should have items from our test data
        assert!(!items.is_empty());
        for item in &items {
            assert_eq!(item.platform, "gerrit");
            assert_eq!(item.category, ActivityCategory::ChangesMerged);
            assert!(!item.id.is_empty());
            assert!(!item.title.is_empty());
        }

        // Non-existent category should return empty
        let empty_items = browser.get_category_items("gerrit", &ActivityCategory::IssuesCreated);
        assert!(empty_items.is_empty());
    }

    #[test]
    fn test_view_mode_titles() {
        let mut platform_names = HashMap::new();
        platform_names.insert("gerrit".to_string(), "Gerrit".to_string());
        platform_names.insert("jira".to_string(), "JIRA".to_string());

        // Test Summary view title
        let summary_view = ViewMode::Summary;
        assert_eq!(
            summary_view.title(&platform_names),
            "📊 Multi-Platform Activity Summary"
        );

        // Test Platform view title
        let platform_view = ViewMode::PlatformView {
            platform_id: "gerrit".to_string(),
        };
        assert_eq!(platform_view.title(&platform_names), "🏢 Gerrit Activity");

        // Test Category view title
        let category_view = ViewMode::CategoryView {
            platform_id: "jira".to_string(),
            category: ActivityCategory::IssuesResolved,
        };
        assert_eq!(
            category_view.title(&platform_names),
            "📋 JIRA - Issues Resolved"
        );

        // Test with unknown platform
        let unknown_platform_view = ViewMode::PlatformView {
            platform_id: "unknown".to_string(),
        };
        assert_eq!(
            unknown_platform_view.title(&platform_names),
            "🏢 unknown Activity"
        );
    }

    #[test]
    fn test_empty_platform_handling() {
        let registry = create_test_registry();
        let browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Test with no data loaded
        let categories = browser.get_available_categories("gerrit");
        assert!(categories.is_empty());

        let items = browser.get_category_items("gerrit", &ActivityCategory::ChangesMerged);
        assert!(items.is_empty());
    }

    #[test]
    fn test_mixed_platform_configuration() {
        let registry = create_mixed_registry();
        let browser = MultiPlatformBrowser::new(
            "John Doe".to_string(),
            "john.doe@example.com".to_string(),
            &registry,
        );

        // Should only include configured platforms in browser
        // The mixed registry has gerrit configured but jira unconfigured
        assert!(browser.platform_names().contains_key("gerrit"));
        assert!(!browser.platform_names().contains_key("jira")); // Unconfigured platforms not included

        // Should only have configured platform icon
        assert_eq!(
            browser.platform_icons().get("gerrit"),
            Some(&"🔧".to_string())
        );
        assert!(browser.platform_icons().get("jira").is_none()); // Unconfigured platform not included
    }
}
