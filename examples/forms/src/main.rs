use nive::prelude::*;
use nive::widget::{column, row};
use nive::prelude::ui::DialogRequest;

struct FormsApp {
    name: String,
    email: String,
    terms: CheckboxState,
    deployment: Option<Deployment>,
    account_tier: Option<AccountTier>,
    organization_query: String,
    organization: Option<Organization>,
    organization_results: OrganizationResultMode,
    organization_open: bool,
    organization_feedback: &'static str,
    sync_updates: bool,
    submit_attempted: bool,
    show_dialog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Deployment {
    Preview,
    Production,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AccountTier {
    Starter,
    Team,
    Enterprise,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Organization {
    NiveCore,
    NiveUi,
    NiveRuntime,
    CafeTelemetry,
}

impl Organization {
    const ALL: [Self; 4] = [
        Self::NiveCore,
        Self::NiveUi,
        Self::NiveRuntime,
        Self::CafeTelemetry,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::NiveCore => "Nive Core",
            Self::NiveUi => "Nive UI",
            Self::NiveRuntime => "Nive Runtime",
            Self::CafeTelemetry => "Café Telemetry",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrganizationResultMode {
    Suggestions,
    Loading,
    Empty,
    Error,
}

#[derive(Debug, Clone)]
enum Message {
    NameChanged(String),
    EmailChanged(String),
    TermsChanged(CheckboxState),
    DeploymentChanged(Deployment),
    AccountTierChanged(AccountTier),
    OrganizationQueryChanged(String),
    OrganizationSelected(Organization),
    OrganizationCleared,
    OrganizationSubmitted,
    OrganizationBlurred,
    OrganizationDismissed,
    OrganizationResultModeChanged(OrganizationResultMode),
    SyncUpdatesChanged(bool),
    Submit,
    OpenDialog,
    CloseDialog,
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FormValidation {
    name_error: &'static str,
    email_error: &'static str,
    group_error: &'static str,
    terms_error: &'static str,
    deployment_error: &'static str,
    account_tier_error: &'static str,
    organization_error: &'static str,
    valid: bool,
}

fn validate(
    name: &str,
    email: &str,
    terms: CheckboxState,
    deployment: Option<Deployment>,
    account_tier: Option<AccountTier>,
    organization: Option<Organization>,
    show_errors: bool,
) -> FormValidation {
    let name_missing = name.trim().is_empty();
    let email_missing = email.trim().is_empty();
    let email_malformed = !email_missing && !email.contains('@');
    let terms_missing = terms != CheckboxState::Checked;
    let deployment_missing = deployment.is_none();
    let account_tier_missing = account_tier.is_none();
    let organization_missing = organization.is_none();
    let valid = !name_missing
        && !email_missing
        && !email_malformed
        && !terms_missing
        && !deployment_missing
        && !account_tier_missing
        && !organization_missing;
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
            "Review the highlighted submission details"
        } else {
            ""
        },
        terms_error: if show_errors && terms_missing {
            "Confirm the submitted terms choice"
        } else {
            ""
        },
        deployment_error: if show_errors && deployment_missing {
            "Select one deployment preference"
        } else {
            ""
        },
        account_tier_error: if show_errors && account_tier_missing {
            "Select an account tier"
        } else {
            ""
        },
        organization_error: if show_errors && organization_missing {
            "Choose an organization from the results"
        } else {
            ""
        },
        valid,
    }
}

fn organization_results(
    mode: OrganizationResultMode,
    query: &str,
) -> AutocompleteResults<'static, Organization> {
    match mode {
        OrganizationResultMode::Loading => AutocompleteResults::Loading,
        OrganizationResultMode::Empty => {
            AutocompleteResults::empty("No organizations match this fixture")
        }
        OrganizationResultMode::Error => {
            AutocompleteResults::error("Could not retrieve organizations")
        }
        OrganizationResultMode::Suggestions => {
            let query = query.trim().to_lowercase();
            let suggestions = Organization::ALL
                .into_iter()
                .filter(|organization| {
                    query.is_empty() || organization.label().to_lowercase().contains(&query)
                })
                .map(|organization| {
                    let suggestion = AutocompleteSuggestion::new(
                        organization,
                        organization.label(),
                    )
                    .leading(IconRole::Identity);
                    match organization {
                        Organization::NiveCore => suggestion.trailing("Foundation"),
                        Organization::NiveUi => suggestion.trailing("Design system"),
                        Organization::NiveRuntime => suggestion.trailing("Runtime"),
                        Organization::CafeTelemetry => suggestion.trailing("Unicode fixture"),
                    }
                })
                .collect::<Vec<_>>();

            if suggestions.is_empty() {
                AutocompleteResults::empty("No organizations match this query")
            } else {
                AutocompleteResults::suggestions(suggestions)
            }
        }
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
                terms: CheckboxState::Mixed,
                deployment: None,
                account_tier: None,
                organization_query: String::new(),
                organization: None,
                organization_results: OrganizationResultMode::Suggestions,
                organization_open: false,
                organization_feedback: "Autocomplete: idle",
                sync_updates: true,
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
            Message::TermsChanged(value) => {
                self.terms = value;
                self.submit_attempted = false;
            }
            Message::DeploymentChanged(value) => {
                self.deployment = Some(value);
                self.submit_attempted = false;
            }
            Message::AccountTierChanged(value) => {
                self.account_tier = Some(value);
                self.submit_attempted = false;
            }
            Message::OrganizationQueryChanged(query) => {
                self.organization_query = query;
                self.organization = None;
                self.organization_results = OrganizationResultMode::Suggestions;
                self.organization_open = true;
                self.organization_feedback = "Autocomplete: query changed";
                self.submit_attempted = false;
            }
            Message::OrganizationSelected(organization) => {
                self.organization_query = organization.label().to_owned();
                self.organization = Some(organization);
                self.organization_open = false;
                self.organization_feedback = "Autocomplete: suggestion selected";
                self.submit_attempted = false;
            }
            Message::OrganizationCleared => {
                self.organization_query.clear();
                self.organization = None;
                self.organization_open = false;
                self.organization_feedback = "Autocomplete: query cleared";
                self.submit_attempted = false;
            }
            Message::OrganizationSubmitted => {
                self.organization_feedback = "Autocomplete: Enter submitted without selection";
            }
            Message::OrganizationBlurred => {
                self.organization_feedback = "Autocomplete: input blurred";
            }
            Message::OrganizationDismissed => {
                self.organization_open = false;
                self.organization_feedback = "Autocomplete: popup dismissed";
            }
            Message::OrganizationResultModeChanged(mode) => {
                self.organization_results = mode;
                self.organization_open = true;
                self.organization_feedback = match mode {
                    OrganizationResultMode::Suggestions => "Autocomplete: suggestions fixture",
                    OrganizationResultMode::Loading => "Autocomplete: loading fixture",
                    OrganizationResultMode::Empty => "Autocomplete: empty fixture",
                    OrganizationResultMode::Error => "Autocomplete: retrieval-error fixture",
                };
            }
            Message::SyncUpdatesChanged(value) => self.sync_updates = value,
            Message::Submit => {
                self.submit_attempted = true;
                if !validate(
                    &self.name,
                    &self.email,
                    self.terms,
                    self.deployment,
                    self.account_tier,
                    self.organization,
                    true,
                )
                .valid
                {
                    return Effect::none();
                }
                self.show_dialog = false;
                return Effect::toast(Toast::success(format!(
                    "Submitted: {} <{}> · {:?} · {:?} · {:?}",
                    self.name,
                    self.email,
                    self.deployment,
                    self.account_tier,
                    self.organization,
                )));
            }
            Message::OpenDialog => self.show_dialog = true,
            Message::CloseDialog => self.show_dialog = false,
            Message::Noop => {}
        }
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let validation = validate(
            &self.name,
            &self.email,
            self.terms,
            self.deployment,
            self.account_tier,
            self.organization,
            self.submit_attempted,
        );

        let account_tier = Select::new(
            vec![
                SelectOption::new(AccountTier::Starter, "Starter"),
                SelectOption::new(AccountTier::Team, "Team"),
                SelectOption::new(AccountTier::Enterprise, "Enterprise").disabled(true),
            ],
            self.account_tier,
        )
        .placeholder("Choose an account tier")
        .on_select(Message::AccountTierChanged)
        .on_open(Message::Noop)
        .on_close(Message::Noop);
        let organization = Autocomplete::new(
            &self.organization_query,
            self.organization,
            organization_results(self.organization_results, &self.organization_query),
        )
        .placeholder("Search organizations")
        .semantic_name("Organization search")
        .open(self.organization_open)
        .highlight(AutocompleteHighlight::First)
        .on_change(Message::OrganizationQueryChanged)
        .on_select(Message::OrganizationSelected)
        .on_clear(Message::OrganizationCleared)
        .on_submit(Message::OrganizationSubmitted)
        .on_blur(Message::OrganizationBlurred)
        .on_dismiss(Message::OrganizationDismissed);

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
            Field::new("Account tier", account_tier)
                .required("Required")
                .hint("A typed bounded choice; Enterprise is unavailable")
                .error(validation.account_tier_error)
                .reserve_support_line(true),
            Field::new("Organization", organization)
                .required("Required")
                .hint("Query, filtering, ordering, and selection are app-owned")
                .error(validation.organization_error)
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
            column![
                text(self.organization_feedback),
                row![
                    nive::widgets::button::tertiary("Suggestions")
                        .on_press(Message::OrganizationResultModeChanged(
                            OrganizationResultMode::Suggestions,
                        )),
                    nive::widgets::button::tertiary("Loading")
                        .on_press(Message::OrganizationResultModeChanged(
                            OrganizationResultMode::Loading,
                        )),
                    nive::widgets::button::tertiary("Empty")
                        .on_press(Message::OrganizationResultModeChanged(
                            OrganizationResultMode::Empty,
                        )),
                    nive::widgets::button::tertiary("Retrieval error")
                        .on_press(Message::OrganizationResultModeChanged(
                            OrganizationResultMode::Error,
                        )),
                ]
                .spacing(8)
                .wrap(),
            ]
            .spacing(8),
            row![
                Field::new(
                    "Empty Select fixture",
                    Select::<AccountTier, Message>::new(Vec::new(), None)
                        .placeholder("No tiers available")
                        .on_select(Message::AccountTierChanged),
                )
                .optional("Empty state")
                .hint("Canonical Select owns the empty popup"),
                Field::new(
                    "Managed organization",
                    Autocomplete::<Organization, Message>::new(
                        "Nive Runtime",
                        Some(Organization::NiveRuntime),
                        organization_results(OrganizationResultMode::Suggestions, "Nive"),
                    ),
                )
                .optional("Disabled")
                .hint("Field propagates disabled context to the control")
                .disabled(true),
            ]
            .spacing(12)
            .wrap(),
            Checkbox::new("I confirm the submitted account terms", self.terms)
                .description("This choice is validated when the form is submitted")
                .error(validation.terms_error)
                .fill_width()
                .on_toggle(Message::TermsChanged),
            RadioGroup::new(
                "Deployment preference",
                self.deployment,
                [
                    RadioOption::new(Deployment::Preview, "Preview environment")
                        .description("Validate changes before production"),
                    RadioOption::new(Deployment::Production, "Production")
                        .description("Apply after submission"),
                    RadioOption::new(Deployment::None, "No deployment"),
                ],
            )
            .required("Required")
            .description("Choose exactly one submitted destination")
            .error(validation.deployment_error)
            .layout(RadioGroupLayout::HorizontalWrap)
            .on_select(Message::DeploymentChanged),
            Switch::setting("Synchronize account updates", self.sync_updates)
                .description("This immediate preference is not deferred until submission")
                .on_toggle(Message::SyncUpdatesChanged),
            row![
                Switch::<Message>::inline("Display-only enabled setting", true),
                Switch::<Message>::inline("Disabled enabled setting", true).disabled(true),
            ]
            .spacing(16),
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
            view.dialog(self.confirmation_dialog_request())
        } else {
            view
        }
    }
}

impl FormsApp {
    fn confirmation_dialog_request(&self) -> DialogRequest<'_, Message> {
        let dialog_body = text(format!(
            "Name: {}\nEmail: {}\nTerms: {:?}\nDeployment: {:?}\nTier: {:?}\nOrganization: {:?}",
            self.name,
            self.email,
            self.terms,
            self.deployment,
            self.account_tier,
            self.organization,
        ));

        let dialog = Dialog::new(dialog_body)
            .size(DialogSize::Md)
            .header(
                DialogHeader::new("Confirm submission")
                    .description("Review the current form values before they are submitted."),
            )
            .footer(DialogActionFooter::with_one(
                DialogAction::cancel("Cancel", Message::CloseDialog),
                DialogTerminalAction::primary("Confirm", Message::Submit),
            ));

        // No backdrop dismissal: an outside click must not silently discard
        // edited form data. Escape follows the same safe Cancel path as the
        // footer's Cancel action.
        DialogRequest::new(dialog).dismiss_on_escape(Message::CloseDialog)
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
        let validation = validate(
            "",
            "",
            CheckboxState::Unchecked,
            None,
            None,
            None,
            true,
        );

        assert!(!validation.valid);
        assert_eq!(validation.name_error, "Enter your name");
        assert_eq!(validation.email_error, "Enter your email");
        assert!(!validation.group_error.is_empty());
        assert!(!validation.terms_error.is_empty());
        assert!(!validation.deployment_error.is_empty());
        assert!(!validation.account_tier_error.is_empty());
        assert!(!validation.organization_error.is_empty());
    }

    #[test]
    fn malformed_email_is_deterministic_before_and_after_submit() {
        let editing = validate(
            "Ada",
            "invalid",
            CheckboxState::Checked,
            Some(Deployment::Preview),
            Some(AccountTier::Team),
            Some(Organization::NiveCore),
            false,
        );
        let submitted = validate(
            "Ada",
            "invalid",
            CheckboxState::Checked,
            Some(Deployment::Preview),
            Some(AccountTier::Team),
            Some(Organization::NiveCore),
            true,
        );

        assert_eq!(editing.email_error, "Enter a valid email address");
        assert!(editing.group_error.is_empty());
        assert_eq!(submitted.email_error, editing.email_error);
        assert!(!submitted.group_error.is_empty());
    }

    #[test]
    fn corrected_values_are_valid_and_clear_all_support_errors() {
        let validation = validate(
            "Ada Lovelace",
            "ada@example.com",
            CheckboxState::Checked,
            Some(Deployment::Preview),
            Some(AccountTier::Team),
            Some(Organization::NiveCore),
            true,
        );

        assert!(validation.valid);
        assert_eq!(validation.name_error, "");
        assert_eq!(validation.email_error, "");
        assert_eq!(validation.group_error, "");
        assert_eq!(validation.terms_error, "");
        assert_eq!(validation.deployment_error, "");
        assert_eq!(validation.account_tier_error, "");
        assert_eq!(validation.organization_error, "");
    }

    #[test]
    fn organization_results_are_atomic_filtered_and_ordered() {
        let AutocompleteResults::Suggestions(all) =
            organization_results(OrganizationResultMode::Suggestions, "")
        else {
            panic!("expected suggestions");
        };
        assert_eq!(all.len(), Organization::ALL.len());
        assert_eq!(all[0].value(), &Organization::NiveCore);
        assert_eq!(all[3].value(), &Organization::CafeTelemetry);

        let AutocompleteResults::Suggestions(filtered) =
            organization_results(OrganizationResultMode::Suggestions, "runtime")
        else {
            panic!("expected filtered suggestions");
        };
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value(), &Organization::NiveRuntime);

        assert!(matches!(
            organization_results(OrganizationResultMode::Suggestions, "missing"),
            AutocompleteResults::Empty(_)
        ));
    }

    #[test]
    fn result_fixtures_do_not_manufacture_field_validation() {
        assert!(matches!(
            organization_results(OrganizationResultMode::Loading, "nive"),
            AutocompleteResults::Loading
        ));
        assert!(matches!(
            organization_results(OrganizationResultMode::Empty, "nive"),
            AutocompleteResults::Empty(_)
        ));
        assert!(matches!(
            organization_results(OrganizationResultMode::Error, "nive"),
            AutocompleteResults::Error(_)
        ));

        let validation = validate(
            "Ada",
            "ada@example.com",
            CheckboxState::Checked,
            Some(Deployment::Preview),
            Some(AccountTier::Team),
            None,
            false,
        );
        assert_eq!(validation.organization_error, "");
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

    fn fixture_app() -> FormsApp {
        FormsApp {
            name: "Ada Lovelace".to_string(),
            email: "ada@example.com".to_string(),
            terms: CheckboxState::Checked,
            deployment: Some(Deployment::Preview),
            account_tier: Some(AccountTier::Team),
            organization_query: String::new(),
            organization: Some(Organization::NiveCore),
            organization_results: OrganizationResultMode::Suggestions,
            organization_open: false,
            organization_feedback: "Autocomplete: idle",
            sync_updates: true,
            submit_attempted: false,
            show_dialog: true,
        }
    }

    #[test]
    fn confirmation_dialog_does_not_enable_backdrop_dismissal_for_edited_data() {
        let app = fixture_app();
        let request = app.confirmation_dialog_request();

        assert!(request.dismiss_policy().on_backdrop().is_none());
    }

    #[test]
    fn confirmation_dialog_escape_follows_the_same_route_as_the_footer_cancel() {
        let app = fixture_app();
        let request = app.confirmation_dialog_request();

        assert!(matches!(
            request.dismiss_policy().on_escape(),
            Some(Message::CloseDialog)
        ));
    }

    #[test]
    fn confirmation_dialog_first_focus_lands_inside_the_dialog_not_the_invoker() {
        // The dialog declares no explicit initial-focus target, so it uses
        // `DialogInitialFocus::First`, which resolves to the footer's
        // Cancel action here (the body is static review text with nothing
        // focusable) rather than leaving focus on the base "Preview
        // dialog" invoker button.
        let app = fixture_app();
        let request = app.confirmation_dialog_request();

        assert_eq!(request.initial_focus_policy(), &DialogInitialFocus::First);
    }
}
