This is a CLI tool to manage employee evaluations.

## Development

To test the tool during development, you can use `cargo run`. For example:

```bash
cargo run -- add "John Doe"
cargo run -- notes "John Doe"
```

## Testing

To test the tool in a clean environment, you can use the `--data-path` flag to specify a directory for storing test data. This is useful for preventing your regular data from being affected by tests.

```bash
cargo run -- --data-path .test_eval_data/ add "John Doe"
cargo run -- --data-path .test_eval_data/ notes "John Doe"
```

## Quality Standards

This project enforces a high standard of code quality through a combination of automated checks and a strict build process.

### Formatting

All code must be formatted with `cargo fmt` before being committed. This ensures a consistent style throughout the project.

### Static Analysis

`cargo clippy` is used for static analysis to identify potential bugs and improve code quality. All Clippy warnings must be addressed before committing.

### Build Process

During the build process, all warnings are treated as errors. This is enforced by the following configuration in `.cargo/config.toml`:

```toml
[build]
rustflags = ["-D", "warnings"]
```
