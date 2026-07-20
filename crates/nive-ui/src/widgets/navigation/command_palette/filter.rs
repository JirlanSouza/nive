use super::CommandPaletteItem;

/// Returns the indices of [`CommandPaletteItem`]s that match `query`.
///
/// The match is a case-insensitive substring on the item's label and optional
/// description. An empty query returns every index in order. The order of
/// the returned indices follows the order of the input items so apps can
/// preserve their declared action ordering while filtering.
pub fn command_palette_filter<M>(query: &str, items: &[CommandPaletteItem<'_, M>]) -> Vec<usize> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return (0..items.len()).collect();
    }

    let needle = trimmed.to_ascii_lowercase();

    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let label_matches = item.label.to_ascii_lowercase().contains(&needle);
            let description_matches = item
                .description
                .is_some_and(|description| description.to_ascii_lowercase().contains(&needle));

            (label_matches || description_matches).then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod filter_tests {
    use super::*;

    fn item<'a>(label: &'a str, description: Option<&'a str>) -> CommandPaletteItem<'a, ()> {
        let mut item = CommandPaletteItem::new("id", label, ());
        if let Some(description) = description {
            item = item.description(description);
        }
        item
    }

    #[test]
    fn empty_query_returns_all_indices_in_order() {
        let items = [
            item("Open file", None),
            item("Save file", Some("Persist the current buffer")),
            item("Close", None),
        ];

        let visible = command_palette_filter("   ", &items);

        assert_eq!(visible, vec![0, 1, 2]);
    }

    #[test]
    fn case_insensitive_label_match() {
        let items = [
            item("Open File", None),
            item("Save", None),
            item("Close", None),
        ];

        let visible = command_palette_filter("open", &items);

        assert_eq!(visible, vec![0]);
    }

    #[test]
    fn description_match_includes_item() {
        let items = [
            item("Open file", Some("Pick a path from disk")),
            item("Save", Some("Persist the current buffer")),
            item("Close", None),
        ];

        let visible = command_palette_filter("disk", &items);

        assert_eq!(visible, vec![0]);
    }

    #[test]
    fn preserves_input_order_across_matches() {
        let items = [
            item("Save", None),
            item("Save As", Some("Pick a new path")),
            item("Save All", None),
        ];

        let visible = command_palette_filter("save", &items);

        assert_eq!(visible, vec![0, 1, 2]);
    }

    #[test]
    fn empty_input_returns_no_matches() {
        let items = [item("Open", None), item("Save", None)];

        let visible = command_palette_filter("nothing", &items);

        assert!(visible.is_empty());
    }

    #[test]
    fn unicode_query_matches_label() {
        let items = [
            item("Salvar", Some("Persistir buffer")),
            item("Abrir", None),
        ];

        let visible = command_palette_filter("salv", &items);

        assert_eq!(visible, vec![0]);
    }
}
