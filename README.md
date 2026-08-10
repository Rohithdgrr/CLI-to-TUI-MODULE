# `universal-tui-generator`

> Automatically transform existing Rust CLI argument definitions into an interactive Terminal User Interface — with minimal or zero additional UI code.

## What is it?

`universal-tui-generator` lets you derive a full TUI from your existing CLI struct. Add `Tui` to your derive list and get an interactive terminal form, navigation, validation, and keyboard controls — automatically.

```rust
use clap::Parser;
use tui_generator::Tui;

#[derive(Parser, Tui)]
#[tui(title = "Image Processor")]
struct Cli {
    /// Input image path
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path
    #[arg(short, long)]
    output: PathBuf,

    /// Number of processing threads
    #[arg(long, default_value_t = 4)]
    threads: usize,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}
```

Run with:

```bash
myapp --tui
```

Get:

```
┌──────────────────────────────────────────────────────────┐
│                    Image Processor                       │
├──────────────────────────────────────────────────────────┤
│                                                          │
│ Input                                                    │
│ ┌──────────────────────────────────────────────────────┐ │
│ │ ./input.txt                                          │ │
│ └──────────────────────────────────────────────────────┘ │
│                                                          │
│ Output                                                   │
│ ┌──────────────────────────────────────────────────────┐ │
│ │ ./output.txt                                         │ │
│ └──────────────────────────────────────────────────────┘ │
│                                                          │
│ Threads                                                  │
│ [ 4 ]                                                    │
│                                                          │
│ Verbose                                                  │
│ [ ]                                                      │
│                                                          │
│             [ Run ]       [ Cancel ]                     │
│                                                          │
├──────────────────────────────────────────────────────────┤
│ ↑↓ Navigate   Enter Edit   Space Toggle   F1 Help       │
└──────────────────────────────────────────────────────────┘
```

## Why?

Most Rust CLI apps already define their full interface in a struct. But the TUI is built manually — forms, widgets, layouts, validation, navigation, all duplicated. This crate eliminates that duplication.

## Installation

```toml
[dependencies]
tui-generator = { version = "0.1", features = ["clap", "ratatui"] }
```

## 30-Second Example

```rust
use clap::Parser;
use tui_generator::Tui;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    age: u32,

    #[arg(long)]
    verbose: bool,
}

fn main() {
    let cli = Cli::parse_or_tui().unwrap();
    // use cli...
}
```

## Supported Types

| Rust Type | Widget |
|-----------|--------|
| `String` | TextInput |
| `PathBuf` | PathInput |
| `bool` | Checkbox |
| `u8/u16/u32/u64/usize/i32/i64` | NumberInput |
| `f32/f64` | NumberInput |
| `Option<T>` | Optional widget |
| `enum` | Select |
| `Vec<T>` | MultiSelect |

## Customization

Override widget, label, section, visibility, and more:

```rust
#[tui(widget = "password")]
api_key: String,

#[tui(label = "Maximum Connections")]
max_connections: usize,

#[tui(section = "Network")]
host: String,
```

## CLI ↔ TUI Compatibility

```bash
# CLI mode
myapp --input file.txt --threads 8

# TUI mode
myapp --tui
```

## Architecture

```
CLI Struct → Derive Macro → TUI Schema → TUI Core → Ratatui Renderer → Terminal
```

See [docs/architecture.md](docs/architecture.md) for details.

## Documentation

- [Architecture](docs/architecture.md)
- [Features](docs/features.md)
- [API Reference](docs/api.md)
- [Backend](docs/backend.md)
- [Frontend](docs/frontend.md)
- [Workflow](docs/workflow.md)
- [Problems & Risks](docs/problems.md)
- [Future Scope](docs/future-scope.md)
- [Git Guide](docs/git.md)

## License

MIT OR Apache-2.0
