# Superfície de API

O contrato que um app implementa, como a superfície pública é organizada em *prelude tiers*,
e quais APIs ficam atrás de feature flags.

---

## 1. Prelude tiers (importação estável em camadas)

A superfície é estratificada para que um app simples compile com um único `use`, e apps
maiores subam de tier sob demanda.

```mermaid
flowchart TD
    app(["use nive::prelude::*"]) --> minimal
    appui(["use nive::prelude::ui::*"]) --> extended

    subgraph tiers["nive::prelude"]
        minimal["**tier mínimo** (template-stable)<br/>Application · AppUpdate · Update · Context · run · ScreenView<br/>Toast · ToastPosition · Theme · ThemeBuilder · ThemeController · ThemePreference · ThemeMode<br/>ShortcutMap · Action · ActionMap · CoreEvent · RuntimeCommand · RuntimeEventLog<br/>WindowSpec · WindowRole · WindowCardinality · WindowCommand · CloseDecision · ExitDecision · Size/Point"]
        extended["**tier estendido (ui)** = mínimo +<br/>Resource · Operation · OperationRegistry<br/>DialogRequest · WindowHandle · WindowRegistry · WindowMode<br/>UserFacingError · BootstrapSpec · BrandContent<br/>ToastDuration · ScreenUpdate"]
        extended --> minimal
    end

    minimal --> uiprelude["+ nive_ui::prelude::* (Element, layout, widgets) + Icon"]
    extended -.feature file-picker.-> fp["FileFilter · PickFileParams · SaveFileParams"]
```

**Regra:** o template do `nive new` (counter) compila só com o tier mínimo — que já inclui
toasts básicos (`Toast`), theming (`Theme`/`ThemeBuilder`/`ThemeController`), atalhos
(`ShortcutMap`), ações, eventos de runtime (`CoreEvent`) e a *declaração* de janelas
(`WindowSpec`/`WindowRole`). Troque para `nive::prelude::ui::*` ao usar **estado async**
(`Resource`/`Operation`), **dialogs**, **`UserFacingError`**, **bootstrap/splash**
(`BootstrapSpec`/`BrandContent`), **handles de janela em runtime**
(`WindowHandle`/`WindowRegistry`/`WindowMode`), **`ToastDuration`** ou **params de file-picker**.

---

## 2. O contrato `Application`

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
        +update(ctx, window, message) AppUpdate
        +view(ctx, window) ScreenView
        +subscription(ctx) Subscription
        +actions(ctx) ActionMap
        +shortcuts(ctx) ShortcutMap
        +window_title(ctx, window) Cow
        +theme(ctx, window) ThemePreference
        +on_core_event(ctx, event) AppUpdate
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

## 3. `Update` — composição de efeitos

O valor de retorno dos hooks. Combina um `Task` async, um `outcome` opcional e comandos de
runtime ordenados, construídos fluentemente.

```mermaid
classDiagram
    class Update {
        -task : Task
        -outcome : Option
        -runtime : Vec
        +none()
        +from_task(task)
        +task(task)
        +outcome(o)
        +toast(Toast)
        +window(WindowCommand)
        +theme(ThemePreference)
        +exit()
    }
    class RuntimeCommand {
        <<enumeration>>
        Toast
        Window
        Theme
        Exit
    }
    Update o-- RuntimeCommand : runtime[]
    note for Update "Generico Update[M,O,K]; AppUpdate[M,K] = Update[M, Never, K]. Hooks de Application nunca produzem outcome."
```

Exemplo de uso fluente:

```rust
AppUpdate::none()
    .task(self.users.load(fetch_users(), Msg::UsersSettled))
    .toast(Toast::success("Salvo"))
    .window(WindowCommand::Open(Window::Details));
```

---

## 4. Feature flags & modularidade

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

**Estabilidade:** prelude tiers já são estáveis; APIs feature-gated (devtools/inspect)
permanecem beta até 1.0. Restrição a `unsafe` honrada — 2 ocorrências: o FFI objc2 do ícone
de app (`platform/app_icon.rs`) e um `transmute_copy` da janela-unit no program runner
(`application/program.rs`).
