# Reviewr

A CLI tool for employee reviews with automatic evidence collection from clipboard URLs.

## Features

- **Employee Management**: Add and manage employee records with names and titles
- **Notes System**: Markdown-based notes with automatic date headers
- **URL Evidence Collection**: Automatically captures and appends clipboard URLs to notes
- **Domain Filtering**: Configure allowed domains for URL evidence collection
- **Configurable Data Path**: Use custom data directories for testing and isolation

## Installation

### From Source

```bash
cargo install --path .
```

### From GitHub Releases

Download the latest release for your platform from the [releases page](https://github.com/kain88-de/eval/releases).

## Usage

### Add an Employee

```bash
reviewr add "John Doe"
```

You'll be prompted to enter the employee's title.

### Open Notes for an Employee

```bash
reviewr notes "John Doe"
```

This will:
1. Create a notes file if it doesn't exist
2. Check your clipboard for URLs and append them as evidence (if domain is allowed)
3. Open the notes file in your default editor

### Configure Allowed Domains

```bash
# Set allowed domains for URL evidence collection
reviewr config set allowed_domains "github.com,google.com,localhost"

# View current configuration
reviewr config get allowed_domains
```

### Custom Data Path

Use a custom directory for data storage:

```bash
reviewr --data-path ./my-eval-data add "Jane Doe"
reviewr --data-path ./my-eval-data notes "Jane Doe"
```

## Data Structure

By default, data is stored in `~/.reviewr/`:

- `employees/{name}.toml` - Employee records
- `notes/{name}.md` - Employee notes
- `config.toml` - Configuration file

## Development

### Requirements

- Rust 1.70+ (uses edition 2024)
- [Just](https://github.com/casey/just) task runner

### Building

```bash
cargo build
```

### Testing

```bash
# Run all tests
cargo test

# Run with test data directory
just run --data-path .test_reviewr_data/ add "Test User"
```

### Code Quality

```bash
# Run all linters
just lint

# Individual linters
just check    # cargo check
just fmt      # cargo fmt
just clippy   # cargo clippy
```

### Pre-commit Requirements

Before committing:
1. `just lint` - All linters must pass
2. `cargo test` - All tests must pass

## License

MIT License - see [LICENSE](LICENSE) for details.
