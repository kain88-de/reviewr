# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Build and Development
- `just run [ARGS]` - Run the application with test data (uses `.test_reviewr_data/` directory)
- `cargo build` - Build the project
- `cargo run -- [ARGS]` - Run the application directly
- `cargo run -- --data-path .test_reviewr_data/ [COMMAND]` - Run with isolated test data

### Testing
- `cargo test` - Run all tests (unit and integration)
- `cargo test --test integration_test` - Run only integration tests
- `cargo test test_name` - Run specific test by name
- **Always add tests when fixing bugs or adding features**

### Linting and Code Quality
- `just lint` - Run all linters (check, fmt, clippy)
- `just check` - Run cargo check
- `just fmt` - Run cargo fmt
- `just clippy` - Run cargo clippy

### Pre-Commit Requirements
Before committing any changes:
1. Run `just lint` - All linters must pass
2. Run `cargo test` - All tests must pass

### Release
- `just release [LEVEL]` - Release a new version (patch, minor, major)

## Architecture

This is a Rust CLI tool for employee reviews with the following structure:

### Core Components
- **CLI Interface**: Uses `clap` for command parsing with subcommands (Add, Notes, Config)
- **Data Storage**: TOML files for employee data and configuration, Markdown files for notes
- **Configuration**: `Config` struct with `allowed_domains` for URL filtering
- **Clipboard Integration**: Automatically captures URLs from clipboard when opening notes

### Key Features
- **Employee Management**: Add employees with name and title
- **Notes System**: Markdown-based notes with automatic date headers
- **URL Evidence**: Automatically appends clipboard URLs to notes if domain is allowed
- **Configurable Data Path**: Can use custom data directory via `--data-path`

### Data Structure
- Default data location: `~/.reviewr/`
- Employee files: `employees/{name}.toml`
- Note files: `notes/{name}.md`
- Config file: `config.toml`

### Dependencies
- `clap` - CLI argument parsing
- `serde` + `toml` - Configuration and data serialization
- `chrono` - Date/time handling
- `arboard` - Clipboard access
- `url` - URL parsing and validation
- `dirs` - System directory paths

### Testing Philosophy
- **Integration Tests**: Primary focus on behavior-driven testing, treating the application as a black box
- **Test User Journeys**: Tests mirror actual user actions from command line to file system effects
- **No Mocks**: Use real file systems with temporary data directories for isolation
- **High-Level Assertions**: Test against observable outputs (exit codes, stdout/stderr, file contents)
- **Unit Tests**: Test individual module functions for granular verification

### Build Configuration
- All warnings are treated as errors (configured in `.cargo/config.toml`)
- Code must be formatted with `cargo fmt` before commits
- All `cargo clippy` warnings must be addressed

### Testing Implementation
Integration tests use `assert_cmd` and `tempfile` for testing CLI commands with timeouts (5 seconds).
