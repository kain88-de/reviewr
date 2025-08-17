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

## Development

We follow a test driven development approach.

- new features start thinking how we can tests it.
- on writing code use the red green refactor cycle
  - write tests showing the behavior we want
  - write the minimal code to make the tests pass, e.g. hard coded returns are ok here
  - refactor the code to be nice and do the work we want, ensure the tests still pass
- when refactoring start with test for your refactor if we do not think the behavior changes

## Architecture

This is a Rust CLI tool for employee reviews with the following structure:

### Data Structure
- Default data location: `~/.reviewr/`
- Employee files: `employees/{name}.toml`
- Note files: `notes/{name}.md`
- Config file: `config.toml`

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

- run pre-commit run --all-files before attempting a commit
