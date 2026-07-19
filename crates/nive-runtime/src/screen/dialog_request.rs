use nive_ui::widgets::overlays::DialogInitialFocus;

use crate::DialogDismiss;

/// A declarative modal dialog request for [`crate::ScreenView`].
///
/// Fields are private; use the builders and accessors below. `dismiss(policy)`
/// replaces the complete dismissal policy, while `dismiss_on_backdrop(...)`
/// and `dismiss_on_escape(...)` each replace only their own route and
/// preserve the other. [`map`](Self::map) maps content and every configured
/// dismissal message exactly once while preserving initial focus and
/// identity. `id(...)` names a stable declarative session: rebuilding an
/// open request with the same (or no) id continues the session without
/// recapturing the invoker or repeating initial focus; a changed id
/// replaces the workflow step and re-runs initial focus without publishing
/// dismissal or losing the original invoker.
///
/// Content, dismissal policy, initial focus, and identity cannot be
/// constructed or mutated by direct field access:
///
/// ```compile_fail
/// use nive_runtime::DialogRequest;
///
/// let request: DialogRequest<'_, ()> = DialogRequest {
///     content: iced::widget::text("content").into(),
///     dismiss: Default::default(),
/// };
/// ```
///
/// ```compile_fail
/// use nive_runtime::DialogRequest;
///
/// let mut request: DialogRequest<'_, ()> = DialogRequest::new(iced::widget::text("content"));
/// request.dismiss = Default::default();
/// ```
pub struct DialogRequest<'a, Message, Theme = nive_ui::Theme, Renderer = nive_ui::Renderer> {
    content: iced::Element<'a, Message, Theme, Renderer>,
    dismiss: DialogDismiss<Message>,
    initial_focus: DialogInitialFocus,
    id: Option<iced::widget::Id>,
}

impl<'a, Message, Theme, Renderer> DialogRequest<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    pub fn new(content: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            dismiss: DialogDismiss::none(),
            initial_focus: DialogInitialFocus::default(),
            id: None,
        }
    }

    /// Replaces the complete dismissal policy.
    pub fn dismiss(mut self, dismiss: DialogDismiss<Message>) -> Self {
        self.dismiss = dismiss;
        self
    }

    /// Replaces only the backdrop dismissal route, preserving Escape.
    pub fn dismiss_on_backdrop(mut self, message: Message) -> Self {
        self.dismiss = self.dismiss.with_backdrop(message);
        self
    }

    /// Replaces only the Escape dismissal route, preserving backdrop.
    pub fn dismiss_on_escape(mut self, message: Message) -> Self {
        self.dismiss = self.dismiss.with_escape(message);
        self
    }

    /// Sets both routes to the same message.
    pub fn dismiss_on_backdrop_or_escape(self, message: Message) -> Self
    where
        Message: Clone,
    {
        self.dismiss(DialogDismiss::backdrop_or_escape(message))
    }

    /// Sets where focus initially lands when the dialog opens.
    pub fn initial_focus(mut self, initial_focus: DialogInitialFocus) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    /// Sets a stable declarative session identity. Rebuilding an open
    /// request with the same (or no) id continues the current modal
    /// session; a changed id replaces the workflow step.
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn content(&self) -> &iced::Element<'a, Message, Theme, Renderer> {
        &self.content
    }

    pub fn dismiss_policy(&self) -> &DialogDismiss<Message> {
        &self.dismiss
    }

    pub fn initial_focus_policy(&self) -> &DialogInitialFocus {
        &self.initial_focus
    }

    pub fn identity(&self) -> Option<&iced::widget::Id> {
        self.id.as_ref()
    }

    /// Splits the request into its independent parts for hosting, without
    /// exposing a public struct-literal construction path.
    pub(crate) fn into_parts(
        self,
    ) -> (
        iced::Element<'a, Message, Theme, Renderer>,
        DialogDismiss<Message>,
        DialogInitialFocus,
        Option<iced::widget::Id>,
    ) {
        (self.content, self.dismiss, self.initial_focus, self.id)
    }

    pub fn map<T: 'a>(
        self,
        map_message: impl Fn(Message) -> T + Copy + 'a,
    ) -> DialogRequest<'a, T, Theme, Renderer>
    where
        Theme: 'a,
        Renderer: iced::advanced::Renderer + 'a,
    {
        DialogRequest {
            content: self.content.map(map_message),
            dismiss: self.dismiss.map(map_message),
            initial_focus: self.initial_focus,
            id: self.id,
        }
    }
}

#[cfg(test)]
mod dialog_request_tests {
    use super::*;
    use iced::widget::text;

    #[test]
    fn new_has_no_dismissal_and_first_initial_focus_and_no_identity() {
        let request: DialogRequest<'_, u8> = DialogRequest::new(text("content"));

        assert_eq!(request.dismiss_policy(), &DialogDismiss::none());
        assert_eq!(request.initial_focus_policy(), &DialogInitialFocus::First);
        assert_eq!(request.identity(), None);
    }

    #[test]
    fn independent_builders_preserve_both_routes() {
        let request: DialogRequest<'_, &str> = DialogRequest::new(text("content"))
            .dismiss_on_backdrop("backdrop")
            .dismiss_on_escape("cancel");

        assert_eq!(request.dismiss_policy().on_backdrop(), Some("backdrop"));
        assert_eq!(request.dismiss_policy().on_escape(), Some("cancel"));
    }

    #[test]
    fn dismiss_replaces_the_complete_policy() {
        let request: DialogRequest<'_, &str> = DialogRequest::new(text("content"))
            .dismiss_on_backdrop("backdrop")
            .dismiss_on_escape("cancel")
            .dismiss(DialogDismiss::escape("only-escape"));

        assert_eq!(request.dismiss_policy().on_backdrop(), None);
        assert_eq!(request.dismiss_policy().on_escape(), Some("only-escape"));
    }

    #[test]
    fn id_sets_the_declarative_session_identity() {
        let id = iced::widget::Id::new("workflow-step");
        let request: DialogRequest<'_, u8> = DialogRequest::new(text("content")).id(id.clone());

        assert_eq!(request.identity(), Some(&id));
    }

    #[test]
    fn map_maps_content_and_dismissal_while_preserving_focus_and_identity() {
        let id = iced::widget::Id::new("workflow-step");
        let request: DialogRequest<'_, u8> = DialogRequest::new(text("content"))
            .dismiss_on_backdrop(1_u8)
            .dismiss_on_escape(2_u8)
            .initial_focus(DialogInitialFocus::Target(iced::widget::Id::new("field")))
            .id(id.clone());

        let mapped = request.map(|value| value * 10);

        assert_eq!(mapped.dismiss_policy().on_backdrop(), Some(10));
        assert_eq!(mapped.dismiss_policy().on_escape(), Some(20));
        assert_eq!(
            mapped.initial_focus_policy(),
            &DialogInitialFocus::Target(iced::widget::Id::new("field"))
        );
        assert_eq!(mapped.identity(), Some(&id));
    }
}
