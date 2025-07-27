# Reviewr User Guide

## Overview

Reviewr is a multi-platform CLI tool for tracking and analyzing employee review activity across different systems. It currently supports **Gerrit** (code review) and **JIRA** (issue tracking) with a unified interface for managing employee data and generating activity reports.

## Quick Start

### Installation

```bash
# Clone and build
git clone <repository-url>
cd reviewr
cargo build --release

# Or install directly
cargo install --path .
```

### Basic Setup

1. **Add an employee:**
   ```bash
   reviewr add "John Doe"
   # Interactive form will ask for title and email
   ```

2. **Configure platforms** (see Platform Configuration section)

3. **Generate review report:**
   ```bash
   reviewr review "John Doe"
   ```

## Platform Configuration

### Gerrit Configuration

Create `~/.reviewr/gerrit_config.toml`:

```toml
gerrit_url = "https://review.example.com"
username = "your-username"
http_password = "your-http-password"
```

**Getting Gerrit credentials:**
1. Go to your Gerrit instance → Settings → HTTP Credentials
2. Generate a new HTTP password
3. Use your username and the generated password

### JIRA Configuration

Create `~/.reviewr/jira_config.toml`:

```toml
jira_url = "https://company.atlassian.net"
username = "user@company.com"
api_token = "your-api-token"

# Optional: Filter by specific projects
project_filter = ["PROJ", "TEAM"]

# Optional: Custom field mappings
[custom_fields]
story_points = "customfield_10001"
epic_link = "customfield_10002"
```

**Getting JIRA credentials:**
1. Go to JIRA → Profile → Personal Access Tokens
2. Create a new token with appropriate permissions
3. Use your email and the generated token

## Command Reference

### Employee Management

```bash
# Add employee (interactive)
reviewr add

# Add employee with name
reviewr add "Jane Smith"

# Edit employee information
reviewr edit "Jane Smith"

# List all employees
reviewr list
```

### Review Activities

```bash
# Generate review report (interactive selection)
reviewr review

# Generate report for specific employee
reviewr review "John Doe"

# Use custom data directory
reviewr --data-path /custom/path review "John Doe"
```

### Configuration Management

```bash
# View current configuration
reviewr config

# Get specific configuration value
reviewr config get allowed_domains

# Set configuration value
reviewr config set allowed_domains "example.com,company.com"
```

### Notes Management

```bash
# Open notes for employee (interactive selection)
reviewr notes

# Open notes for specific employee
reviewr notes "John Doe"
```

## Multi-Platform TUI Interface

When you run `reviewr review`, the multi-platform TUI provides:

### Navigation Hierarchy

1. **Summary View** - Overview of all configured platforms
2. **Platform View** - Categories within a specific platform
3. **Category View** - Individual items (changes, tickets, etc.)

### Controls

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between platforms (Summary view) |
| `Enter` | Select item / Drill down |
| `Backspace` | Go back to previous level |
| `↑` / `↓` | Navigate within lists |
| `s` | Go to Summary view |
| `h` / `?` | Show/hide help |
| `q` / `Esc` | Quit application |

### Platform Features

#### Gerrit Integration
- **Changes Created** - Code changes authored by the employee
- **Changes Merged** - Successfully merged commits
- **Reviews Given** - Code reviews provided by the employee
- **Reviews Received** - Reviews received on employee's changes

#### JIRA Integration
- **Issues Created** - Tickets created by the employee
- **Issues Resolved** - Tickets resolved/closed by the employee
- **Issues Assigned** - Currently assigned tickets
- **Issues Commented** - Tickets with employee comments

### Opening Items in Browser

- Press `Enter` in Category View to open items directly in your web browser
- Works with both Gerrit changes and JIRA tickets
- Automatically generates correct URLs for each platform

## Data Structure

### Employee Data

```toml
# ~/.reviewr/employees/john-doe.toml
name = "John Doe"
title = "Senior Software Engineer"
committer_email = "john.doe@company.com"
```

### Notes

Notes are stored as Markdown files with automatic date headers:

```markdown
# Notes for John Doe

## 2024-01-15
- Started work on feature X
- Completed code review for project Y

## 2024-01-10
- Weekly team meeting discussion
```

### Configuration Files

```
~/.reviewr/
├── config.toml                 # Main configuration
├── gerrit_config.toml         # Gerrit platform config
├── jira_config.toml           # JIRA platform config
├── employees/                 # Employee data files
│   ├── john-doe.toml
│   └── jane-smith.toml
└── notes/                     # Employee notes
    ├── john-doe.md
    └── jane-smith.md
```

## Advanced Usage

### Custom Data Directory

```bash
# Use project-specific data
reviewr --data-path ./project-reviews review "Team Lead"

# Useful for different teams or projects
export REVIEWR_DATA_PATH="/team/alpha/reviews"
reviewr review "Alpha Team Member"
```

### URL Evidence in Notes

Reviewr automatically captures URLs from your clipboard when opening notes, if the domain is in your `allowed_domains` configuration:

```bash
# Configure allowed domains
reviewr config set allowed_domains "github.com,jira.company.com"

# Open notes - any copied URLs from allowed domains will be appended
reviewr notes "John Doe"
```

### Batch Operations

```bash
# Generate reports for all employees
for employee in $(reviewr list --names-only); do
    echo "=== $employee ==="
    reviewr review "$employee" --summary-only
done
```

## Testing Your Setup

### 1. Test Platform Connections

```bash
# This will test all configured platforms
reviewr review --connection-test
```

### 2. Verify Employee Setup

```bash
# Add test employee
reviewr add "Test Employee"
# Enter: "Test Engineer" for title
# Enter: "test@company.com" for email

# Verify in list
reviewr list
```

### 3. Test Review Generation

```bash
# Generate review (should show platform data or graceful errors)
reviewr review "Test Employee"
```

## Troubleshooting

### Common Issues

#### "No platforms configured"
- Ensure you have created `gerrit_config.toml` and/or `jira_config.toml`
- Check file permissions and TOML syntax
- Verify file locations in `~/.reviewr/`

#### "Authentication failed"
- **Gerrit**: Verify username and HTTP password (not your login password)
- **JIRA**: Verify email and API token (not your login password)
- Check URL formats (must include `https://`)

#### "Employee has no committer email"
- Edit employee to add email: `reviewr edit "Employee Name"`
- Email must match the one used in Gerrit/JIRA

#### "Connection timeout"
- Check network connectivity to platform URLs
- Verify URLs are accessible from your machine
- Check firewall/proxy settings

### Debug Mode

```bash
# Enable debug logging
RUST_LOG=debug reviewr review "John Doe"

# Check configuration files
ls -la ~/.reviewr/
cat ~/.reviewr/config.toml
```

### Validation Commands

```bash
# Test TOML syntax
toml-test ~/.reviewr/config.toml

# Test network connectivity
curl -u username:password https://gerrit.example.com/a/changes/?q=limit:1
curl -u email:token https://jira.example.com/rest/api/3/myself
```

## Performance Tips

### Large Datasets
- Default query period is 30 days
- Platforms automatically limit results (Gerrit: no limit, JIRA: 50 items)
- Use project filters in JIRA config to reduce data volume

### Network Optimization
- Configure reasonable timeouts (30 seconds default)
- Use pagination for large result sets
- Consider caching for frequently accessed data

### Memory Usage
- TUI efficiently handles large datasets
- Data is loaded on-demand per platform
- Memory usage scales with number of items displayed

## Security Considerations

### Credential Storage
- Credentials stored in plain text TOML files
- Ensure proper file permissions: `chmod 600 ~/.reviewr/*_config.toml`
- Consider using environment variables for credentials

### Network Security
- All communications use HTTPS
- Certificates are validated
- No credential logging in debug mode

### Data Privacy
- All data stored locally
- No telemetry or external reporting
- User controls all data retention

## Best Practices

### Employee Management
- Use consistent naming conventions
- Include committer emails for all platforms
- Keep titles up-to-date for accurate reporting

### Platform Configuration
- Use dedicated service accounts when possible
- Regularly rotate API tokens/passwords
- Test configurations after updates

### Review Workflow
- Regular review cycles (weekly/monthly)
- Document findings in employee notes
- Use URL evidence for reference tracking

### Data Organization
- Use descriptive employee names (avoid abbreviations)
- Organize notes with clear date sections
- Archive old data periodically

## Integration Examples

### Shell Scripts

```bash
#!/bin/bash
# weekly-reviews.sh

EMPLOYEES=(
    "John Doe"
    "Jane Smith"
    "Bob Wilson"
)

for employee in "${EMPLOYEES[@]}"; do
    echo "=== Weekly Review: $employee ==="
    reviewr review "$employee"
    echo "Press Enter to continue..."
    read
done
```

### CI/CD Integration

```yaml
# .github/workflows/team-metrics.yml
name: Weekly Team Metrics

on:
  schedule:
    - cron: '0 9 * * MON'  # Every Monday at 9 AM

jobs:
  team-metrics:
    runs-on: ubuntu-latest
    steps:
      - name: Generate team reports
        run: |
          # Configure reviewr with CI credentials
          reviewr --data-path ./reports list | while read employee; do
            reviewr --data-path ./reports review "$employee" --format json > "reports/${employee// /_}.json"
          done
```

### Reporting Integration

```python
#!/usr/bin/env python3
# generate-team-report.py

import subprocess
import json
from datetime import datetime

def get_team_metrics():
    """Generate team-wide metrics using reviewr"""
    employees = subprocess.check_output(['reviewr', 'list', '--json']).decode()

    for employee in json.loads(employees):
        try:
            metrics = subprocess.check_output([
                'reviewr', 'review', employee['name'], '--json'
            ]).decode()

            # Process metrics data
            data = json.loads(metrics)
            print(f"Employee: {employee['name']}")
            print(f"  - Changes: {data.get('gerrit', {}).get('changes_created', 0)}")
            print(f"  - Tickets: {data.get('jira', {}).get('tickets_resolved', 0)}")

        except subprocess.CalledProcessError as e:
            print(f"Error processing {employee['name']}: {e}")

if __name__ == "__main__":
    get_team_metrics()
```

## FAQ

### Q: Can I use multiple JIRA instances?
A: Currently, one JIRA instance per installation. You can use project filters to limit scope within a single instance.

### Q: How do I handle employees with multiple email addresses?
A: Use their primary work email that's consistent across platforms. Add aliases in notes if needed.

### Q: Can I export data to Excel/CSV?
A: Not yet implemented. Use JSON output with custom scripts for now. CSV export is planned for future releases.

### Q: How do I backup my data?
A: Simply backup the `~/.reviewr/` directory. All data is stored in human-readable TOML/Markdown files.

### Q: Can I use this for multiple teams?
A: Yes, use different `--data-path` directories for different teams or projects.

### Q: What if a platform is temporarily unavailable?
A: Reviewr gracefully handles platform failures. Available platforms will still work normally.

This user guide provides comprehensive information for evaluating and using the current multi-platform implementation effectively.
