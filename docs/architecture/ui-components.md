# L3 — `nive-ui` Components

The design system's internal modules. The flow runs bottom-up: **tokens → theme →
widgets → hosts**. It depends on `iced` and on `nive-core` (the presentation
contracts for error, toast, and status).

```mermaid
flowchart BT
    subgraph ui["nive-ui"]
        direction BT
        tokens["tokens<br/>module<br/>Primitive constants: color (hex/RGB), spacing (compact base-4), radius, shadow, typography"]
        theme["theme<br/>module<br/>Semantic roles, palette, Theme/ThemeData/ThemeId, ThemeBuilder, catalog (Iced's Catalog), the global active(), density (ThemeDensity)"]
        widgets["widgets<br/>module<br/>40+ type-safe widgets organised into primitives, controls, display, containers, navigation, overlays (including DialogHost/ToastHost), and feedback"]
        layoutfacade["layout<br/>facade<br/>Surfaces and containers, for focused imports"]
        graphicsfacade["graphics<br/>facade<br/>Icons, swatches, and SVG"]
        a11yfacade["accessibility<br/>facade<br/>FocusRoot, FocusDirection, and focus/keyboard helpers"]
        advancedfocus["advanced::focus<br/>widget authoring<br/>FocusState and FocusVisibility"]
        bootstrapview["bootstrap<br/>module<br/>BootstrapView: a generic splash/loading/failure template"]
        focustrap["focus_trap<br/>module<br/>Tab/Shift+Tab focus cycling for overlays"]
    end

    iced["Iced<br/>Element, Widget, advanced, Catalog, canvas, svg"]
    core["nive-core<br/>ErrorPresentation, ResourceStatusPresentation,<br/>OperationStatusPresentation, ToastPresentation, ToastTone"]

    theme -->|derives values from| tokens
    widgets -->|styles through the roles/catalog of| theme
    widgets -->|uses spacing/radius from| tokens
    widgets -->|re-exports contracts from| core
    layoutfacade -->|re-exports| widgets
    graphicsfacade -->|re-exports| widgets
    a11yfacade -->|re-exports| focustrap
    a11yfacade -->|coordinates the states of| advancedfocus
    bootstrapview -->|composes| widgets
    focustrap -->|reads keyboard events from| iced
    theme -->|implements Catalog from| iced
    widgets -->|implements Widget from| iced

    classDef component fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class tokens,theme,widgets,layoutfacade,graphicsfacade,a11yfacade,advancedfocus,bootstrapview,focustrap component;
    class iced,core external;
```

## The design system stack

```mermaid
flowchart BT
    iced["iced (Widget · Catalog · canvas · svg)"]
    tokens["tokens<br/>color · spacing · radius · shadow · typography"]
    theme["theme<br/>roles · palette · Theme · ThemeBuilder · catalog · active()"]
    widgets["widgets (40+)<br/>primitives · controls · display · containers · navigation · overlays · feedback"]
    hosts["hosts &amp; templates<br/>DialogHost · ToastHost · BootstrapView · focus_trap"]

    tokens --> theme --> widgets --> hosts
    tokens --> iced
    widgets --> iced
    theme --> iced

    classDef ext fill:#eee,stroke:#999,color:#333;
    class iced ext;
```

## Notes

- **`nive-ui` is the base layer and knows nothing about `nive-runtime`.**
  Presentation contracts — `ResourceStatusPresentation`, `ToastPresentation`,
  `ErrorPresentation`, `ToastTone` — live in `nive-core` (zero dependencies) and
  are re-exported by `nive-ui`; the runtime implements them. That keeps the design
  system reusable on its own without inverting ownership of a headless contract
  into the UI layer.
- **`active()` exposes a global theme** (thread-local state) so widgets can read
  the active theme without prop drilling — worth it at high density, where passing
  `&Theme` everywhere would be noise.
- **Managed focus has exactly one owner per tree or window.** `FocusRoot` holds a
  single sequential logical anchor and propagates it through the whole overlay
  chain; next/previous ordering is still computed by Iced's native operations.
  `nive-runtime` installs the root automatically, while a standalone `nive-ui` app
  opts in explicitly at the final composition.
- **`FocusState` is advanced authoring surface, not application state.** Each
  external target registers a persistent state; active focus and visible
  indication belong to that layer. A composite's roving/highlighted item, durable
  selection, and overlay entry/restore policy stay with their own components.
- The full widget catalogue is in [`design-system.md`](design-system.md).
