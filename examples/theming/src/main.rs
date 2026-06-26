use nive::prelude::*;
use nive::widget::{column, row};
use std::borrow::Cow;

struct ThemingApp {
    preference: ThemePreference,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    System,
    Light,
    Dark,
}

impl Application for ThemingApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-theming").name("Theming")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
        (
            Self {
                preference: ThemePreference::System,
            },
            (),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
        message: Self::Message,
    ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
        match message {
            Message::System => self.preference = ThemePreference::System,
            Message::Light => self.preference = ThemePreference::Light,
            Message::Dark => self.preference = ThemePreference::Dark,
        }
        AppUpdate::none().theme(self.preference)
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let active = match self.preference {
            ThemePreference::System => "System",
            ThemePreference::Light => "Light",
            ThemePreference::Dark => "Dark",
        };

        let content = column![
            text("Theming Example").size(24),
            text(format!("Current theme: {}", active)),
            row![
                button("System").on_press(Message::System),
                button("Light").on_press(Message::Light),
                button("Dark").on_press(Message::Dark),
            ]
            .spacing(12),
            text("Click a button to switch themes at runtime"),
        ]
        .padding(40)
        .spacing(16);

        ScreenView::new(content)
    }

    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        self.preference
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("Theming")
    }
}

fn main() -> nive::Result {
    nive::run::<ThemingApp>()
}
