//! Optional adapters for `nive-runtime`.
//!
//! These helpers are feature-gated so the core workbench shell remains usable
//! without runtime lifecycle types.

use nive_runtime::ActionMap;
use nive_ui::widgets::CommandPaletteItem;

/// Projects a shared action map into canonical command-palette items.
pub fn action_palette_items<M>(actions: &ActionMap<M>) -> Vec<CommandPaletteItem<'_, M>>
where
    M: Clone,
{
    actions
        .iter()
        .map(CommandPaletteItem::from_action)
        .collect()
}

#[cfg(test)]
mod tests {
    use nive_runtime::{Action, ActionMap};

    use super::*;

    #[test]
    fn maps_runtime_actions_to_palette_items() {
        let actions = ActionMap::new().action(Action::new("file.save", "Save", ()));

        let items = action_palette_items(&actions);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, "file.save");
    }
}
