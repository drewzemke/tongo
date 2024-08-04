#![allow(clippy::module_name_repetitions)]

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

#[derive(Debug)]
pub enum Command {
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,

    FocusUp,
    FocusDown,
    FocusLeft,
    FocusRight,

    CreateNew,
    Select,
    NextField,
    PreviousField,
    Delete,
    Back,
    Quit,

    InsertDoc,
    EditDoc,
    DuplicateDoc,
    DeleteDoc,
}

#[derive(Debug)]
pub struct CommandGroup {
    pub commands: Vec<Command>,
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
