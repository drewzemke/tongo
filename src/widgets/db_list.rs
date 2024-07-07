use super::list_widget::ListWidget;
use crate::state::{State, WidgetFocus};
use mongodb::results::DatabaseSpecification;
use ratatui::{prelude::*, widgets::ListState};

#[derive(Debug, Default)]
pub struct DatabaseListState {
    pub items: Vec<DatabaseSpecification>,
    pub state: ListState,
}

#[derive(Debug, Default)]
pub struct DbList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> ListWidget for DbList<'a> {
    type Item = DatabaseSpecification;
    type State = State<'a>;

    fn title() -> &'static str {
        "Databases"
    }

    fn list_state(state: &mut Self::State) -> &mut ListState {
        &mut state.db_list.state
    }

    fn items(state: &Self::State) -> std::slice::Iter<Self::Item> {
        state.db_list.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> Text<'static> {
        item.name.clone().into()
    }

    fn is_focused(state: &Self::State) -> bool {
        state.focus == WidgetFocus::DatabaseList
    }

    fn on_change(state: &mut Self::State) {
        state.exec_get_collections();
    }

    fn on_select(state: &mut Self::State) {
        state.focus = WidgetFocus::CollectionList;
    }
}
