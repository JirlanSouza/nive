use crate::{theme::ToneRole, widgets::Badge, Element};

pub struct VersionBadge<'a, Message> {
    label: &'a str,
    _marker: std::marker::PhantomData<Message>,
}

impl<'a, Message> VersionBadge<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            _marker: std::marker::PhantomData,
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        Badge::new(self.label).tone(ToneRole::Neutral).xs().into()
    }
}

impl<'a, Message> From<VersionBadge<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(badge: VersionBadge<'a, Message>) -> Self {
        badge.into_element()
    }
}
