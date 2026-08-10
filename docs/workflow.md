# Workflow

> Development workflow, build process, and release cycle for `universal-tui-generator`.

## Prerequisites

- Rust stable toolchain
- cargo-edit (optional, for `cargo add`)
- just (optional, for task running)

## Getting Started

```bash
# Clone the repository
git clone https://github.com/user/universal-tui-generator.git
cd universal-tui-generator

# Build the workspace
cargo build

# Run tests
cargo test

# Run examples
cargo run --example clap-basic
```

## Project Structure

```
universal-tui-generator/
├── Cargo.toml              # workspace root
├── crates/
│   ├── tui-generator/      # public API
│   ├── tui-generator-core/ # schema, types
│   ├── tui-generator-macros/ # proc macro
│   ├── tui-generator-clap/  # clap adapter
│   ├── tui-generator-ratatui/ # renderer
│   └── tui-generator-cli/   # optional CLI tool
├── examples/
├── tests/
└── docs/
```

## Build Commands

```bash
# Build everything
cargo build

# Build specific crate
cargo build -p tui-generator-core
cargo build -p tui-generator-macros

# Build with all features
cargo build --all-features

# Build in release mode
cargo build --release
```

## Test Commands

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p tui-generator-core
cargo test -p tui-generator-macros

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run compile-fail tests
cargo test --test compile_fail
```

## Example Commands

```bash
# Run basic example
cargo run --example clap-basic

# Run with TUI mode
cargo run --example clap-basic -- --tui

# Run with CLI mode
cargo run --example clap-basic -- --input file.txt
```

## Lint & Format

```bash
# Check formatting
cargo fmt --check

# Apply formatting
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run clippy with all features
cargo clippy --all-features -- -D warnings
```

## Feature Flags

```toml
[features]
default = ["ratatui"]
clap = ["dep:clap"]
argh = ["dep:argh"]
ratatui = ["dep:ratatui", "dep:crossterm"]
mouse = []
serde = ["dep:serde"]
```

## Development Cycle

### 1. Pick a Task

From the roadmap or issue tracker. Work in small, focused units.

### 2. Create Branch

```bash
git checkout -b feature/my-feature
```

### 3. Implement

Write code. Follow existing patterns. Keep changes minimal.

### 4. Test

```bash
cargo test
cargo clippy
cargo fmt --check
```

### 5. Document

Update relevant docs. Add examples if needed.

### 6. Commit

```bash
git add .
git commit -m "feat: add support for enum widgets"
```

### 7. Push & PR

```bash
git push origin feature/my-feature
```

Create pull request with description.

## Commit Convention

Follow Conventional Commits:

```
feat: add support for enum widgets
fix: handle empty required fields
docs: update API reference
test: add snapshot tests for subcommands
refactor: extract widget rendering into module
chore: update dependencies
```

## Release Process

### 1. Update Version

```toml
# Cargo.toml
version = "0.2.0"
```

### 2. Update CHANGELOG

```markdown
## [0.2.0] - 2026-08-15

### Added
- Enum widget support
- Subcommand navigation

### Fixed
- Empty required field validation

### Changed
- Improved error messages
```

### 3. Tag Release

```bash
git tag v0.2.0
git push origin v0.2.0
```

### 4. Publish to crates.io

```bash
cargo publish -p tui-generator-core
cargo publish -p tui-generator-macros
cargo publish -p tui-generator-clap
cargo publish -p tui-generator-ratatui
cargo publish -p tui-generator
```

## Dependency Updates

```bash
# Check for outdated dependencies
cargo outdated

# Update dependencies
cargo update

# Update specific dependency
cargo update -p clap
```

## CI/CD

### GitHub Actions

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - run: cargo test --all-features
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo fmt --check
```

## Troubleshooting

### Macro Not Expanding

Check that `tui-generator-macros` is in dependencies. Ensure `#[derive(Tui)]` is after `#[derive(Parser)]`.

### Compile Errors from Macro

Enable `RUSTFLAGS=-Zmacro-backtrace` (nightly) or use `cargo expand` to see expanded code.

### Terminal Issues

Test with different terminals. Check `TERM` environment variable. Use `--mouse` flag for mouse support.

## Performance Profiling

```bash
# Profile macro expansion
cargo build --timings

# Profile runtime
cargo build --release
perf target/release/myapp

# Benchmark
cargo bench
```
