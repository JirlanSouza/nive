use super::CommandPaletteRow;

/// Returns the indices of [`CommandPaletteRow`]s that match `query`.
///
/// The match is a case-insensitive substring on the row's label and optional
/// description. An empty query returns every index in order. The order of
/// the returned indices follows the order of the input rows so apps can
/// preserve their declared action ordering while filtering.
pub fn command_palette_filter<M>(query: &str, rows: &[CommandPaletteRow<'_, M>]) -> Vec<usize> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return (0..rows.len()).collect();
    }

    let needle = trimmed.to_ascii_lowercase();

    rows.iter()
        .enumerate()
        .filter_map(|(index, row)| {
            let label_matches = row.label.to_ascii_lowercase().contains(&needle);
            let description_matches = row
                .description
                .is_some_and(|description| description.to_ascii_lowercase().contains(&needle));

            (label_matches || description_matches).then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod filter_tests {
    use super::*;

    fn row<'a>(label: &'a str, description: Option<&'a str>) -> CommandPaletteRow<'a, ()> {
        let mut row = CommandPaletteRow::new("id", label, ());
        if let Some(description) = description {
            row = row.description(description);
        }
        row
    }

    #[test]
    fn empty_query_returns_all_indices_in_order() {
        let rows = [
            row("Open file", None),
            row("Save file", Some("Persist the current buffer")),
            row("Close", None),
        ];

        let visible = command_palette_filter("   ", &rows);

        assert_eq!(visible, vec![0, 1, 2]);
    }

    #[test]
    fn case_insensitive_label_match() {
        let rows = [
            row("Open File", None),
            row("Save", None),
            row("Close", None),
        ];

        let visible = command_palette_filter("open", &rows);

        assert_eq!(visible, vec![0]);
    }

    #[test]
    fn description_match_includes_row() {
        let rows = [
            row("Open file", Some("Pick a path from disk")),
            row("Save", Some("Persist the current buffer")),
            row("Close", None),
        ];

        let visible = command_palette_filter("disk", &rows);

        assert_eq!(visible, vec![0]);
    }

    #[test]
    fn preserves_input_order_across_matches() {
        let rows = [
            row("Save", None),
            row("Save As", Some("Pick a new path")),
            row("Save All", None),
        ];

        let visible = command_palette_filter("save", &rows);

        assert_eq!(visible, vec![0, 1, 2]);
    }

    #[test]
    fn empty_input_returns_no_matches() {
        let rows = [row("Open", None), row("Save", None)];

        let visible = command_palette_filter("nothing", &rows);

        assert!(visible.is_empty());
    }

    #[test]
    fn unicode_query_matches_label() {
        let rows = [row("Salvar", Some("Persistir buffer")), row("Abrir", None)];

        let visible = command_palette_filter("salv", &rows);

        assert_eq!(visible, vec![0]);
    }
}
