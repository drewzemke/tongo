#![allow(clippy::module_name_repetitions)]

use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

#[derive(Debug)]
pub enum Command {
    Navigate,
    ChangeFocus,
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
pub struct CommandInfo {
    pub command: Command,
    pub key: &'static str,
    pub text: &'static str,
}

impl CommandInfo {
    pub const fn new(command: Command, key: &'static str, text: &'static str) -> Self {
        Self { command, key, text }
    }
}

impl<'a> From<&CommandInfo> for Vec<Span<'a>> {
    fn from(hint: &CommandInfo) -> Self {
        let hint_style = Style::default();
        vec![
            Span::styled(hint.key, hint_style.add_modifier(Modifier::BOLD)),
            Span::styled(": ", hint_style),
            Span::styled(hint.text, hint_style.fg(Color::Gray)),
            Span::raw("  "),
        ]
    }
}
