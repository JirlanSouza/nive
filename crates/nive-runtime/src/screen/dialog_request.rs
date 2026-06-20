use crate::DialogDismiss;

pub struct DialogRequest<'a, Message, Theme = nive_ui::Theme, Renderer = nive_ui::Renderer> {
    pub content: iced::Element<'a, Message, Theme, Renderer>,
    pub dismiss: DialogDismiss<Message>,
}

impl<'a, Message, Theme, Renderer> DialogRequest<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    pub fn new(content: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            dismiss: DialogDismiss::None,
        }
    }

    pub fn dismiss(mut self, dismiss: DialogDismiss<Message>) -> Self {
        self.dismiss = dismiss;
        self
    }

    pub fn dismiss_on_backdrop(self, message: Message) -> Self {
        self.dismiss(DialogDismiss::OnBackdrop(message))
    }

    pub fn dismiss_on_escape(self, message: Message) -> Self {
        self.dismiss(DialogDismiss::OnEscape(message))
    }

    pub fn dismiss_on_backdrop_or_escape(self, message: Message) -> Self {
        self.dismiss(DialogDismiss::OnBackdropOrEscape(message))
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
        }
    }
}
