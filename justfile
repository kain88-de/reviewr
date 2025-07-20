# Justfile for eval

# Run all linters
lint:
    just check
    just fmt
    just clippy

# Run cargo check
check:
    cargo check

# Run cargo fmt
fmt:
    cargo fmt

# Run cargo clippy
clippy:
    cargo clippy
