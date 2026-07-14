use iced::{
    widget::{column, container, row, scrollable, text, Space},
    Alignment, Background, Length,
};

use super::CommandPaletteRow;
use crate::theme::{
    self, spacing::SpacingScale, text as theme_text, ControlRole, ControlState, SurfaceRole,
    TextRole, TypographyRole,
};
use crate::widgets::{Input, Panel};
use crate::Element;

/// Renders a desktop-standard command palette content tree.
///
/// The view is intentionally a pure render helper: it does not own the
/// search query, the highlighted row, the open/closed state, or the
/// keyboard navigation. Apps wrap the result in a `DialogRequest` (or
/// app-owned overlay) and route `ArrowUp`/`ArrowDown`/`Enter`/`Escape`
/// themselves. Keyboard navigation is the host's responsibility so the
/// palette does not steal focus from the search input.
pub fn command_palette_view<'a, M>(
    placeholder: &'a str,
    query: &'a str,
    rows: impl IntoIterator<Item = CommandPaletteRow<'a, M>>,
    highlighted: Option<usize>,
    on_query_change: impl Fn(String) -> M + 'a,
    on_submit: Option<M>,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let spacing = theme::spacing();
    let palette_width = 480.0;

    let mut input = Input::new(placeholder, query)
        .appearance(crate::widgets::TextInputAppearance::Standard)
        .md()
        .on_change(on_query_change);

    if let Some(message) = on_submit {
        input = input.on_submit(message);
    }

    let input: Element<'a, M> = input.into();

    let collected: Vec<CommandPaletteRow<'a, M>> = rows.into_iter().collect();
    let list: Element<'a, M> = if collected.is_empty() {
        empty_state(query, spacing)
    } else {
        let list = list_content(&collected, highlighted, spacing);
        scrollable(list).into()
    };

    let content = column![input, list].spacing(spacing.xs).padding(spacing.sm);

    Panel::new(content)
        .role(SurfaceRole::Popover)
        .width(Length::Fixed(palette_width))
        .into()
}

fn list_content<'a, M>(
    rows: &[CommandPaletteRow<'a, M>],
    highlighted: Option<usize>,
    spacing: SpacingScale,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let mut column = iced::widget::Column::new().spacing(0.0).width(Length::Fill);

    for (position, row) in rows.iter().enumerate() {
        let is_highlighted = highlighted == Some(position);
        column = column.push(row_element(row, is_highlighted, spacing));
    }

    column.into()
}

fn row_element<'a, M>(
    row: &CommandPaletteRow<'a, M>,
    is_highlighted: bool,
    spacing: SpacingScale,
) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let label_color = if row.enabled {
        TextRole::Primary
    } else {
        TextRole::Muted
    };

    let label = text(row.label)
        .size(theme::typography(TypographyRole::Body).size)
        .style(theme_text::style(label_color));

    let leading: Element<'a, M> = if let Some(description) = row.description {
        column![
            label,
            text(description)
                .size(theme::typography(TypographyRole::Caption).size)
                .style(theme_text::style(TextRole::Muted)),
        ]
        .spacing(spacing.xxs)
        .width(Length::Fill)
        .into()
    } else {
        label.width(Length::Fill).into()
    };

    let trailing: Element<'a, M> = match row.shortcut_label.clone() {
        Some(shortcut) => text(shortcut)
            .size(theme::typography(TypographyRole::Caption).size)
            .style(theme_text::style(TextRole::Muted))
            .into(),
        None => Space::new().width(Length::Shrink).into(),
    };

    let content = row![leading, trailing]
        .align_y(Alignment::Center)
        .spacing(spacing.md)
        .width(Length::Fill)
        .padding([spacing.xs, spacing.md]);

    container(content)
        .style(row_style(is_highlighted))
        .width(Length::Fill)
        .into()
}

/// Highlighted rows use the accent/selection fill, not `SurfaceRole::Elevated`
/// (reserved for genuine elevation, e.g. floating cards) — see
/// `surface-hierarchy`'s "Elevated is reserved for genuine elevation".
fn row_style(is_highlighted: bool) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme: &crate::theme::Theme| {
        if is_highlighted {
            let control = theme.control(ControlRole::Selectable, ControlState::SELECTED);

            container::Style {
                text_color: Some(control.foreground),
                background: Some(Background::Color(control.background)),
                ..container::Style::default()
            }
        } else {
            theme::surface::style(SurfaceRole::Popover)(theme)
        }
    }
}

fn empty_state<'a, M>(query: &str, spacing: SpacingScale) -> Element<'a, M>
where
    M: Clone + 'a,
{
    let message: &str = if query.trim().is_empty() {
        "Type to search commands"
    } else {
        "No commands match this search"
    };

    let text = text(message)
        .size(theme::typography(TypographyRole::Body).size)
        .style(theme_text::style(TextRole::Muted));

    container(text)
        .width(Length::Fill)
        .padding([spacing.lg, spacing.xl])
        .center_x(Length::Fill)
        .into()
}
