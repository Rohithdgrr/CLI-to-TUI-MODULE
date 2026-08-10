# Architecture

> How `universal-tui-generator` is structured internally.

## High-Level Data Flow

```
                Rust CLI Struct
                       │
                       ▼
              ┌─────────────────┐
              │ Procedural Macro │
              └────────┬────────┘
                       │
                       ▼
                TUI Schema
                       │
          ┌────────────┼────────────┐
          │            │            │
          ▼            ▼            ▼
        clap         argh        custom
          │            │            │
          └────────────┼────────────┘
                       ▼
                  TUI Core
                       │
                       ▼
                TUI Application
                       │
                       ▼
                 Ratatui Layer
                       │
                       ▼
                    Terminal
                       │
                       ▼
                      User
                       │
                     Submit
                       │
                       ▼
                Generated Cli → Application Logic
```

## Most Important Rule

Never connect `clap → ratatui` directly.

```
clap/argh → normalized schema → TUI core → renderer
```

This keeps the project extensible.

## Crate Structure

```
universal-tui-generator/
├── crates/
│   ├── tui-generator/           # Main public crate
│   ├── tui-generator-core/      # Framework-independent core
│   ├── tui-generator-macros/    # Procedural macro
│   ├── tui-generator-clap/      # Clap adapter
│   ├── tui-generator-argh/      # Argh adapter
│   ├── tui-generator-ratatui/   # Ratatui renderer
│   └── tui-generator-cli/       # CLI utility (optional)
├── examples/
├── tests/
└── docs/
```

## Crate Responsibilities

### `tui-generator`

Main public crate. Re-exports components:

```rust
use tui_generator::Tui;
```

### `tui-generator-macros`

Procedural macro crate. Parses Rust AST, extracts field metadata, generates schema and adapter code. Does NOT contain terminal rendering logic.

```
TokenStream → syn::DeriveInput → Struct Analysis → Internal AST → Schema Generator → Code Generator → quote!
```

### `tui-generator-core`

Framework-independent core. Contains:

- `TuiSchema` — field definitions, metadata
- `Field` — individual field representation
- `Value` — generic value type
- `WidgetKind` — widget enum
- `Constraint` — validation rules
- `ValidationError` — error types
- `FormState` — runtime state
- `Action` — user actions
- `Event` — normalized terminal events

Does NOT depend on ratatui, crossterm, clap, or argh.

### `tui-generator-clap`

Clap integration. Inspects clap metadata, converts definitions to TUI schema. Supports subcommands, defaults, possible values, help text.

### `tui-generator-argh`

Argh integration. Same pattern as clap adapter.

### `tui-generator-ratatui`

Default renderer. Handles terminal lifecycle, rendering, widgets, focus, keyboard events, dialogs, help, themes.

### `tui-generator-cli`

Optional CLI utility. Future `cargo tui-gen` commands. Not required for normal usage.

## Core Data Model

### TuiSchema

```rust
pub struct TuiSchema {
    pub name: String,
    pub description: Option<String>,
    pub fields: Vec<Field>,
    pub subcommands: Vec<Command>,
}
```

### Field

```rust
pub struct Field {
    pub name: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    pub default: Option<Value>,
    pub widget: WidgetKind,
    pub constraints: Vec<Constraint>,
}
```

### Value

```rust
pub enum Value {
    String(String),
    Bool(bool),
    Integer(i64),
    Float(f64),
    Path(PathBuf),
    List(Vec<Value>),
    None,
}
```

### WidgetKind

```rust
pub enum WidgetKind {
    TextInput,
    PasswordInput,
    NumberInput,
    Checkbox,
    Select,
    MultiSelect,
    PathInput,
    FileInput,
    DirectoryInput,
    Confirm,
    TextArea,
}
```

## State Machine

```
             ┌─────────────┐
             │    Start    │
             └──────┬──────┘
                    │
                    ▼
             ┌─────────────┐
             │ CommandMenu │
             └──────┬──────┘
                    │
                    ▼
             ┌─────────────┐
             │    Form     │
             └──────┬──────┘
                    │
          ┌─────────┴─────────┐
          ▼                   ▼
      Validation           Cancel
          │
          ▼
       Execute
          │
          ▼
        Result
```

## Widget Abstraction

```rust
pub trait WidgetController {
    fn render(...);
    fn handle_event(...);
    fn value(&self) -> Value;
    fn set_value(&mut self, value: Value);
}
```

## Renderer Abstraction

```rust
pub trait Renderer {
    fn render(&mut self, app: &AppState);
}
```

Ratatui implements this trait. Future renderers (Cursive, Termion, web, GUI) can implement the same trait.

## Conversion Flow

```
TUI State → Value Map → Generated Conversion → Cli struct
```

```rust
let cli: Cli = tui_state.into_value()?;
```

## Terminal Lifecycle

```
initialize terminal
    ↓
build schema
    ↓
initialize state
    ↓
render
    ↓
receive event
    ↓
update state
    ↓
render
    ↓
submit/cancel
    ↓
restore terminal
```

Terminal restoration must happen even after errors/panic. A `TerminalGuard` struct ensures cleanup.

## Future Architecture

```
                 TUI Schema
                     │
       ┌─────────────┼─────────────┐
       │             │             │
       ▼             ▼             ▼
   Ratatui       Web TUI        GUI
       │             │             │
       ▼             ▼             ▼
   Terminal       Browser        Window
```

The project evolves into a general Schema → UI system without changing the original CLI API.
