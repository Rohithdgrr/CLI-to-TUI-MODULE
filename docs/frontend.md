# Frontend Architecture

> Terminal rendering, ratatui layer, layout system, and widget implementations.

## Rendering Pipeline

```
TuiState
    ↓
Calculate layout (terminal size, field count)
    ↓
Render header
    ↓
Render form area
    ├── Render sections
    │   └── Render fields
    │       ├── Render label
    │       ├── Render widget
    │       └── Render validation error
    ├── Render help panel (if visible)
    └── Render command preview (if active)
    ↓
Render footer (keyboard hints)
    ↓
Render overlays (help, confirmation, error dialog)
```

## Terminal Backend

Uses `crossterm` for:

- Raw mode
- Alternate screen
- Cursor visibility
- Keyboard events
- Mouse events (optional)
- Terminal resize detection
- Color/style rendering

## Layout System

### Automatic Layout

Layout calculated from:

- Terminal width/height
- Number of fields
- Field types (text inputs need more space)
- Section grouping
- Help panel visibility

### Layout Modes

**Wide terminal (>120 columns):**

```
┌──────────────────────────────────────────────────────┐
│ Header                                               │
├──────────────────────────────┬───────────────────────┤
│ Form                         │ Help / Preview        │
│                              │                       │
│ Input                        │ Description of        │
│ [........................]   │ focused field         │
│                              │                       │
│ Output                       │                       │
│ [........................]   │                       │
│                              │                       │
│ Threads                      │                       │
│ [4]                          │                       │
├──────────────────────────────┴───────────────────────┤
│ ↑↓ Navigate   Enter Edit   F1 Help   Ctrl+Enter Run │
└──────────────────────────────────────────────────────┘
```

**Medium terminal (80-120 columns):**

```
┌────────────────────────────────────────┐
│ Header                                 │
├────────────────────────────────────────┤
│ Input                                  │
│ [........................]             │
│                                        │
│ Output                                 │
│ [........................]             │
│                                        │
│ Threads                                │
│ [4]                                    │
├────────────────────────────────────────┤
│ Description of focused field           │
├────────────────────────────────────────┤
│ ↑↓ Navigate   Enter Edit   F1 Help    │
└────────────────────────────────────────┘
```

**Small terminal (<80 columns):**

```
┌──────────────────────┐
│ Header               │
├──────────────────────┤
│ Input                │
│ [................]   │
├──────────────────────┤
│ ↓ Next   Enter Edit  │
└──────────────────────┘
```

### Minimum Size

```
MIN_WIDTH = 40
MIN_HEIGHT = 10
```

Below minimum, show:

```
Terminal too small.
Minimum: 40x10
Current: 30x8
```

## Widget Rendering

### TextInput

```
Label
┌──────────────────────────────────────┐
│ placeholder text or current value    │
└──────────────────────────────────────┘
```

Focused:

```
Label
┌──────────────────────────────────────┐
│ current value█                       │
└──────────────────────────────────────┘
```

### PasswordInput

```
Label
┌──────────────────────────────────────┐
│ ****************                      │
└──────────────────────────────────────┘
```

### NumberInput

```
Label
[ 4 ]
```

With increment/decrement:

```
Label
[-] 4 [+]
```

### Checkbox

```
Label
[✓] Enable option
```

or

```
Label
[ ] Enable option
```

### Select

```
Label

> Option A
  Option B
  Option C
```

### MultiSelect

```
Label

[✓] Option A
[ ] Option B
[✓] Option C
```

### PathInput

```
Label
┌──────────────────────────────────────┐
│ /home/user/file.txt                  │
└──────────────────────────────────────┘

Tab completion available
```

### Confirm

```
Message?

[Y] Yes   [N] No
```

## Focus Management

### Focus Indicator

Focused field highlighted:

```
Input
▸ ┌────────────────────────────────────┐
  │ value                              │
  └────────────────────────────────────┘
```

or with color:

```
Input
┌──────────────────────────────────────┐  ← border color: accent
│ value█                               │
└──────────────────────────────────────┘
```

### Focus Cycling

```
Field 0 → Field 1 → ... → Field N → Field 0
```

With sections:

```
Section 1, Field 0
Section 1, Field 1
Section 2, Field 0
...
```

### Auto-Scroll

When focused field is below visible area:

```
scroll_offset = focused_field - visible_fields + 1
```

When focused field is above visible area:

```
scroll_offset = focused_field
```

## Navigation

### Vertical Navigation

```
↑ / Shift+Tab → previous field
↓ / Tab → next field
```

### Page Navigation

```
Page Up → previous section
Page Down → next section
```

### Home/End

```
Home → first field
End → last field
```

## Keyboard Handling

### Edit Mode

When a field is focused and user presses Enter:

```
Normal mode → Edit mode
```

Edit mode:

```
Input
┌──────────────────────────────────────┐
│ current value█                       │  ← cursor visible
└──────────────────────────────────────┘

Enter: confirm edit
Esc: cancel edit
```

### Toggle Mode

For checkboxes and multi-select:

```
Space → toggle current item
```

### Select Mode

For select widgets:

```
↑/↓ → navigate options
Enter → confirm selection
Esc → cancel selection
```

## Scrolling

### Vertical Scroll

```rust
let visible_start = scroll_offset;
let visible_end = scroll_offset + visible_height;

if focused_field >= visible_end {
    scroll_offset = focused_field - visible_height + 1;
}
if focused_field < visible_start {
    scroll_offset = focused_field;
}
```

### Scroll Indicator

```
Field 5 / 42
```

or

```
[████████░░░░] 5/42
```

## Section Rendering

```
NETWORK
────────────────────────────
Host
[........................]

Port
[8080]

SECURITY
────────────────────────────
TLS
[✓] Enable TLS
```

## Help Panel

Toggle with F1:

```
┌─ Help ────────────────────────────────┐
│                                       │
│ Input                                  │
│ Path to the input file.               │
│                                       │
│ Short: -i                             │
│ Long:  --input                        │
│ Type:  PathBuf                        │
│ Required: yes                         │
│                                       │
│ ↑↓ Navigate   Esc Close              │
└───────────────────────────────────────┘
```

## Error Display

### Inline Errors

```
Input *
┌──────────────────────────────────────┐
│                                      │
└──────────────────────────────────────┘
✗ Input is required
```

### Error Dialog

```
┌─ Validation Errors ──────────────────┐
│                                      │
│ ✗ Input is required                  │
│ ✗ Threads must be 1-64              │
│                                      │
│         [ Back to Edit ]             │
└──────────────────────────────────────┘
```

## Command Preview

Before submission:

```
┌─ Command Preview ────────────────────┐
│                                      │
│ myapp                                │
│   --input ./data.json                │
│   --output ./result.json             │
│   --threads 8                        │
│   --verbose                          │
│                                      │
│ [ Run ]     [ Edit ]     [ Cancel ]  │
└──────────────────────────────────────┘
```

## Themes

### Default Theme

```rust
Theme {
    background: Color::Black,
    foreground: Color::White,
    accent: Color::Cyan,
    error: Color::Red,
    success: Color::Green,
    border: Color::DarkGray,
}
```

### Dark Theme

```rust
Theme {
    background: Color::Rgb(30, 30, 30),
    foreground: Color::Rgb(220, 220, 220),
    accent: Color::Rgb(100, 149, 237),
    error: Color::Rgb(255, 85, 85),
    success: Color::Rgb(80, 250, 123),
    border: Color::Rgb(80, 80, 80),
}
```

### Light Theme

```rust
Theme {
    background: Color::White,
    foreground: Color::Black,
    accent: Color::Blue,
    error: Color::Red,
    success: Color::Green,
    border: Color::Gray,
}
```

### Monochrome

```rust
Theme {
    background: Color::Black,
    foreground: Color::White,
    accent: Color::White,
    error: Color::White,
    success: Color::White,
    border: Color::DarkGray,
}
```

### High Contrast

```rust
Theme {
    background: Color::Black,
    foreground: Color::White,
    accent: Color::Yellow,
    error: Color::Red,
    success: Color::Green,
    border: Color::White,
}
```

## Mouse Support (Optional)

When enabled:

- Click on field → focus it
- Click on checkbox → toggle
- Click on select option → select
- Scroll up → previous field
- Scroll down → next field

Keyboard remains primary input method.

## Performance

### Frame Rate

Target: 60 FPS where terminal supports it.

### Partial Rendering

Only re-render changed areas when practical:

```rust
if state.changed_fields.contains(&field.name) {
    render_field(area, frame, &field, &state);
}
```

### Lazy Rendering

For large forms, only render visible fields:

```rust
let visible_fields: Vec<_> = schema.fields
    .iter()
    .skip(scroll_offset)
    .take(visible_height)
    .collect();
```
