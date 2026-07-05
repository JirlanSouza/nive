# L3 — Componentes de `nive-ui`

Módulos internos do design system. Fluxo de baixo para cima: **tokens → theme → widgets →
hosts**. Depende de `iced` e de `nive-core` (contratos de apresentação de erro/toast/status).

```mermaid
flowchart BT
    subgraph ui["nive-ui"]
        direction BT
        tokens["tokens<br/>módulo<br/>Constantes primitivas: color (hex/RGB), spacing (base-4 compacto), radius, shadow, typography"]
        theme["theme<br/>módulo<br/>Roles semânticos, palette, Theme/ThemeData/ThemeId, ThemeBuilder, catalog (Catalog do Iced), active() global, density (ThemeDensity)"]
        widgets["widgets<br/>módulo<br/>40+ widgets type-safe organizados por primitives, controls, display, containers, navigation, overlays (inclui DialogHost/ToastHost) e feedback"]
        layoutfacade["layout<br/>facade<br/>Surfaces e contêineres para imports focados"]
        graphicsfacade["graphics<br/>facade<br/>Ícones, swatches e SVG"]
        a11yfacade["accessibility<br/>facade<br/>FocusDirection e helpers de foco/teclado"]
        bootstrapview["bootstrap<br/>módulo<br/>BootstrapView: template genérico de splash/loading/falha"]
        focustrap["focus_trap<br/>módulo<br/>Ciclo de foco Tab/Shift+Tab para overlays"]
    end

    iced["Iced<br/>Element, Widget, advanced, Catalog, canvas, svg"]
    core["nive-core<br/>ErrorPresentation, ResourceStatusPresentation,<br/>OperationStatusPresentation, ToastPresentation, ToastTone"]

    theme -->|deriva valores de| tokens
    widgets -->|estiliza via roles/catalog de| theme
    widgets -->|usa spacing/radius de| tokens
    widgets -->|reexporta contratos de| core
    layoutfacade -->|reexporta| widgets
    graphicsfacade -->|reexporta| widgets
    a11yfacade -->|reexporta| focustrap
    bootstrapview -->|compõe| widgets
    focustrap -->|lê eventos de teclado de| iced
    theme -->|implementa Catalog de| iced
    widgets -->|implementa Widget de| iced

    classDef component fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class tokens,theme,widgets,layoutfacade,graphicsfacade,a11yfacade,bootstrapview,focustrap component;
    class iced,core external;
```

## Pilha do design system

```mermaid
flowchart BT
    iced["iced (Widget · Catalog · canvas · svg)"]
    tokens["tokens<br/>color · spacing · radius · shadow · typography"]
    theme["theme<br/>roles · palette · Theme · ThemeBuilder · catalog · active()"]
    widgets["widgets (40+)<br/>primitives · controls · display · containers · navigation · overlays · feedback"]
    hosts["hosts & templates<br/>DialogHost · ToastHost · BootstrapView · focus_trap"]

    tokens --> theme --> widgets --> hosts
    tokens --> iced
    widgets --> iced
    theme --> iced

    classDef ext fill:#eee,stroke:#999,color:#333;
    class iced ext;
```

## Notas

- **`nive-ui` é a camada base e não conhece `nive-runtime`.** Contratos de apresentação
  (ex.: `ResourceStatusPresentation`, `ToastPresentation`, `ErrorPresentation`, `ToastTone`)
  vivem em `nive-core` (zero dependências) e são reexportados por `nive-ui`; o runtime os
  implementa. Isso mantém o design system reutilizável de forma isolada sem inverter a
  fronteira de ownership de um contrato headless para dentro da camada de UI.
- **`active()` expõe um tema global** (estado em thread-local) para que widgets leiam o tema
  ativo sem prop-drilling — útil em densidade alta onde passar `&Theme` em todo lugar
  seria ruído.
- Catálogo de widgets detalhado em [`design-system.md`](design-system.md).
