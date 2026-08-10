# `universal-tui-generator` — Documentation Index

> Automatically transform existing Rust CLI argument definitions into an interactive Terminal User Interface.

## Overview

`universal-tui-generator` is a Rust crate that derives a complete TUI from your existing CLI argument struct. Add `Tui` to your derive list and get an interactive terminal form — with navigation, validation, keyboard controls, and submit/cancel — automatically.

## Quick Start

```rust
use clap::Parser;
use tui_generator::Tui;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    input: String,

    #[arg(long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse_or_tui().unwrap();
}
```

## Documentation Pages

| Document | Description |
|----------|-------------|
| [architecture.md](architecture.md) | System architecture, crate responsibilities, data flow |
| [features.md](features.md) | Full feature list, widget types, type mappings |
| [working.md](working.md) | How it works internally — macro pipeline, schema, state |
| [problems.md](problems.md) | Known risks, limitations, mitigation strategies |
| [workflow.md](workflow.md) | Development workflow, build, test, release process |
| [git.md](git.md) | Git conventions, branching, commit messages |
| [api.md](api.md) | Public API reference, macros, traits, config |
| [backend.md](backend.md) | Backend architecture — core schema, adapters, renderer |
| [frontend.md](frontend.md) | Terminal rendering — ratatui layer, layout, widgets |
| [future-scope.md](future-scope.md) | Roadmap, planned features, long-term vision |

## Project Status

- **Status:** Planned
- **Language:** Rust
- **License:** MIT OR Apache-2.0
- **CLI parsers:** clap, argh
- **TUI backend:** ratatui + crossterm

## Core Principle

> **Define once, generate everywhere.**

The CLI struct remains the single source of truth. The TUI is derived, not manually maintained.
