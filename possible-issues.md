# Possible Issues & Improvements

This file tracks issues, bugs, and potential improvements discovered during development and testing. These can be used to inform future requirements and development priorities.

## TUI/Display Issues

### Fixed Issues
- **Layout Overlap Problem** (2025-07-26)
  - **Issue**: Change details panel was overlapping with the main list area
  - **Cause**: Incorrect layout constraints causing elements to render on top of each other
  - **Fix**: Proper separation of list and details areas with fixed constraints
  - **Impact**: High - Made the interface unusable

- **Text Wrapping Problems** (2025-07-26)
  - **Issue**: Long text (subjects, project names) was not properly contained within borders
  - **Cause**: No text truncation or proper width calculations
  - **Fix**: Added intelligent text truncation with ellipsis for long subjects and project names
  - **Impact**: Medium - Affected readability

- **Gerrit URL Format** (2025-07-26)
  - **Issue**: Generated URLs missing project path component
  - **Cause**: Used `/c/<change-number>` instead of `/c/<project>/+/<change-number>`
  - **Fix**: Updated URL generation to include project in path
  - **Impact**: High - Links were completely broken

### Potential Future Issues

- **Terminal Compatibility**
  - **Risk**: TUI may not work correctly in all terminal environments
  - **Mitigation**: Test across different terminal emulators (xterm, tmux, screen, etc.)
  - **Priority**: Medium

- **Unicode/Emoji Support**
  - **Risk**: Emoji characters in TUI may not render on all systems
  - **Mitigation**: Add fallback ASCII mode option
  - **Priority**: Low

- **Performance with Large Change Lists**
  - **Risk**: Rendering hundreds of changes may cause lag
  - **Mitigation**: Implement pagination or virtual scrolling
  - **Priority**: Medium

## API Integration Issues

### Known Limitations
- **Gerrit Query Compatibility** (Fixed 2025-07-26)
  - **Issue**: `has:reviewer` operator not supported in all Gerrit versions
  - **Workaround**: Use simpler queries that approximate the intended metric
  - **Future**: Implement version detection and adaptive queries

- **Network Timeout Handling**
  - **Current**: Fixed 30-second timeout
  - **Issue**: May be too short for large datasets or slow networks
  - **Future**: Make timeout configurable

### Future Integration Challenges
- **JIRA API Rate Limiting**
  - **Risk**: JIRA Cloud has strict rate limits
  - **Mitigation**: Implement request batching and caching
  - **Priority**: High for JIRA integration

- **GitLab API Pagination**
  - **Risk**: Large repositories may have thousands of MRs
  - **Mitigation**: Implement proper pagination handling
  - **Priority**: High for GitLab integration

## Data Management Issues

### Potential Problems
- **Configuration Validation**
  - **Current**: Basic TOML parsing with minimal validation
  - **Risk**: Invalid configurations cause runtime errors
  - **Future**: Add comprehensive config validation with helpful error messages

- **Concurrent Access**
  - **Current**: File locking implemented for employee data
  - **Risk**: May not be sufficient for high-concurrency scenarios
  - **Future**: Consider database backend for production use

- **Data Migration**
  - **Risk**: Changes to data formats will break existing installations
  - **Future**: Implement versioned data schemas with migration scripts

## Usability Issues

### Identified Pain Points
- **Employee Email Setup**
  - **Issue**: Users must manually configure committer emails
  - **Future**: Auto-detect from git config or provide import options

- **Multi-Platform Configuration**
  - **Risk**: Managing credentials for multiple platforms becomes complex
  - **Future**: Implement secure credential storage (keyring integration)

- **URL Opening Reliability**
  - **Current**: Uses platform-specific commands (`xdg-open`, `open`, `cmd`)
  - **Risk**: May fail if default browser not configured
  - **Future**: Add fallback options and error handling

### Missing Features
- **Export Functionality**
  - **Need**: Users want to export data for reports
  - **Future**: Add CSV/JSON export options

- **Historical Data**
  - **Need**: Compare performance over time
  - **Future**: Add trend analysis and historical reporting

- **Team Analytics**
  - **Need**: Aggregate metrics across team members
  - **Future**: Add team dashboard and comparison features

## Security Considerations

### Current Risks
- **Credential Storage**
  - **Issue**: Passwords stored in plain text TOML files
  - **Risk**: High if config files are compromised
  - **Mitigation**: Use tokens instead of passwords where possible
  - **Future**: Implement encrypted credential storage

- **Network Security**
  - **Current**: HTTPS for API calls but no certificate validation specifics
  - **Future**: Add certificate pinning options for enterprise environments

## Platform-Specific Issues

### Linux
- **Clipboard Access**: Works well with arboard crate
- **Browser Opening**: `xdg-open` generally reliable

### macOS
- **Potential Issues**: Not extensively tested
- **Browser Opening**: Should work with `open` command

### Windows
- **Potential Issues**: Path handling differences
- **Browser Opening**: `cmd /c start` may have escaping issues

## Development/Testing Issues

### Test Coverage Gaps
- **TUI Testing**: No automated tests for TUI components
- **Integration Testing**: Limited API integration tests
- **Error Path Testing**: Insufficient testing of error conditions

### Build/Distribution
- **Cross-Platform Builds**: Not automated
- **Dependency Management**: No automated security scanning
- **Version Management**: Manual version bumping

## Documentation Issues

### Missing Documentation
- **User Guide**: No comprehensive user manual
- **API Documentation**: Limited inline documentation
- **Troubleshooting Guide**: No systematic troubleshooting docs
- **Installation Guide**: Basic instructions only

## Performance Issues

### Identified Bottlenecks
- **API Call Serialization**: All API calls are sequential
- **Future**: Implement concurrent API calls where possible

- **Memory Usage**: Stores all changes in memory
- **Future**: Implement streaming for large datasets

## Monitoring/Observability

### Missing Features
- **Usage Analytics**: No metrics on feature usage
- **Error Reporting**: No centralized error tracking
- **Performance Monitoring**: No performance metrics collection

---

## Issue Classification

### Priority Levels
- **High**: Blocks core functionality or causes data loss
- **Medium**: Impacts usability or performance significantly
- **Low**: Minor inconveniences or nice-to-have improvements

### Categories
- **Bug**: Something that doesn't work as intended
- **Enhancement**: New functionality or improvement
- **Performance**: Speed or resource usage issues
- **Security**: Security vulnerabilities or concerns
- **Usability**: User experience improvements
- **Documentation**: Missing or inadequate documentation
