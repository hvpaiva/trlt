# Contributing

Thank you for considering contributing to trlt!

## Getting Started

1. Fork the repository
2. Clone your fork
3. Run the setup:

```bash
just setup
```

This installs all required tools (cocogitto, cargo-deny, cargo-nextest, cargo-llvm-cov) and configures git hooks.

## Pull Requests

1. Create a branch from `main`
2. Make your changes
3. Ensure `just check` passes
4. Commit using [Conventional Commits](#conventional-commits)
5. Push and open a Pull Request against `main`

Keep PRs focused and small. Separate refactoring from functional changes.

## Project Structure

```
src/
  cli.rs           # CLI argument definitions (clap)
  config.rs        # Configuration loading, saving, and migration
  input.rs         # Input resolution (text, file, stdin)
  main.rs          # Entry point and command dispatch
  translate.rs     # Translation orchestration
  provider/
    mod.rs         # Provider trait and factory
    openai.rs      # OpenAI implementation
    anthropic.rs   # Anthropic implementation
    ollama.rs      # Ollama implementation
tests/
  cli.rs           # Integration tests for the CLI binary
```

## Conventional Commits

This project enforces [Conventional Commits](https://www.conventionalcommits.org/) via git hooks.

You can use `cog commit` for an interactive guide:

```bash
cog commit feat "add streaming support" api
```

Or use `git commit` directly -- the commit-msg hook validates the format.

Format:

```
type(scope): description

[optional body]

[optional footer(s)]
```

**Types:** `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`, `revert`

**Scopes:** `api`, `cli`, `config`, `deps`, `version` (optional)

**Examples:**

```
feat(api): add Gemini provider support
fix(cli): handle empty stdin gracefully
docs: update configuration examples
build(deps): bump reqwest to 0.13
```

## Automated Versioning

Versions and changelogs are managed automatically by [cocogitto](https://docs.cocogitto.io/) via the CD workflow. **Do not** edit `CHANGELOG.md` or the `version` field in `Cargo.toml` manually.

## Development Commands

```bash
just check      # run all checks (fmt, lint, test, audit)
just fmt        # format code
just lint       # clippy
just test       # run tests with nextest
just audit      # dependency audit (cargo-deny)
just coverage   # HTML coverage report
just run        # run the trlt CLI
just doc        # generate and open docs
just hooks      # install/update git hooks
```

## Code Style

- Follow idiomatic Rust
- Run `cargo fmt` before committing
- All public items must be documented
- Write tests for new functionality
- Minimum 90% line coverage is enforced in CI

## Reporting Issues

Before opening an issue, check if a similar one already exists. Include reproduction steps and your `trlt --version` output when reporting bugs.
