use nive::prelude::*;
use nive::widget::{column, row};

mod icons;

fn role_icon(role: IconRole) -> Element<'static, Message> {
    Icon::role(role).into()
}

fn role_icon_sm(role: IconRole) -> Element<'static, Message> {
    Icon::role(role).xs().into()
}

fn role_icon_lg(role: IconRole) -> Element<'static, Message> {
    Icon::role(role).lg().into()
}

fn role_icon_32(role: IconRole) -> Element<'static, Message> {
    Icon::role(role).custom_size(32.0).into()
}

fn role_icon_color(role: IconRole, color: Color) -> Element<'static, Message> {
    Icon::role(role).color(color).into()
}

fn symbol_icon(symbol: icons::IconSymbol) -> Element<'static, Message> {
    Icon::symbol(symbol).lg().into()
}

struct IconsApp;

#[derive(Debug, Clone, Copy)]
enum Message {}

impl Application for IconsApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-icons")
            .name("Icons")
            .theme_catalog(app_theme_catalog())
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self, ())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let content = column![
            text("Icons Example").size(24),
            text("Semantic roles:"),
            row![
                role_tile(IconRole::EditFind),
                role_tile(IconRole::PreferencesSystem),
                role_tile(IconRole::ActionConfirm),
                role_tile(IconRole::DialogInformation),
                role_tile(IconRole::DialogError),
            ]
            .spacing(12),
            text("Small role icons:"),
            row![
                role_icon_sm(IconRole::EditFind),
                role_icon_sm(IconRole::PreferencesSystem),
                role_icon_sm(IconRole::ActionConfirm),
            ]
            .spacing(12),
            text("Large role icons:"),
            row![
                role_icon_lg(IconRole::EditFind),
                role_icon_lg(IconRole::PreferencesSystem),
                role_icon_lg(IconRole::ActionConfirm),
            ]
            .spacing(12),
            text("Custom size:"),
            row![
                role_icon_32(IconRole::EditFind),
                role_icon_32(IconRole::PreferencesSystem),
                role_icon_32(IconRole::ActionConfirm),
            ]
            .spacing(12),
            text("Color overrides:"),
            row![
                role_icon_color(IconRole::ActionConfirm, Color::from_rgb(0.0, 0.8, 0.0)),
                role_icon_color(IconRole::DialogError, Color::from_rgb(0.9, 0.2, 0.2)),
                role_icon_color(IconRole::DialogInformation, Color::from_rgb(0.2, 0.5, 0.9)),
            ]
            .spacing(12),
            text("Generated custom symbol and theme role override:"),
            row![
                symbol_icon(icons::IconSymbol::BrandMark),
                role_tile(IconRole::WindowClose),
            ]
            .spacing(12),
        ]
        .padding(40)
        .spacing(16);

        ScreenView::new(content)
    }
}

fn role_tile(role: IconRole) -> Element<'static, Message> {
    column![role_icon(role), text(role.canonical_name()).size(12)]
        .spacing(4)
        .into()
}

fn app_theme_catalog() -> ThemeCatalog {
    ThemeCatalog::new(
        Theme::builder("Icons Light", ThemeMode::Light)
            .icons(icons::APP_ICON_CATALOG)
            .build(),
        Theme::builder("Icons Dark", ThemeMode::Dark)
            .icons(icons::APP_ICON_CATALOG)
            .build(),
    )
}

fn main() -> nive::Result {
    nive::run::<IconsApp>()
}
