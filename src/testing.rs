use crate::{
    components::Component,
    config::key_map::Key,
    system::{
        command::Command,
        event::Event,
        message::Message,
        signal::{Signal, SignalQueue},
    },
};
use crossterm::event::KeyCode;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyModifiers};

pub mod mock_storage;

pub struct ComponentTestHarness<C: Component> {
    component: C,
    events: Vec<Event>,
    messages: Vec<Message>,
}

impl<C: Component> ComponentTestHarness<C> {
    pub const fn new(component: C) -> Self {
        Self {
            component,
            events: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub const fn component(&self) -> &C {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut C {
        &mut self.component
    }

    pub fn given_command(&mut self, command: Command) {
        let mut queue = SignalQueue::default();
        self.component.handle_command(&command, &mut queue);
        self.process_signals(queue);
    }

    pub fn given_key(&mut self, string: &str) {
        let key = Key::try_from(string).expect("key codes in tests should be correct");
        let raw_event = CrosstermEvent::Key(KeyEvent::new(key.code, KeyModifiers::empty()));
        let mut queue = SignalQueue::default();
        self.component.handle_raw_event(&raw_event, &mut queue);
        self.process_signals(queue);
    }

    pub fn given_string(&mut self, string: &str) {
        for c in string.chars() {
            let raw_event =
                CrosstermEvent::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::empty()));
            let mut queue = SignalQueue::default();
            self.component.handle_raw_event(&raw_event, &mut queue);
            self.process_signals(queue);
        }
    }

    pub fn given_event(&mut self, event: Event) {
        let mut queue = SignalQueue::default();
        queue.push(event);
        self.process_signals(queue);
    }

    fn process_signals(&mut self, mut queue: SignalQueue) {
        while let Some(signal) = queue.pop() {
            let mut new_queue = SignalQueue::default();

            match &signal {
                Signal::Event(event) => self.component.handle_event(event, &mut new_queue),
                Signal::Message(message) => self.component.handle_message(message, &mut new_queue),
            }

            // Add any new signals back to the main queue for processing
            while let Some(new_signal) = new_queue.pop() {
                queue.push(new_signal);
            }

            // Store the processed signal for test assertions
            match signal {
                Signal::Event(event) => self.events.push(event),
                Signal::Message(message) => self.messages.push(message),
            }
        }
    }

    pub fn expect_event<P: FnMut(&&Event) -> bool>(&self, predicate: P) {
        let event = self.events.iter().find(predicate);
        assert!(
            event.is_some(),
            "Matching event not found. These events were recorded:\n{:?}",
            self.events
        );
    }

    pub fn expect_message<P: FnMut(&&Message) -> bool>(&self, predicate: P) {
        let event = self.messages.iter().find(predicate);
        assert!(
            event.is_some(),
            "Matching message not found. These message were recorded:\n{:?}",
            self.messages
        );
    }

    pub fn expect_no_messages(&self) {
        assert!(
            self.messages.is_empty(),
            "Message list not empty. These message were recorded:\n{:?}",
            self.messages
        );
    }
}
