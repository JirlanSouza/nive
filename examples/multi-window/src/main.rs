use nive::prelude::*;
use nive::widget::column;
use std::borrow::Cow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Window {
    Main,
    Detail,
}

struct MultiWindowApp {
    count: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    OpenDetail,
}

impl Application for MultiWindowApp {
    type Message = Message;
    type Window = Window;
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-multi-window")
            .name("Multi Window")
            .window(Window::Main, WindowSpec::app())
            .window(Window::Detail, WindowSpec::auxiliary())
            .initial_window(Window::Main)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self { count: 0 }, ())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        match message {
            Message::Increment => self.count += 1,
            Message::OpenDetail => {
                return Effect::window(WindowCommand::Open(Window::Detail));
            }
        }
        Effect::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        match window.kind {
            Window::Main => {
                let content = column![
                    text("Main Window").size(24),
                    text(format!("Count: {}", self.count)),
                    button("+").on_press(Message::Increment),
                    button("Open Detail Window").on_press(Message::OpenDetail),
                ]
                .padding(40)
                .spacing(16);

                ScreenView::new(content)
            }
            Window::Detail => {
                let content = column![
                    text("Detail Window").size(24),
                    text(format!("Shared count: {}", self.count)),
                    button("Increment from Detail").on_press(Message::Increment),
                ]
                .padding(40)
                .spacing(16);

                ScreenView::new(content)
            }
        }
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        match window.kind {
            Window::Main => Cow::Borrowed("Main Window"),
            Window::Detail => Cow::Borrowed("Detail Window"),
        }
    }
}

fn main() -> nive::Result {
    nive::run::<MultiWindowApp>()
}
