use std::borrow::Cow;

use iced::{
    widget::{column, row, Space},
    Alignment, Length,
};

use crate::theme::{self, GapRole, TextRole, TypographyRole};
use crate::widgets::controls::button;
use crate::widgets::primitives::icon;
use crate::widgets::primitives::text as theme_text;
use crate::widgets::primitives::IconRole;
use crate::Element;

/// Canonical Dialog header: a required title, optional supporting
/// description, optional semantic leading icon, and an optional safe close
/// affordance.
///
/// The title always renders the complete [`TypographyRole::Heading`] style
/// with [`TextRole::Primary`]; a meaningful description renders the complete
/// [`TypographyRole::Body`] style with [`TextRole::Secondary`].
pub struct DialogHeader<'a, Message> {
    title: Cow<'a, str>,
    description: Option<Cow<'a, str>>,
    icon: Option<IconRole>,
    close: Option<DialogHeaderClose<'a, Message>>,
}

struct DialogHeaderClose<'a, Message> {
    name: Cow<'a, str>,
    message: Message,
}

impl<'a, Message> DialogHeader<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(title: impl Into<Cow<'a, str>>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
            close: None,
        }
    }

    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Adds a safe icon-only close affordance. `name` is the required
    /// localizable accessible name; `message` is published on activation.
    /// The affordance always represents safe Cancel/Close behavior and is
    /// absent from layout unless configured.
    pub fn close(mut self, name: impl Into<Cow<'a, str>>, message: Message) -> Self {
        self.close = Some(DialogHeaderClose {
            name: name.into(),
            message,
        });
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let gap = theme::gap(GapRole::Related);

        let mut title_column = column![theme_text::heading(self.title.into_owned())]
            .spacing(theme::gap(GapRole::Tight));

        if let Some(description) = self.description {
            title_column = title_column.push(theme_text::with_role(
                description.into_owned(),
                TypographyRole::Body,
                TextRole::Secondary,
            ));
        }

        let mut content = row![]
            .spacing(gap)
            .align_y(Alignment::Start)
            .width(Length::Fill);

        if let Some(icon_role) = self.icon {
            content = content.push(icon::role(icon_role));
        }

        content = content.push(title_column.width(Length::Fill));

        if let Some(close) = self.close {
            content = content
                .push(button::icon(IconRole::WindowClose, close.name).on_press(close.message));
        } else {
            content = content.push(Space::new().width(0.0).height(0.0));
        }

        content.into()
    }
}

impl<'a, Message> From<DialogHeader<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(header: DialogHeader<'a, Message>) -> Self {
        header.into_element()
    }
}

#[cfg(test)]
mod dialog_header_tests {
    use super::*;

    #[test]
    fn accepts_borrowed_and_owned_titles() {
        let borrowed: DialogHeader<'_, ()> = DialogHeader::new("Borrowed title");
        let owned: DialogHeader<'_, ()> = DialogHeader::new(String::from("Owned title"));

        assert_eq!(borrowed.title, Cow::Borrowed("Borrowed title"));
        assert_eq!(owned.title, Cow::Owned::<str>(String::from("Owned title")));
    }

    #[test]
    fn close_is_absent_when_unconfigured() {
        let header: DialogHeader<'_, ()> = DialogHeader::new("Title");

        assert!(header.close.is_none());
    }

    #[test]
    fn close_requires_a_localizable_name_and_message() {
        let header: DialogHeader<'_, &'static str> =
            DialogHeader::new("Title").close("Close dialog", "close");

        let close = header.close.expect("close should be configured");
        assert_eq!(close.name, Cow::Borrowed("Close dialog"));
        assert_eq!(close.message, "close");
    }

    #[test]
    fn icon_and_description_are_optional() {
        let header: DialogHeader<'_, ()> = DialogHeader::new("Title");
        assert!(header.icon.is_none());
        assert!(header.description.is_none());

        let configured: DialogHeader<'_, ()> = DialogHeader::new("Title")
            .description("Supporting copy")
            .icon(IconRole::DialogWarning);
        assert_eq!(configured.icon, Some(IconRole::DialogWarning));
        assert_eq!(
            configured.description,
            Some(Cow::Borrowed("Supporting copy"))
        );
    }
}
