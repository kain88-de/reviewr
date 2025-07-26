# Multi-System Integration Plan

## UX Design Plan

### Overview
Transform the current Gerrit-only review browser into a unified multi-system dashboard that seamlessly integrates Gerrit, JIRA, and GitLab while maintaining simplicity and discoverability.

### Core UX Principles

1. **Progressive Disclosure**: Start simple, reveal complexity as needed
2. **Consistent Mental Model**: Similar navigation patterns across all systems
3. **Context Preservation**: Remember user's last view and selections
4. **Quick Switching**: Fast transitions between systems and views
5. **Unified Search**: Cross-system search and filtering capabilities

### Primary Navigation Design

#### Tab-Based System Switching
```
[📊 Summary] [🔧 Gerrit] [🎫 JIRA] [🦊 GitLab] [⚙️ Config]
```

- **Summary Tab**: Unified dashboard showing activity across all configured systems
- **System Tabs**: Dedicated views for each platform (only show if configured)
- **Config Tab**: System configuration and credential management
- **Visual Indicators**: Show activity counts, error states, loading status per tab

#### Tab Behavior
- **Auto-hide unconfigured systems**: Only show tabs for configured platforms
- **Badge notifications**: Show pending items count on each tab
- **Status indicators**:
  - ✅ Green: Connected and working
  - ⚠️ Yellow: Configured but connection issues
  - ❌ Red: Configuration errors
  - ⚪ Gray: Not configured

### Unified Summary Dashboard

#### Overview Section
```
📊 Cross-Platform Activity Summary
=====================================
Employee: Max Linke
Period: Last 30 days
Status: 🔧 Gerrit ✅ | 🎫 JIRA ✅ | 🦊 GitLab ❌

📈 Combined Metrics:
Code Reviews    │ Project Management  │ Repository Activity
• Gerrit: 42    │ • JIRA: 15         │ • GitLab: N/A
• Total: 42     │ • Total: 15        │ • Total: 0
```

#### Quick Access Panel
```
🔍 Recent Activity (All Systems):
1. 🔧 [MERGED] Fix authentication bug          (checkmk)
2. 🎫 [DONE]   Implement user dashboard        (PROJ-123)
3. 🔧 [NEW]    Add logging configuration       (core-utils)
4. 🎫 [IN PROGRESS] Database migration task    (PROJ-124)

⚡ Quick Actions:
[G] Gerrit Details  [J] JIRA Details  [L] GitLab Details  [C] Configure
```

### System-Specific Views

#### Enhanced Gerrit View
- **Current functionality preserved** with improvements
- **Additional context**: Link to related JIRA tickets if project mapping configured
- **Cross-references**: "Related JIRA: PROJ-123" in change details

#### JIRA Ticket View
```
🎫 JIRA Activity (15 items)
===========================

📋 Issue Categories:
[A]ssigned (5)  │ [C]reated (8)  │ [R]esolved (2)  │ [W]atching (12)

Current View: Assigned Issues

 Status    │ Key       │ Summary                           │ Project
───────────┼───────────┼───────────────────────────────────┼─────────
 TODO      │ PROJ-123  │ Implement user dashboard          │ MyProject
 PROGRESS  │ PROJ-124  │ Database migration task           │ MyProject
 REVIEW    │ PROJ-125  │ API endpoint optimization         │ CoreAPI

📋 Selected: PROJ-123
Project: MyProject | Assignee: Max Linke
Created: 2025-07-20 | Updated: 2025-07-25
Labels: backend, dashboard, high-priority
URL: https://company.atlassian.net/browse/PROJ-123
Related Gerrit Changes: checkmk:+/54321

Press Enter to open in browser | [A/C/R/W] to switch categories
```

#### GitLab Merge Request View
```
🦊 GitLab Activity (28 items)
============================

📋 MR Categories:
[A]uthored (12)  │ [R]eviewed (16)  │ [M]erged (8)  │ [O]pen (4)

Current View: Authored MRs

 Status │ MR    │ Title                              │ Project
────────┼───────┼────────────────────────────────────┼─────────────
 OPEN   │ !123  │ Feature: Add user authentication   │ web-frontend
 MERGED │ !122  │ Fix: Resolve CORS issues           │ api-backend
 DRAFT  │ !121  │ WIP: New dashboard layout          │ web-frontend

📋 Selected: !123
Project: web-frontend | Author: Max Linke
Created: 2025-07-22 | Last Updated: 2025-07-26
Source: feature/auth → Target: main
URL: https://gitlab.company.com/frontend/web/-/merge_requests/123
Related JIRA: PROJ-123

Press Enter to open in browser | [A/R/M/O] to switch categories
```

### Cross-System Features

#### Unified Search
```
🔍 Search across all systems:
> auth dashboard

Results (23 items):
🔧 Gerrit (8):  "authentication", "dashboard" in subjects
🎫 JIRA (12):   "auth", "dashboard" in summaries
🦊 GitLab (3):  "authentication", "dashboard" in MR titles

[Enter] to explore results by system | [Esc] to cancel
```

#### Smart Linking
- **Auto-detect references**: PROJ-123 mentioned in Gerrit commits → show JIRA link
- **Bidirectional navigation**: From JIRA ticket, show related Gerrit changes
- **Project mapping**: Configure relationships between systems

#### Export & Reporting
```
📊 Generate Report:
• Time period: [Last 30 days ▼]
• Systems: [✓] Gerrit [✓] JIRA [✓] GitLab
• Format: [CSV ▼] JSON | PDF
• Include: [✓] Metrics [✓] Item details [✓] Cross-references

[Generate] [Preview] [Cancel]
```

### Progressive Configuration

#### First-Time Setup Wizard
```
🚀 Welcome to ReviewR Multi-System Setup

Step 1/3: Choose your platforms
[✓] Gerrit - Code review system
[ ] JIRA   - Issue tracking
[ ] GitLab - Repository management

Step 2/3: Configure Gerrit
URL: https://review.company.com
Username: max.linke
Token: [hidden] ••••••••••••
[Test Connection] → ✅ Connected successfully

Step 3/3: Summary
✅ Gerrit configured and tested
⚪ JIRA (optional) - Add later in Config tab
⚪ GitLab (optional) - Add later in Config tab

[Finish Setup] [Add More Systems]
```

#### Incremental Configuration
- **Add systems later**: Don't require all systems upfront
- **Test connections**: Immediate feedback on configuration
- **Optional mapping**: Set up cross-system relationships later

### Error Handling & Offline UX

#### Graceful Degradation
```
🔧 Gerrit: ✅ Connected (42 items)
🎫 JIRA:  ⚠️ Connection timeout - showing cached data (15 items, 2h old)
🦊 GitLab: ❌ Authentication failed - check credentials

💡 Some systems are unavailable. You can:
• [R]etry connections
• [V]iew cached data
• [C]onfigure credentials
• [W]ork offline with available systems
```

#### Offline Capabilities
- **Cache recent data**: Show last successful fetch with timestamps
- **Partial functionality**: Work with available systems
- **Smart retries**: Automatic reconnection attempts
- **Clear status**: Always show what's working and what's not

---

## Implementation Plan

### Phase 1: Architecture Refactoring

#### 1.1 Platform Abstraction Layer
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

#### 1.2 Configuration Management
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

#### 1.3 Data Layer Unification
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

### Phase 2: JIRA Integration

#### 2.1 JIRA API Client
```rust
// src/core/jira.rs
pub struct JiraClient {
    client: Client,
    base_url: String,
    auth_header: String,
}

impl JiraClient {
    pub async fn get_assigned_issues(&self, user: &str, days: u32) -> Result<Vec<IssueInfo>>;
    pub async fn get_created_issues(&self, user: &str, days: u32) -> Result<Vec<IssueInfo>>;
    pub async fn get_resolved_issues(&self, user: &str, days: u32) -> Result<Vec<IssueInfo>>;
    pub async fn search_issues(&self, jql: &str) -> Result<Vec<IssueInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueInfo {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub issue_type: String,
    pub priority: String,
    pub assignee: Option<String>,
    pub created: String,
    pub updated: String,
    pub resolved: Option<String>,
    pub project: ProjectInfo,
    pub labels: Vec<String>,
}
```

#### 2.2 JIRA Platform Implementation
```rust
impl ReviewPlatform for JiraClient {
    async fn get_activity_metrics(&self, user: &str, days: u32) -> Result<ActivityMetrics> {
        // Implement JIRA-specific metrics collection
    }

    fn get_platform_name(&self) -> &str { "JIRA" }
    fn get_platform_icon(&self) -> &str { "🎫" }
    // ... other implementations
}
```

### Phase 3: GitLab Integration

#### 3.1 GitLab API Client
```rust
// src/core/gitlab.rs
pub struct GitLabClient {
    client: Client,
    base_url: String,
    access_token: String,
}

impl GitLabClient {
    pub async fn get_authored_mrs(&self, user: &str, days: u32) -> Result<Vec<MergeRequestInfo>>;
    pub async fn get_reviewed_mrs(&self, user: &str, days: u32) -> Result<Vec<MergeRequestInfo>>;
    pub async fn get_created_issues(&self, user: &str, days: u32) -> Result<Vec<GitLabIssueInfo>>;
}
```

### Phase 4: TUI Multi-System Interface

#### 4.1 Enhanced Navigation
```rust
// src/tui/multi_browser.rs
pub struct MultiSystemBrowser {
    current_tab: SystemTab,
    platforms: HashMap<String, Box<dyn ReviewPlatform>>,
    unified_data: UnifiedActivityMetrics,
    tab_state: TabState,
    search_mode: bool,
    search_query: String,
}

#[derive(Clone, Copy)]
pub enum SystemTab {
    Summary,
    Gerrit,
    Jira,
    GitLab,
    Config,
}
```

#### 4.2 Tab Management
```rust
impl MultiSystemBrowser {
    fn handle_tab_switch(&mut self, tab: SystemTab) -> Result<()>;
    fn render_tab_bar(&self, f: &mut Frame, area: Rect);
    fn get_active_tabs(&self) -> Vec<SystemTab>; // Only show configured systems
    fn update_tab_status(&mut self); // Update connection status, item counts
}
```

#### 4.3 Unified Search Interface
```rust
// src/tui/search.rs
pub struct SearchInterface {
    query: String,
    results: Vec<SearchResult>,
    selected_platform: Option<String>,
    search_state: SearchState,
}

#[derive(Debug)]
pub struct SearchResult {
    pub item: ActivityItem,
    pub relevance_score: f64,
    pub matched_fields: Vec<String>,
}
```

### Phase 5: Cross-System Features

#### 5.1 Smart Linking Engine
```rust
// src/core/linking.rs
pub struct LinkingEngine {
    patterns: Vec<LinkPattern>,
    project_mappings: Vec<ProjectMapping>,
}

impl LinkingEngine {
    pub fn find_cross_references(&self, item: &ActivityItem) -> Vec<CrossReference>;
    pub fn suggest_links(&self, items: &[ActivityItem]) -> Vec<LinkSuggestion>;
}
```

#### 5.2 Export System
```rust
// src/core/export.rs
pub struct ExportEngine;

impl ExportEngine {
    pub fn export_csv(&self, data: &UnifiedActivityMetrics) -> Result<String>;
    pub fn export_json(&self, data: &UnifiedActivityMetrics) -> Result<String>;
    pub fn generate_report(&self, config: &ReportConfig) -> Result<Report>;
}
```

### Phase 6: Configuration & Setup

#### 6.1 Setup Wizard
```rust
// src/tui/setup_wizard.rs
pub struct SetupWizard {
    current_step: SetupStep,
    platforms_to_configure: Vec<String>,
    configurations: HashMap<String, PlatformConfig>,
}

impl SetupWizard {
    pub fn run(&mut self) -> Result<UnifiedConfig>;
    fn test_platform_config(&self, platform: &str, config: &PlatformConfig) -> Result<()>;
}
```

#### 6.2 Incremental Configuration
```rust
// src/tui/config_manager.rs
pub struct ConfigManager {
    config: UnifiedConfig,
    test_results: HashMap<String, ConnectionStatus>,
}

impl ConfigManager {
    pub fn add_platform(&mut self, platform: String) -> Result<()>;
    pub fn test_all_connections(&mut self) -> Result<()>;
    pub fn configure_cross_system_mapping(&mut self) -> Result<()>;
}
```

### Implementation Timeline

#### Phase 1: Architecture (Week 1)
- [ ] Create platform abstraction trait
- [ ] Refactor existing Gerrit code to implement trait
- [ ] Create unified configuration system
- [ ] Set up data models for multi-system support

#### Phase 2: JIRA Integration (Week 2)
- [ ] Implement JIRA API client
- [ ] Create JIRA platform implementation
- [ ] Add JIRA-specific TUI views
- [ ] Test with live JIRA instance

#### Phase 3: GitLab Integration (Week 3)
- [ ] Implement GitLab API client
- [ ] Create GitLab platform implementation
- [ ] Add GitLab-specific TUI views
- [ ] Test with live GitLab instance

#### Phase 4: Multi-System TUI (Week 4)
- [ ] Implement tab-based navigation
- [ ] Create unified summary dashboard
- [ ] Add cross-system search
- [ ] Polish UX and handle edge cases

#### Phase 5: Advanced Features (Week 5)
- [ ] Implement smart linking
- [ ] Add export functionality
- [ ] Create setup wizard
- [ ] Add configuration management

#### Phase 6: Polish & Testing (Week 6)
- [ ] Comprehensive testing across all systems
- [ ] Error handling and offline support
- [ ] Performance optimization
- [ ] Documentation and examples

### Technical Considerations

#### Async Architecture
- All platform clients use async/await
- Concurrent data fetching from multiple systems
- Timeout handling per platform
- Graceful degradation for slow/offline systems

#### Error Handling Strategy
- Platform-specific error types
- Graceful fallbacks when systems are unavailable
- Clear error messaging to users
- Retry mechanisms with exponential backoff

#### Performance Optimization
- Lazy loading of system data
- Caching with TTL for each platform
- Pagination for large datasets
- Background refresh capabilities

#### Security Considerations
- Secure credential storage (keyring integration future)
- Token-based authentication preferred over passwords
- Certificate validation for enterprise environments
- Audit logging for credential access

This plan provides a comprehensive roadmap for transforming reviewr into a powerful multi-system review dashboard while maintaining excellent UX and a clean, maintainable codebase.
