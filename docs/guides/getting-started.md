# Getting Started with Nive

This guide will help you create your first Nive application.

## Prerequisites

- Rust 1.92 or later
- Cargo
- just (optional, for convenience commands)

## Creating a New App

```bash
# Using the framework repo
cargo run --package create-nive-app -- my-app

# Or using just
just create-app my-app
```

This creates a new directory with:
- `Cargo.toml` with Nive dependencies
- `src/main.rs` with a basic counter app
- `justfile` with development commands
- `icons/lucide.toml` for icon management

## Understanding the Structure

### Application Trait

Every Nive app implements the `Application` trait:

```rust
use nive::prelude::*;

struct MyApp;

impl Application for MyApp {
    type Message = Message;
    type Window = Window;
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("my-app")
            .window(Window::Main, WindowSpec::app().size(600.0, 400.0))
            .initial_window(Window::Main)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, AppUpdate<Self::Message, Self::Window>) {
        (Self, AppUpdate::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
        message: Self::Message,
    ) -> AppUpdate<Self::Message, Self::Window> {
        AppUpdate::none()
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        ScreenView::new(text("Hello, Nive!"))
    }
}
```

### Running the App

```rust
fn main() -> iced::Result {
    nive::run::<MyApp>()
}
```

## Next Steps

- [Adding Icons](adding-icons.md)
- Explore the [API documentation](https://docs.rs/nive)
- Check out the [architecture guide](../agents/architecture.md)
