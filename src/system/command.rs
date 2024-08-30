#![allow(clippy::match_same_arms)]

use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

#[derive(Debug, Clone, Copy)]
pub enum Command {
    NavUp,
    NavDown,
    NavLeft,
    NavRight,

    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,

    CreateNew,
    Confirm,
    Reset,
    Refresh,
    ExpandCollapse,
    NextPage,
    PreviousPage,
    Delete,
    Back,
    Quit,

    InsertDoc,
    EditDoc,
    DuplicateDoc,
    DeleteDoc,
}

impl Command {
    // TODO: make configurable
    pub const fn key(self) -> KeyCode {
        match self {
            Self::NavUp => KeyCode::Up,
            Self::NavDown => KeyCode::Down,
            Self::NavLeft => KeyCode::Left,
            Self::NavRight => KeyCode::Right,

            Self::FocusUp => KeyCode::Char('K'),
            Self::FocusDown => KeyCode::Char('J'),
            Self::FocusLeft => KeyCode::Char('H'),
            Self::FocusRight => KeyCode::Char('L'),

            Self::CreateNew => KeyCode::Char('n'),
            Self::Confirm => KeyCode::Enter,
            Self::Reset => KeyCode::Char('R'),
            Self::Refresh => KeyCode::Char('r'),
            Self::ExpandCollapse => KeyCode::Char(' '),
            Self::NextPage => KeyCode::Char('n'),
            Self::PreviousPage => KeyCode::Char('p'),
            Self::Delete => KeyCode::Char('D'),
            Self::Back => KeyCode::Esc,
            Self::Quit => KeyCode::Char('q'),

            Self::InsertDoc => KeyCode::Char('I'),
            Self::EditDoc => KeyCode::Char('E'),
            Self::DuplicateDoc => KeyCode::Char('C'),
            Self::DeleteDoc => KeyCode::Char('D'),
        }
    }

    fn key_to_string(key: KeyCode) -> String {
        match key {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Home => "home".to_string(),
            KeyCode::End => "end".to_string(),
            KeyCode::Tab => "tab".to_string(),
            KeyCode::Delete => "del".to_string(),
            KeyCode::Esc => "esc".to_string(),
            KeyCode::CapsLock => "caps".to_string(),
            KeyCode::Char(c) => match c {
                ' ' => "space".to_string(),
                c => c.to_string(),
            },
            _ => "?".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
    key_hint: String,
    name: &'static str,
}

impl CommandGroup {
    pub fn new(commands: Vec<Command>, name: &'static str) -> Self {
        let key_hint = commands
            .iter()
            .map(|c| Command::key(*c))
            .map(Command::key_to_string)
            .collect();

        Self {
            commands,
            key_hint,
            name,
        }
    }
}

impl<'a> From<&'a CommandGroup> for Vec<Span<'a>> {
    fn from(hint: &'a CommandGroup) -> Self {
        let hint_style = Style::default();
        vec![
            Span::styled(&hint.key_hint, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(hint.name, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}
