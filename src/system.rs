/// Commands represent user intentions. They correspond one-to-one with key
/// presses, and are the things that users can configure key maps for. Each
/// command is passed to the root component (`App`) as well as to every
/// component in the component tree between the root and the currently-focused
/// component.
pub mod command;

/// Events represent things that happen within the program. Events are emitted
/// by components as a result of handling a command, a message, or another
/// event, or when an async process (such as a db query) has completed. The
/// root component (`App`), the client, the status bar, the tab bar, as well
/// as *every* component in the currently-visible tab (even those that are not
/// visible themselves) receive every event.
pub mod event;

/// Messages represent direct imperative communications between components. Like
/// events, a component may emit a message when it handles a command, event,
/// or another message. Unlike events, messages are tagged with a recipient
/// component, and are only handled by that component. A component may send a
/// message to the root component (`App`), the client, the status bar, the tab
/// bar, or any other component within the currently-visible tab.
pub mod message;

/// Signal utilities including the Signal enum and `SignalQueue` for managing
/// the communication between components in the event-driven system.
pub mod signal;

// Re-export Signal for backward compatibility
