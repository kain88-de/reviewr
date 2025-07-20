# Justfile for reviewr

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

# Run the application with test data
run *ARGS:
    cargo run -- --data-path .test_reviewr_data/ {{ARGS}}

# Release a new version
release LEVEL:
    just lint
    cargo test
    cargo install cargo-edit
    cargo set-version --bump {{LEVEL}}
    git commit -am "chore(release): v$(cargo pkgid | awk -F# '{print $2}')"
    git tag "v$(cargo pkgid | awk -F# '{print $2}')"
    git push
    git push --tags
