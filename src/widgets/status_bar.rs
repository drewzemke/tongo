#![allow(clippy::module_name_repetitions)]

use crate::state::State;
use ratatui::{
    prelude::*,
    widgets::{Block, Padding, Paragraph, StatefulWidget, Wrap},
};

#[derive(Debug, Default)]
pub struct StatusBarState {
    pub message: Option<String>,
}

#[derive(Debug, Default)]
pub struct StatusBar<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> StatefulWidget for StatusBar<'a> {
    type State = State<'a>;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if let Some(ref message) = state.status_bar.message {
            let line = Line::from(vec![
                Span::styled("Error: ", Style::default().red()),
                Span::from(message.clone()),
            ]);
            let paragraph = Paragraph::new(line)
                .wrap(Wrap::default())
                .block(Block::default().padding(Padding::horizontal(1)));

            Widget::render(paragraph, area, buf);
        }
    }
}
