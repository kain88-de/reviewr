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