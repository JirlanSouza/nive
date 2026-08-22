use std::borrow::Cow;

use nive::prelude::*;
use nive::prelude::ui::{
    CancelSignal, Operation, Request, Resource, ScopeId, SettleOutcome, Settled, TaskScope,
    UserFacingError,
};
use nive::widget::column;

#[derive(Debug, Clone)]
struct Project {
    name: String,
}

#[derive(Debug)]
struct LoadProjects;

#[derive(Debug, Clone)]
struct Draft(String);

#[derive(Debug)]
struct SaveDraft(String);

#[derive(Debug, Clone)]
struct SavedRevision(u64);

#[derive(Clone)]
struct Services;

impl Services {
    async fn load(
        &self,
        _intent: LoadProjects,
        cancel: CancelSignal,
    ) -> std::result::Result<Vec<Project>, UserFacingError> {
        if cancel.is_cancelled() {
            return Err(UserFacingError::custom("load", "Load stopped"));
        }
        Ok(vec![Project {
            name: String::from("Tracked request"),
        }])
    }

    async fn save(
        &self,
        intent: SaveDraft,
        cancel: CancelSignal,
    ) -> std::result::Result<SavedRevision, UserFacingError> {
        if cancel.is_cancelled() {
            return Err(UserFacingError::custom("save", "Save stopped"));
        }
        Ok(SavedRevision(intent.0.len() as u64))
    }
}

struct ProjectScreen {
    scope: TaskScope,
    projects: Resource<Vec<Project>>,
}

struct AsyncActionsApp {
    screen: Option<ProjectScreen>,
    save: Operation<Draft, SavedRevision>,
    services: Services,
    last_revision: Option<u64>,
}

#[derive(Debug, Clone)]
enum Message {
    Load,
    CancelLoad,
    OpenScreen,
    CloseScreen,
    Save,
    ProjectsSettled(Settled<Vec<Project>>),
    SaveSettled(Settled<SavedRevision>),
}

enum Action {
    None,
    Load(Request<Vec<Project>, LoadProjects>),
    Save(Request<SavedRevision, SaveDraft>),
}

impl AsyncActionsApp {
    fn reduce(&mut self, scope: ScopeId, message: Message) -> Action {
        match message {
            Message::Load => self
                .screen
                .as_mut()
                .map(|screen| {
                    Action::Load(screen.projects.request_with(screen.scope.id(), LoadProjects))
                })
                .unwrap_or(Action::None),
            Message::CancelLoad => Action::None,
            Message::OpenScreen => {
                if self.screen.is_none() {
                    self.screen = Some(ProjectScreen {
                        scope: scope.child("projects-screen"),
                        projects: Resource::idle(),
                    });
                }
                Action::None
            }
            Message::CloseScreen => {
                self.screen = None;
                Action::None
            }
            Message::Save => self
                .save
                .request_with(
                    scope,
                    Draft(String::from("example draft")),
                    SaveDraft(String::from("example draft")),
                )
                .map(Action::Save)
                .unwrap_or(Action::None),
            Message::ProjectsSettled(settled) => {
                if let Some(screen) = self.screen.as_mut() {
                    let _outcome = screen.projects.settle(settled);
                }
                Action::None
            }
            Message::SaveSettled(settled) => {
                if let SettleOutcome::Succeeded((draft, revision)) = self.save.settle(settled) {
                    self.last_revision = Some(revision.0 + draft.0.len() as u64);
                }
                Action::None
            }
        }
    }

    fn run_action(&self, action: Action) -> Effect<Message, ()> {
        match action {
            Action::None => Effect::none(),
            Action::Load(request) => {
                let services = self.services.clone();
                request
                    .perform(
                        move |intent, cancel| async move { services.load(intent, cancel).await },
                        Message::ProjectsSettled,
                    )
                    .into()
            }
            Action::Save(request) => {
                let services = self.services.clone();
                request
                    .perform(
                        move |intent, cancel| async move { services.save(intent, cancel).await },
                        Message::SaveSettled,
                    )
                    .into()
            }
        }
    }
}

impl Application for AsyncActionsApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-async-actions").name("Async Actions")
    }

    fn init(
        context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (
            Self {
                screen: Some(ProjectScreen {
                    scope: context.app_scope().child("projects-screen"),
                    projects: Resource::idle(),
                }),
                save: Operation::idle(),
                services: Services,
                last_revision: None,
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
        if matches!(message, Message::CancelLoad) {
            return Effect::cancel(
                self.screen
                    .as_mut()
                    .and_then(|screen| screen.projects.cancel()),
            );
        }
        let action = self.reduce(context.app_scope(), message);
        self.run_action(action)
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let project = self
            .screen
            .as_ref()
            .and_then(|screen| screen.projects.value())
            .and_then(|projects| projects.first())
            .map(|project| project.name.as_str())
            .unwrap_or("No active project screen or data");
        let revision = self
            .last_revision
            .map(|revision| format!("Saved revision {revision}"))
            .unwrap_or_else(|| String::from("Not saved"));
        ScreenView::new(
            column![
                text("Reducer-friendly async actions").size(24),
                text(project),
                text(revision),
                button("Load").on_press(Message::Load),
                button("Cancel load").on_press(Message::CancelLoad),
                button("Open project screen").on_press(Message::OpenScreen),
                button("Close project screen").on_press(Message::CloseScreen),
                button("Save once").on_press(Message::Save),
            ]
            .padding(40)
            .spacing(16),
        )
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("Async Actions")
    }
}

fn main() -> nive::Result {
    nive::run::<AsyncActionsApp>()
}
