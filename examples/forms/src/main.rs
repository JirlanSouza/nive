use nive::prelude::*;
use nive::widget::{column, row};
use nive::prelude::ui::DialogRequest;

struct FormsApp {
    name: String,
    email: String,
    show_dialog: bool,
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    EmailChanged(String),
    Submit,
    OpenDialog,
    CloseDialog,
}

impl Application for FormsApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-forms").name("Forms")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (
            Self {
                name: String::new(),
                email: String::new(),
                show_dialog: false,
            },
            (),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::NameChanged(value) => self.name = value,
            Message::EmailChanged(value) => self.email = value,
            Message::Submit => {
                return Effect::toast(Toast::success(format!(
                    "Submitted: {} <{}>",
                    self.name, self.email
                )));
            }
            Message::OpenDialog => self.show_dialog = true,
            Message::CloseDialog => self.show_dialog = false,
        }
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let email_invalid = !self.email.is_empty() && !self.email.contains('@');

        let form = column![
            text("Contact Form").size(24),
            Field::new(
                Input::new("Enter your name", &self.name)
                    .on_input(Message::NameChanged)
            )
            .label("Name"),
            Field::new(
                Input::new("Enter your email", &self.email)
                    .on_input(Message::EmailChanged)
                    .validation(if email_invalid {
                        FieldValidation::Invalid
                    } else {
                        FieldValidation::Valid
                    })
            )
            .label("Email")
            .error(if email_invalid { "Invalid email" } else { "" }),
            row![
                button("Submit").on_press(Message::Submit),
                button("Open Dialog").on_press(Message::OpenDialog),
            ]
            .spacing(12),
        ]
        .padding(40)
        .spacing(16);

        let view = ScreenView::new(form);

        if self.show_dialog {
            let dialog_content = column![
                text("Confirm Submission"),
                text(format!("Name: {}\nEmail: {}", self.name, self.email)),
                row![
                    button("Cancel").on_press(Message::CloseDialog),
                    button("Confirm")
                        .on_press(Message::Submit)
                        .on_press(Message::CloseDialog),
                ]
                .spacing(12),
            ]
            .padding(24)
            .spacing(12);

            view.dialog(
                DialogRequest::new(dialog_content)
                    .dismiss_on_backdrop(Message::CloseDialog)
                    .dismiss_on_escape(Message::CloseDialog),
            )
        } else {
            view
        }
    }
}

fn main() -> nive::Result {
    nive::run::<FormsApp>()
}
