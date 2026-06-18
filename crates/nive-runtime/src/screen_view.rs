use crate::DialogRequest;

pub struct ScreenView<'a, Message, Theme = nive_ui::Theme, Renderer = nive_ui::Renderer> {
    pub content: iced::Element<'a, Message, Theme, Renderer>,
    pub dialog: Option<DialogRequest<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> ScreenView<'a, Message, Theme, Renderer>
where
    Message: 'a,
{
    pub fn new(content: impl Into<iced::Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
        }
    }

    pub fn dialog(mut self, dialog: DialogRequest<'a, Message, Theme, Renderer>) -> Self {
        self.dialog = Some(dialog);
        self
    }

    pub fn dialog_maybe(
        mut self,
        dialog: Option<DialogRequest<'a, Message, Theme, Renderer>>,
    ) -> Self {
        self.dialog = dialog;
        self
    }

    pub fn map<T: 'a>(
        self,
        map_message: impl Fn(Message) -> T + Copy + 'a,
    ) -> ScreenView<'a, T, Theme, Renderer>
    where
        Theme: 'a,
        Renderer: iced::advanced::Renderer + 'a,
    {
        ScreenView {
            content: self.content.map(map_message),
            dialog: self.dialog.map(|dialog| dialog.map(map_message)),
        }
    }
}

impl<'a, Message> ScreenView<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn has_dialog(&self) -> bool {
        self.dialog.is_some()
    }

    pub fn into_element(self) -> nive_ui::Element<'a, Message> {
        match self.dialog {
            Some(dialog) => {
                let on_backdrop = dialog.dismiss.on_backdrop();
                let on_escape = dialog.dismiss.on_escape();

                nive_ui::DialogHost::new(self.content)
                    .dialog(dialog.content, on_backdrop, on_escape)
                    .into()
            }
            None => nive_ui::DialogHost::new(self.content).into(),
        }
    }
}

#[cfg(test)]
mod screen_view_tests {
    use super::*;
    use crate::DialogDismiss;
    use iced::widget::text;

    #[test]
    fn map_preserves_dialog_and_maps_dismiss_message() {
        let screen: ScreenView<'_, u8> = ScreenView::new(text("content")).dialog(
            DialogRequest::new(text("dialog")).dismiss(DialogDismiss::OnBackdropOrEscape(3_u8)),
        );

        let mapped = screen.map(|message| message + 1);

        match mapped.dialog {
            Some(dialog) => assert_eq!(dialog.dismiss, DialogDismiss::OnBackdropOrEscape(4)),
            None => panic!("dialog must exist"),
        }
    }
}
