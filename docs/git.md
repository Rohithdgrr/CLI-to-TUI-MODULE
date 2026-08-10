# Git Guide

> Git conventions, branching strategy, and commit standards for `universal-tui-generator`.

## Repository

```
https://github.com/user/universal-tui-generator.git
```

## Branching Strategy

### Main Branches

- `main` — stable, release-ready code
- `develop` — integration branch for features (optional, can use PRs to main)

### Feature Branches

```
feature/feature-name
fix/bug-description
docs/documentation-update
test/test-addition
refactor/refactor-description
```

Examples:

```
feature/enum-widget
feature/argh-adapter
fix/empty-required-validation
docs/api-reference
test/snapshot-tests
```

### Naming Convention

```
<type>/<short-description>
```

Types:

- `feature` — new functionality
- `fix` — bug fix
- `docs` — documentation only
- `test` — adding tests
- `refactor` — code restructure
- `chore` — maintenance
- `perf` — performance improvement

## Commit Messages

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat` — new feature
- `fix` — bug fix
- `docs` — documentation
- `test` — tests
- `refactor` — code restructure
- `chore` — maintenance
- `perf` — performance
- `style` — formatting
- `ci` — CI/CD

### Scope

Optional, indicates which crate is affected:

```
feat(core): add Value enum
feat(macros): generate schema from struct
feat(clap): support subcommands
fix(ratatui): handle terminal resize
```

### Examples

```
feat(macros): add Tui derive macro

Implements #[derive(Tui)] that generates TuiSchema from
struct definitions with clap metadata.

Closes #12
```

```
fix(core): handle empty required fields

Previously, submitting with empty required fields would
panic. Now returns ValidationError.

Fixes #45
```

```
docs: update architecture diagram

Add subcommand flow and conversion pipeline.
```

```
test(macros): add trybuild tests for unsupported types

Tests compile-fail behavior for HashMap, Box<dyn Trait>,
and generic types.
```

## Pull Requests

### Title Format

Same as commit messages:

```
feat(core): add WidgetKind enum
```

### Description Template

```markdown
## Summary

Brief description of changes.

## Changes

- Change 1
- Change 2
- Change 3

## Testing

How were these changes tested?

## Related Issues

Closes #123
```

### Review Checklist

- [ ] Code follows existing patterns
- [ ] Tests pass
- [ ] Clippy warnings addressed
- [ ] Formatting applied
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
- [ ] Feature flags considered

## Tags

### Format

```
v<major>.<minor>.<patch>
```

Examples:

```
v0.1.0
v0.2.0
v1.0.0
```

### Creating Tags

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

## Git Hooks

### Pre-commit

Run before each commit:

```bash
cargo fmt --check
cargo clippy -- -D warnings
```

Setup with `pre-commit` or `husky`.

### Commit-msg

Validate commit message format:

```bash
# .git/hooks/commit-msg
#!/bin/sh
regex='^(feat|fix|docs|test|refactor|chore|perf|style|ci)(\(.+\))?: .{1,72}'

if ! grep -qE "$regex" "$1"; then
    echo "Invalid commit message format"
    exit 1
fi
```

## Git Configuration

### .gitignore

```gitignore
/target
Cargo.lock
*.swp
*.swo
*~
.DS_Store
```

### .gitattributes

```
*.rs linguist-language=Rust
*.md linguist-documentation
```

## Workflow

### Starting Work

```bash
git checkout main
git pull origin main
git checkout -b feature/my-feature
```

### During Work

```bash
git add .
git commit -m "feat(scope): description"
```

### Finishing Work

```bash
git push origin feature/my-feature
# Create PR on GitHub
# After review and merge, clean up:
git checkout main
git pull origin main
git branch -d feature/my-feature
```

### Syncing Fork

```bash
git remote add upstream https://github.com/user/universal-tui-generator.git
git fetch upstream
git merge upstream/main
git push origin main
```

## Releases

### Checklist

1. [ ] All tests pass
2. [ ] Version bumped in all Cargo.toml files
3. [ ] CHANGELOG updated
4. [ ] README updated if needed
5. [ ] Tag created
6. [ ] Published to crates.io
7. [ ] GitHub release created

### Publishing Order

```bash
# Core first (no dependencies on other workspace crates)
cargo publish -p tui-generator-core

# Macros (depends on core)
cargo publish -p tui-generator-macros

# Adapters (depends on core)
cargo publish -p tui-generator-clap
cargo publish -p tui-generator-argh

# Renderer (depends on core)
cargo publish -p tui-generator-ratatui

# Public API (depends on all above)
cargo publish -p tui-generator
```
