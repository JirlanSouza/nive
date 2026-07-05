use nive::prelude::*;
use nive::widget::{column, row};

struct CounterApp {
    count: i32,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
    Decrement,
}

impl Application for CounterApp {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-counter").name("Counter")
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
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let content = column![
            row![
                button("-").on_press(Message::Decrement),
                text(self.count).size(32),
                button("+").on_press(Message::Increment),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .padding(40)
        .spacing(16)
        .align_x(Alignment::Center);

        ScreenView::new(content)
    }
}

fn main() -> nive::Result {
    nive::run::<CounterApp>()
}
