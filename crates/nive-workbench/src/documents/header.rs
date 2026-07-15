use std::borrow::Cow;

use iced::{
    widget::{container, row, text},
    Alignment, Length,
};
use nive_ui::theme::{self, TextRole, TypographyRole};
use nive_ui::widgets::{icon, tooltip};
use nive_ui::{Element, IconRole};

/// Principal document-title composition for application-owned document content.
///
/// `DocumentHeader` is transparent content, not a [`crate::WorkbenchShell`]
/// slot. The document host continues to own its Canvas surface and inset.
pub struct DocumentHeader<'a, Message> {
    title: Cow<'a, str>,
    icon: Option<IconRole>,
    trailing: Option<Element<'a, Message>>,
    title_tooltip: Option<Cow<'a, str>>,
}

impl<'a, Message> DocumentHeader<'a, Message>
where
    Message: 'a,
{
    /// Creates a transparent principal document header.
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            trailing: None,
            title_tooltip: None,
        }
    }

    /// Adds a secondary decorative icon before the title.
    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds protected application-owned content after the flexible title.
    pub fn trailing(mut self, trailing: impl Into<Element<'a, Message>>) -> Self {
        self.trailing = Some(trailing.into());
        self
    }

    /// Exposes the full title when its visible single line is clipped.
    pub fn title_tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.title_tooltip = Some(tooltip.into());
        self
    }

    /// Configures the full-title tooltip when one is available.
    pub fn title_tooltip_maybe(mut self, tooltip: Option<impl Into<Cow<'a, str>>>) -> Self {
        self.title_tooltip = tooltip.map(Into::into);
        self
    }

    /// Renders the one-line header without adding a surface, seam, or inset.
    pub fn view(self) -> Element<'a, Message> {
        let theme = theme::active();
        let title_style = theme.typography(TypographyRole::Heading);
        let icon_size = theme.control_metrics(theme::ControlSize::Sm).icon_size;

        let title: Element<'a, Message> = text(self.title)
            .font(title_style.font)
            .size(title_style.size)
            .line_height(title_style.line_height)
            .wrapping(text::Wrapping::None)
            .style(theme::text::style(TextRole::Primary))
            .into();
        let title: Element<'a, Message> = container(title).width(Length::Fill).clip(true).into();
        let title = match self.title_tooltip {
            Some(label) => tooltip::bottom(title, label),
            None => title,
        };

        let mut leading = row![]
            .spacing(theme.spacing().gap(theme::GapRole::Tight))
            .align_y(Alignment::Center)
            .width(Length::Fill);
        if let Some(icon_role) = self.icon {
            leading = leading.push(
                icon::role(icon_role)
                    .custom_size(icon_size)
                    .color(theme.text(TextRole::Secondary).color),
            );
        }
        leading = leading.push(title);

        let mut content = row![leading]
            .spacing(theme.spacing().gap(theme::GapRole::Content))
            .align_y(Alignment::Center)
            .width(Length::Fill);
        if let Some(trailing) = self.trailing {
            content = content.push(trailing);
        }

        container(content).width(Length::Fill).clip(true).into()
    }
}

impl<'a, Message> From<DocumentHeader<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(header: DocumentHeader<'a, Message>) -> Self {
        header.view()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_contract_is_sixteen_pixel_semibold() {
        let style = theme::active().typography(TypographyRole::Heading);

        assert_eq!(style.size, 16.0);
        assert_eq!(style.font.weight, iced::font::Weight::Semibold);
    }

    #[test]
    fn optional_content_is_kept_without_creating_shell_state() {
        let header = DocumentHeader::<()>::new("Document")
            .icon(IconRole::Folder)
            .title_tooltip("Full document title")
            .trailing(text("Healthy"));

        assert_eq!(header.title, "Document");
        assert_eq!(header.icon, Some(IconRole::Folder));
        assert_eq!(header.title_tooltip.as_deref(), Some("Full document title"));
        assert!(header.trailing.is_some());
    }
}
