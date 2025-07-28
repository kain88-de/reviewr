use crate::core::platform::{ActivityCategory, ActivityItem, ActivityMetrics, DetailedActivities};
use std::collections::HashMap;

pub fn create_sample_activity_metrics() -> ActivityMetrics {
    let mut metrics = ActivityMetrics::default();
    metrics.total_items = 15;

    // Add sample category data
    metrics
        .items_by_category
        .insert(ActivityCategory::ChangesCreated, 5);
    metrics
        .items_by_category
        .insert(ActivityCategory::ChangesMerged, 3);
    metrics
        .items_by_category
        .insert(ActivityCategory::ReviewsGiven, 4);
    metrics
        .items_by_category
        .insert(ActivityCategory::ReviewsReceived, 3);

    // Add platform-specific data
    metrics
        .platform_specific
        .insert("changes_created".to_string(), 5);
    metrics
        .platform_specific
        .insert("commits_merged".to_string(), 3);
    metrics
        .platform_specific
        .insert("reviews_given".to_string(), 4);
    metrics
        .platform_specific
        .insert("reviews_received".to_string(), 3);

    metrics
}

pub fn create_sample_jira_metrics() -> ActivityMetrics {
    let mut metrics = ActivityMetrics::default();
    metrics.total_items = 12;

    metrics
        .items_by_category
        .insert(ActivityCategory::IssuesCreated, 4);
    metrics
        .items_by_category
        .insert(ActivityCategory::IssuesResolved, 3);
    metrics
        .items_by_category
        .insert(ActivityCategory::IssuesAssigned, 3);
    metrics
        .items_by_category
        .insert(ActivityCategory::IssuesCommented, 2);

    metrics
        .platform_specific
        .insert("tickets_created".to_string(), 4);
    metrics
        .platform_specific
        .insert("tickets_resolved".to_string(), 3);
    metrics
        .platform_specific
        .insert("tickets_assigned".to_string(), 3);
    metrics
        .platform_specific
        .insert("comments_added".to_string(), 2);

    metrics
}

pub fn create_sample_detailed_activities() -> DetailedActivities {
    let mut activities = DetailedActivities::default();

    // Sample Gerrit changes
    let gerrit_changes = vec![
        ActivityItem {
            id: "12345".to_string(),
            title: "Fix critical bug in authentication module".to_string(),
            status: "MERGED".to_string(),
            created: "2024-01-15T10:30:00Z".to_string(),
            updated: "2024-01-15T16:45:00Z".to_string(),
            url: "https://gerrit.example.com/c/project/+/12345".to_string(),
            platform: "gerrit".to_string(),
            category: ActivityCategory::ChangesMerged,
            project: "auth-service".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("change_id".to_string(), "I1234567890abcdef".to_string());
                map.insert("owner_name".to_string(), "John Doe".to_string());
                map
            },
        },
        ActivityItem {
            id: "12346".to_string(),
            title: "Add comprehensive unit tests for user service".to_string(),
            status: "NEW".to_string(),
            created: "2024-01-16T09:15:00Z".to_string(),
            updated: "2024-01-16T14:20:00Z".to_string(),
            url: "https://gerrit.example.com/c/project/+/12346".to_string(),
            platform: "gerrit".to_string(),
            category: ActivityCategory::ChangesCreated,
            project: "user-service".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("change_id".to_string(), "I9876543210fedcba".to_string());
                map.insert("owner_name".to_string(), "John Doe".to_string());
                map
            },
        },
    ];

    activities.items_by_category.insert(
        ActivityCategory::ChangesMerged,
        vec![gerrit_changes[0].clone()],
    );
    activities.items_by_category.insert(
        ActivityCategory::ChangesCreated,
        vec![gerrit_changes[1].clone()],
    );

    activities
}

pub fn create_sample_jira_activities() -> DetailedActivities {
    let mut activities = DetailedActivities::default();

    let jira_issues = vec![
        ActivityItem {
            id: "PROJ-123".to_string(),
            title: "Implement OAuth2 integration for third-party services".to_string(),
            status: "Done".to_string(),
            created: "2024-01-10T08:00:00Z".to_string(),
            updated: "2024-01-14T17:30:00Z".to_string(),
            url: "https://jira.example.com/browse/PROJ-123".to_string(),
            platform: "jira".to_string(),
            category: ActivityCategory::IssuesResolved,
            project: "PROJ".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("issue_type".to_string(), "Story".to_string());
                map.insert("priority".to_string(), "High".to_string());
                map.insert("assignee".to_string(), "John Doe".to_string());
                map
            },
        },
        ActivityItem {
            id: "PROJ-124".to_string(),
            title: "Investigate performance issues in search functionality".to_string(),
            status: "In Progress".to_string(),
            created: "2024-01-12T11:30:00Z".to_string(),
            updated: "2024-01-16T15:45:00Z".to_string(),
            url: "https://jira.example.com/browse/PROJ-124".to_string(),
            platform: "jira".to_string(),
            category: ActivityCategory::IssuesAssigned,
            project: "PROJ".to_string(),
            metadata: {
                let mut map = HashMap::new();
                map.insert("issue_type".to_string(), "Bug".to_string());
                map.insert("priority".to_string(), "Medium".to_string());
                map.insert("assignee".to_string(), "John Doe".to_string());
                map
            },
        },
    ];

    activities.items_by_category.insert(
        ActivityCategory::IssuesResolved,
        vec![jira_issues[0].clone()],
    );
    activities.items_by_category.insert(
        ActivityCategory::IssuesAssigned,
        vec![jira_issues[1].clone()],
    );

    activities
}

pub fn create_empty_activities() -> DetailedActivities {
    DetailedActivities::default()
}

pub fn create_large_dataset_activities() -> DetailedActivities {
    let mut activities = DetailedActivities::default();

    // Create a large number of items to test rendering performance and pagination
    let changes: Vec<ActivityItem> = (1..=50)
        .map(|i| ActivityItem {
            id: format!("CHANGE-{}", i),
            title: format!(
                "Sample change #{}: Refactor component for better maintainability",
                i
            ),
            status: if i % 3 == 0 {
                "MERGED".to_string()
            } else {
                "NEW".to_string()
            },
            created: format!("2024-01-{:02}T{:02}:00:00Z", (i % 30) + 1, (i % 24)),
            updated: format!("2024-01-{:02}T{:02}:30:00Z", (i % 30) + 1, (i % 24)),
            url: format!("https://gerrit.example.com/c/project/+/{}", 10000 + i),
            platform: "gerrit".to_string(),
            category: ActivityCategory::ChangesCreated,
            project: format!("project-{}", (i % 5) + 1),
            metadata: HashMap::new(),
        })
        .collect();

    activities
        .items_by_category
        .insert(ActivityCategory::ChangesCreated, changes);

    activities
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_metrics_consistency() {
        let metrics = create_sample_activity_metrics();

        // Verify total matches sum of categories
        let category_sum: u32 = metrics.items_by_category.values().sum();
        assert_eq!(metrics.total_items, category_sum);

        // Verify essential categories are present
        assert!(
            metrics
                .items_by_category
                .contains_key(&ActivityCategory::ChangesCreated)
        );
        assert!(
            metrics
                .items_by_category
                .contains_key(&ActivityCategory::ChangesMerged)
        );
    }

    #[test]
    fn test_detailed_activities_structure() {
        let activities = create_sample_detailed_activities();

        // Verify items exist in expected categories
        assert!(
            activities
                .items_by_category
                .contains_key(&ActivityCategory::ChangesMerged)
        );
        assert!(
            activities
                .items_by_category
                .contains_key(&ActivityCategory::ChangesCreated)
        );

        // Verify items have required fields
        for items in activities.items_by_category.values() {
            for item in items {
                assert!(!item.id.is_empty());
                assert!(!item.title.is_empty());
                assert!(!item.url.is_empty());
                assert!(!item.platform.is_empty());
            }
        }
    }

    #[test]
    fn test_large_dataset_generation() {
        let activities = create_large_dataset_activities();
        let changes = activities
            .items_by_category
            .get(&ActivityCategory::ChangesCreated)
            .unwrap();

        assert_eq!(changes.len(), 50);

        // Verify variety in the generated data
        let projects: std::collections::HashSet<_> = changes.iter().map(|c| &c.project).collect();
        assert!(projects.len() > 1, "Should have multiple projects");

        let statuses: std::collections::HashSet<_> = changes.iter().map(|c| &c.status).collect();
        assert!(statuses.len() > 1, "Should have multiple statuses");
    }
}
