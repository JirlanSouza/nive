use nive::prelude::*;
use nive::prelude::ui::{Operation, Resource, SettleOutcome, Settled};
use nive::widget::column;
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct Project {
    name: String,
}

#[derive(Debug, Clone, Default)]
struct SaveInput;

#[derive(nive::Inspect)]
struct AppState {
    #[inspect(sample = sample_projects)]
    projects: Resource<Vec<Project>>,
    #[inspect(input = sample_save_input)]
    save: Operation<SaveInput>,
}

struct DevtoolsExample {
    state: AppState,
}

#[derive(Debug, Clone)]
enum Message {
    Load,
    ProjectsSettled(Settled<Vec<Project>>),
}

impl Application for DevtoolsExample {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-devtools").name("Devtools")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (
            Self {
                state: AppState {
                    projects: Resource::idle(),
                    save: Operation::idle(),
                },
            },
            (),
        )
    }

    fn update(
        &mut self,
        context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::Load => {
                self
                    .state
                    .projects
                    .load(
                        context.app_scope(),
                        |_cancel| fetch_projects(),
                        Message::ProjectsSettled,
                    )
                    .into()
            }
            Message::ProjectsSettled(settled) => {
                let _outcome: SettleOutcome = self.state.projects.settle(settled);
                Effect::none()
            }
        }
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let data_view: Element<'_, Message> =
            if let Some(projects) = self.state.projects.value() {
                let items: Vec<Element<'_, Message>> =
                    projects.iter().map(|p| text(&p.name).into()).collect();
                column(items).spacing(8).into()
            } else if self.state.projects.is_loading() {
                text("Loading...").into()
            } else {
                text("No data loaded").into()
            };

        let content = column![
            text("Devtools Example").size(24),
            text("Press Cmd+Option+I (macOS) or Ctrl+Alt+I to open devtools"),
            text("Use the simulator to force Resource/Operation into any state"),
            button("Load Projects").on_press(Message::Load),
            data_view,
        ]
        .padding(40)
        .spacing(16);

        ScreenView::new(content)
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("Devtools")
    }
}

impl nive::DevtoolsApp for DevtoolsExample {
    type State = AppState;

    fn devtool_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }
}

async fn fetch_projects() -> std::result::Result<Vec<Project>, nive::UserFacingError> {
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(sample_projects())
}

fn sample_projects() -> Vec<Project> {
    vec![
        Project {
            name: "Alpha".to_string(),
        },
        Project {
            name: "Beta".to_string(),
        },
        Project {
            name: "Gamma".to_string(),
        },
    ]
}

fn sample_save_input() -> SaveInput {
    SaveInput
}

fn main() -> nive::Result {
    nive::run_with_devtools::<DevtoolsExample>()
}
