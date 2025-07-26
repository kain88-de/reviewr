# Multi-System Review Dashboard - UX Design

## Core UX Principles

1. **Progressive Disclosure**: Start simple, reveal complexity as needed
2. **Consistent Mental Model**: Similar navigation patterns across all systems
3. **Context Preservation**: Remember user's last view and selections
4. **Quick Switching**: Fast transitions between systems and views
5. **Unified Search**: Cross-system search and filtering capabilities

## Primary Navigation Design

### Tab-Based System Switching
```
[📊 Summary] [🔧 Gerrit] [🎫 JIRA] [🦊 GitLab] [⚙️ Config]
```

- **Summary Tab**: Unified dashboard showing activity across all configured systems
- **System Tabs**: Dedicated views for each platform (only show if configured)
- **Config Tab**: System configuration and credential management
- **Visual Indicators**: Show activity counts, error states, loading status per tab

### Tab Behavior
- **Auto-hide unconfigured systems**: Only show tabs for configured platforms
- **Badge notifications**: Show pending items count on each tab
- **Status indicators**:
  - ✅ Green: Connected and working
  - ⚠️ Yellow: Configured but connection issues
  - ❌ Red: Configuration errors
  - ⚪ Gray: Not configured

## Unified Summary Dashboard

### Overview Section
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

### Quick Access Panel
```
🔍 Recent Activity (All Systems):
1. 🔧 [MERGED] Fix authentication bug          (checkmk)
2. 🎫 [DONE]   Implement user dashboard        (PROJ-123)
3. 🔧 [NEW]    Add logging configuration       (core-utils)
4. 🎫 [IN PROGRESS] Database migration task    (PROJ-124)

⚡ Quick Actions:
[G] Gerrit Details  [J] JIRA Details  [L] GitLab Details  [C] Configure
```

## System-Specific Views

### Enhanced Gerrit View
- **Current functionality preserved** with improvements
- **Additional context**: Link to related JIRA tickets if project mapping configured
- **Cross-references**: "Related JIRA: PROJ-123" in change details

### JIRA Ticket View
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

### GitLab Merge Request View
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

## Cross-System Features

### Unified Search
```
🔍 Search across all systems:
> auth dashboard

Results (23 items):
🔧 Gerrit (8):  "authentication", "dashboard" in subjects
🎫 JIRA (12):   "auth", "dashboard" in summaries
🦊 GitLab (3):  "authentication", "dashboard" in MR titles

[Enter] to explore results by system | [Esc] to cancel
```

### Smart Linking
- **Auto-detect references**: PROJ-123 mentioned in Gerrit commits → show JIRA link
- **Bidirectional navigation**: From JIRA ticket, show related Gerrit changes
- **Project mapping**: Configure relationships between systems

### Export & Reporting
```
📊 Generate Report:
• Time period: [Last 30 days ▼]
• Systems: [✓] Gerrit [✓] JIRA [✓] GitLab
• Format: [CSV ▼] JSON | PDF
• Include: [✓] Metrics [✓] Item details [✓] Cross-references

[Generate] [Preview] [Cancel]
```

## Progressive Configuration

### First-Time Setup Wizard
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

### Incremental Configuration
- **Add systems later**: Don't require all systems upfront
- **Test connections**: Immediate feedback on configuration
- **Optional mapping**: Set up cross-system relationships later

## Error Handling & Offline UX

### Graceful Degradation
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

### Offline Capabilities
- **Cache recent data**: Show last successful fetch with timestamps
- **Partial functionality**: Work with available systems
- **Smart retries**: Automatic reconnection attempts
- **Clear status**: Always show what's working and what's not

## Navigation Patterns

### Consistent Keyboard Shortcuts
- **System switching**: Number keys 1-5 for tabs
- **Category switching**: Letter keys (A/C/R/W for JIRA, M/C/G/R for Gerrit)
- **Universal actions**:
  - `Enter` - Open item in browser
  - `/` - Start search
  - `h` - Help
  - `q` - Quit
  - `r` - Refresh current view
  - `c` - Configure current system

### Visual Consistency
- **Color coding**: Consistent across systems (green=good, red=error, yellow=warning)
- **Icons**: Each system has recognizable icon (🔧 Gerrit, 🎫 JIRA, 🦊 GitLab)
- **Layout patterns**: Similar structures across all system views
- **Status indicators**: Unified approach to showing connection/data status

## Information Architecture

### Hierarchy
1. **Global Level**: Cross-system summary and navigation
2. **System Level**: Platform-specific dashboards
3. **Category Level**: Grouped items (assigned/created/resolved)
4. **Item Level**: Individual changes/tickets/MRs with full details

### Context Switching
- **Preserve context**: Remember last viewed category when switching systems
- **Breadcrumbs**: Show current location (System > Category > Item)
- **Quick return**: Easy way to get back to summary or previous view

## Responsive Design Principles

### Terminal Size Adaptation
- **Minimum viable**: Works in 80x24 terminal
- **Optimal**: Takes advantage of larger terminals
- **Graceful degradation**: Hide less critical info in small terminals
- **Horizontal scrolling**: For wide content like URLs

### Content Prioritization
1. **Essential**: Core metrics, navigation, current selection
2. **Important**: Detailed item information, status indicators
3. **Helpful**: Cross-references, additional metadata
4. **Optional**: Help text, extended descriptions

This UX design ensures a cohesive, intuitive experience across all systems while maintaining the efficiency and power that makes reviewr valuable for employee review processes.
