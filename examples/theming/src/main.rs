use nive::prelude::*;
use nive::widget::{column, row};
use std::borrow::Cow;

struct ThemingApp {
    preference: ThemePreference,
}

/// A branded light/dark pair, built from the six colors an app supplies.
///
/// Every semantic color a widget reads — panel and sidebar surfaces, border
/// weights, the hover and pressed fills — is derived from this palette rather
/// than set here. That derivation is what keeps contrast floors and the
/// idle → hover → pressed ladder true for a custom theme, not only for the
/// framework defaults.
fn brand_catalog() -> ThemeCatalog {
    let light = ThemeBuilder::new("Brand Light", ThemeMode::Light)
        .palette(ThemePalette {
            background: Color::from_rgb8(0xF7, 0xF5, 0xF2),
            text: Color::from_rgb8(0x24, 0x1E, 0x1A),
            primary: BRAND_ACCENT,
            success: Color::from_rgb8(0x15, 0x80, 0x3D),
            warning: Color::from_rgb8(0xC5, 0x66, 0x05),
            danger: Color::from_rgb8(0xDC, 0x26, 0x26),
        })
        .density(ThemeDensity::Comfortable)
        .build();

    // The other door: start from a framework theme and change only what the
    // brand requires, leaving the rest of the dark palette alone.
    let dark = ThemeBuilder::from_theme(Theme::Dark)
        .name("Brand Dark")
        .accent(BRAND_ACCENT)
        .app_background(Color::from_rgb8(0x1A, 0x17, 0x14))
        .density(ThemeDensity::Comfortable)
        .build();

    ThemeCatalog::new(light, dark)
}

const BRAND_ACCENT: Color = Color::from_rgb(0.85, 0.42, 0.13);

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
        ApplicationConfig::new("nive-example-theming")
            .name("Theming")
            .theme_catalog(brand_catalog())
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
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
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::System => self.preference = ThemePreference::System,
            Message::Light => self.preference = ThemePreference::Light,
            Message::Dark => self.preference = ThemePreference::Dark,
        }
        Effect::theme(self.preference)
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
            text(
                "Both themes are built by this app from six colors; \
                 every other color is derived from them.",
            ),
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
