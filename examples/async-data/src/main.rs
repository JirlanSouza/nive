use nive::prelude::*;
use nive::prelude::ui::{Resource, SettleOutcome, Settled, UserFacingError};
use nive::widget::column;
use std::borrow::Cow;

#[derive(Debug, Clone)]
struct Project {
    name: String,
}

struct AsyncDataApp {
    projects: Resource<Vec<Project>>,
}

#[derive(Debug, Clone)]
enum Message {
    Load,
    Cancel,
    ProjectsSettled(Settled<Vec<Project>>),
}

impl Application for AsyncDataApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-async-data").name("Async Data")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self { projects: Resource::idle() }, ())
    }

    fn update(
        &mut self,
        context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::Load => {
                self.projects
                    .load(
                        context.app_scope(),
                        |_cancel| fetch_projects(),
                        Message::ProjectsSettled,
                    )
                    .into()
            }
            Message::Cancel => {
                Effect::cancel(self.projects.cancel())
            }
            Message::ProjectsSettled(settled) => {
                let _outcome: SettleOutcome = self.projects.settle(settled);
                Effect::none()
            }
        }
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let data_view: Element<'_, Message> = if let Some(projects) = self.projects.value() {
            let items: Vec<Element<'_, Message>> =
                projects.iter().map(|p| text(&p.name).into()).collect();
            column(items).spacing(8).into()
        } else if self.projects.is_loading() {
            text("Loading...").into()
        } else if let Some(error) = self.projects.error() {
            text(format!("Error: {}", error.summary())).into()
        } else {
            text("No data loaded").into()
        };

        let content = column![
            text("Async Data Example").size(24),
            text("Demonstrates Resource with automatic stale-request guarding"),
            button("Load Projects").on_press(Message::Load),
            button("Cancel Load").on_press_maybe(self.projects.is_loading().then_some(Message::Cancel)),
            text(if self.projects.is_refreshing() { "Refreshing retained data…" } else { "" }),
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
        Cow::Borrowed("Async Data")
    }
}

async fn fetch_projects() -> std::result::Result<Vec<Project>, UserFacingError> {
    std::thread::sleep(std::time::Duration::from_secs(2));
    Ok(vec![
        Project { name: "Alpha".to_string() },
        Project { name: "Beta".to_string() },
        Project { name: "Gamma".to_string() },
    ])
}

fn main() -> nive::Result {
    nive::run::<AsyncDataApp>()
}
