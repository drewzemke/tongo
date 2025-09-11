use std::collections::VecDeque;

use super::{event::Event, message::Message};

#[derive(Debug, Clone)]
pub enum Signal {
    Event(Event),
    Message(Message),
}

impl std::fmt::Display for Signal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Event(event) => write!(f, "{event}"),
            Self::Message(message) => write!(f, "{message}"),
        }
    }
}

impl From<Event> for Signal {
    fn from(e: Event) -> Self {
        Self::Event(e)
    }
}

impl From<Message> for Signal {
    fn from(m: Message) -> Self {
        Self::Message(m)
    }
}

#[derive(Debug, Default)]
pub struct SignalQueue {
    queue: VecDeque<Signal>,
}

impl SignalQueue {
    pub fn push(&mut self, signal: impl Into<Signal>) {
        self.queue.push_back(signal.into());
    }

    pub fn pop(&mut self) -> Option<Signal> {
        self.queue.pop_front()
    }
}