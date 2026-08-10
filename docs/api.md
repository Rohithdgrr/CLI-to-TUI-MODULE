# API Reference

> Public API for `universal-tui-generator`.

## Main Crate: `tui-generator`

### Derive Macro

```rust
use tui_generator::Tui;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,

    #[arg(long, default_value_t = 4)]
    threads: usize,
}
```

### Generated Trait

```rust
pub trait Tui {
    fn schema() -> TuiSchema;
    fn from_values(values: HashMap<String, Value>) -> Result<Self, TuiError>;
    fn to_values(&self) -> HashMap<String, Value>;
}
```

### Entry Points

```rust
// Parse or run TUI based on arguments
let cli = Cli::parse_or_tui()?;

// Run TUI explicitly
let cli = Cli::run_tui()?;

// Run TUI with configuration
let cli = Cli::run_tui_with(TuiConfig::default())?;

// Parse from CLI arguments
let cli = Cli::parse();
```

## Attributes

### Struct Attributes

```rust
#[tui(title = "My Application")]
#[tui(description = "A helpful tool")]
```

### Field Attributes

```rust
#[tui(label = "Custom Label")]
#[tui(widget = "select")]
#[tui(section = "Network")]
#[tui(order = 10)]
#[tui(skip)]
#[tui(readonly)]
#[tui(password)]
#[tui(placeholder = "Enter value...")]
#[tui(validate = my_validator)]
#[tui(visible_if = "other_field")]
#[tui(enabled_if = "other_field")]
```

## Core Types

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
    pub section: Option<String>,
    pub order: Option<u32>,
    pub readonly: bool,
    pub password: bool,
    pub placeholder: Option<String>,
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

### Constraint

```rust
pub enum Constraint {
    Required,
    MinLength(usize),
    MaxLength(usize),
    MinValue(f64),
    MaxValue(f64),
    Pattern(String),
    Custom(Box<dyn Fn(&Value) -> Result<(), String>>),
}
```

### ValidationError

```rust
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub kind: ValidationKind,
}

pub enum ValidationKind {
    Required,
    TypeMismatch,
    OutOfRange,
    Custom,
}
```

### TuiError

```rust
pub enum TuiError {
    TerminalError(String),
    ValidationError(Vec<ValidationError>),
    ConversionError(String),
    UnsupportedType(String),
    Cancelled,
    IoError(io::Error),
}
```

## Configuration

### TuiConfig

```rust
pub struct TuiConfig {
    pub title: Option<String>,
    pub description: Option<String>,
    pub theme: Theme,
    pub mouse: bool,
    pub keybindings: KeyBindings,
}
```

Builder pattern:

```rust
TuiConfig::default()
    .title("My App")
    .theme(Theme::dark())
    .mouse(true)
```

### Theme

```rust
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub accent: Color,
    pub error: Color,
    pub success: Color,
    pub border: Color,
}
```

Built-in themes:

```rust
Theme::default()
Theme::dark()
Theme::light()
Theme::monochrome()
Theme::high_contrast()
```

### KeyBindings

```rust
pub struct KeyBindings {
    pub navigate_up: KeyCode,
    pub navigate_down: KeyCode,
    pub select: KeyCode,
    pub toggle: KeyCode,
    pub submit: KeyCode,
    pub cancel: KeyCode,
    pub help: KeyCode,
    pub quit: KeyCode,
}
```

## Traits

### WidgetController

```rust
pub trait WidgetController {
    fn render(&self, area: Rect, frame: &mut Frame, state: &FieldState);
    fn handle_event(&mut self, event: KeyEvent) -> Option<Action>;
    fn value(&self) -> Value;
    fn set_value(&mut self, value: Value);
    fn is_readonly(&self) -> bool;
}
```

### Renderer

```rust
pub trait Renderer {
    fn render(&mut self, state: &TuiState, schema: &TuiSchema);
}
```

### Validator

```rust
pub trait Validator {
    fn validate(&self, value: &Value, field: &Field) -> Result<(), ValidationError>;
}
```

## State

### TuiState

```rust
pub struct TuiState {
    pub active_screen: Screen,
    pub focused_field: usize,
    pub values: HashMap<String, Value>,
    pub errors: Vec<ValidationError>,
    pub help_visible: bool,
}
```

### Screen

```rust
pub enum Screen {
    CommandMenu,
    Form(String),
    Help,
    Confirmation,
}
```

### Action

```rust
pub enum Action {
    NavigateNext,
    NavigatePrevious,
    Select,
    Cancel,
    Submit,
    Toggle,
    Edit,
    Help,
    Quit,
    None,
}
```

### Event

```rust
pub enum Event {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}
```

## Conversion

### From CLI to TUI

```rust
let cli = Cli::parse();
let values = cli.to_values();
let state = TuiState::from_values(schema, values);
```

### From TUI to CLI

```rust
let cli = Cli::from_values(state.values)?;
```

### To CLI Arguments

```rust
let args: Vec<String> = state.to_cli_args();
// ["--input", "file.txt", "--threads", "8"]
```

## Integration

### With clap

```rust
use clap::Parser;
use tui_generator::Tui;

#[derive(Parser, Tui)]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
}
```

### With argh

```rust
use argh::FromArgs;
use tui_generator::Tui;

#[derive(FromArgs, Tui)]
struct Cli {
    #[argh(option)]
    input: PathBuf,
}
```

### With anyhow

```rust
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_or_tui()?;
    process(cli)?;
    Ok(())
}
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

## Re-exports

```rust
pub use tui_generator_core::{
    TuiSchema, Field, Value, WidgetKind, Constraint,
    ValidationError, TuiError, Action, Event,
};
pub use tui_generator_macros::Tui;
```
