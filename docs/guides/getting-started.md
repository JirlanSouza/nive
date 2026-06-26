# Getting Started with Nive

This guide will help you create your first Nive application.

## Prerequisites

- Rust 1.92 or later
- Cargo
- just (optional, for convenience commands)

## Creating a New App

```bash
# Install the CLI
cargo install nive-cli

# Create a new app
nive new my-app

# Run it
cd my-app
cargo run
```

This creates a new directory with:
- `Cargo.toml` with Nive dependencies
- `src/main.rs` with a basic counter app
- `justfile` with development commands
- `icons/lucide.toml` for icon management

## Understanding the Structure

### Application Trait

Every Nive app implements the `Application` trait. The simplest template uses
the unit marker types (`type Window = ()` and `type Bootstrap = ()`), which
opts into the `SimpleApplication` marker: the runtime auto-registers one
default `WindowSpec::app()` for `Window = ()` and skips the splash flow when
`Bootstrap = ()`. Update hooks can return `()`, which the runtime treats as
`AppUpdate::none()` when there are no side effects.

```rust
use nive::prelude::*;

struct MyApp;

#[derive(Debug, Clone, Copy)]
enum Message {
    Increment,
}

impl Application for MyApp {
    type Message = Message;
    // Single-window + no-splash marker — the runtime auto-registers
    // one WindowSpec::app(); no `enum Window` declaration needed.
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("my-app")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
        (Self, AppUpdate::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
        _message: Self::Message,
    ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
        // Returning `()` is `AppUpdate::none()`.
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

Multi-window apps set `type Window = MyWindow` (an `enum`) and call
`.window(MyWindow::Main, WindowSpec::app())` on `ApplicationConfig`.

### Running the App

```rust
fn main() -> nive::Result {
    nive::run::<MyApp>()
}
```

## Next Steps

- [Adding Icons](adding-icons.md)
- Explore the [API documentation](https://docs.rs/nive)
- Check out the [architecture guide](../agents/architecture.md)
