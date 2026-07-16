use nive::prelude::*;
use nive::widget::{column, row};
use nive::prelude::ui::DialogRequest;

struct FormsApp {
    name: String,
    email: String,
    submit_attempted: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormValidation {
    name_error: &'static str,
    email_error: &'static str,
    group_error: &'static str,
    valid: bool,
}

fn validate(name: &str, email: &str, show_errors: bool) -> FormValidation {
    let name_missing = name.trim().is_empty();
    let email_missing = email.trim().is_empty();
    let email_malformed = !email_missing && !email.contains('@');
    let valid = !name_missing && !email_missing && !email_malformed;
    let name_error = if show_errors && name_missing {
        "Enter your name"
    } else {
        ""
    };
    let email_error = if show_errors && email_missing {
        "Enter your email"
    } else if email_malformed {
        "Enter a valid email address"
    } else {
        ""
    };

    FormValidation {
        name_error,
        email_error,
        group_error: if show_errors && !valid {
            "Review the highlighted contact details"
        } else {
            ""
        },
        valid,
    }
}

impl Application for FormsApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-forms")
            .name("Forms")
            .window((), WindowSpec::app().min_size(480.0, 480.0))
            .initial_window(())
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (
            Self {
                name: String::new(),
                email: String::new(),
                submit_attempted: false,
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
            Message::NameChanged(value) => {
                self.name = value;
                self.submit_attempted = false;
            }
            Message::EmailChanged(value) => {
                self.email = value;
                self.submit_attempted = false;
            }
            Message::Submit => {
                self.submit_attempted = true;
                if !validate(&self.name, &self.email, true).valid {
                    return Effect::none();
                }
                self.show_dialog = false;
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
        let validation = validate(&self.name, &self.email, self.submit_attempted);

        let contact_fields = vec![
            Field::new(
                "Name",
                Input::new("Enter your name", &self.name).on_change(Message::NameChanged),
            )
            .required("Required")
            .hint("The name shown on your account")
            .error(validation.name_error)
            .reserve_support_line(true),
            Field::new(
                "Email",
                InputGroup::new(
                    Input::new("name@example.com", &self.email)
                        .on_change(Message::EmailChanged),
                )
                .semantic_icon(IconRole::Identity)
                .clear_action(
                    nive::widgets::button::icon(IconRole::WindowClose, "Clear email")
                        .on_press(Message::EmailChanged(String::new())),
                ),
            )
            .required("Required")
            .hint("Used for submission confirmation")
            .error(validation.email_error)
            .reserve_support_line(true),
            Field::new(
                "Account reference",
                Input::new("Reference", "ACC-1042").read_only(true),
            )
            .optional("Read only")
            .hint("Selectable and copyable, but not editable")
            .reserve_support_line(true),
            Field::new(
                "Provisioning key",
                Input::new("Unavailable", "Created after approval").disabled(true),
            )
            .optional("Disabled")
            .hint("This value becomes available after approval")
            .reserve_support_line(true),
        ];

        let form = column![
            text("Contact Form").size(24),
            FieldGroup::new("Contact details", contact_fields)
                .description("Provide the identity used for this submission")
                .error(validation.group_error)
                .md()
                .wrap(260.0),
            row![
                nive::widgets::button::primary("Submit").on_press(Message::Submit),
                nive::widgets::button::secondary("Preview dialog")
                    .on_press(Message::OpenDialog),
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
                    nive::widgets::button::secondary("Cancel")
                        .on_press(Message::CloseDialog),
                    nive::widgets::button::primary("Confirm").on_press(Message::Submit),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_submit_exposes_field_and_group_errors() {
        let validation = validate("", "", true);

        assert!(!validation.valid);
        assert_eq!(validation.name_error, "Enter your name");
        assert_eq!(validation.email_error, "Enter your email");
        assert!(!validation.group_error.is_empty());
    }

    #[test]
    fn malformed_email_is_deterministic_before_and_after_submit() {
        let editing = validate("Ada", "invalid", false);
        let submitted = validate("Ada", "invalid", true);

        assert_eq!(editing.email_error, "Enter a valid email address");
        assert!(editing.group_error.is_empty());
        assert_eq!(submitted.email_error, editing.email_error);
        assert!(!submitted.group_error.is_empty());
    }

    #[test]
    fn corrected_values_are_valid_and_clear_all_support_errors() {
        let validation = validate("Ada Lovelace", "ada@example.com", true);

        assert!(validation.valid);
        assert_eq!(validation.name_error, "");
        assert_eq!(validation.email_error, "");
        assert_eq!(validation.group_error, "");
    }

    #[test]
    fn canonical_group_builds_with_read_only_disabled_and_wrapping_fields() {
        let read_only = Input::<Message>::new("Reference", "ACC-1042").read_only(true);
        assert!(read_only.is_read_only());
        assert!(!read_only.is_disabled());

        let disabled = Input::<Message>::new("Unavailable", "Pending").disabled(true);
        assert!(disabled.is_disabled());

        let fields = vec![
            Field::new("Account reference", read_only)
                .optional("Read only")
                .reserve_support_line(true),
            Field::new("Provisioning key", disabled)
                .optional("Disabled")
                .reserve_support_line(true),
        ];
        let _: Element<'_, Message> = FieldGroup::new("Contact details", fields)
            .description("Canonical form smoke fixture")
            .md()
            .wrap(260.0)
            .into();
    }

    #[test]
    fn forms_window_can_reach_the_single_column_review_width() {
        let config = FormsApp::config();
        let [window] = config.windows() else {
            panic!("forms must register one review window");
        };

        assert_eq!(window.spec.min_size, Some(Size::new(480.0, 480.0)));
        assert_eq!(config.initial_windows(), &[()]);
    }
}
