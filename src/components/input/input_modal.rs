use crate::{
    components::{tab::TabFocus, Component, ComponentCommand},
    system::{
        command::{Command, CommandGroup},
        event::Event,
        Signal,
    },
};
use ratatui::{prelude::*, widgets::Clear};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use super::{DefaultFormatter, InnerInput};

const INPUT_MODAL_WIDTH: u16 = 40;
const INPUT_MODAL_HEIGHT: u16 = 1;

#[derive(Debug, Clone, Copy)]
pub enum InputKind {
    NewCollectionName,
    NewDatabaseName,
}

impl InputKind {
    fn modal_title(&self) -> &'static str {
        match self {
            InputKind::NewCollectionName => "New Connection's Name",
            InputKind::NewDatabaseName => "New Database's Name",
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputModal {
    focus: Rc<RefCell<TabFocus>>,
    kind: Option<InputKind>,
    input: InnerInput<DefaultFormatter>,
}
impl InputModal {
    pub fn new(focus: Rc<RefCell<TabFocus>>, cursor_pos: Rc<Cell<(u16, u16)>>) -> Self {
        let mut input = InnerInput::new("", cursor_pos, DefaultFormatter::default());
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
        *self.focus.borrow() == TabFocus::InputModal
    }

    fn focus(&self) {
        *self.focus.borrow_mut() = TabFocus::InputModal;
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::vertical(vec![
            Constraint::Fill(1),
            Constraint::Length(INPUT_MODAL_HEIGHT + 4),
            Constraint::Fill(1),
        ])
        .split(area);
        let layout = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Length(INPUT_MODAL_WIDTH + 6),
            Constraint::Fill(1),
        ])
        .split(layout[1]);

        frame.render_widget(Clear, layout[1]);
        self.input
            .render(frame, layout[1].inner(Margin::new(2, 1)), true);
    }

    fn commands(&self) -> Vec<CommandGroup> {
        match self.kind {
            Some(InputKind::NewCollectionName) => vec![
                CommandGroup::new(vec![Command::Confirm], "create collection"),
                CommandGroup::new(vec![Command::Back], "cancel"),
            ],
            Some(InputKind::NewDatabaseName) => vec![
                CommandGroup::new(vec![Command::Confirm], "create database"),
                CommandGroup::new(vec![Command::Back], "cancel"),
            ],
            _ => vec![],
        }
    }

    fn handle_command(&mut self, command: &ComponentCommand) -> Vec<Signal> {
        match command {
            ComponentCommand::RawEvent(event) => self.input.handle_raw_event(event),
            ComponentCommand::Command(command) => match command {
                Command::Confirm => {
                    let value = self.input.value().to_string();
                    self.input.set_value("");
                    vec![
                        Event::InputConfirmed(
                            self.kind.expect("input should not be shown without a kind"),
                            value,
                        )
                        .into(),
                        Event::RawModeExited.into(),
                    ]
                }
                Command::Back => {
                    self.input.set_value("");
                    vec![Event::InputCanceled.into(), Event::RawModeExited.into()]
                }
                _ => vec![],
            },
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

        test.expect_event(|e| {
            matches!(e, Event::InputConfirmed(InputKind::NewCollectionName, s) if *s == "text!".to_string())
        });
        test.expect_event(|e| matches!(e, Event::RawModeExited));
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
        test.expect_event(|e| matches!(e, Event::RawModeExited));
        assert_eq!(test.component_mut().input.value(), "");
    }
}
