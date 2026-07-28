# Getting Started with Nive

This guide will help you create your first Nive application.

## Prerequisites

- Rust 1.92 or later
- Cargo
- just (optional, for convenience commands)

## Installing the CLI

### GitHub alpha (pre-crates.io)

```bash
cargo install --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1 --locked nive-cli
```

> **crates.io (final release):** `cargo install nive-cli` is the install path
> after the v0.1.0 crates.io publication.

## Creating a New App

```bash
# Create a new app depending on the GitHub alpha tag
nive new my-app --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1

# Run it
cd my-app
cargo run
```

This creates a new directory with:
- `Cargo.toml` with Nive dependencies
- `src/main.rs` with a basic counter app
- `src/icons.rs` and `src/icons/` with generated icon catalog/symbol modules
- `justfile` with development commands
- `icons.toml` for provider-neutral icon management

Creating the app inside an existing Cargo workspace works: it is registered as
a member for you, so the first `cargo build` succeeds.

## Adding Nive to a Crate You Already Have

```bash
cd my-crate
nive init --git https://github.com/JirlanSouza/nive --tag v0.1.0-alpha.1
```

`nive init` adds the `nive` dependency, sets up the icon workflow, and fills in
whatever boilerplate the crate is missing.

It never overwrites a file you wrote. Anything it skips is listed in the output,
so you can merge by hand what it refused to replace, and running it twice
changes nothing.

## Understanding the Structure

### Application Trait

Every Nive app implements the `Application` trait. The simplest template uses
the unit marker types (`type Window = ()` and `type Bootstrap = ()`), which
opts into the `SimpleApplication` marker: the runtime auto-registers one
default `WindowSpec::app()` for `Window = ()` and skips the splash flow when
`Bootstrap = ()`. Update hooks can return `()`, which the runtime treats as
`Effect::none()` when there are no side effects.

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
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (Self, Effect::none())
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        _message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        // Returning `()` is `Effect::none()`.
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

- [Theming](theming.md) — build your own light/dark pair
- [Adding Icons](adding-icons.md)
- [Architecture diagrams](../architecture/README.md) — how the framework is put
  together
- Explore the [API documentation](https://docs.rs/nive) _(available after crates.io publication)_
