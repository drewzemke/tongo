use crate::{
    components::{Component, ComponentCommand},
    system::{command::Command, event::Event},
};
use crossterm::event::KeyCode;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyModifiers};
use std::collections::VecDeque;

pub mod mock_storage;

pub struct ComponentTestHarness<C: Component> {
    component: C,
    events: Vec<Event>,
}

impl<C: Component> ComponentTestHarness<C> {
    pub const fn new(component: C) -> Self {
        Self {
            component,
            events: Vec::new(),
        }
    }

    pub const fn component(&self) -> &C {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }

    pub fn given_command(&mut self, command: Command) {
        let events = self
            .component
            .handle_command(&ComponentCommand::Command(command));
        self.process_events(events);
    }

    pub fn given_string(&mut self, string: &str) {
        for c in string.chars() {
            let ct_event =
                CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
            let command = ComponentCommand::RawEvent(ct_event);
            let events = self.component.handle_command(&command);
            self.process_events(events);
        }
    }

    pub fn given_event(&mut self, event: Event) {
        self.process_events(vec![event]);
    }

    fn process_events(&mut self, events: Vec<Event>) {
        let mut events_deque = VecDeque::from(events);

        while let Some(event) = events_deque.pop_front() {
            let new_events = self.component.handle_event(&event);

            events_deque.append(&mut new_events.into());

            self.events.push(event);
        }
    }

    pub fn expect_event<P: FnMut(&&Event) -> bool>(&self, predicate: P) {
        let event = self.events.iter().find(predicate);
        assert!(event.is_some(), "Matching event not found");
    }
}
