use crate::{
    components::{tab::TabFocus, Component},
    config::Config,
    system::{
        command::{Command, CommandCategory, CommandGroup},
        event::Event,
        message::{AppAction, Message},
        Signal,
    },
};
use ratatui::prelude::*;
use std::{cell::Cell, rc::Rc};

use super::{DefaultFormatter, InnerInput};

const INPUT_MODAL_WIDTH: u16 = 40;
const INPUT_MODAL_HEIGHT: u16 = 1;

#[derive(Debug, Clone, Copy)]
pub enum InputKind {
    NewCollectionName,
    NewDatabaseName,
}

impl InputKind {
    const fn modal_title(self) -> &'static str {
        match self {
            Self::NewCollectionName => "New Connection's Name",
            Self::NewDatabaseName => "New Database's Name",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputModal {
    focus: Rc<Cell<TabFocus>>,
    kind: Option<InputKind>,
    input: InnerInput<DefaultFormatter>,
}
impl InputModal {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let mut input = InnerInput::new("", cursor_pos, config, DefaultFormatter::default());
        input.start_editing();

        Self {
            focus,
            input,
            ..Default::default()
        }
    }

    pub fn show_with(&mut self, kind: InputKind) {
        // HACK: it would be better if we could just compute this at render time,
        // like if the input didn't insist on rendering its own title
        self.input.set_title(kind.modal_title());

        self.kind = Some(kind);
        self.focus();
    }
}

impl Component for InputModal {
    fn is_focused(&self) -> bool {
        self.focus.get() == TabFocus::InputModal
    }

    fn focus(&self) {
        self.focus.set(TabFocus::InputModal);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(INPUT_MODAL_HEIGHT + 2),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(INPUT_MODAL_WIDTH + 2),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        self.input.render(frame, layout[1], true);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        match self.kind {
            Some(InputKind::NewCollectionName) => vec![
                CommandGroup::new(vec![Command::Confirm], "create collection")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "cancel")
                    .in_cat(CommandCategory::StatusBarOnly),
            ],
            Some(InputKind::NewDatabaseName) => vec![
                CommandGroup::new(vec![Command::Confirm], "create database")
                    .in_cat(CommandCategory::StatusBarOnly),
                CommandGroup::new(vec![Command::Back], "cancel")
                    .in_cat(CommandCategory::StatusBarOnly),
            ],
            _ => vec![],
        }
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event) -> Vec<Signal> {
        self.input.handle_raw_event(event)
    }

    fn handle_command(&mut self, command: &Command) -> Vec<Signal> {
        match command {
            Command::Confirm => {
                let value = self.input.value().to_string();
                self.input.set_value("");
                vec![
                    Event::InputConfirmed(
                        self.kind.expect("input should not be shown without a kind"),
                        value,
                    )
                    .into(),
                    Message::to_app(AppAction::ExitRawMode).into(),
                ]
            }
            Command::Back => {
                self.input.set_value("");
                vec![
                    Event::InputCanceled.into(),
                    Message::to_app(AppAction::ExitRawMode).into(),
                ]
            }
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::ComponentTestHarness;

    #[test]
    fn enter_text_then_confirm() {
        let mut test = ComponentTestHarness::new(InputModal::default());

        test.component_mut().input.start_editing();
        test.component_mut().show_with(InputKind::NewCollectionName);

        test.given_string("text!");
        test.given_command(Command::Confirm);

        test.expect_event(
            |e| matches!(e, Event::InputConfirmed(InputKind::NewCollectionName, s) if s == "text!"),
        );
        test.expect_message(|m| matches!(m.read_as_app(), Some(AppAction::ExitRawMode)));
        assert_eq!(test.component_mut().input.value(), "");
    }

    #[test]
    fn enter_text_then_cancel() {
        let mut test = ComponentTestHarness::new(InputModal::default());

        test.component_mut().input.start_editing();
        test.component_mut().show_with(InputKind::NewCollectionName);

        test.given_string("text!");
        test.given_command(Command::Back);

        test.expect_event(|e| matches!(e, Event::InputCanceled));
        test.expect_message(|m| matches!(m.read_as_app(), Some(AppAction::ExitRawMode)));
        assert_eq!(test.component_mut().input.value(), "");
    }
}
