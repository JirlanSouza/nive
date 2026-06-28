# L3 — Componentes de `nive-runtime`

Módulos internos do crate de runtime e como se relacionam. O módulo `application` é o
orquestrador; o *program runner* é a única ponte com o loop do Iced.

```mermaid
flowchart LR
    subgraph rt["nive-runtime"]
        direction TB
        app["application<br/>módulo<br/>Trait Application, run(), program runner (ponte Iced), Update, Context, Config, ThemeController"]
        actions["actions<br/>módulo<br/>ActionMap / Action: catálogo de comandos (alimenta atalhos e command palette)"]
        lifecycle["lifecycle<br/>módulo<br/>BootstrapSpec (splash), WindowSpec/Registry/Command (multi-janela), handshakes close/exit"]
        state["state<br/>módulo<br/>Resource&lt;T&gt;, Operation&lt;C&gt;, OperationRegistry, RequestId, clock"]
        feedback["feedback<br/>módulo<br/>ToastState (fila), Toast/ToastTone, UserFacingError"]
        screen["screen<br/>módulo<br/>ScreenView, ScreenUpdate, DialogRequest/Dismiss"]
        input["input<br/>módulo<br/>ShortcutMap, keyboard_navigation_subscription, KeyboardNavigation"]
        settings["settings<br/>módulo<br/>SettingsConfig, RuntimeSession, store (persistência serde)"]
        platform["platform<br/>módulo<br/>app_icon (objc2/winres), file_picker (rfd, feature-gated)"]
        support["support<br/>módulo<br/>RuntimeEventLog, DiagnosticSnapshot, panic hook diagnóstico"]
        devtools["devtools<br/>módulo (feature)<br/>Painel: host, view, probe, simuladores de estado"]
        inspect["inspect<br/>módulo (feature)<br/>Trait Inspect, ResourceSimulator, OperationSimulator"]
    end

    ui["nive-ui<br/>Design system + widgets"]
    derive["nive-runtime-derive<br/>#[derive(Inspect)]"]
    iced["Iced<br/>Task, Subscription, window, Element"]

    app -->|consome<br/>bootstrap + janelas| lifecycle
    app -->|renderiza via| screen
    app -->|resolve atalhos/foco| input
    app -->|lê catálogo| actions
    app -->|drena toasts do Update| feedback
    app -->|carrega/salva preferências| settings
    app -->|program runner traduz para<br/>Iced Program| iced

    state -->|usa UserFacingError| feedback
    screen -->|compõe widgets de| ui
    feedback -->|implementa contratos de apresentação de| ui
    app -->|tema, hosts de overlay| ui

    devtools -->|coleta snapshot via| inspect
    devtools -->|embrulha (run_with_devtools)| app
    inspect -->|derivado por| derive
    inspect -->|simula Resource/Operation| state

    classDef component fill:#e8f1ff,stroke:#4b77be,color:#111;
    classDef external fill:#eee,stroke:#999,color:#333;
    class app,actions,lifecycle,state,feedback,screen,input,settings,platform,support,devtools,inspect component;
    class ui,derive,iced external;
```

## Mapa de responsabilidades

```mermaid
flowchart LR
    subgraph core["Núcleo (sempre presente)"]
        application
        lifecycle
        state
        feedback
        screen
        input
        actions
        settings
        support
        platform
    end
    subgraph opt["Opt-in (feature flags)"]
        devtools["devtools<br/>(feature: devtools)"]
        inspect["inspect<br/>(feature: devtools)"]
        filepicker["platform::file_picker<br/>(feature: file-picker)"]
    end

    application --> lifecycle & state & feedback & screen & input & actions & settings
    devtools --> inspect --> state
    devtools -.embrulha.-> application
```

## Notas

- **`application` é o orquestrador**; tudo o mais é biblioteca que ele compõe. O *program
  runner* (`application/program.rs`) é privado — apps nunca falam com o Iced diretamente.
- **`state` e `feedback` são headless:** máquinas de estado (`Resource`, `Operation`) e
  `UserFacingError` não desenham pixels; implementam *contratos de apresentação*
  (`ResourceStatusPresentation`, etc.) que os widgets de `nive-ui` consomem. Isso mantém
  `nive-ui` ignorante do runtime.
- **`devtools` + `inspect` são totalmente feature-gated** — zero peso em release.
