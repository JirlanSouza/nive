# L1 — Context Diagram

Nive as a black box between the people who use it and the external systems it
rests on.

```mermaid
flowchart LR
    dev([App Developer<br/>Builds Rust/Iced desktop applications with Nive])
    user([End User<br/>Uses the desktop app built with Nive])

    nive["Nive Framework<br/>General-purpose Rust/Iced framework, also suited to data-dense UIs.<br/>Design system + runtime + DX tooling."]

    iced["Iced 0.14<br/>Elm-architecture GUI runtime, rendering through wgpu, canvas, and svg"]
    os["OS Platform<br/>macOS, Windows, Linux: windows, input, app icon, file dialogs"]
    registry["crates.io / docs.rs<br/>Crate distribution and documentation"]
    icons["Icon providers<br/>Lucide SVGs and local custom SVGs, compiled in at build time by the CLI"]

    dev -->|Programs against / depends on<br/>the Rust API| nive
    user -->|Interacts with the app built with| nive
    nive -->|Built on| iced
    iced -->|Renders, opens windows, and captures input through| os
    nive -->|Installs the app icon / opens file dialogs<br/>objc2 / winres / rfd| os
    dev -->|cargo install nive-cli / cargo add nive| registry
    nive -->|nive icons syncs glyphs from| icons

    classDef person fill:#f7f7f7,stroke:#666,color:#222;
    classDef system fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class dev,user person;
    class nive system;
    class iced,os,registry,icons external;
```

## Notes

- **Domain-agnostic at the boundary:** the runtime never depends on the app's
  domain types; product clients and services are built during *bootstrap* and
  injected into `Application::init`.
- **Two human consumers:** the *App Developer* (API and DX) and the *End User*
  (the rendered UI). Nive balances both: DX for the developer, density and
  performance for the user.
- **Two doors in:** `nive new <name>` creates a project — registering it as a
  member when a workspace sits above it, without which Cargo refuses the build —
  and `nive init` adopts Nive in a crate that already exists, writing only what is
  missing and never overwriting a file the author wrote. `cargo add nive` covers
  the dependency but not the icon workflow the CLI establishes.
- Coupling to the OS is thin and isolated in `platform/` (app icon, file picker).
  The workspace holds only **two occurrences of `unsafe`**: the objc2 FFI for the
  macOS app icon (`platform/app_icon.rs`), and a `transmute_copy::<(), Window>` in
  the *program runner* (`application/program.rs`) that materialises the unit
  window of single-window apps.
