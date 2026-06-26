use nive::prelude::*;
use nive::widget::{column, row};

fn icon(name: IconName) -> Element<'static, Message> {
    Icon::new(name).into()
}

fn icon_sm(name: IconName) -> Element<'static, Message> {
    Icon::new(name).xs().into()
}

fn icon_lg(name: IconName) -> Element<'static, Message> {
    Icon::new(name).lg().into()
}

fn icon_32(name: IconName) -> Element<'static, Message> {
    Icon::new(name).size(32.0).into()
}

fn icon_color(name: IconName, color: Color) -> Element<'static, Message> {
    Icon::new(name).color(color).into()
}

struct IconsApp;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Application for IconsApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-icons").name("Icons")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
        (Self, ())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
        _message: Self::Message,
    ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
        AppUpdate::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let content = column![
            text("Icons Example").size(24),
            text("Default size (16px):"),
            row![
                icon(IconName::Search),
                icon(IconName::Settings),
                icon(IconName::Check),
                icon(IconName::Info),
                icon(IconName::AlertCircle),
            ]
            .spacing(12),
            text("Small (12px):"),
            row![
                icon_sm(IconName::Search),
                icon_sm(IconName::Settings),
                icon_sm(IconName::Check),
            ]
            .spacing(12),
            text("Large (20px):"),
            row![
                icon_lg(IconName::Search),
                icon_lg(IconName::Settings),
                icon_lg(IconName::Check),
            ]
            .spacing(12),
            text("Custom size (32px):"),
            row![
                icon_32(IconName::Search),
                icon_32(IconName::Settings),
                icon_32(IconName::Check),
            ]
            .spacing(12),
            text("With color:"),
            row![
                icon_color(IconName::Check, Color::from_rgb(0.0, 0.8, 0.0)),
                icon_color(IconName::AlertCircle, Color::from_rgb(0.9, 0.2, 0.2)),
                icon_color(IconName::Info, Color::from_rgb(0.2, 0.5, 0.9)),
            ]
            .spacing(12),
        ]
        .padding(40)
        .spacing(16);

        ScreenView::new(content)
    }
}

fn main() -> nive::Result {
    nive::run::<IconsApp>()
}
