//! Optional adapters for `nive-runtime`.
//!
//! These helpers are feature-gated so the core workbench shell remains usable
//! without runtime lifecycle types.

use nive_runtime::ActionMap;
use nive_ui::widgets::CommandPaletteRow;

/// Builds command palette rows from a runtime action map.
pub fn action_palette_rows<M>(actions: &ActionMap<M>) -> Vec<CommandPaletteRow<'_, M>>
where
    M: Clone,
{
    nive_runtime::command_palette_rows(actions)
}

#[cfg(test)]
mod tests {
    use nive_runtime::{Action, ActionMap};

    use super::*;

    #[test]
    fn maps_runtime_actions_to_palette_rows() {
        let actions = ActionMap::new().action(Action::new("file.save", "Save", ()));

        let rows = action_palette_rows(&actions);

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "file.save");
    }
}
