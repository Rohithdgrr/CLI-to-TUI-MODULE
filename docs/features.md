# Features

> Complete feature list for `universal-tui-generator`.

## Core Features

### Automatic TUI Generation

Add `Tui` to your derive list. Get a working interactive form — no manual layout code.

```rust
#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(long)]
    verbose: bool,
}
```

### CLI ↔ TUI Compatibility

Same application supports both modes:

```bash
# CLI mode
myapp --input file.txt --threads 8

# TUI mode
myapp --tui
```

### Minimal Developer Code

Target: `<5 minutes` from existing CLI to working TUI.

### Single Source of Truth

The CLI struct is the primary source of truth. No separate `AppState` struct needed.

## Supported Parsers

### clap

First-class support for `clap::Parser`.

```rust
#[derive(Parser, Tui)]
struct Cli { ... }
```

### argh

Support for `argh::FromArgs`.

```rust
#[derive(FromArgs, Tui)]
struct Cli { ... }
```

## Type → Widget Mapping

| Rust Type | Widget |
|-----------|--------|
| `String` | TextInput |
| `PathBuf` | PathInput |
| `bool` | Checkbox |
| `u8` | NumberInput |
| `u16` | NumberInput |
| `u32` | NumberInput |
| `u64` | NumberInput |
| `usize` | NumberInput |
| `i32` | NumberInput |
| `i64` | NumberInput |
| `f32` | NumberInput |
| `f64` | NumberInput |
| `Option<T>` | Optional T widget |
| `enum` | Select |
| `Vec<T>` | MultiSelect/List |
| password-like | PasswordInput |

## Widget Types

### Initial Widgets

- `TextInput` — free text entry
- `PasswordInput` — masked text entry
- `NumberInput` — numeric entry with validation
- `Checkbox` — boolean toggle
- `Select` — single-choice from enum
- `MultiSelect` — multi-choice from list
- `PathInput` — file/directory path
- `FileInput` — file path with picker (future)
- `DirectoryInput` — directory path with picker (future)
- `Confirm` — yes/no dialog
- `TextArea` — multi-line text (future)

### Future Widgets

- `Slider` — numeric range
- `DateInput` — date picker
- `ColorPicker` — color selection
- `KeyValueEditor` — map editor
- `ListEditor` — list editor
- `TableEditor` — table editor
- `CommandSelector` — command picker

## Enum Support

Given:

```rust
#[derive(ValueEnum)]
enum Format { Json, Yaml, Toml }
```

Generate select widget with keyboard navigation.

## Boolean Support

```rust
#[arg(long)]
verbose: bool
```

Generates checkbox. Space toggles.

## Optional Arguments

```rust
option: Option<String>
```

Rendered as optional widget with enable/disable toggle.

## Default Values

```rust
#[arg(default_value_t = 4)]
threads: usize
```

Loaded into initial TUI state.

## Required Values

Required fields marked with `*`. Validation prevents submission with empty required values.

## Positional Arguments

Positional args rendered as text inputs.

## Short and Long Flags

Metadata shown in help:

```
Input
-i, --input
```

## Help Text

```rust
#[arg(help = "Number of worker threads")]
workers: usize
```

Displayed in help panel and below field labels.

## Subcommands

```rust
#[derive(Subcommand)]
enum Commands {
    Build(BuildArgs),
    Clean(CleanArgs),
}
```

Generates command menu with nested form navigation.

## Nested Subcommands

```
app → project → create → form
```

Breadcrumb navigation: `Home > Project > Create`.

## Validation

Pipeline: Type → Required → Range → Custom → Application → Execute.

### Numeric Validation

Rejects out-of-range values with clear messages.

### Custom Validation

```rust
#[tui(validate = validate_input)]
input: PathBuf
```

## Customization

### Widget Override

```rust
#[tui(widget = "select")]
format: Format
```

### Label Override

```rust
#[tui(label = "Maximum Connections")]
max_connections: usize
```

### Section Grouping

```rust
#[tui(section = "Network")]
host: String
```

### Hidden Fields

```rust
#[tui(skip)]
internal: String
```

### Read-Only Fields

```rust
#[tui(readonly)]
version: String
```

### Password Fields

```rust
#[tui(password)]
token: String
```

### Ordering

```rust
#[tui(order = 10)]
field: String
```

Default: struct field declaration order.

## Layout

### Automatic Layout

Calculates layout from field count and terminal size.

### Responsive Layout

- Wide: form | description side by side
- Medium: form with description below
- Small: one field at a time

### Scrolling

Vertical scrolling, focus auto-scroll, page/section navigation.

## Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` | Previous field |
| `↓` | Next field |
| `Tab` | Next field |
| `Shift+Tab` | Previous field |
| `Enter` | Select/Edit |
| `Space` | Toggle checkbox |
| `Esc` | Back/Cancel |
| `F1` | Help |
| `q` | Quit |
| `Ctrl+C` | Quit |
| `Ctrl+Enter` | Submit |

All keys remappable.

## Themes

Built-in: Default, Dark, Light, Monochrome, HighContrast.

```rust
TuiConfig::default().theme(Theme::dark())
```

## Accessibility

- Keyboard-only operation
- High contrast mode
- Clear focus indicators
- No color-only information
- Readable error messages
- Terminal resize handling

## Mouse Support

Optional. Click, scroll, selection. Keyboard remains primary.

## Configuration API

```rust
Cli::run_tui_with(
    TuiConfig::default()
        .title("My Application")
        .theme(Theme::dark())
        .mouse(true)
)
```

## Command Preview

Before execution, show generated CLI command:

```
myapp --input ./data.json --threads 8 --verbose
```

## CLI Command Export

```rust
let args = state.to_cli_args();
```

## Security

- Never log passwords, API keys, tokens
- Debug output redacts secret fields
- `api_key = ********`

## Platform Support

- Linux (Ubuntu, Fedora, Arch, Debian)
- macOS
- Windows (Terminal, PowerShell, cmd)
- SSH sessions
- UTF-8 terminals

## Feature Flags

```toml
[features]
default = ["ratatui"]
clap = ["dep:clap"]
argh = ["dep:argh"]
ratatui = ["dep:ratatui", "dep:crossterm"]
mouse = [...]
serde = [...]
```

## Performance

- Macro generation: compile-time only
- Runtime schema construction: < 1ms
- TUI frame rendering: 60 FPS
- 100-field navigation: < 1ms
