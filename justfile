# Run all checks
check: fmt-check lint test audit

# Format code
fmt:
    cargo fmt

# Verify formatting
fmt-check:
    cargo fmt --check

# Lint with clippy
lint:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo nextest run

# Audit dependencies
audit:
    cargo deny check

# Generate coverage report
coverage:
    cargo llvm-cov --html
    @echo "Report at target/llvm-cov/html/index.html"

# Generate coverage summary
coverage-summary:
    cargo llvm-cov --fail-under-lines 90

# Run the trlt CLI
run *args:
    cargo run -- {{args}}

# Generate and open docs
doc:
    cargo doc --no-deps --open

# Install/update git hooks
hooks:
    cog install-hook --all --overwrite

# Setup development environment
setup:
    ./scripts/setup.sh
