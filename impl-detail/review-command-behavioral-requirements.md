# Review Command Behavioral Requirements - Detailed Specification

## Overview

This document provides detailed behavioral requirements specifically for the `reviewr review` command, its TUI interface, configuration management, and data fetching behaviors. The system should be "talkative" - providing clear feedback about what data is being fetched, from where, and the current status of all operations.

## 1. Review Command Invocation Behaviors

### 1.1 Command Execution Flow

#### Behavior: `reviewr review` (No Employee Specified)
**Expected Observable Behavior**:
1. **Employee Discovery**: System scans `{data-path}/employees/` directory
   - Console output: "Scanning for employees in {data-path}/employees/"
   - Console output: "Found {N} employees: {names...}"
   - If no employees found: "No employees found. Run 'reviewr add' to create an employee first."

2. **Employee Selection**: Interactive selector appears
   - Display format: "Select employee to review:" followed by numbered list
   - Each entry shows: "{N}. {Name} - {Title} ({email})"
   - If employee missing email: "{N}. {Name} - {Title} (⚠️ No email configured)"
   - User selects by number or arrow keys + Enter

3. **Proceed to Review Generation**: Selected employee passed to review process

#### Behavior: `reviewr review "Employee Name"`
**Expected Observable Behavior**:
1. **Employee Validation**: System checks if employee exists
   - Console output: "Looking up employee: Employee Name"
   - If found: "Found employee: {Name} - {Title} ({email})"
   - If not found: "Employee 'Employee Name' not found. Available employees: {list}"
   - If found but no email: "Employee {Name} has no committer email configured. Run 'reviewr edit \"{Name}\"' to add email."

2. **Proceed to Review Generation**: Valid employee passed to review process

### 1.2 Configuration Loading and Validation

#### Behavior: Configuration Discovery
**Expected Observable Behavior**:
1. **Configuration File Detection**:
   - Console output: "Loading configuration from {data-path}/config.toml"
   - If file doesn't exist: "No configuration found, creating default configuration"
   - If file exists but malformed: "Configuration file has syntax errors: {details}"

2. **Platform Configuration Validation**:
   - Console output: "Checking platform configurations..."
   - For each platform section found:
     - "✓ Gerrit configuration found (URL: {gerrit_url})"
     - "✓ JIRA configuration found (URL: {jira_url})"
     - "ℹ️ GitLab configuration not found (optional)"
   - For invalid configurations:
     - "❌ Gerrit configuration invalid: {specific_error}"
     - "❌ JIRA configuration invalid: {specific_error}"

3. **Configuration Summary**:
   - Console output: "Configuration summary:"
   - "  - {N} platforms configured"
   - "  - Default time period: {days} days"
   - "  - Data refresh interval: {minutes} minutes"

## 2. Unified Configuration File Specification

### 2.1 Single Configuration File Structure

The system uses a single `config.toml` file containing all configuration:

```toml
# Reviewr Unified Configuration File
version = 1

# UI and General Preferences
[ui]
default_time_period_days = 30
auto_refresh_interval_minutes = 15
show_loading_details = true
theme = "default"  # "default", "dark", "light", "high-contrast"

# URL Evidence Collection
[evidence]
allowed_domains = ["github.com", "jira.company.com", "review.company.com"]
auto_capture_urls = true

# Platform Configurations
[platforms.gerrit]
enabled = true
url = "https://review.company.com"
username = "max.linke"
http_password = "generated-password-here"
# Optional connection settings
timeout_seconds = 30
retry_attempts = 3

[platforms.jira]
enabled = true
url = "https://company.atlassian.net"
username = "max.linke@company.com"
api_token = "ATATT3xFfGF0..."
# Optional filters
project_filter = ["PROJ", "TEAM", "CORE"]
max_results = 100

[platforms.gitlab]
enabled = false
url = "https://gitlab.company.com"
access_token = ""
username = ""
group_filter = []

```

### 2.2 Configuration Management Behaviors

#### Behavior: Configuration Creation
**Expected Observable Behavior**:
1. **Initial Setup Detection**:
   - Console output: "First time setup detected"
   - "Creating default configuration at {data-path}/config.toml"
   - "Run 'reviewr config' to customize platform settings"

2. **Platform Auto-Detection**:
   - Console output: "Scanning for existing platform configurations..."
   - If legacy files found: "Found legacy {platform}_config.toml, migrating settings"
   - "Migration complete, legacy files backed up as {filename}.backup"

#### Behavior: Configuration Modification
**Trigger**: `reviewr config set SECTION.KEY VALUE`
**Expected Observable Behavior**:
1. **Setting Update**:
   - Console output: "Updating configuration: {section}.{key} = {value}"
   - Validation: "✓ Configuration value validated"
   - Save: "Configuration saved to {data-path}/config.toml"

2. **Platform Specific Settings**:
   - `reviewr config set platforms.gerrit.url "https://new-gerrit.com"`
   - Console output: "Updated Gerrit URL, testing connection..."
   - Connection test result: "✓ Gerrit connection successful" or "❌ Connection failed: {reason}"

## 3. Data Fetching and Progress Reporting

### 3.1 Platform Connection and Data Loading

#### Behavior: Platform Connection Testing
**Expected Observable Behavior**:
1. **Connection Sequence**:
   - Console output: "Testing platform connections..."
   - For each enabled platform:
     - "Connecting to {Platform} at {URL}..."
     - Progress indicator: "🔄 Authenticating with {Platform}..."
     - Result: "✓ {Platform} connected successfully" or "❌ {Platform} connection failed: {reason}"

2. **Connection Summary**:
   - Console output: "Connection summary:"
   - "  ✓ Gerrit: Connected (last test: successful)"
   - "  ⚠️ JIRA: Connected with warnings (rate limit: 80% used)"
   - "  ❌ GitLab: Connection failed (check credentials)"

#### Behavior: Data Fetching Process
**Expected Observable Behavior**:
1. **Data Collection Initialization**:
   - Console output: "Collecting activity data for {Employee Name} ({email})"
   - "Time period: Last {days} days (from {start_date} to {end_date})"

2. **Per-Platform Data Fetching**:
   - **Gerrit Data Collection**:
     - "🔄 Fetching Gerrit data from {gerrit_url}..."
     - "  → Querying changes created by {email}..."
     - "  → Found {N} changes created"
     - "  → Querying changes merged by {email}..."
     - "  → Found {N} changes merged"
     - "  → Querying reviews given by {email}..."
     - "  → Found {N} reviews given"
     - "  → Querying reviews received by {email}..."
     - "  → Found {N} reviews received"
     - "✓ Gerrit data collection complete: {total} items"

   - **JIRA Data Collection**:
     - "🔄 Fetching JIRA data from {jira_url}..."
     - "  → Searching issues created by {email}..."
     - "  → JQL: reporter = \"{email}\" AND created >= -{days}d"
     - "  → Found {N} issues created"
     - "  → Searching issues assigned to {email}..."
     - "  → JQL: assignee = \"{email}\" AND status != Done"
     - "  → Found {N} issues assigned"
     - "  → Searching issues resolved by {email}..."
     - "  → Found {N} issues resolved"
     - "  → Searching issues commented by {email}..."
     - "  → Found {N} issues with comments"
     - "✓ JIRA data collection complete: {total} items"

3. **Data Processing**:
   - Console output: "Processing collected data..."
   - "  → Categorizing {total} items across {N} platforms"
   - "  → Calculating activity metrics"
   - "✓ Data processing complete"

### 3.2 Platform Unavailability Handling

#### Behavior: Temporary Platform Unavailability
**Expected Observable Behavior**:
1. **Connection Failure Detection**:
   - Console output: "❌ {Platform} connection failed: {specific_error}"
   - "Attempting to use cached data for {Platform}..."
   - If cache available: "✓ Using cached {Platform} data (last updated: {timestamp})"
   - If no cache: "⚠️ No cached data available for {Platform}"

2. **Partial Data Mode**:
   - Console output: "Proceeding with partial data from available platforms"
   - "Available: {list_of_working_platforms}"
   - "Unavailable: {list_of_failed_platforms}"

3. **TUI Launch with Warnings**:
   - TUI displays warning banner: "⚠️ Some platforms unavailable - showing partial data"
   - Platform tabs show status indicators
   - Unavailable platforms show "❌ Connection failed" in tab

## 4. TUI Interface Detailed Behaviors

### 4.1 TUI Launch and Initial Display

#### Behavior: TUI Initialization
**Expected Observable Behavior**:
1. **Screen Setup**:
   - Clear terminal screen
   - Display header: "Reviewr - Employee Review Dashboard"
   - Show employee info: "Employee: {Name} ({email}) | Period: Last {days} days"

2. **Loading Screen**:
   - Display loading spinner with messages:
   - "🔄 Loading Gerrit data..."
   - "🔄 Loading JIRA data..."
   - "🔄 Processing activity metrics..."
   - Progress bar: "[████████░░] 80% complete"

3. **Initial View**:
   - Default to Summary view
   - Show platform tabs across top
   - Display data refresh timestamp: "Last updated: {time} ago"

### 4.2 Summary View Behaviors

#### Behavior: Cross-Platform Summary Display
**Expected Observable Behavior**:
```
┌─────────────────────────────────────────────────────────────────┐
│ 📊 Cross-Platform Activity Summary                             │
│ Employee: John Doe (john.doe@company.com)                      │
│ Period: Last 30 days (2024-01-01 to 2024-01-31)              │
│ Status: 🔧 Gerrit ✓ | 🎫 JIRA ✓ | 🦊 GitLab ❌               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 📈 Activity Metrics:                                           │
│ ┌─────────────────┬─────────────────┬─────────────────────────┐ │
│ │ Code Reviews    │ Issue Tracking  │ Repository Activity     │ │
│ │ • Created: 15   │ • Created: 8    │ • GitLab: Not Available │ │
│ │ • Merged: 12    │ • Resolved: 5   │ • Total: 0              │ │
│ │ • Reviewed: 23  │ • Assigned: 3   │                         │ │
│ │ • Total: 50     │ • Total: 16     │                         │ │
│ └─────────────────┴─────────────────┴─────────────────────────┘ │
│                                                                 │
│ 🔍 Recent Activity (All Platforms):                           │
│ 1. 🔧 [MERGED] Fix authentication bug          (checkmk)      │
│ 2. 🎫 [DONE]   Implement user dashboard        (PROJ-123)     │
│ 3. 🔧 [NEW]    Add logging configuration       (core-utils)   │
│ 4. 🎫 [PROGRESS] Database migration task       (PROJ-124)     │
│                                                                 │
│ ⚡ Quick Actions:                                              │
│ [G] Gerrit Details  [J] JIRA Details  [C] Config  [R] Refresh │
├─────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate | [Enter] Select | [Tab] Switch Tabs | [Q] Quit  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.3 Platform-Specific View Behaviors

#### Behavior: Gerrit Platform View
**Expected Observable Behavior**:
```
┌─────────────────────────────────────────────────────────────────┐
│ 🔧 Gerrit Activity (50 items) - review.company.com             │
│ Last Updated: 2 minutes ago | Connection: ✓ Healthy            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 📋 Change Categories:                                           │
│ [C]reated (15) │ [M]erged (12) │ [G]iven Reviews (23) │ [R]eceived (18) │
│                                                                 │
│ Current View: Changes Created (15 items)                       │
│                                                                 │
│ Status  │ Change │ Subject                        │ Project     │
│ ────────┼────────┼────────────────────────────────┼─────────────│
│ ▶ NEW   │ 12345  │ Add user authentication        │ main-app    │
│   MERGED│ 12344  │ Fix database connection pool   │ core-db     │
│   NEW   │ 12343  │ Update API documentation       │ docs        │
│   REVIEW│ 12342  │ Implement caching layer        │ cache-lib   │
│                                                                 │
│ 📋 Selected: Change 12345 (NEW)                               │
│ Subject: Add user authentication                                │
│ Project: main-app | Branch: main                               │
│ Created: 2024-01-15 14:30 | Updated: 2024-01-16 09:15         │
│ URL: https://review.company.com/c/main-app/+/12345             │
│ Related: JIRA ticket MAIN-456 (if cross-reference detected)    │
│                                                                 │
│ Press [Enter] to open in browser | [C/M/G/R] to switch views  │
├─────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate | [Enter] Open | [Backspace] Back | [R] Refresh  │
└─────────────────────────────────────────────────────────────────┘
```

#### Behavior: JIRA Platform View
**Expected Observable Behavior**:
```
┌─────────────────────────────────────────────────────────────────┐
│ 🎫 JIRA Activity (16 items) - company.atlassian.net            │
│ Last Updated: 1 minute ago | Connection: ⚠️ Rate limited       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│ 📋 Issue Categories:                                            │
│ [A]ssigned (3) │ [C]reated (8) │ [R]esolved (5) │ [W]atched (12) │
│                                                                 │
│ Current View: Issues Created (8 items)                         │
│                                                                 │
│ Status    │ Key       │ Summary                     │ Project   │
│ ──────────┼───────────┼─────────────────────────────┼───────────│
│ ▶ TODO    │ PROJ-123  │ Implement user dashboard    │ MainProj  │
│   PROGRESS│ PROJ-124  │ Database migration task     │ MainProj  │
│   DONE    │ PROJ-122  │ Fix login redirect issue    │ MainProj  │
│   REVIEW  │ CORE-456  │ Update cache configuration  │ CoreInfra │
│                                                                 │
│ 📋 Selected: PROJ-123 (TODO)                                  │
│ Summary: Implement user dashboard                               │
│ Project: MainProj | Assignee: John Doe                        │
│ Created: 2024-01-10 | Updated: 2024-01-15                     │
│ Priority: High | Labels: frontend, dashboard                   │
│ URL: https://company.atlassian.net/browse/PROJ-123             │
│ Related: Gerrit change main-app:+/12345                        │
│                                                                 │
│ Press [Enter] to open in browser | [A/C/R/W] to switch views  │
├─────────────────────────────────────────────────────────────────┤
│ [↑↓] Navigate | [Enter] Open | [Backspace] Back | [R] Refresh  │
└─────────────────────────────────────────────────────────────────┘
```

### 4.4 Data Refresh Behaviors in TUI

#### Behavior: Current View Refresh (`r` key)
**Expected Observable Behavior**:
1. **Refresh Initiation**:
   - Status bar shows: "🔄 Refreshing {Platform} data..."
   - Progress indicator appears in current view
   - Current selection and scroll position preserved

2. **Data Fetching Display**:
   - In-place loading messages:
   - "Fetching latest {category} data..."
   - "Processing {N} new items..."
   - "Detecting changes since last update..."

3. **Refresh Completion**:
   - Status bar shows: "✓ Refresh complete - {N} items updated"
   - New items highlighted with "NEW" indicator
   - Updated timestamp displayed: "Last updated: Just now"
   - If no changes: "No new activity since last update"

#### Behavior: Force Refresh All Platforms (`R` key)
**Expected Observable Behavior**:
1. **Global Refresh Initiation**:
   - Full screen overlay appears: "🔄 Refreshing all platforms..."
   - Progress bar shows overall completion
   - Individual platform progress: "Gerrit: 80%, JIRA: 60%, GitLab: Failed"

2. **Platform-by-Platform Updates**:
   - "✓ Gerrit refresh complete: 3 new changes, 1 merged"
   - "✓ JIRA refresh complete: 2 new issues, 1 resolved"
   - "❌ GitLab refresh failed: Connection timeout"

3. **Return to Summary**:
   - Automatically returns to Summary view
   - Shows updated metrics across all platforms
   - Displays refresh summary: "Updated {N} platforms, {M} new items total"

### 4.5 Error Display and Recovery Behaviors

#### Behavior: Platform Connection Error Display
**Expected Observable Behavior**:
1. **Error Banner**:
   - Top of screen shows: "⚠️ Platform Issues Detected"
   - Detailed status in platform tabs:
   - "🔧 Gerrit ❌" with hover text: "Authentication failed - check credentials"
   - "🎫 JIRA ⚠️" with hover text: "Rate limited - retry in 5 minutes"

2. **In-Platform Error Display**:
   ```
   ┌─────────────────────────────────────────────────────────────────┐
   │ 🦊 GitLab Activity - gitlab.company.com                        │
   │ Connection: ❌ Authentication Failed                             │
   ├─────────────────────────────────────────────────────────────────┤
   │                                                                 │
   │ ❌ Unable to connect to GitLab                                 │
   │                                                                 │
   │ Error: Invalid access token (HTTP 401)                         │
   │                                                                 │
   │ 💡 Troubleshooting steps:                                      │
   │ 1. Check your access token in config file                      │
   │ 2. Verify token has required scopes: api, read_user            │
   │ 3. Generate new token: Settings → Access Tokens                │
   │                                                                 │
   │ ⚡ Quick Actions:                                               │
   │ [C] Edit Config  [T] Test Connection  [R] Retry                │
   │                                                                 │
   │ Last successful connection: 2024-01-15 14:30                   │
   │ Using cached data: 15 merge requests (may be outdated)         │
   └─────────────────────────────────────────────────────────────────┘
   ```

#### Behavior: Partial Data Mode Operation
**Expected Observable Behavior**:
1. **Clear Status Communication**:
   - Summary view shows: "⚠️ Showing partial data - some platforms unavailable"
   - Each metric section indicates data completeness:
   - "Code Reviews: 50 items (Gerrit only)"
   - "Issue Tracking: 16 items (JIRA only)"
   - "Repository Activity: No data (GitLab unavailable)"

2. **Graceful Degradation**:
   - All available features continue to work normally
   - Cross-references limited to available platforms
   - Search functionality works within available data
   - Export includes disclaimer about missing platforms


### 4.6 Navigation Behaviors

#### Behavior: Tab Navigation
**Controls**: `Tab`, `Shift+Tab`, Number keys 1-5
**Expected Observable Behavior**:
- Switches between Summary, Platform tabs, and Config
- Visual indicator shows current active tab
- Tab badges display item counts when data available
- Status indicators show connection health (✅⚠️❌⚪)

#### Behavior: Hierarchical Navigation
**Controls**: `Enter` (drill down), `Backspace` (go up)
**Expected Observable Behavior**:
- Summary → Platform View → Category View → Item Details
- Breadcrumb indicator shows current location
- Context preserved when navigating back
- Selection maintained within same level

#### Behavior: List Navigation
**Controls**: `↑`, `↓` arrow keys
**Expected Observable Behavior**:
- Highlight moves through available items
- Scrolling when list exceeds screen size
- Selection wraps around at top/bottom
- Selected item details shown in info panel

### 4.7 Help and Information Behaviors

#### Behavior: Help Display
**Control**: `h` or `?` key
**Expected Observable Behavior**:
- Toggles help panel visibility
- Context-sensitive help based on current view
- Lists all available keyboard shortcuts
- Shows navigation hierarchy explanation

#### Behavior: Status Information
**Expected Observable Behavior**:
- Platform status always visible in tabs
- Data age indicators: "Last updated: Xm ago"
- Connection status with descriptive messages
- Item counts per category when available

## 5. Configuration Interface Behaviors

### 5.1 Configuration Command Behaviors

#### Behavior: Platform Configuration Setup
**Trigger**: `reviewr config setup`
**Expected Observable Behavior**:
1. **Interactive Configuration Wizard**:
   - "Welcome to Reviewr platform setup!"
   - "This wizard will help you configure platform connections."

2. **Platform Selection**:
   - "Which platforms would you like to configure?"
   - "[ ] Gerrit (code review)"
   - "[ ] JIRA (issue tracking)"
   - "[ ] GitLab (repository management)"
   - User can toggle with space, confirm with Enter

3. **Per-Platform Configuration**:
   - For Gerrit:
     - "Enter Gerrit URL (e.g., https://review.company.com): "
     - "Enter your username: "
     - "Enter your HTTP password (Settings → HTTP Credentials): "
     - "Testing connection..." → "✓ Connected successfully!"

   - For JIRA:
     - "Enter JIRA URL (e.g., https://company.atlassian.net): "
     - "Enter your email address: "
     - "Enter your API token (Profile → Personal Access Tokens): "
     - "Optional: Enter project filter (comma-separated, blank for all): "
     - "Testing connection..." → "✓ Connected successfully!"

4. **Configuration Save**:
   - "Saving configuration to {data-path}/config.toml"
   - "✓ Configuration saved successfully!"
   - "Run 'reviewr review' to start using your configured platforms"

#### Behavior: Configuration Viewing and Editing
**Trigger**: `reviewr config show`
**Expected Observable Behavior**:
```
Current Reviewr Configuration
=============================

General Settings:
  Default time period: 30 days
  Auto-refresh interval: 15 minutes
  Show loading details: enabled

Platform Configurations:
  🔧 Gerrit:
    URL: https://review.company.com
    Username: max.linke
    Status: ✓ Connected (last tested: 5 minutes ago)

  🎫 JIRA:
    URL: https://company.atlassian.net
    Username: max.linke@company.com
    Project filter: PROJ, TEAM, CORE
    Status: ⚠️ Rate limited (retry in 3 minutes)

  🦊 GitLab:
    Status: Not configured

Cross-Platform Mappings:
  Main Product: gerrit:main-product ↔ jira:MAIN
  Core Utils: gerrit:core-utils ↔ jira:UTIL

Configuration file: /home/user/.reviewr/config.toml

To modify configuration:
  reviewr config edit           # Open in editor
  reviewr config set KEY VALUE  # Set specific value
  reviewr config setup          # Re-run setup wizard
```

## 6. Data Persistence and Caching Behaviors

### 6.1 Cache Management
**Expected Observable Behavior**:
1. **Cache Creation**:
   - Console output: "Caching {Platform} data for faster access"
   - "Cache expires in {minutes} minutes"

2. **Cache Usage**:
   - When using cache: "Loading cached {Platform} data (age: {time})"
   - When cache expired: "Cache expired, fetching fresh data from {Platform}"
   - When force refresh: "Bypassing cache, fetching latest data"

3. **Cache Status Display**:
   - In TUI status bar: "Data age: Gerrit 2m, JIRA 5m, GitLab cached"
   - Cache health indicator: "🟢 Fresh | 🟡 Aging | 🔴 Stale"

This detailed specification ensures the `reviewr review` command provides comprehensive feedback about all operations, maintains a single unified configuration file, and gracefully handles platform unavailability while keeping users informed about what's happening behind the scenes.
