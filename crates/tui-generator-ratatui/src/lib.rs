use tui_generator_core::schema::TuiSchema;
use tui_generator_core::state::FormState;
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::event::Action;
use tui_generator_core::error::TuiError;

use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct RatatuiRenderer;

impl Default for RatatuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl RatatuiRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn run(schema: &TuiSchema) -> Result<FormState, TuiError> {
        let mut state = FormState::from_schema(schema);

        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        crossterm::execute!(
            stdout,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::cursor::Hide
        )?;

        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let result = run_loop(&mut terminal, &mut state, schema);

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        )?;

        result
    }
}

fn run_loop(
    terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    state: &mut FormState,
    schema: &TuiSchema,
) -> Result<FormState, TuiError> {
    loop {
        terminal.draw(|f| {
            let area = f.area();
            render(f, area, state, schema);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if state.editing {
                    handle_edit_key(key, state, schema);
                } else {
                    let action = key_to_action(key, state, schema);
                    match action {
                        Action::FocusNext => state.focus_next(schema.fields.len()),
                        Action::FocusPrev => state.focus_prev(schema.fields.len()),
                        Action::ToggleEdit => {
                            state.editing = true;
                            let field = &schema.fields[state.focused_index];
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
                        _ => {}
                    }
                }
            }
        }
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
    state: &FormState,
    schema: &TuiSchema,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, chunks[0], schema);
    render_form(f, chunks[1], state, schema);
    render_footer(f, chunks[2], state, schema);
}

fn render_header(f: &mut ratatui::Frame, area: Rect, schema: &TuiSchema) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Style, Color};

    let title = schema.name.clone();
    let desc = schema.description.as_deref().unwrap_or("");
    let text = format!("{}  {}", title, desc);
    let para = Paragraph::new(text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(" TUI Generator "));
    f.render_widget(para, area);
}

fn render_form(f: &mut ratatui::Frame, area: Rect, state: &FormState, schema: &TuiSchema) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Style, Color};

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Fields ")
        .style(Style::default());

    let inner = block.inner(area);
    f.render_widget(block, area);

    let field_height: usize = 3;
    let visible = (inner.height as usize) / field_height;
    let total = schema.fields.len();

    if state.focused_index >= state.scroll_offset + visible {
        // Use saturating sub to avoid underflow
        let _new_offset = state.focused_index.saturating_sub(visible - 1);
        // We don't mutate state here; scroll is handled by clamp in rendering
    }

    let scroll = state.scroll_offset.min(total.saturating_sub(1));

    for (vis_idx, field_idx) in (scroll..total).enumerate().take(visible) {
        let field = &schema.fields[field_idx];
        let y_offset = (vis_idx as u16) * field_height as u16;

        if y_offset + field_height as u16 > inner.height {
            break;
        }

        let is_focused = field_idx == state.focused_index;
        render_field(f, inner, y_offset, field, state, is_focused);
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
        let err_para = Paragraph::new(err_text).style(Style::default().fg(Color::Red));
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
) {
    use ratatui::widgets::{Paragraph};
    use ratatui::style::{Style, Color};

    let label_style = if is_focused {
        Style::default().fg(Color::Cyan).add_modifier(ratatui::style::Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    // Label
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

    // Widget
    let widget_area = Rect {
        x: parent.x,
        y: parent.y + y_offset + 1,
        width: parent.width,
        height: 1,
    };

    match field.widget {
        WidgetKind::Checkbox => render_checkbox(f, widget_area, field, state, is_focused),
        WidgetKind::NumberInput => render_number_input(f, widget_area, field, state, is_focused),
        WidgetKind::Select => render_select(f, widget_area, field, state, is_focused),
        _ => render_text_input(f, widget_area, field, state, is_focused),
    }
}

fn render_text_input(
    f: &mut ratatui::Frame,
    area: Rect,
    field: &tui_generator_core::schema::Field,
    state: &FormState,
    is_focused: bool,
) {
    use ratatui::widgets::{Paragraph, Block, Borders};
    use ratatui::style::{Style, Color};

    let value_str = match state.get_value(&field.name) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Path(p)) => p.to_string_lossy().to_string(),
        Some(Value::Integer(n)) => n.to_string(),
        Some(Value::Float(n)) => n.to_string(),
        _ => String::new(),
    };

    let display = if is_focused && state.editing {
        format!("{}█", value_str)
    } else if value_str.is_empty() {
        "(empty)".to_string()
    } else {
        value_str
    };

    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let para = Paragraph::new(display).block(block);
    f.render_widget(para, area);

    // Show cursor in edit mode
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
) {
    use ratatui::widgets::Paragraph;
    use ratatui::style::{Style, Color};

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
        Style::default().fg(Color::Cyan)
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
) {
    use ratatui::widgets::Paragraph;
    use ratatui::style::{Style, Color};

    let checked = match state.get_value(&field.name) {
        Some(Value::Bool(b)) => *b,
        _ => false,
    };

    let marker = if is_focused { "▸" } else { " " };
    let check = if checked { "✓" } else { " " };
    let text = format!("{} [{}] {}", marker, check, field.label);

    let style = if is_focused {
        Style::default().fg(Color::Cyan)
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
) {
    use ratatui::widgets::{Paragraph};
    use ratatui::style::{Style, Color};

    let current = match state.get_value(&field.name) {
        Some(Value::String(s)) => s.clone(),
        _ => String::new(),
    };

    let display = if is_focused && state.editing {
        // Show selection cursor
        let idx = state.select_index;
        field.options.iter().enumerate().map(|(i, opt)| {
            let marker = if i == idx { "▸ " } else { "  " };
            format!("{}{}", marker, opt)
        }).collect::<Vec<_>>().join("\n")
    } else {
        format!("▸ {}", if current.is_empty() { "(none)" } else { &current })
    };

    let style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    };

    let para = Paragraph::new(display).style(style);
    f.render_widget(para, area);
}

fn render_footer(
    f: &mut ratatui::Frame,
    area: Rect,
    state: &FormState,
    schema: &TuiSchema,
) {
    use ratatui::widgets::{Block, Borders, Paragraph};
    use ratatui::style::{Style, Color};

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

    let text = format!("{}{}", hints, error_indicator);
    let style = if !state.errors.is_empty() {
        Style::default().fg(Color::Red)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let para = Paragraph::new(text)
        .style(style)
        .block(Block::default().borders(Borders::ALL).title(" Help "));
    f.render_widget(para, area);
}
