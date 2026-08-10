#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    FocusNext,
    FocusPrev,
    ToggleEdit,
    ConfirmEdit,
    CancelEdit,
    ToggleValue,
    SelectOption(usize),
    Submit,
    Cancel,
    ShowHelp,
    HideHelp,
    ScrollUp,
    ScrollDown,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Key(Key),
    Resize(u16, u16),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Esc,
    Tab,
    BackTab,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Home,
    End,
    Backspace,
    Delete,
    F(u8),
}

impl Event {
    pub fn to_action(&self, editing: bool) -> Action {
        match self {
            Event::Key(key) => match key {
                Key::Char('\t') if !editing => Action::FocusNext,
                Key::BackTab => Action::FocusPrev,
                Key::Down if !editing => Action::FocusNext,
                Key::Up if !editing => Action::FocusPrev,
                Key::Enter if !editing => Action::ToggleEdit,
                Key::Enter if editing => Action::ConfirmEdit,
                Key::Esc if editing => Action::CancelEdit,
                Key::Esc if !editing => Action::Cancel,
                Key::Char(' ') if !editing => Action::ToggleValue,
                Key::Char('j') if !editing => Action::FocusNext,
                Key::Char('k') if !editing => Action::FocusPrev,
                Key::Char('q') if !editing => Action::Cancel,
                Key::Char('\n') if !editing => Action::Submit,
                Key::F(1) => Action::ShowHelp,
                Key::PageUp => Action::ScrollUp,
                Key::PageDown => Action::ScrollDown,
                Key::Home => Action::ScrollUp,
                Key::End => Action::ScrollDown,
                _ => Action::None,
            },
            Event::Resize(_, _) => Action::None,
        }
    }
}
