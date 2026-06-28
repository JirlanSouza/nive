# L3 — Componentes de `nive-ui`

Módulos internos do design system. Fluxo de baixo para cima: **tokens → theme → widgets →
hosts**. Tudo depende apenas de `iced`.

```mermaid
flowchart BT
    subgraph ui["nive-ui"]
        direction BT
        tokens["tokens<br/>módulo<br/>Constantes primitivas: color (hex/RGB), spacing (base-4 compacto), radius, shadow, typography"]
        theme["theme<br/>módulo<br/>Roles semânticos, palette, Theme/ThemeData/ThemeId, ThemeBuilder, catalog (Catalog do Iced), active() global"]
        widgets["widgets<br/>módulo<br/>40+ widgets primitivos type-safe (inputs, botões, overlays, feedback, dados, motion)"]
        dialoghost["dialog_host<br/>módulo<br/>DialogHost: composição de overlay modal"]
        toasthost["toast_host<br/>módulo<br/>ToastHost + ToastPresentation: overlay de toasts"]
        bootstrapview["bootstrap<br/>módulo<br/>BootstrapView: template genérico de splash/loading/falha"]
        focustrap["focus_trap<br/>módulo<br/>Ciclo de foco Tab/Shift+Tab para overlays"]
    end

    iced["Iced<br/>Element, Widget, advanced, Catalog, canvas, svg"]

    theme -->|deriva valores de| tokens
    widgets -->|estiliza via roles/catalog de| theme
    widgets -->|usa spacing/radius de| tokens
    dialoghost -->|compõe| widgets
    toasthost -->|compõe| widgets
    bootstrapview -->|compõe| widgets
    focustrap -->|lê eventos de teclado de| iced
    theme -->|implementa Catalog de| iced
    widgets -->|implementa Widget de| iced

    classDef component fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class tokens,theme,widgets,dialoghost,toasthost,bootstrapview,focustrap component;
    class iced external;
```

## Pilha do design system

```mermaid
flowchart BT
    iced["iced (Widget · Catalog · canvas · svg)"]
    tokens["tokens<br/>color · spacing · radius · shadow · typography"]
    theme["theme<br/>roles · palette · Theme · ThemeBuilder · catalog · active()"]
    widgets["widgets (40+)"]
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
  (ex.: `ResourceStatusPresentation`, `ToastPresentation`, `ErrorPresentation`) são traits
  *no `nive-ui`* que o runtime implementa — invertendo a dependência para manter o design
  system reutilizável de forma isolada.
- **`active()` expõe um tema global** (estado em thread-local) para que widgets leiam o tema
  ativo sem prop-drilling — útil em densidade alta onde passar `&Theme` em todo lugar
  seria ruído.
- Catálogo de widgets detalhado em [`design-system.md`](design-system.md).
