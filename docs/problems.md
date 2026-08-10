# Problems & Risks

> Known risks, limitations, and mitigation strategies for `universal-tui-generator`.

## Risk 1 — clap Metadata Limitations

**Problem:** Some clap information may not be directly accessible to a procedural macro. Clap's internal representation is not fully public.

**Mitigation:**
- Build explicit normalization attributes (`#[tui(...)]`)
- Provide adapter APIs that inspect what's available
- Fall back to user-provided metadata when clap doesn't expose enough

## Risk 2 — Complex Rust Types

**Problem:** Not every Rust type can automatically become a meaningful widget. `HashMap<String, Foo>`, `Box<dyn Trait>`, generics — these don't map cleanly.

**Mitigation:**
- Support a well-defined set of automatic types
- Provide custom widget overrides for unsupported types
- Compile-time errors with clear messages for unhandled types

```rust
// Bad error:
// expected expression

// Good error:
// `tui` cannot automatically generate a widget for field `config`
// of type `HashMap<String, String>`.
//
// Consider:
// #[tui(widget = "custom")]
```

## Risk 3 — Ugly Layouts for Complex CLIs

**Problem:** Automatic generation can produce poor layouts for CLIs with many fields, nested subcommands, or unusual structures.

**Mitigation:**
- Provide `section` grouping
- Provide `order` control
- Provide `label` overrides
- Provide `widget` overrides
- Provide layout configuration API
- Allow custom renderers for full control

## Risk 4 — Macro Complexity

**Problem:** Proc macros can become difficult to maintain, debug, and extend. Compile errors from macros are often cryptic.

**Mitigation:**
- Keep AST parsing separate from schema generation
- Keep schema generation separate from runtime
- Keep runtime separate from rendering
- Invest in clear compile-time diagnostics
- Test macros extensively with `trybuild`

## Risk 5 — Scope Creep

**Problem:** A project like this can easily become another TUI framework, losing focus on the core value proposition.

**Mitigation:**
- Keep the core promise: CLI struct → TUI
- Move advanced functionality behind optional modules
- Say "no" to features that don't serve the core use case
- The project is NOT a replacement for ratatui

## Risk 6 — argh Metadata Differences

**Problem:** argh has different metadata conventions than clap. The adapter must handle these differences.

**Mitigation:**
- Study argh's derive internals
- Build separate adapter crate
- Share core schema between adapters

## Risk 7 — Terminal Compatibility

**Problem:** Different terminals handle colors, Unicode, resize events, and keyboard input differently.

**Mitigation:**
- Use crossterm for cross-platform terminal handling
- Test on Windows Terminal, PowerShell, cmd, iTerm, Alacritty, Kitty
- Gracefully degrade for limited terminals
- SSH session compatibility

## Risk 8 — Performance with Large Structs

**Problem:** Structs with 100+ fields may cause slow rendering or navigation.

**Mitigation:**
- Lazy rendering (only render visible fields)
- O(1) field lookup via HashMap
- Pagination/scrolling for large forms
- Benchmark with 50, 100, 500 fields

## Risk 9 — Version Compatibility

**Problem:** Clap and argh may change their internal APIs between versions.

**Mitigation:**
- Pin compatible versions
- Test against multiple versions
- Keep adapter layer thin
- Use public APIs only

## Risk 10 — Macro Compile Time

**Problem:** Complex proc macros can slow down compilation.

**Mitigation:**
- Keep macro logic minimal
- Generate static structures, not dynamic code
- Profile macro expansion time
- Consider caching strategies

## Limitations

### Not Supported Initially

- Generic structs
- Tuple structs
- Unit structs
- Enums with data (only value enums)
- Nested structs beyond subcommands
- Async validation
- Custom terminal escape sequences

### Requires Manual Work

- Complex layouts beyond sections
- Custom keyboard shortcuts (beyond remapping)
- Custom themes beyond color overrides
- File picker dialogs
- Network requests in validation
- Multi-window TUIs

### Never Supported

- GUI generation
- Web UI generation
- Mobile UI generation
- IDE integration
- Code completion

## Known Issues

None yet — project is planned, not implemented.

## Testing Gaps

Will need extensive testing for:

- Macro expansion edge cases
- Terminal resize behavior
- Keyboard input on different platforms
- Color rendering in limited terminals
- Unicode handling
- Large form performance
- Subcommand navigation depth

## Mitigation Priority

1. Start with narrow scope (clap + ratatui + basic types)
2. Add features incrementally
3. Test each addition thoroughly
4. Keep the core schema clean
5. Never break the basic derive(Tui) API
