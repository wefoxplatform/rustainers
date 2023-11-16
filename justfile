
# List all just receipes
default:
    @just --list --unsorted

# Install requirement for recipes
requirement:
    cargo install cargo-watch
    cargo install cargo-nextest
    cargo install cargo-llvm-cov
    cargo install cargo-sort
    cargo install cargo-deny
    cargo install cargo-hack

# Format the code and sort dependencies
format:
    cargo fmt
    cargo sort --workspace --grouped

_check_format:
    cargo fmt --all -- --check
    cargo sort --workspace --grouped --check

deny:
    cargo deny check advisories
    cargo deny check bans licenses sources

# Lint the rust code
lint:
    cargo clippy --workspace --all-features --all-targets -- --deny warnings --allow deprecated

# Launch tests
test:
    cargo nextest run
    cargo test --doc

# Test with features combination
test-with-features:
  cargo hack check --each-feature --no-dev-deps

# Check code coverage
coverage:
    cargo llvm-cov --open

# Check the code (formatting, lint, and tests)
check: && _check_format lint test

# Run TDD mode
tdd:
    cargo watch -c -s "just check"

# Build documentation (rustdoc, book)
doc:
    cargo doc --all-features --no-deps
