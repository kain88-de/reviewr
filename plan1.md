# Implementation Plan - Phase 1: Architecture Refactoring

## Overview
Create platform abstraction layer to support multiple review systems (Gerrit, JIRA, GitLab) while maintaining existing Gerrit functionality.

## Phase 1: Architecture Refactoring

### 1.1 Platform Abstraction Layer
```rust
// src/core/platform.rs
pub trait ReviewPlatform {
    async fn get_activity_metrics(&self, user: &str, days: u32) -> Result<ActivityMetrics>;
    async fn get_detailed_activities(&self, user: &str, days: u32) -> Result<DetailedActivities>;
    async fn search_items(&self, query: &str, user: &str) -> Result<Vec<ActivityItem>>;

    fn get_platform_name(&self) -> &str;
    fn get_platform_icon(&self) -> &str;
    fn is_configured(&self) -> bool;
    fn test_connection(&self) -> Result<ConnectionStatus>;
    fn get_item_url(&self, item: &ActivityItem) -> String;
}

#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub id: String,
    pub title: String,
    pub status: String,
    pub created: String,
    pub updated: String,
    pub url: String,
    pub platform: String,
    pub category: ActivityCategory,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum ActivityCategory {
    CodeReview,
    Issue,
    MergeRequest,
    PullRequest,
}
```

### 1.2 Configuration Management
```rust
// src/core/config.rs
#[derive(Serialize, Deserialize)]
pub struct UnifiedConfig {
    pub systems: SystemConfigs,
    pub cross_references: CrossSystemMapping,
    pub ui_preferences: UiPreferences,
}

#[derive(Serialize, Deserialize)]
pub struct SystemConfigs {
    pub gerrit: Option<GerritConfig>,
    pub jira: Option<JiraConfig>,
    pub gitlab: Option<GitLabConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct CrossSystemMapping {
    pub project_mappings: Vec<ProjectMapping>,
    pub auto_link_patterns: Vec<LinkPattern>,
}
```

### 1.3 Data Layer Unification
```rust
// src/core/unified_metrics.rs
#[derive(Debug, Clone)]
pub struct UnifiedActivityMetrics {
    pub platforms: HashMap<String, PlatformMetrics>,
    pub cross_references: Vec<CrossReference>,
    pub summary: ActivitySummary,
}

#[derive(Debug, Clone)]
pub struct ActivitySummary {
    pub total_items: u32,
    pub items_by_category: HashMap<ActivityCategory, u32>,
    pub recent_activity: Vec<ActivityItem>,
    pub productivity_score: Option<f64>,
}
```

## Implementation Steps

### Step 1: Create Platform Abstraction
- [ ] Create `src/core/platform.rs` with trait definition
- [ ] Define unified data models (`ActivityItem`, `ActivityCategory`)
- [ ] Create connection status and error types

### Step 2: Refactor Gerrit Implementation
- [ ] Implement `ReviewPlatform` trait for existing Gerrit client
- [ ] Update Gerrit data models to use unified types
- [ ] Maintain backward compatibility with existing API

### Step 3: Create Unified Configuration
- [ ] Extend config system to support multiple platforms
- [ ] Create migration path from existing `gerrit_config.toml`
- [ ] Add configuration validation for new structure

### Step 4: Update CLI Layer
- [ ] Create platform registry/manager
- [ ] Update review command to use platform abstraction
- [ ] Maintain existing CLI interface

### Step 5: Prepare TUI for Multi-System
- [ ] Refactor TUI to accept platform-agnostic data
- [ ] Create foundation for tab-based navigation
- [ ] Keep existing Gerrit TUI working

## Success Criteria
- Existing Gerrit functionality unchanged from user perspective
- Clean platform abstraction ready for additional systems
- Configuration supports multiple platforms
- TUI architecture prepared for tabs
- All tests pass
- Clean, documented code ready for JIRA integration

## Technical Debt Addressed
- Tight coupling between TUI and Gerrit-specific data
- Hard-coded platform assumptions in CLI layer
- Configuration system limited to single platform
- No framework for cross-system features

This phase sets the foundation for seamless multi-platform integration while preserving all existing functionality.
