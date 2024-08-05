#![allow(clippy::module_name_repetitions, clippy::match_same_arms)]

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
}

#[derive(Debug)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
    // TODO: get from key fn
    pub key: &'static str,
    pub text: &'static str,
}

impl CommandGroup {
    pub const fn new(commands: Vec<Command>, key: &'static str, text: &'static str) -> Self {
        Self {
            commands,
            key,
            text,
        }
    }
}

impl<'a> From<&CommandGroup> for Vec<Span<'a>> {
    fn from(hint: &CommandGroup) -> Self {
        let hint_style = Style::default();
        vec![
            Span::styled(hint.key, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(hint.text, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}
