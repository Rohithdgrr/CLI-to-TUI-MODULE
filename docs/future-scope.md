# Future Scope

> Roadmap, planned features, and long-term vision for `universal-tui-generator`.

## Version Roadmap

### v0.1 — Core MVP

**Target:** Working TUI from clap struct with basic widgets.

- [x] Workspace setup
- [x] Core schema types
- [x] `#[derive(Tui)]` macro
- [x] Clap adapter
- [x] Ratatui renderer
- [x] String, bool, number widgets
- [x] PathBuf widget
- [x] Option<T> support
- [x] Default values
- [x] Required field validation
- [x] Keyboard navigation
- [x] Submit/cancel
- [x] Terminal cleanup
- [x] Basic examples

### v0.2 — Enhanced Widgets

**Target:** Enums, subcommands, better UX.

- [ ] Enum → Select widget
- [ ] Subcommand menu
- [ ] Nested subcommand navigation
- [ ] Breadcrumb navigation
- [ ] Responsive layouts
- [ ] Scrolling for large forms
- [ ] Help panel
- [ ] Section grouping
- [ ] Argh adapter
- [ ] Custom widget overrides
- [ ] Label overrides
- [ ] Hidden fields (skip)
- [ ] Read-only fields
- [ ] Password fields

### v0.3 — Advanced Features

**Target:** Conditional fields, custom widgets, command preview.

- [ ] Conditional visibility (visible_if)
- [ ] Conditional enablement (enabled_if)
- [ ] Custom widget trait
- [ ] Command preview before execution
- [ ] CLI argument export
- [ ] Mouse support
- [ ] Search fields (Ctrl+F)
- [ ] Numeric range validation
- [ ] Custom validators
- [ ] Theme system
- [ ] Built-in themes (dark, light, mono, high-contrast)
- [ ] Key binding customization
- [ ] Configuration API

### v0.4 — Developer Experience

**Target:** Profiles, persistence, undo/redo.

- [ ] Configuration profiles
- [ ] Save/load form state
- [ ] Undo/redo (Ctrl+Z, Ctrl+Y)
- [ ] Reset fields (Ctrl+R)
- [ ] Command palette (Ctrl+P)
- [ ] Multi-value arguments (Vec<T>)
- [ ] Repeatable arguments
- [ ] Environment variable display
- [ ] Shell completion compatibility
- [ ] Compile-time diagnostics improvement

### v0.5 — Plugin System

**Target:** Extensibility and custom renderers.

- [ ] Plugin architecture
- [ ] Widget plugins
- [ ] Renderer plugins
- [ ] Validator plugins
- [ ] Theme plugins
- [ ] Layout plugins
- [ ] Custom renderer trait
- [ ] Cursive renderer (alternative TUI)
- [ ] Web TUI renderer (terminal in browser)

### v1.0 — Stable Release

**Target:** Stable public API, production-ready.

- [ ] API stabilization
- [ ] Comprehensive documentation
- [ ] Full test coverage
- [ ] Performance benchmarks
- [ ] Security audit
- [ ] Accessibility audit
- [ ] Cross-platform testing matrix
- [ ] Migration guide from 0.x

## Long-Term Vision

### Schema-Driven UI

The project evolves from "CLI → TUI" to "Schema → UI":

```
CLI Struct
    │
    ├── CLI
    │
    ├── TUI (terminal)
    │
    ├── Web TUI (browser-based terminal)
    │
    └── GUI (future)
```

All from one source of truth.

### Future Renderer Targets

```
Ratatui (terminal)
    ↓
Cursive (alternative terminal)
    ↓
Termion (Unix terminal)
    ↓
Web terminal (WASM in browser)
    ↓
egui (GUI)
    ↓
iced (GUI)
    ↓
Custom renderers
```

### Future CLI Framework Support

```
clap (primary)
    ↓
argh
    ↓
bpaf
    ↓
gumdrop
    ↓
lexopt
    ↓
structopt (legacy)
    ↓
custom schema adapter
```

### Advanced Layout System

Future layout capabilities:

- Grid layout
- Flexbox-like constraints
- Responsive breakpoints
- Drag-and-drop reordering (in GUI)
- Custom layout widgets
- Form wizards (multi-step)
- Tabbed interfaces
- Accordion sections

### Advanced Widget System

Future widgets:

- Date/time picker
- Color picker
- Slider/range
- Key-value editor
- Table editor
- Tree view
- Code editor (syntax highlighted)
- Markdown preview
- JSON/YAML editor
- File tree browser
- Process manager
- Log viewer

### Configuration Persistence

Future capabilities:

```rust
// Save form state
state.save("my-profile")?;

// Load form state
let state = TuiState::load("my-profile")?;

// List profiles
let profiles = TuiState::list_profiles()?;

// Delete profile
TuiState::delete_profile("my-profile")?;
```

Storage locations:

```
~/.config/myapp/profiles/
~/.local/share/myapp/profiles/
$APPDATA/myapp/profiles/
```

### Multi-Language Support

Future: i18n for TUI labels and messages.

```rust
TuiConfig::default()
    .locale("ja-JP")
```

### Accessibility Enhancements

- Screen reader support
- High contrast mode improvements
- Keyboard-only operation verification
- ARIA-like labels for terminal widgets
- Color-blind friendly themes
- Font size awareness

### Performance Optimizations

- Incremental rendering (only redraw changed fields)
- Virtual scrolling for 1000+ fields
- Parallel validation
- Cached schema construction
- Lazy widget initialization
- Async event handling

### Testing Improvements

- Snapshot testing with `insta`
- Property testing with `proptest`
- Compile-fail testing with `trybuild`
- Integration testing framework
- Visual regression testing
- Performance benchmarking suite
- Cross-platform CI matrix

### Documentation Improvements

- Interactive examples in docs
- Video tutorials
- Cookbook recipes
- Migration guides
- API reference with examples
- Architecture decision records (ADRs)
- Contributing guide with diagrams

### Community Features

- Plugin marketplace
- Theme gallery
- Widget library
- Example gallery
- Community adapters
- Community renderers

## Research Areas

### Compile-Time Reflection

Investigate Rust compile-time reflection to extract more metadata from structs without runtime overhead.

### Macro Expansion Optimization

Reduce macro compilation time through:

- Caching
- Incremental expansion
- Parallel expansion

### Terminal Detection

Auto-detect terminal capabilities:

- Color support
- Unicode support
- Mouse support
- Size limits
- Font support

### Mobile Terminal Support

Future: support for mobile terminal apps (Termux, iSH).

### Voice Control

Future: voice commands for accessibility.

### AI Integration

Future: AI-assisted form filling, auto-completion, smart defaults.

## Deprecation Policy

Before v1.0:

- Breaking changes allowed with deprecation warnings
- Migration guides provided

After v1.0:

- Semantic versioning strictly followed
- Breaking changes only in major versions
- Deprecated features marked with `#[deprecated]`
- Minimum 2 minor versions before removal

## Success Metrics

### Adoption

- crates.io downloads
- GitHub stars
- Community contributions
- Real-world projects using it

### Quality

- Test coverage > 80%
- Zero known security vulnerabilities
- Cross-platform compatibility
- Performance benchmarks met

### Developer Experience

- Time from CLI to TUI < 5 minutes
- Compile-time errors are clear
- Documentation covers all use cases
- Examples cover common patterns
