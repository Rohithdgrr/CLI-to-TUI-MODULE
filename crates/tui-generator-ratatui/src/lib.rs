use tui_generator_core::schema::TuiSchema;
use tui_generator_core::state::FormState;
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::event::Action;
use tui_generator_core::error::TuiError;

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Color, Modifier};

pub struct Theme {
    pub primary: Color,
    pub text: Color,
    pub border_focused: Color,
    pub border_unfocused: Color,
    pub error: Color,
    pub success: Color,
    pub section: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            primary: Color::Cyan,
            text: Color::White,
            border_focused: Color::Cyan,
            border_unfocused: Color::DarkGray,
            error: Color::Red,
            success: Color::Green,
            section: Color::Yellow,
        }
    }
}

impl Theme {
    pub fn dark() -> Self {
        Self::default()
    }

    pub fn light() -> Self {
        Theme {
            primary: Color::Blue,
            text: Color::Black,
            border_focused: Color::Blue,
            border_unfocused: Color::Gray,
            error: Color::Red,
            success: Color::Green,
            section: Color::Magenta,
        }
    }

    pub fn monochrome() -> Self {
        Theme {
            primary: Color::White,
            text: Color::White,
            border_focused: Color::White,
            border_unfocused: Color::DarkGray,
            error: Color::White,
            success: Color::White,
            section: Color::White,
        }
    }
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
    }
}

pub struct RatatuiRenderer {
    theme: Theme,
    mouse_enabled: bool,
}

impl Default for RatatuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl RatatuiRenderer {
    pub fn new() -> Self {
        RatatuiRenderer {
            theme: Theme::default(),
            mouse_enabled: false,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_mouse(mut self, enabled: bool) -> Self {
        self.mouse_enabled = enabled;
        self
    }

    pub fn run(schema: &TuiSchema) -> Result<FormState, TuiError> {
        Self::new().run_tui(schema)
    }

    pub fn run_tui(&self, schema: &TuiSchema) -> Result<FormState, TuiError> {
        let mut state = FormState::from_schema(schema);

        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;

        if self.mouse_enabled {
            crossterm::execute!(stdout, crossterm::event::EnableMouseCapture)?;
        }

        let _guard = TerminalGuard;

        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = self.run_loop(&mut terminal, &mut state, schema);

        if self.mouse_enabled {
            crossterm::execute!(
                terminal.backend_mut(),
                crossterm::event::DisableMouseCapture
            )?;
        }
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;

        result
    }
}

impl RatatuiRenderer {
    fn run_loop(
        &self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        state: &mut FormState,
        schema: &TuiSchema,
    ) -> Result<FormState, TuiError> {
        loop {
            terminal.draw(|f| {
                let area = f.area();
                render(f, area, state, schema, &self.theme);
            })?;

            if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                match crossterm::event::read()? {
                    crossterm::event::Event::Key(key) => {
                        if state.help_visible {
                            if key.code == crossterm::event::KeyCode::Esc
                                || key.code == crossterm::event::KeyCode::F(1)
                            {
                                state.help_visible = false;
                            }
                            continue;
                        }
                        if state.editing {
                            handle_edit_key(key, state, schema);
                        } else if state.focused_index < schema.fields.len()
                            && schema.fields[state.focused_index].widget
                                == WidgetKind::MultiSelect
                        {
                            match key.code {
                                crossterm::event::KeyCode::Enter => {
                                    handle_multi_select_toggle(state, schema);
                                }
                                crossterm::event::KeyCode::Up
                                | crossterm::event::KeyCode::Char('k') => {
                                    let field = &schema.fields[state.focused_index];
                                    if state.select_index > 0 && !field.options.is_empty() {
                                        state.select_index -= 1;
                                    }
                                }
                                crossterm::event::KeyCode::Down
                                | crossterm::event::KeyCode::Char('j') => {
                                    let field = &schema.fields[state.focused_index];
                                    if state.select_index + 1 < field.options.len() {
                                        state.select_index += 1;
                                    }
                                }
                                crossterm::event::KeyCode::Esc
                                | crossterm::event::KeyCode::Char('q') => {
                                    state.select_index = 0;
                                }
                                _ => {}
                            }
                        } else {
                            let action = key_to_action(key, state, schema);
                            match action {
                                Action::FocusNext => state.focus_next(schema),
                                Action::FocusPrev => state.focus_prev(schema),
                                Action::ToggleEdit => {
                                    let field = &schema.fields[state.focused_index];
                                    if field.readonly {
                                        continue;
                                    }
                                    state.editing = true;
                                    state.cursor_pos = state.edit_buffer(&field.name).len();
                                }
                                Action::ToggleValue => {
                                    if let Some(field) = schema.fields.get(state.focused_index) {
                                        if field.widget == WidgetKind::Checkbox {
                                            let current = state.get_value(&field.name).cloned();
                                            let new_val = match current {
                                                Some(Value::Bool(b)) => Value::Bool(!b),
                                                _ => Value::Bool(true),
                                            };
                                            state.set_value(&field.name, new_val);
                                        }
                                    }
                                }
                                Action::Cancel => return Err(TuiError::Cancelled),
                                Action::Submit => {
                                    state.validate(schema);
                                    if state.errors.is_empty() {
                                        return Ok(state.clone());
                                    }
                                }
                                Action::ShowHelp => state.help_visible = true,
                                _ => {}
                            }
                        }
                    }
                    crossterm::event::Event::Mouse(mouse) => {
                        if self.mouse_enabled {
                            handle_mouse(mouse, state, schema);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

fn handle_mouse(
    mouse: crossterm::event::MouseEvent,
    state: &mut FormState,
    schema: &TuiSchema,
) {
    match mouse.kind {
        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let row = mouse.row;
            let field_height: u16 = 3;
            let clicked_row = row.saturating_sub(4);
            let field_idx = (clicked_row / field_height) as usize + state.scroll_offset;
            if field_idx < schema.fields.len() && !schema.fields[field_idx].skip {
                state.focused_index = field_idx;
                state.editing = false;
            }
        }
        crossterm::event::MouseEventKind::ScrollUp => {
            state.focus_prev(schema);
        }
        crossterm::event::MouseEventKind::ScrollDown => {
            state.focus_next(schema);
        }
        _ => {}
    }
}

fn handle_edit_key(
    key: crossterm::event::KeyEvent,
    state: &mut FormState,
    schema: &TuiSchema,
) {
    let field = &schema.fields[state.focused_index];

    match field.widget {
        WidgetKind::Select => handle_select_key(key, state, field),
        WidgetKind::Checkbox => {
            // Checkbox doesn't need edit mode, but handle Esc
            if key.code == crossterm::event::KeyCode::Esc {
                state.editing = false;
                state.cursor_pos = 0;
            }
        }
        _ => handle_text_edit_key(key, state, field),
    }
}

fn handle_multi_select_toggle(state: &mut FormState, schema: &TuiSchema) {
    let field = &schema.fields[state.focused_index];
    let current_items: Vec<Value> = match state.get_value(&field.name) {
        Some(Value::List(items)) => items.clone(),
        _ => vec![],
    };

    if let Some(opt) = field.options.get(state.select_index) {
        let opt_val = Value::String(opt.clone());
        let mut new_items = current_items.clone();
        if let Some(pos) = new_items.iter().position(|v| v == &opt_val) {
            new_items.remove(pos);
        } else {
            new_items.push(opt_val);
        }
        state.set_value(&field.name, Value::List(new_items));
    }
}

fn handle_text_edit_key(
    key: crossterm::event::KeyEvent,
    state: &mut FormState,
    field: &tui_generator_core::schema::Field,
) {
    let mut buffer = state.edit_buffer(&field.name);

    match key.code {
        crossterm::event::KeyCode::Esc => {
            state.editing = false;
            state.cursor_pos = 0;
            return;
        }
        crossterm::event::KeyCode::Enter => {
            state.set_edit_buffer(&field.name, buffer, field.widget);
            state.editing = false;
            state.cursor_pos = 0;
            return;
        }
        crossterm::event::KeyCode::Char(c) => {
            if state.cursor_pos <= buffer.len() {
                buffer.insert(state.cursor_pos, c);
                state.cursor_pos += 1;
            }
        }
        crossterm::event::KeyCode::Backspace => {
            if state.cursor_pos > 0 {
                state.cursor_pos -= 1;
                buffer.remove(state.cursor_pos);
            }
        }
        crossterm::event::KeyCode::Delete => {
            if state.cursor_pos < buffer.len() {
                buffer.remove(state.cursor_pos);
            }
        }
        crossterm::event::KeyCode::Left => {
            state.cursor_pos = state.cursor_pos.saturating_sub(1);
        }
        crossterm::event::KeyCode::Right => {
            if state.cursor_pos < buffer.len() {
                state.cursor_pos += 1;
            }
        }
        crossterm::event::KeyCode::Home => {
            state.cursor_pos = 0;
        }
        crossterm::event::KeyCode::End => {
            state.cursor_pos = buffer.len();
        }
        _ => return,
    }

    state.set_edit_buffer(&field.name, buffer, field.widget);
}

fn handle_select_key(
    key: crossterm::event::KeyEvent,
    state: &mut FormState,
    field: &tui_generator_core::schema::Field,
) {
    match key.code {
        crossterm::event::KeyCode::Esc => {
            state.editing = false;
            state.select_index = 0;
        }
        crossterm::event::KeyCode::Enter => {
            if let Some(opt) = field.options.get(state.select_index) {
                state.set_value(&field.name, Value::String(opt.clone()));
            }
            state.editing = false;
            state.select_index = 0;
        }
        crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
            state.select_index = state.select_index.saturating_sub(1);
        }
        crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j')
            if state.select_index + 1 < field.options.len() =>
        {
            state.select_index += 1;
        }
        _ => {}
    }
}

fn key_to_action(key: crossterm::event::KeyEvent, state: &FormState, schema: &TuiSchema) -> Action {
    let field = &schema.fields[state.focused_index];

    if state.editing {
        match key.code {
            crossterm::event::KeyCode::Esc => Action::CancelEdit,
            crossterm::event::KeyCode::Enter => Action::ConfirmEdit,
            _ => Action::None,
        }
    } else {
        match key.code {
            crossterm::event::KeyCode::Tab | crossterm::event::KeyCode::Down => Action::FocusNext,
            crossterm::event::KeyCode::BackTab | crossterm::event::KeyCode::Up => Action::FocusPrev,
            crossterm::event::KeyCode::Enter => {
                if field.widget == WidgetKind::Checkbox {
                    Action::ToggleValue
                } else {
                    Action::ToggleEdit
                }
            }
            crossterm::event::KeyCode::Char(' ') => {
                if field.widget == WidgetKind::Checkbox {
                    Action::ToggleValue
                } else {
                    Action::None
                }
            }
            crossterm::event::KeyCode::Esc => Action::Cancel,
            crossterm::event::KeyCode::Char('q') => Action::Cancel,
            crossterm::event::KeyCode::Char('j') => Action::FocusNext,
            crossterm::event::KeyCode::Char('k') => Action::FocusPrev,
            crossterm::event::KeyCode::F(1) => Action::ShowHelp,
            _ => Action::None,
        }
    }
}

fn render(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &mut FormState,
    schema: &TuiSchema,
    theme: &Theme,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, chunks[0], schema, theme);
    render_form(f, chunks[1], state, schema, theme);
    render_footer(f, chunks[2], state, schema, theme);

    if state.help_visible {
        render_help_popup(f, area, theme);
    }
}

fn render_header(f: &mut ratatui::Frame, area: Rect, schema: &TuiSchema, theme: &Theme) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let title = schema.name.clone();
    let desc = schema.description.as_deref().unwrap_or("");
    let text = format!("{}  {}", title, desc);

    // Pake-style: dark header background with orange accent line (hardcoded orange #FF9F1C)
    let orange = Color::Rgb(255, 159, 28);
    let top_bar = Paragraph::new(" ")
        .style(Style::default().bg(orange));
    let top_bar_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    f.render_widget(top_bar, top_bar_area);

    let para = Paragraph::new(text)
        .style(Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" TUI Generator "));
    f.render_widget(para, area);
}

fn render_form(f: &mut ratatui::Frame, area: Rect, state: &mut FormState, schema: &TuiSchema, theme: &Theme) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Fields ")
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let field_height: usize = 3;
    let visible = (inner.height as usize) / field_height;
    let total = schema.fields.len();

    if visible > 0 {
        if state.focused_index < state.scroll_offset {
            state.scroll_offset = state.focused_index;
        } else if state.focused_index >= state.scroll_offset + visible {
            state.scroll_offset = state.focused_index.saturating_sub(visible - 1);
        }
    }

    let scroll = state.scroll_offset.min(total.saturating_sub(1));

    let mut current_section: Option<String> = None;
    let mut y_used: u16 = 0;

    for field_idx in scroll..total {
        let field = &schema.fields[field_idx];

        if let Some(ref section_name) = field.section {
            if current_section.as_ref() != Some(section_name) {
                current_section = Some(section_name.clone());
                if y_used < inner.height {
                    let section_para = Paragraph::new(format!("── {} ──", section_name))
                        .style(Style::default().fg(theme.section).add_modifier(Modifier::BOLD));
                    let section_area = Rect {
                        x: inner.x,
                        y: inner.y + y_used,
                        width: inner.width,
                        height: 1,
                    };
                    f.render_widget(section_para, section_area);
                    y_used += 1;
                }
            }
        }

        let remaining = (inner.height - y_used) as usize;
        if remaining < field_height {
            break;
        }

        let is_focused = field_idx == state.focused_index;
        render_field(f, inner, y_used, field, state, is_focused, theme);
        y_used += field_height as u16;
    }

    // Render validation errors at bottom
    if !state.errors.is_empty() {
        let err_lines: Vec<String> = state.errors.iter().map(|e| format!("✗ {}", e.message)).collect();
        let err_text = err_lines.join("\n");
        let err_area = Rect {
            x: inner.x,
            y: inner.y + inner.height.saturating_sub(2),
            width: inner.width,
            height: 2.min(inner.height),
        };
        let err_para = Paragraph::new(err_text).style(Style::default().fg(theme.error));
        f.render_widget(err_para, err_area);
    }
}

fn render_field(
    f: &mut ratatui::Frame,
    parent: Rect,
    y_offset: u16,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::{Paragraph};

    let label_style = if is_focused {
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    };

    let required_mark = if field.required { " *" } else { "" };
    let label_text = format!("{}{}", field.label, required_mark);
    let label = Paragraph::new(label_text).style(label_style);
    let label_area = Rect {
        x: parent.x,
        y: parent.y + y_offset,
        width: parent.width,
        height: 1,
    };
    f.render_widget(label, label_area);

    let widget_area = Rect {
        x: parent.x,
        y: parent.y + y_offset + 1,
        width: parent.width,
        height: 1,
    };

    match field.widget {
        WidgetKind::Checkbox => render_checkbox(f, widget_area, field, state, is_focused, theme),
        WidgetKind::NumberInput => render_number_input(f, widget_area, field, state, is_focused, theme),
        WidgetKind::Select => render_select(f, widget_area, field, state, is_focused, theme),
        WidgetKind::MultiSelect => render_multi_select(f, widget_area, field, state, is_focused, theme),
        _ => render_text_input(f, widget_area, field, state, is_focused, theme),
    }
}

fn render_text_input(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::{Paragraph, Block, Borders};

    let value_str = match state.get_value(&field.name) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Path(p)) => p.to_string_lossy().to_string(),
        Some(Value::Integer(n)) => n.to_string(),
        Some(Value::Float(n)) => n.to_string(),
        _ => String::new(),
    };

    let is_password = field.widget == WidgetKind::PasswordInput;

    let display = if is_password && !is_focused {
        "•".repeat(value_str.len()).to_string()
    } else if is_focused && state.editing {
        format!("{}█", value_str)
    } else if value_str.is_empty() {
        "(empty)".to_string()
    } else if is_password {
        format!("{}█", "•".repeat(value_str.len()))
    } else {
        value_str
    };

    let border_style = if is_focused {
        Style::default().fg(theme.border_focused)
    } else {
        Style::default().fg(theme.border_unfocused)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let para = Paragraph::new(display).block(block);
    f.render_widget(para, area);

    if is_focused && state.editing {
        let cursor_x = area.x + 1 + (state.cursor_pos as u16).min(area.width.saturating_sub(2));
        f.set_cursor_position((cursor_x, area.y + 1));
    }
}

fn render_number_input(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::Paragraph;

    let value_str = match state.get_value(&field.name) {
        Some(Value::Integer(n)) => n.to_string(),
        Some(Value::Float(n)) => n.to_string(),
        _ => String::new(),
    };

    let display = if is_focused && state.editing {
        format!("[ {}█ ]", value_str)
    } else {
        format!("[ {} ]", if value_str.is_empty() { "0" } else { &value_str })
    };

    let style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default()
    };

    let para = Paragraph::new(display).style(style);
    f.render_widget(para, area);

    if is_focused && state.editing {
        let bracket_offset = 3 + (state.cursor_pos as u16).min(value_str.len() as u16);
        f.set_cursor_position((area.x + bracket_offset, area.y));
    }
}

fn render_checkbox(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::Paragraph;

    let checked = match state.get_value(&field.name) {
        Some(Value::Bool(b)) => *b,
        _ => false,
    };

    let marker = if is_focused { "▸" } else { " " };
    let check = if checked { "✓" } else { " " };
    let text = format!("{} [{}] {}", marker, check, field.label);

    let style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default()
    };

    let para = Paragraph::new(text).style(style);
    f.render_widget(para, area);
}

fn render_select(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::{Paragraph};

    let current = match state.get_value(&field.name) {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    let display = if is_focused && state.editing {
        let idx = state.select_index;
        field.options.iter().enumerate().map(|(i, opt)| {
            let marker = if i == idx { "▸ " } else { "  " };
            format!("{}{}", marker, opt)
        }).collect::<Vec<_>>().join("\n")
    } else {
        format!("▸ {}", if current.is_empty() { "(none)" } else { &current })
    };

    let style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default()
    };

    let para = Paragraph::new(display).style(style);
    f.render_widget(para, area);
}

fn render_multi_select(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
    theme: &Theme,
) {
    use ratatui::widgets::Paragraph;

    let current: Vec<String> = match state.get_value(&field.name) {
        Some(Value::List(items)) => items.iter()
            .filter_map(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        Some(Value::String(s)) => vec![s.clone()],
        _ => vec![],
    };

    let display = if current.is_empty() {
        "(none selected)".to_string()
    } else if current.len() <= 3 {
        current.join(", ")
    } else {
        format!("{} selected: {}", current.len(), current[..3].join(", "))
    };

    let style = if is_focused {
        Style::default().fg(theme.primary)
    } else {
        Style::default()
    };

    let para = Paragraph::new(display).style(style);
    f.render_widget(para, area);
}

fn build_cli_preview(schema: &TuiSchema, state: &FormState) -> String {
    let mut args = Vec::new();
    for field in &schema.fields {
        if field.skip {
            continue;
        }
        if let Some(val) = state.get_value(&field.name) {
            let arg_str = match val {
                Value::String(s) => format!("--{} \"{}\"", field.name, s),
                Value::Integer(n) => format!("--{} {}", field.name, n),
                Value::Float(f) => format!("--{} {}", field.name, f),
                Value::Bool(b) => {
                    if *b { format!("--{}", field.name) } else { String::new() }
                }
                Value::Path(p) => format!("--{} \"{}\"", field.name, p.to_string_lossy()),
                Value::List(_) | Value::None => String::new(),
            };
            if !arg_str.is_empty() {
                args.push(arg_str);
            }
        }
    }
    if args.is_empty() {
        "(no args set)".to_string()
    } else {
        format!("CLI: {}", args.join(" "))
    }
}

fn render_footer(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &FormState,
    schema: &TuiSchema,
    theme: &Theme,
) {
    use ratatui::widgets::{Block, Borders, Paragraph};

    let field = &schema.fields[state.focused_index];

    let hints = if state.editing {
        match field.widget {
            WidgetKind::Select => "↑↓ Select  Enter Confirm  Esc Cancel",
            _ => "Enter Confirm  Esc Cancel",
        }
    } else {
        "↑↓ Navigate  Enter Edit  Space Toggle  q Quit  F1 Help"
    };

    let error_indicator = if !state.errors.is_empty() {
        format!("  ✗ {} errors", state.errors.len())
    } else {
        String::new()
    };

    let preview_raw = build_cli_preview(schema, state);
    let preview = if preview_raw.len() > 60 {
        format!("{}...", &preview_raw[..57])
    } else {
        preview_raw
    };
    let text = format!("{}    {}{}", hints, error_indicator, preview);
    let style = if !state.errors.is_empty() {
        Style::default().fg(theme.error)
    } else {
        Style::default().fg(theme.border_unfocused)
    };

    let para = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(para, area);
}

fn render_help_popup(f: &mut ratatui::Frame, area: Rect, theme: &Theme) {
    use ratatui::widgets::{Block, Borders, Paragraph, Clear};

    let w = 50.min(area.width);
    let h = 14.min(area.height);
    let x = (area.width - w) / 2;
    let y = (area.height - h) / 2;

    let popup_area = Rect { x, y, width: w, height: h };

    f.render_widget(Clear, popup_area);

    let help_text = "\
Keyboard Shortcuts

↑↓ / j/k     Navigate fields
Enter        Edit field / Confirm
Space        Toggle checkbox
Esc          Cancel / Quit
q            Quit
Tab          Next field
Shift+Tab    Previous field
F1           Toggle this help

Mouse (if enabled)
Click        Focus field
Scroll       Navigate fields

Press Esc or F1 to close";

    let para = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary))
                .title(" Help (F1/Esc to close) "),
        );

    f.render_widget(para, popup_area);
}
