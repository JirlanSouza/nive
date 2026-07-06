/// Shared horizontal/vertical orientation for interactive widgets.
///
/// `Horizontal` means content is laid out side by side (left/right), matching
/// GTK's `GtkOrientable` and Qt's `QSplitter` semantics. This is the opposite
/// of iced's `pane_grid::Axis`, where `Axis::Horizontal` describes a
/// horizontal split *line* (panes stacked top/bottom) a false friend to
/// watch for when porting `pane_grid` code onto widgets that consume this
/// type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Panes/content are laid out side by side (left/right).
    #[default]
    Horizontal,
    /// Panes/content are stacked top/bottom.
    Vertical,
}
