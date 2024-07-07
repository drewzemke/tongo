#![allow(clippy::module_name_repetitions)]

use crate::{key_hint::KeyHint, state::State};
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
        let content = state.status_bar.message.as_ref().map_or_else(
            || {
                let key_hints = KeyHint::from_state(state);
                Line::from(
                    key_hints
                        .into_iter()
                        .flat_map(Into::<Vec<Span>>::into)
                        .collect::<Vec<Span>>(),
                )
            },
            |message| {
                Line::from(vec![
                    Span::styled("Error: ", Style::default().red()),
                    Span::from(message.clone()),
                ])
            },
        );

        let paragraph = Paragraph::new(content)
            .wrap(Wrap::default())
            .block(Block::default().padding(Padding::horizontal(1)));
        Widget::render(paragraph, area, buf);

        // // this is to debug computing keys based on selected stuff
        // else {
        //     let text = state.main_view.state.selected().join(".");
        //     let paragraph = Paragraph::new(text)
        //         .wrap(Wrap::default())
        //         .block(Block::default().padding(Padding::horizontal(1)));
        //     Widget::render(paragraph, area, buf);
        // }
    }
}
