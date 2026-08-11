use iced::{event, Event, Size};
use iced_runtime::{user_interface, UserInterface};

pub(super) struct RuntimeUpdate<Message> {
    pub(super) messages: Vec<Message>,
    pub(super) status: event::Status,
}

pub(super) struct RuntimeUiState {
    cache: user_interface::Cache,
    renderer: nive_ui::Renderer,
    cursor: iced::advanced::mouse::Cursor,
    bounds: Size,
}

impl RuntimeUiState {
    pub(super) fn new(bounds: Size) -> Self {
        Self {
            cache: user_interface::Cache::new(),
            renderer: iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                iced::Font::default(),
                iced::Pixels(14.0),
            )),
            cursor: iced::advanced::mouse::Cursor::Unavailable,
            bounds,
        }
    }

    pub(super) fn dispatch<Message>(
        &mut self,
        element: nive_ui::Element<'_, Message>,
        event: Event,
    ) -> RuntimeUpdate<Message> {
        let mut interface = UserInterface::build(
            element,
            self.bounds,
            std::mem::take(&mut self.cache),
            &mut self.renderer,
        );
        let mut clipboard = iced::advanced::clipboard::Null;
        let mut messages = Vec::new();
        let (_, statuses) = interface.update(
            std::slice::from_ref(&event),
            self.cursor,
            &mut self.renderer,
            &mut clipboard,
            &mut messages,
        );
        self.cache = interface.into_cache();

        let status = statuses
            .into_iter()
            .next()
            .expect("one event produces one status");

        RuntimeUpdate { messages, status }
    }
}
