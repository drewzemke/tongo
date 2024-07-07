use super::list_widget::ListWidget;
use crate::state::{State, WidgetFocus};
use mongodb::results::CollectionSpecification;
use ratatui::widgets::ListState;

#[derive(Debug, Default)]
pub struct CollectionListState {
    pub items: Vec<CollectionSpecification>,
    pub state: ListState,
}

#[derive(Debug, Default)]
pub struct CollList<'a> {
    marker: std::marker::PhantomData<State<'a>>,
}

impl<'a> ListWidget for CollList<'a> {
    type Item = CollectionSpecification;
    type State = State<'a>;

    fn title() -> &'static str {
        "Collections"
    }

    fn list_state(state: &mut Self::State) -> &mut ListState {
        &mut state.coll_list.state
    }

    fn items(state: &Self::State) -> std::slice::Iter<Self::Item> {
        state.coll_list.items.iter()
    }

    fn item_to_str(item: &Self::Item) -> String {
        item.name.clone()
    }

    fn is_focused(state: &Self::State) -> bool {
        state.focus == WidgetFocus::CollectionList
    }

    fn on_select(state: &mut Self::State) {
        state.main_view.page = 0;
        state.exec_query();
        state.exec_count();
        state.focus = WidgetFocus::MainView;
    }
}
