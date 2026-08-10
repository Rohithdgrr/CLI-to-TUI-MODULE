# Backend Architecture

> Core schema system, adapters, and renderer abstractions.

## Backend Layers

```
┌─────────────────────────────────────────┐
│           CLI Struct (User Code)        │
└──────────────────┬──────────────────────┘
                   │
         ┌─────────┴─────────┐
         │   Proc Macro      │
         │   (Compile Time)  │
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │   Core Schema     │
         │   (Framework-     │
         │    Independent)   │
         └─────────┬─────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
┌───┴───┐    ┌────┴────┐   ┌────┴────┐
│ clap  │    │  argh   │   │ custom  │
│adapter│    │ adapter │   │ adapter │
└───┬───┘    └────┬────┘   └────┬────┘
    │              │              │
    └──────────────┼──────────────┘
                   │
         ┌─────────┴─────────┐
         │   TUI Core        │
         │   (State, Events, │
         │    Validation)    │
         └─────────┬─────────┘
                   │
         ┌─────────┴─────────┐
         │   Renderer        │
         │   (Ratatui)       │
         └─────────┬─────────┘
                   │
              Terminal
```

## Core Schema (`tui-generator-core`)

### Dependencies

None (except `std`). No ratatui, no clap, no crossterm.

### Purpose

Define the intermediate representation that bridges CLI parsers and TUI renderers.

### Key Types

```rust
// Schema
pub struct TuiSchema { ... }
pub struct Field { ... }
pub struct Command { ... }

// Values
pub enum Value { ... }

// Widgets
pub enum WidgetKind { ... }

// Validation
pub struct Constraint { ... }
pub struct ValidationError { ... }

// State
pub struct FormState { ... }
pub struct Navigation { ... }

// Events
pub enum Action { ... }
pub enum Event { ... }
```

### Design Principles

- Framework-agnostic
- Serializable (optional serde feature)
- Deterministic construction
- No runtime reflection

## Adapters

### Clap Adapter (`tui-generator-clap`)

**Purpose:** Convert clap metadata to `TuiSchema`.

**Input:** `#[derive(Parser)]` struct with `#[arg(...)]` attributes.

**Output:** `TuiSchema` with fields, types, defaults, constraints.

**Process:**

```
clap struct
    ↓
inspect #[arg(...)] attributes
    ↓
extract: name, type, default, help, required, possible_values
    ↓
map to Field + WidgetKind
    ↓
TuiSchema
```

**Supported clap features:**

- `#[arg(short, long)]`
- `#[arg(default_value_t = ...)]`
- `#[arg(help = "...")]`
- `#[arg(value_parser = ...)]`
- `#[arg(value_enum)]`
- `#[command(subcommand)]`
- `#[command(name = "...")]`
- Required vs optional detection
- Possible values extraction
- Range constraints

**Limitations:**

- Some clap internals not publicly accessible
- Complex `value_parser` chains may not fully map

### Argh Adapter (`tui-generator-argh`)

**Purpose:** Convert argh metadata to `TuiSchema`.

**Input:** `#[derive(FromArgs)]` struct with `#[argh(...)]` attributes.

**Output:** `TuiSchema`.

**Process:**

```
argh struct
    ↓
inspect #[argh(...)] attributes
    ↓
extract: name, type, default, description, subcommand
    ↓
map to Field + WidgetKind
    ↓
TuiSchema
```

**Supported argh features:**

- `#[argh(option)]`
- `#[argh(switch)]`
- `#[argh(subcommand)]`
- `#[argh(description = "...")]`
- Default values

### Custom Adapter (future)

Allow users to implement their own adapter:

```rust
impl SchemaAdapter for MyAdapter {
    fn to_schema(&self) -> TuiSchema { ... }
}
```

## Renderer Abstraction

### Trait

```rust
pub trait Renderer {
    fn render(&mut self, state: &TuiState, schema: &TuiSchema);
    fn handle_event(&mut self, event: Event) -> Action;
    fn cleanup(&mut self);
}
```

### Implementations

- `RatatuiRenderer` — default terminal renderer
- Future: `CursiveRenderer`, `TermionRenderer`, `WebRenderer`, `GuiRenderer`

## State Management

### State Flow

```
Schema
    ↓
initialize (defaults from schema)
    ↓
TuiState
    ↓
Event → Action → State update
    ↓
Render (read state)
    ↓
Submit → values → Cli struct
```

### State Isolation

Each subcommand gets isolated state:

```rust
pub struct AppState {
    main: TuiState,
    subcommands: HashMap<String, TuiState>,
    active: String,
}
```

## Validation System

### Pipeline

```
Value
    ↓
Type check
    ↓
Required check
    ↓
Range check (min/max)
    ↓
Length check (min/max)
    ↓
Pattern check (regex)
    ↓
Custom validator
    ↓
Result
```

### Custom Validators

```rust
fn validate_input(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err("File does not exist".into());
    }
    Ok(())
}
```

Registered via:

```rust
#[tui(validate = validate_input)]
input: PathBuf
```

## Conversion System

### TUI → CLI

```
TuiState.values
    ↓
HashMap<String, Value>
    ↓
Generated From impls
    ↓
Cli struct
```

### CLI → TUI

```
Cli struct
    ↓
Generated ToValues impl
    ↓
HashMap<String, Value>
    ↓
TuiState (with defaults loaded)
```

### Round-Trip Guarantee

```
Cli → Values → Tui → Values → Cli
```

Must be lossless for supported types.

## Error Handling

### Error Types

```rust
pub enum TuiError {
    TerminalError(String),      // crossterm failures
    ValidationError(Vec<...>),  // field validation
    ConversionError(String),    // type conversion
    UnsupportedType(String),    // unknown Rust type
    Cancelled,                  // user pressed Esc
    IoError(io::Error),         // IO failures
}
```

### Error Display

Errors render in the TUI as:

```
┌─ Validation Errors ─────────────────┐
│                                     │
│ ✗ Input is required                 │
│ ✗ Threads must be between 1 and 64  │
│                                     │
│         [ Back to Edit ]            │
└─────────────────────────────────────┘
```

## Terminal Lifecycle

### Initialization

```rust
fn init_terminal() -> Result<Terminal, TuiError> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    Ok(terminal)
}
```

### Cleanup (RAII)

```rust
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        disable_raw_mode().ok();
        execute!(stdout, LeaveAlternateScreen, cursor::Show).ok();
    }
}
```

### Panic Safety

```rust
std::panic::set_hook(Box::new(|_| {
    // ensure terminal cleanup
    cleanup_terminal();
}));
```
