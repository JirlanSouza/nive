use std::{borrow::Cow, marker::PhantomData};

use crate::{widgets::MetadataTag, Element};

/// One-release migration wrapper over [`MetadataTag::code`].
#[deprecated(note = "use MetadataTag::code for versions and technical identifiers")]
pub struct VersionBadge<'a, Message> {
    value: Cow<'a, str>,
    _marker: PhantomData<Message>,
}

#[allow(deprecated)]
impl<'a, Message> VersionBadge<'a, Message>
where
    Message: 'a,
{
    /// Retains the exact value and delegates directly to `MetadataTag::code`.
    pub fn new(value: impl Into<Cow<'a, str>>) -> Self {
        Self {
            value: value.into(),
            _marker: PhantomData,
        }
    }

    fn into_element(self) -> Element<'a, Message> {
        MetadataTag::code(self.value).into()
    }
}

#[allow(deprecated)]
impl<'a, Message> From<VersionBadge<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(badge: VersionBadge<'a, Message>) -> Self {
        badge.into_element()
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use super::*;

    #[test]
    fn wrapper_retains_borrowed_and_owned_exact_values() {
        let borrowed = VersionBadge::<()>::new("v1.4.0-beta.2+build.7");
        let owned = VersionBadge::<()>::new(String::from("v1.4.0-beta.2+build.7"));
        assert_eq!(borrowed.value, owned.value);
        assert_eq!(borrowed.value, "v1.4.0-beta.2+build.7");
    }
}
