use std::time::{Duration, Instant};
use tui_generator_core::schema::{Field, TuiSchema};
use tui_generator_core::state::FormState;
use tui_generator_core::value::Value;
use tui_generator_core::widget::WidgetKind;
use tui_generator_core::event::Action;
use tui_generator_core::error::TuiError;

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Color, Modifier};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{
    Block, Paragraph, Clear, Padding
};
use ratatui::Frame;

// ============================================================================
// PAKE THEME — Matching exact pake aesthetic
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct PakeTheme {
    pub bg: Color,
    pub surface: Color,
    pub primary: Color,        // Cyan/teal (#4EC9B0 / #50FA7B)
    pub primary_dim: Color,    // Dim teal
    pub text: Color,           // Bright off-white
    pub text_muted: Color,     // Muted grey/blue
    pub text_dim: Color,       // Deep dim
    pub border_focused: Color, // Cyan
    pub border_unfocused: Color,
    pub error: Color,          // Soft red
    pub success: Color,        // Bright green (#50FA7B)
    pub warning: Color,
    pub accent: Color,         // Pake Pumpkin Orange (#FF9F1C)
    pub selection_bg: Color,
}

impl Default for PakeTheme {
    fn default() -> Self {
        Self::pake()
    }
}

impl PakeTheme {
    pub fn pake() -> Self {
        PakeTheme {
            bg: Color::Rgb(14, 16, 22),              // Deep terminal dark
            surface: Color::Rgb(22, 25, 35),         // Subtle panel
            primary: Color::Rgb(78, 201, 176),       // Cyan/teal #4EC9B0
            primary_dim: Color::Rgb(58, 145, 130),
            text: Color::Rgb(235, 240, 250),         // Off-white
            text_muted: Color::Rgb(140, 145, 165),   // Soft grey
            text_dim: Color::Rgb(85, 90, 110),
            border_focused: Color::Rgb(78, 201, 176),
            border_unfocused: Color::Rgb(45, 48, 62),
            error: Color::Rgb(255, 107, 107),
            success: Color::Rgb(80, 250, 123),
            warning: Color::Rgb(255, 200, 100),
            accent: Color::Rgb(255, 159, 28),        // Pake Orange #FF9F1C
            selection_bg: Color::Rgb(32, 45, 58),
        }
    }

    pub fn dark() -> Self {
        Self::pake()
    }
}

// ============================================================================
// TERMINAL GUARD
// ============================================================================

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

// ============================================================================
// RENDERER
// ============================================================================

pub struct RatatuiRenderer {
    theme: PakeTheme,
    mouse_enabled: bool,
    show_splash: bool,
    splash_duration_ms: u64,
}

impl Default for RatatuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl RatatuiRenderer {
    pub fn new() -> Self {
        RatatuiRenderer {
            theme: PakeTheme::pake(),
            mouse_enabled: false,
            show_splash: false,
            splash_duration_ms: 800,
        }
    }

    pub fn with_theme(mut self, theme: PakeTheme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_mouse(mut self, enabled: bool) -> Self {
        self.mouse_enabled = enabled;
        self
    }

    pub fn with_splash(mut self, show: bool) -> Self {
        self.show_splash = show;
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

        if self.show_splash {
            self.run_splash(&mut terminal, schema)?;
        }

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

    fn run_splash(
        &self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        schema: &TuiSchema,
    ) -> Result<(), TuiError> {
        let start = Instant::now();
        let duration = Duration::from_millis(self.splash_duration_ms);

        let logo_lines = get_ascii_art(&schema.name);

        while start.elapsed() < duration {
            let progress = (start.elapsed().as_millis() as f64 / duration.as_millis() as f64).min(1.0);
            terminal.draw(|f| {
                let area = f.size();
                f.render_widget(
                    Block::default().style(Style::default().bg(self.theme.bg)),
                    area,
                );

                let logo_text: Text = logo_lines.iter().map(|line| {
                    Line::from(Span::styled(
                        *line,
                        Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD),
                    ))
                }).collect();

                let logo_para = Paragraph::new(logo_text).alignment(Alignment::Center);
                let logo_height = logo_lines.len() as u16 + 4;
                let logo_area = Rect {
                    x: area.x,
                    y: area.y + (area.height.saturating_sub(logo_height)) / 2,
                    width: area.width,
                    height: logo_height,
                };
                f.render_widget(logo_para, logo_area);

                let subtitle = format!(
                    "  {}  —  Universal TUI  ",
                    schema.name
                );
                let sub_area = Rect {
                    x: area.x,
                    y: logo_area.y + logo_height,
                    width: area.width,
                    height: 1,
                };
                f.render_widget(
                    Paragraph::new(Span::styled(
                        subtitle,
                        Style::default().fg(self.theme.text_muted),
                    ))
                    .alignment(Alignment::Center),
                    sub_area,
                );

                let bar_width = 36u16.min(area.width.saturating_sub(10));
                let filled = (progress * bar_width as f64) as u16;
                let empty = bar_width.saturating_sub(filled);
                let bar = format!(
                    " [{}>{}]",
                    "█".repeat(filled as usize),
                    " ".repeat(empty as usize)
                );
                let bar_area = Rect {
                    x: area.x,
                    y: sub_area.y + 2,
                    width: area.width,
                    height: 1,
                };
                f.render_widget(
                    Paragraph::new(Span::styled(
                        bar,
                        Style::default().fg(self.theme.primary),
                    ))
                    .alignment(Alignment::Center),
                    bar_area,
                );
            })?;

            std::thread::sleep(Duration::from_millis(16));
        }

        Ok(())
    }

    fn run_loop(
        &self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
        state: &mut FormState,
        schema: &TuiSchema,
    ) -> Result<FormState, TuiError> {
        let mut last_tick = Instant::now();
        let tick_rate = Duration::from_millis(50);

        loop {
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            terminal.draw(|f| {
                let area = f.size();
                f.render_widget(
                    Block::default().style(Style::default().bg(self.theme.bg)),
                    area,
                );
                self.render_main(f, area, state, schema);
            })?;

            if crossterm::event::poll(timeout)? {
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
                            && schema.fields[state.focused_index].widget == WidgetKind::MultiSelect
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
                                    if state.focused_index < schema.fields.len() {
                                        let field = &schema.fields[state.focused_index];
                                        if !field.readonly {
                                            state.editing = true;
                                            state.cursor_pos = state.edit_buffer(&field.name).len();
                                        }
                                    }
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
                                        self.render_success(terminal)?;
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

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }
    }

    fn render_success(
        &self,
        terminal: &mut ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), TuiError> {
        let start = Instant::now();
        let duration = Duration::from_millis(500);

        while start.elapsed() < duration {
            terminal.draw(|f| {
                let area = f.size();
                f.render_widget(
                    Block::default().style(Style::default().bg(self.theme.bg)),
                    area,
                );

                let msg = "✓ Done!";
                let para = Paragraph::new(Span::styled(
                    msg,
                    Style::default()
                        .fg(self.theme.success)
                        .add_modifier(Modifier::BOLD),
                ))
                .alignment(Alignment::Center);
                let msg_area = Rect {
                    x: area.x,
                    y: area.y + area.height / 2,
                    width: area.width,
                    height: 1,
                };
                f.render_widget(para, msg_area);
            })?;
            std::thread::sleep(Duration::from_millis(16));
        }
        Ok(())
    }

    fn render_main(
        &self,
        f: &mut Frame,
        area: Rect,
        state: &mut FormState,
        schema: &TuiSchema,
    ) {
        // Responsive Layout:
        // Compact header if terminal height < 18 lines
        let header_height: u16 = if area.height < 18 { 3 } else { 9 };
        let footer_height: u16 = 3;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(header_height),
                Constraint::Min(4),
                Constraint::Length(footer_height),
            ])
            .split(area);

        self.render_pake_header(f, chunks[0], schema);
        self.render_pake_form(f, chunks[1], state, schema);
        self.render_pake_footer(f, chunks[2], state, schema);

        if state.help_visible {
            self.render_help_popup(f, area);
        }
    }

    fn render_pake_header(&self, f: &mut Frame, area: Rect, schema: &TuiSchema) {
        let app_name = schema.name.to_lowercase().replace(' ', "-");
        let repo_url = "https://github.com/Rohithdgrr/CLI-to-TUI-MODULE";
        let tagline = schema
            .description
            .clone()
            .unwrap_or_else(|| "can turn any CLI into a TUI with Rust.".to_string());

        let mut lines = Vec::new();

        // 1. Top prompt: ~ 🎃 pake
        lines.push(Line::from(vec![
            Span::styled("~ ", Style::default().fg(self.theme.primary)),
            Span::styled("🎃 ", Style::default().fg(self.theme.accent)),
            Span::styled(&app_name, Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD)),
        ]));

        // Compact view for smaller terminals
        if area.height < 6 {
            lines.push(Line::from(vec![
                Span::styled("Usage: ", Style::default().fg(self.theme.text).add_modifier(Modifier::BOLD)),
                Span::styled(&app_name, Style::default().fg(self.theme.text_muted)),
                Span::styled(" [options]", Style::default().fg(self.theme.text_dim)),
            ]));
            let para = Paragraph::new(lines).block(Block::default().padding(Padding::horizontal(2)));
            f.render_widget(para, area);
            return;
        }

        let ascii_logo = get_ascii_art(&schema.name);

        // 2-5. ASCII Logo + side info (URL & Tagline)
        for (i, art_line) in ascii_logo.iter().enumerate() {
            let mut spans = vec![
                Span::styled(
                    format!("{:<26}", art_line),
                    Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD),
                ),
            ];

            if i == 2 {
                spans.push(Span::styled(format!("  {}", repo_url), Style::default().fg(self.theme.accent)));
            } else if i == 3 {
                spans.push(Span::styled(format!("  {}", tagline), Style::default().fg(self.theme.primary)));
            }

            lines.push(Line::from(spans));
        }

        // Blank separator
        lines.push(Line::from(""));

        // 6. Usage line
        lines.push(Line::from(vec![
            Span::styled("Usage: ", Style::default().fg(self.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled(&app_name, Style::default().fg(self.theme.text_muted)),
            Span::styled(" [options]", Style::default().fg(self.theme.text_dim)),
        ]));

        let para = Paragraph::new(lines).block(Block::default().padding(Padding::horizontal(2)));
        f.render_widget(para, area);
    }

    fn render_pake_form(
        &self,
        f: &mut Frame,
        area: Rect,
        state: &mut FormState,
        schema: &TuiSchema,
    ) {
        let block = Block::default().padding(Padding::horizontal(2));
        let inner = block.inner(area);
        f.render_widget(block, area);

        let total = schema.fields.len();
        if total == 0 {
            return;
        }

        // Each field takes 1 clean line in pake style
        let visible_lines = inner.height as usize;
        if visible_lines > 0 {
            if state.focused_index < state.scroll_offset {
                state.scroll_offset = state.focused_index;
            } else if state.focused_index >= state.scroll_offset + visible_lines {
                state.scroll_offset = state.focused_index.saturating_sub(visible_lines - 1);
            }
        }

        let scroll = state.scroll_offset.min(total.saturating_sub(1));
        let mut y_used: u16 = 0;
        let mut current_section: Option<String> = None;

        // Section header label (Options:)
        let first_section = schema.fields.first().and_then(|f| f.section.clone()).unwrap_or_else(|| "Options".to_string());
        
        for field_idx in scroll..total {
            if y_used >= inner.height {
                break;
            }

            let field = &schema.fields[field_idx];

            // Section Header
            let field_sec = field.section.clone().unwrap_or_else(|| first_section.clone());
            if current_section.as_ref() != Some(&field_sec) {
                current_section = Some(field_sec.clone());
                if y_used + 1 < inner.height {
                    let sec_line = Line::from(vec![
                        Span::styled(
                            format!("{}:", field_sec),
                            Style::default()
                                .fg(self.theme.text)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]);
                    f.render_widget(
                        Paragraph::new(sec_line),
                        Rect {
                            x: inner.x,
                            y: inner.y + y_used,
                            width: inner.width,
                            height: 1,
                        },
                    );
                    y_used += 1;
                }
            }

            if y_used >= inner.height {
                break;
            }

            let is_focused = field_idx == state.focused_index;
            self.render_pake_field_line(
                f,
                Rect {
                    x: inner.x,
                    y: inner.y + y_used,
                    width: inner.width,
                    height: 1,
                },
                field,
                state,
                is_focused,
            );
            y_used += 1;
        }

        // Validation errors at bottom if any
        if !state.errors.is_empty() && y_used + 1 <= inner.height {
            let err_line = Line::from(vec![
                Span::styled("✗ ", Style::default().fg(self.theme.error).add_modifier(Modifier::BOLD)),
                Span::styled(&state.errors[0].message, Style::default().fg(self.theme.error)),
            ]);
            f.render_widget(
                Paragraph::new(err_line),
                Rect {
                    x: inner.x,
                    y: inner.y + inner.height.saturating_sub(1),
                    width: inner.width,
                    height: 1,
                },
            );
        }
    }

    fn render_pake_field_line(
        &self,
        f: &mut Frame,
        area: Rect,
        field: &Field,
        state: &FormState,
        is_focused: bool,
    ) {
        let type_tag = match field.widget {
            WidgetKind::TextInput | WidgetKind::TextArea => "<string>",
            WidgetKind::PasswordInput => "<password>",
            WidgetKind::PathInput | WidgetKind::FileInput => "<path>",
            WidgetKind::DirectoryInput => "<dir>",
            WidgetKind::NumberInput => "<number>",
            WidgetKind::Checkbox | WidgetKind::Confirm => "",
            WidgetKind::Select => "<select>",
            WidgetKind::MultiSelect => "<files>",
        };

        let opt_name = format!("--{}", field.name);
        let flag_col = if type_tag.is_empty() {
            format!("{:<22}", opt_name)
        } else {
            format!("{:<14} {:<7}", opt_name, type_tag)
        };

        // Formatted value string
        let val_display = match state.get_value(&field.name) {
            Some(Value::String(s)) => {
                if field.widget == WidgetKind::PasswordInput && !is_focused {
                    "•".repeat(s.len())
                } else {
                    s.clone()
                }
            }
            Some(Value::Path(p)) => p.to_string_lossy().to_string(),
            Some(Value::Integer(n)) => n.to_string(),
            Some(Value::Float(n)) => n.to_string(),
            Some(Value::Bool(b)) => if *b { "✓ true".to_string() } else { "false".to_string() },
            Some(Value::List(l)) => {
                let items: Vec<String> = l.iter().map(|v| match v {
                    Value::String(s) => s.clone(),
                    _ => "".to_string(),
                }).collect();
                items.join(", ")
            }
            _ => String::new(),
        };

        let desc = field.description.as_deref().unwrap_or(&field.label);

        let mut spans = Vec::new();

        if is_focused {
            // Focused Indicator: ▸
            spans.push(Span::styled("▸ ", Style::default().fg(self.theme.accent).add_modifier(Modifier::BOLD)));
            // Focused Flag in bold teal
            spans.push(Span::styled(
                flag_col,
                Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD),
            ));

            // Interactive Editor pill or value
            if state.editing {
                let edit_buf = state.edit_buffer(&field.name);
                spans.push(Span::styled(" [ ", Style::default().fg(self.theme.accent)));
                spans.push(Span::styled(
                    format!("{}_", edit_buf),
                    Style::default().fg(self.theme.text).add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::styled(" ] ", Style::default().fg(self.theme.accent)));
            } else if !val_display.is_empty() {
                spans.push(Span::styled(" [ ", Style::default().fg(self.theme.border_focused)));
                spans.push(Span::styled(
                    &val_display,
                    Style::default().fg(self.theme.text),
                ));
                spans.push(Span::styled(" ] ", Style::default().fg(self.theme.border_focused)));
            } else {
                spans.push(Span::styled("   ", Style::default()));
            }

            // Description in off-white
            spans.push(Span::styled(
                format!("  {}", desc),
                Style::default().fg(self.theme.text_muted),
            ));
        } else {
            // Unfocused
            spans.push(Span::styled("  ", Style::default()));
            spans.push(Span::styled(
                flag_col,
                Style::default().fg(self.theme.text),
            ));

            if !val_display.is_empty() {
                let val_preview = if val_display.len() > 16 {
                    format!("{}…", &val_display[..15])
                } else {
                    val_display
                };
                spans.push(Span::styled(
                    format!(" [ {} ]", val_preview),
                    Style::default().fg(self.theme.text_dim),
                ));
            }

            spans.push(Span::styled(
                format!("  {}", desc),
                Style::default().fg(self.theme.text_dim),
            ));
        }

        let para = Paragraph::new(Line::from(spans));
        f.render_widget(para, area);
    }

    fn render_pake_footer(
        &self,
        f: &mut Frame,
        area: Rect,
        state: &FormState,
        schema: &TuiSchema,
    ) {
        let preview_raw = build_cli_preview(schema, state);
        let preview = if preview_raw.len() > 60 {
            format!("{}...", &preview_raw[..57])
        } else {
            preview_raw
        };

        let mut lines = Vec::new();

        // Line 1: Shortcuts bar
        let hints = if state.editing {
            "  Enter: Confirm   Esc: Cancel"
        } else {
            "  ↑↓: Navigate   Enter: Edit / Toggle   Space: Toggle   q: Quit   F1: Help"
        };
        lines.push(Line::from(Span::styled(hints, Style::default().fg(self.theme.text_dim))));

        // Line 2: Pake style command prompt preview: ~ 🎃 <command>
        lines.push(Line::from(vec![
            Span::styled("~ ", Style::default().fg(self.theme.primary)),
            Span::styled("🎃 ", Style::default().fg(self.theme.accent)),
            Span::styled(
                preview,
                Style::default().fg(self.theme.text).add_modifier(Modifier::BOLD),
            ),
            Span::styled(" █", Style::default().fg(self.theme.primary)),
        ]));

        let para = Paragraph::new(lines).block(Block::default().padding(Padding::horizontal(2)));
        f.render_widget(para, area);
    }

    fn render_help_popup(&self, f: &mut Frame, area: Rect) {
        let w = 52u16.min(area.width.saturating_sub(4));
        let h = 16u16.min(area.height.saturating_sub(4));
        let x = (area.width.saturating_sub(w)) / 2;
        let y = (area.height.saturating_sub(h)) / 2;

        let popup_area = Rect { x, y, width: w, height: h };
        f.render_widget(Clear, popup_area);

        let help_text = Text::from(vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("  Keyboard Shortcuts", Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  ↑↓ / j/k", Style::default().fg(self.theme.primary)),
                Span::styled("     Navigate fields", Style::default().fg(self.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(self.theme.primary)),
                Span::styled("          Edit / Confirm value", Style::default().fg(self.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  Space", Style::default().fg(self.theme.primary)),
                Span::styled("          Toggle checkbox", Style::default().fg(self.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  Esc / q", Style::default().fg(self.theme.primary)),
                Span::styled("        Cancel / Quit", Style::default().fg(self.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  Tab", Style::default().fg(self.theme.primary)),
                Span::styled("            Next field", Style::default().fg(self.theme.text)),
            ]),
            Line::from(vec![
                Span::styled("  F1", Style::default().fg(self.theme.primary)),
                Span::styled("             Toggle help", Style::default().fg(self.theme.text)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  Press Esc or F1 to close", Style::default().fg(self.theme.text_muted)),
            ]),
        ]);

        let para = Paragraph::new(help_text)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_type(ratatui::widgets::BorderType::Rounded)
                    .border_style(Style::default().fg(self.theme.primary))
                    .title(Span::styled(" Help ", Style::default().fg(self.theme.primary).add_modifier(Modifier::BOLD))),
            );

        f.render_widget(para, popup_area);
    }
}

// ============================================================================
// ASCII ART GENERATOR
// ============================================================================

fn get_ascii_art(name: &str) -> Vec<&'static str> {
    let lower = name.to_lowercase();
    if lower.contains("pake") {
        vec![
            r" ____       _       ",
            r"|  _ \ __ _| | _____ ",
            r"| |_) / _` | |/ / _ \",
            r"|  __/ (_| |   <  __/",
            r"|_|   \__,_|_|\_\___|",
        ]
    } else if lower.contains("image") {
        vec![
            r" ___                            ",
            r"|_ _|_ __ ___   __ _  __ _  ___ ",
            r" | || '_ ` _ \ / _` |/ _` |/ _ \",
            r" | || | | | | | (_| | (_| |  __/",
            r"|___|_| |_| |_|\__,_|\__, |\___|",
        ]
    } else {
        vec![
            r" _____ _   _ ___ ",
            r"|_   _| | | |_ _|",
            r"  | | | | | || | ",
            r"  | | | |_| || | ",
            r"  |_|  \___/|___|",
        ]
    }
}

// ============================================================================
// INPUT HANDLERS
// ============================================================================

fn handle_mouse(
    mouse: crossterm::event::MouseEvent,
    state: &mut FormState,
    schema: &TuiSchema,
) {
    match mouse.kind {
        crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
            let row = mouse.row;
            let header_offset: u16 = 10;
            let clicked_row = row.saturating_sub(header_offset);
            let field_idx = clicked_row as usize + state.scroll_offset;
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

fn handle_edit_key(
    key: crossterm::event::KeyEvent,
    state: &mut FormState,
    schema: &TuiSchema,
) {
    let field = &schema.fields[state.focused_index];
    match field.widget {
        WidgetKind::Select => handle_select_key(key, state, field),
        WidgetKind::Checkbox => {
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
    field: &Field,
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
    field: &Field,
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

fn key_to_action(
    key: crossterm::event::KeyEvent,
    state: &FormState,
    schema: &TuiSchema,
) -> Action {
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

fn build_cli_preview(schema: &TuiSchema, state: &FormState) -> String {
    let app_name = schema.name.to_lowercase().replace(' ', "-");
    let mut args = Vec::new();
    for field in &schema.fields {
        if field.skip {
            continue;
        }
        if let Some(val) = state.get_value(&field.name) {
            let arg_str = match val {
                Value::String(s) => {
                    if s.is_empty() {
                        String::new()
                    } else {
                        format!("--{} \"{}\"", field.name, s)
                    }
                }
                Value::Integer(n) => format!("--{} {}", field.name, n),
                Value::Float(f) => format!("--{} {}", field.name, f),
                Value::Bool(b) => {
                    if *b {
                        format!("--{}", field.name)
                    } else {
                        String::new()
                    }
                }
                Value::Path(p) => {
                    let s = p.to_string_lossy();
                    if s.is_empty() {
                        String::new()
                    } else {
                        format!("--{} \"{}\"", field.name, s)
                    }
                }
                Value::List(l) => {
                    let items: Vec<String> = l.iter().map(|v| match v {
                        Value::String(s) => s.clone(),
                        _ => String::new(),
                    }).filter(|s| !s.is_empty()).collect();
                    if items.is_empty() {
                        String::new()
                    } else {
                        format!("--{} \"{}\"", field.name, items.join(","))
                    }
                }
                Value::None => String::new(),
            };
            if !arg_str.is_empty() {
                args.push(arg_str);
            }
        }
    }
    if args.is_empty() {
        app_name
    } else {
        format!("{} {}", app_name, args.join(" "))
    }
}
