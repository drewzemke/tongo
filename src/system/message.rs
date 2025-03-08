#[derive(Debug, Clone, strum_macros::Display, PartialEq, Eq)]
pub enum Action {
    EnterRawMode,
    ExitRawMode,
}

#[derive(Debug, Clone, strum_macros::Display, PartialEq, Eq)]
pub enum Target {
    App,
    Tab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message(Action, Target);

impl Message {
    pub fn new(action: Action, target: Target) -> Self {
        Self(action, target)
    }

    pub fn action(&self) -> &Action {
        &self.0
    }

    pub fn target(&self) -> &Target {
        &self.1
    }
}
