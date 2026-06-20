#[derive(Debug, Default)]
pub(super) struct AutocompleteState {
    pub highlighted: Option<usize>,
    pub dismissed: bool,
    pub input_focused: bool,
    pub open: bool,
    pub item_count: usize,
    pub input_value: String,
}

pub(super) fn initial_highlight(open: bool, item_count: usize) -> Option<usize> {
    (open && item_count > 0).then_some(0)
}
