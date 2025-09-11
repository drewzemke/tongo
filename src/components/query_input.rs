use std::{cell::Cell, rc::Rc};

use ratatui::{layout::Rect, Frame};
use serde::{Deserialize, Serialize};

use crate::{
    components::{
        input::filter::FilterInput,
        tab::{CloneWithFocus, TabFocus},
        Component,
    },
    config::Config,
    persistence::PersistedComponent,
    system::{
        command::{Command, CommandGroup},
        signal::SignalQueue,
    },
};

#[derive(Debug, Default, Clone)]
pub struct QueryInput {
    #[expect(dead_code)]
    focus: Rc<Cell<TabFocus>>,
    filter_input: FilterInput,
}

impl CloneWithFocus for QueryInput {
    fn clone_with_focus(&self, focus: Rc<Cell<TabFocus>>) -> Self {
        Self {
            filter_input: self.filter_input.clone_with_focus(focus.clone()),
            focus,
        }
    }
}

impl QueryInput {
    pub fn new(
        focus: Rc<Cell<TabFocus>>,
        cursor_pos: Rc<Cell<(u16, u16)>>,
        config: Config,
    ) -> Self {
        let filter_input = FilterInput::new(focus.clone(), cursor_pos, config);
        Self {
            focus,
            filter_input,
        }
    }

    pub const fn is_editing(&self) -> bool {
        self.filter_input.is_editing()
    }
}

impl Component for QueryInput {
    fn commands(&self) -> Vec<CommandGroup> {
        self.filter_input.commands()
    }

    fn handle_command(&mut self, command: &Command, queue: &mut SignalQueue) {
        self.filter_input.handle_command(command, queue);
    }

    fn handle_raw_event(&mut self, event: &crossterm::event::Event, queue: &mut SignalQueue) {
        self.filter_input.handle_raw_event(event, queue);
    }

    fn focus(&self) {
        self.filter_input.focus();
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.filter_input.render(frame, area);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedQueryInput {
    filter_input: String,
}

impl PersistedComponent for QueryInput {
    type StorageType = PersistedQueryInput;

    fn persist(&self) -> Self::StorageType {
        PersistedQueryInput {
            filter_input: self.filter_input.persist(),
        }
    }

    fn hydrate(&mut self, storage: Self::StorageType) {
        self.filter_input.hydrate(storage.filter_input);
    }
}
