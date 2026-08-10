# How It Works

> Internal mechanics of `universal-tui-generator`.

## Overview

The system works in three phases:

1. **Compile time** — macro parses struct, generates schema metadata and conversion code
2. **Runtime** — schema builds state, renders TUI, handles events
3. **Submission** — TUI state converts back to CLI struct

## Phase 1: Macro Pipeline (Compile Time)

```
TokenStream
    │
    ▼
syn::DeriveInput
    │
    ▼
Struct Analysis
    ├── Fields
    ├── Attributes
    ├── Types
    └── Parser metadata
    │
    ▼
Internal AST
    │
    ▼
Schema Generator
    │
    ▼
Code Generator
    │
    ▼
quote!
    │
    ▼
Rust compiler
```

### Step-by-step

1. Developer writes `#[derive(Tui)]`
2. Compiler invokes proc macro
3. Macro receives `TokenStream` of the struct
4. Parse into `syn::DeriveInput`
5. Extract fields, types, attributes (`#[tui(...)]`, `#[arg(...)]`)
6. Build internal `TuiField` representations
7. Map Rust types to `WidgetKind`
8. Generate `TuiSchema` construction code
9. Generate `From<Cli>` and `Into<Cli>` conversion code
10. Output expanded code via `quote!`

### Internal AST

```rust
struct TuiField {
    ident: Ident,
    name: String,
    ty: TypeInfo,
    widget: WidgetKind,
    required: bool,
    default: Option<TokenStream>,
}
```

This is NOT coupled to ratatui or any renderer.

### What the Macro Generates

```rust
impl Tui for Cli {
    fn schema() -> TuiSchema {
        TuiSchema {
            name: "Cli".into(),
            fields: vec![
                Field {
                    name: "input".into(),
                    label: "Input".into(),
                    widget: WidgetKind::PathInput,
                    required: true,
                    default: None,
                    ..
                },
                // ...
            ],
            subcommands: vec![],
        }
    }

    fn from_values(values: HashMap<String, Value>) -> Result<Self, TuiError> {
        Ok(Cli {
            input: values.get("input")?.try_into()?,
            // ...
        })
    }

    fn to_values(&self) -> HashMap<String, Value> {
        let mut map = HashMap::new();
        map.insert("input".into(), Value::Path(self.input.clone()));
        // ...
        map
    }
}
```

### What the Macro Does NOT Generate

- No terminal rendering code
- No event loop code
- No widget implementations
- No layout calculations

These belong to the runtime crates.

## Phase 2: Runtime (Schema → State → Render)

### Schema Construction

At startup, the runtime calls the generated `schema()` method:

```rust
let schema = Cli::schema();
```

This is a static, deterministic structure. No runtime reflection.

### State Initialization

```rust
let state = TuiState::new(schema);
```

State contains:

```rust
pub struct TuiState {
    pub active_screen: Screen,
    pub focused_field: usize,
    pub values: HashMap<String, Value>,
    pub errors: Vec<ValidationError>,
}
```

Initial values come from CLI defaults.

### Event Loop

```
initialize terminal (raw mode, alternate screen)
    ↓
loop {
    render(state)
        ↓
    receive event (key, resize, tick)
        ↓
    convert to Action
        ↓
    update state
        ↓
    if Submit → break
    if Cancel → break
}
    ↓
restore terminal
    ↓
return Cli or error
```

### Event → Action Conversion

```
KeyEvent → Action
    ↓
ArrowUp → NavigatePrevious
ArrowDown → NavigateNext
Tab → NavigateNext
ShiftTab → NavigatePrevious
Enter → Select/Edit
Space → Toggle
Esc → Back
F1 → Help
q → Quit
Ctrl+C → Quit
Ctrl+Enter → Submit
```

### State Update

```
Action
    ↓
NavigateNext → focused_field += 1
NavigatePrevious → focused_field -= 1
Toggle → values[field] = !values[field]
Edit → enter edit mode for field
Submit → validate all fields → return Cli
Cancel → return error
```

### Rendering

The renderer reads state and draws to terminal:

```
State
    ↓
Calculate layout (terminal size, field count)
    ↓
Render header
    ↓
Render fields (focus indicator, values, errors)
    ↓
Render footer (keyboard hints)
    ↓
Render help panel (if F1 pressed)
    ↓
Render validation errors
```

## Phase 3: Submission (State → CLI)

```
TUI State
    ↓
Validation
    ↓
Value Map (HashMap<String, Value>)
    ↓
Generated conversion (From<Value> for each type)
    ↓
Cli struct
    ↓
Application logic
```

### Validation Pipeline

```
TUI value
    ↓
Type validation (is it a valid integer?)
    ↓
Required validation (is it present?)
    ↓
Range validation (is it within bounds?)
    ↓
Custom validation (user-provided function)
    ↓
Application validation (domain rules)
    ↓
Execute
```

### Conversion

The macro generates conversion code:

```rust
impl TryFrom<Value> for PathBuf {
    type Error = TuiError;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v {
            Value::Path(p) => Ok(p),
            Value::String(s) => Ok(PathBuf::from(s)),
            _ => Err(TuiError::ConversionError("expected path")),
        }
    }
}
```

## Type Inference

The macro maps Rust types to widgets at compile time:

```
PathBuf → PathInput
bool → Checkbox
u32 → NumberInput
String → TextInput
Option<T> → optional wrapper
enum → Select
Vec<T> → MultiSelect
```

For enums, the macro inspects variants to build the select options.

## Subcommand Handling

```
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
```

Macro detects `#[command(subcommand)]` and generates:

1. Command menu screen
2. Per-subcommand form screens
3. Navigation between screens
4. State isolation per subcommand

## Default Value Reconstruction

Clap stores defaults as `default_value_t = 4` or `default_value = "hello"`.

The macro extracts these at compile time:

```rust
default: Some(TokenStream::from("4usize")),
```

At runtime, this becomes:

```rust
Value::Integer(4)
```

Loaded into initial state.

## Error Recovery

Terminal cleanup happens via RAII:

```rust
struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // restore terminal
        // disable raw mode
        // hide cursor
        // leave alternate screen
    }
}
```

Even on panic, the terminal is restored.

## Performance Characteristics

- Macro expansion: compile-time only
- Schema construction: O(fields), < 1ms
- State update: O(1) per action
- Rendering: O(fields) per frame
- Conversion: O(fields) on submit

No runtime reflection. No dynamic type inspection. All metadata is generated at compile time.
