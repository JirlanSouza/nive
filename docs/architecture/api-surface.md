# Superfície de API

O contrato que um app implementa, como a superfície pública é organizada em *prelude tiers*,
e quais APIs ficam atrás de feature flags.

> Este arquivo descreve a superfície existente. As decisões de contrato alvo
> pré-publicação, renomeações e remoções ficam em
> [`api-target.md`](api-target.md).

---

## 1. Prelude tiers (importação estável em camadas)

A superfície é estratificada para que um app simples compile com um único `use`, e apps
maiores subam de tier sob demanda.

```mermaid
flowchart TD
    app(["use nive::prelude::*"]) --> minimal
    appui(["use nive::prelude::ui::*"]) --> extended

    subgraph tiers["nive::prelude"]
        minimal["**tier mínimo** (template-stable)<br/>Application · Effect · MessageContext · MessageSource · Context · run · ScreenView<br/>Toast · ToastPosition · Theme · ThemeBuilder · ThemeController · ThemePreference · ThemeMode<br/>ShortcutMap · Action · ActionMap · RuntimeEvent · DiagnosticEventLog<br/>WindowSpec · WindowRole · WindowCardinality · WindowCommand · CloseDecision · ExitDecision · Size/Point"]
        extended["**tier estendido (ui)** = mínimo +<br/>Resource · Operation · OperationRegistry<br/>DialogRequest · WindowHandle · WindowRegistry · WindowMode<br/>UserFacingError · BootstrapSpec · BrandContent<br/>ToastDuration · ScreenEffect"]
        extended --> minimal
    end

    minimal --> uiprelude["+ nive_ui::prelude::* (Element, layout, widgets) + Icon"]
    extended -.feature file-picker.-> fp["FileFilter · PickFileParams · SaveFileParams"]
```

**Regra:** o template do `nive new` (counter) compila só com o tier mínimo — que já inclui
toasts básicos (`Toast`), theming (`Theme`/`ThemeBuilder`/`ThemeController`), atalhos
(`ShortcutMap`), ações, eventos de runtime (`RuntimeEvent`) e a *declaração* de janelas
(`WindowSpec`/`WindowRole`). Troque para `nive::prelude::ui::*` ao usar **estado async**
(`Resource`/`Operation`), **dialogs**, **`UserFacingError`**, **bootstrap/splash**
(`BootstrapSpec`/`BrandContent`), **handles de janela em runtime**
(`WindowHandle`/`WindowRegistry`/`WindowMode`), **`ToastDuration`** ou **params de file-picker**.

---

## 2. Módulos runtime por área

Além dos preludes, `nive-runtime` expõe módulos públicos por área para consumidores
diretos da camada runtime:

| Módulo | Papel |
|--------|-------|
| `nive_runtime::application` | Contrato `Application`, `ApplicationConfig`, `Context`, `Effect`, `MessageContext`, eventos de runtime e runner público. |
| `nive_core::actions` | `Action`, `ActionId`, `ActionMap` e atalhos neutros compartilhados. |
| `nive_runtime::input` | Atalhos e navegação por teclado. |
| `nive_runtime::lifecycle` | Bootstrap, decisões de close/exit, `WindowCommand`, specs e registry de janelas. |
| `nive_runtime::state` | `Resource`, `Operation`, `OperationRegistry`, `RequestId`, `Settled` e helpers de tempo. |
| `nive_runtime::feedback` | `Toast`, `ToastState`, `UserFacingError` e tipos relacionados. |
| `nive_runtime::screen` | `ScreenView`, `ScreenEffect` e contratos de dialog. |
| `nive_runtime::settings` | `SettingsConfig`, `RuntimeSession` e sessão de janelas. |
| `nive_runtime::support` | Diagnósticos, panic hook e runtime event log. |

O crate root continua como conveniência beta. Apps devem preferir `nive::prelude::*`,
`nive::prelude::ui::*` ou os módulos por área acima quando quiserem imports mais
explícitos. Helpers de runner, como abertura direta de janela Iced, permanecem internos;
apps emitem `WindowCommand` por `Effect`.

---

## 3. O contrato `Application`

Quatro métodos obrigatórios; o resto tem default. O runtime nunca depende de tipos de
domínio — clientes/serviços entram via `Bootstrap` em `init`.

```mermaid
classDiagram
    class Application {
        <<trait>>
        +Message
        +Window
        +Bootstrap
        +config() ApplicationConfig
        +init(ctx, bootstrap)
        +update(ctx, message_context, message) Effect
        +view(ctx, window) ScreenView
        +subscription(ctx) Subscription
        +actions(ctx) ActionMap
        +shortcuts(ctx) ShortcutMap
        +window_title(ctx, window) Cow
        +theme(ctx, window) ThemePreference
        +on_runtime_event(ctx, event) Effect
        +on_window_close_requested(ctx, window) CloseDecision
        +on_exit_requested(ctx) ExitDecision
    }
    class SimpleApplication {
        <<marker trait>>
    }
    Application <|-- SimpleApplication : blanket impl
    note for Application "Tipos associados Message/Window/Bootstrap (ou unit). Obrigatorios: config, init, update, view. O resto tem default."
```

- **`type Window = ()`** → app de janela única (auto-registra `WindowSpec::app()`).
- **`type Bootstrap = ()`** → sem splash, init imediato.
- **`SimpleApplication`** é um marcador automático (blanket impl) — apps nunca o implementam
  à mão; existe porque defaults de tipo associado são instáveis no Rust stable.

---

## 4. `Effect` — composição de efeitos

O valor de retorno dos hooks. Combina um `Task` async e comandos de runtime ordenados,
construídos com construtores diretos (`Effect::task`, `Effect::toast`, ...) e combinadores
`with_*` para compor múltiplos efeitos.

```mermaid
classDiagram
    class Effect {
        -task : Task
        -runtime : Vec~RuntimeCommand~
        +none()
        +task(task)
        +toast(Toast)
        +window(WindowCommand)
        +theme(ThemePreference)
        +exit()
        +with_task(task)
        +with_toast(Toast)
        +with_window(WindowCommand)
        +with_theme(ThemePreference)
        +with_exit()
    }
    class RuntimeCommand {
        <<enumeration, interno>>
        Toast
        Window
        Theme
        Exit
    }
    Effect o-- RuntimeCommand : runtime[]
    note for Effect "Effect[M, K = Never]; sem eixo de outcome (outcome tipado de tela/componente vive em ScreenEffect). RuntimeCommand nao e reexportado — apps constroem efeitos via Effect."
```

Exemplo de uso direto:

```rust
Effect::task(self.users.load(fetch_users(), Msg::UsersSettled))
    .with_toast(Toast::success("Salvo"))
    .with_window(WindowCommand::Open(Window::Details));
```

---

## 5. Feature flags & modularidade

```mermaid
flowchart LR
    subgraph core["default — core mínimo (zero opt-in)"]
        direction TB
        c1["design system + 40 widgets"]
        c2["Application · lifecycle · multi-janela"]
        c3["Resource/Operation · feedback · settings"]
        c4["atalhos · navegação por teclado · command palette"]
    end
    subgraph opt["opt-in"]
        direction TB
        f1["**devtools**<br/>painel + #[derive(Inspect)] + simuladores"]
        f2["**file-picker**<br/>pick_file/files/folder · save_file (rfd)"]
    end

    nive["nive (umbrella)"] -->|forward| f1
    nive -->|forward| f2

    classDef todo fill:#fff3cd,stroke:#cc9a06,color:#663c00;
    t1["tables (roadmap)"]:::todo
    t2["charts (roadmap)"]:::todo
    t3["i18n (roadmap)"]:::todo
    opt -.futuro.-> t1 & t2 & t3
```

| Feature | Status | Expõe |
|---------|--------|-------|
| `devtools` | ✅ | `devtools`, `run_with_devtools`, `Inspect`, simuladores |
| `file-picker` | ✅ | `pick_*`, `save_file`, `FileFilter`, `PickFileParams`, `SaveFileParams` |
| `tables`, `charts`, `i18n` | ⬜ roadmap | (ver [`../roadmap.md`](../roadmap.md)) |

**Estabilidade:** os prelude tiers são o contrato atual de app, mas ainda podem receber
breaking changes pré-publicação conforme [`api-target.md`](api-target.md). APIs feature-gated
(devtools/inspect) permanecem beta até 1.0. Restrição a `unsafe` honrada — 2 ocorrências: o
FFI objc2 do ícone de app (`platform/app_icon.rs`) e um `transmute_copy` da janela-unit no
program runner (`application/program.rs`).
