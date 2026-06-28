# Fluxos de Runtime & Máquinas de Estado

Comportamento dinâmico do `nive-runtime`: o loop Elm, o bootstrap/splash, o handshake de
janelas e as máquinas de estado assíncrono.

---

## 1. Loop de atualização (arquitetura Elm)

O *program runner* (privado) media entre o Iced e o seu `Application`. Cada `update`
devolve um `AppUpdate` que combina um `Task`, um `outcome` opcional e uma lista ordenada de
`RuntimeCommand`s que o runtime **drena** após o update.

```mermaid
sequenceDiagram
    actor User as Usuário
    participant Iced as Iced runtime
    participant Runner as Nive program runner
    participant App as Application (seu app)

    User->>Iced: input (click, tecla, ...)
    Iced->>Runner: evento
    Runner->>App: update(ctx, window, Message)
    App-->>Runner: AppUpdate { task, outcome, runtime[] }
    Runner->>Runner: drena RuntimeCommand[]
    Note right of Runner: Toast → ToastState<br/>Window → WindowRegistry<br/>Theme → ThemeController<br/>Exit → encerra
    Runner->>Iced: Task<Message> (efeitos async)
    Iced-->>Runner: Message (quando Task resolve)
    Runner->>App: view(ctx, window) → ScreenView
    App-->>User: re-render

    Note over Iced,App: subscription(), shortcuts() e actions()<br/>também emitem Message no mesmo loop
```

**Tipos-chave:** `Update<M, O, K>` (genérico) · `AppUpdate<M, K> = Update<M, Never, K>` (o
que os hooks de `Application` retornam) · `RuntimeCommand<K>` = `Toast | Window | Theme |
Exit`.

---

## 2. Bootstrap & Splash

`BootstrapSpec` encapsula a tarefa de inicialização (construir clientes/serviços do
produto), com duração mínima de splash, rejeição de resultado obsoleto e retry. Apps sem
splash usam `type Bootstrap = ()` e recebem um bootstrap instantâneo.

```mermaid
sequenceDiagram
    participant Main as fn main()
    participant Runner
    participant Splash as BootstrapView
    participant Task as Bootstrap Task
    participant App

    Main->>Runner: nive::run::<A>()
    Runner->>Runner: A::config() → ApplicationConfig
    alt tem BootstrapSpec
        Runner->>Splash: mostra splash (brand + loading)
        Runner->>Task: attempt() → Task<UserFacingResult<B>>
        alt sucesso
            Task-->>Runner: Ok(B)  (respeita duração mínima)
            Runner->>App: init(ctx, B) → (estado, AppUpdate)
            Runner->>App: abre 1ª janela → view()
        else falha
            Task-->>Runner: Err(UserFacingError)
            Runner->>Splash: tela de falha + ação de retry
            Splash->>Task: retry → attempt()
        end
    else Bootstrap = ()
        Runner->>App: init(ctx, ()) imediatamente
    end
```

---

## 3. Handshake de Janela (close / exit)

Multi-janela é de primeira classe: `WindowSpec`/`WindowRegistry` orquestram abertura, foco e
fechamento. O app pode interceptar o fechamento e a saída.

```mermaid
flowchart TD
    closeReq["Usuário fecha janela"] --> onClose["on_window_close_requested(ctx, window)"]
    onClose --> decision{CloseDecision}
    decision -->|Close| doClose["fecha a janela"]
    decision -->|Keep| keep["mantém aberta (cancela)"]
    decision -->|com Tasks| tasks["roda tasks e então fecha"]
    doClose --> last{era a última<br/>janela de app?}
    last -->|sim| lastEvt["CoreEvent::LastAppWindowClosed"]
    last -->|não| idle["continua"]

    exitReq["Pedido de sair do app"] --> onExit["on_exit_requested(ctx)"]
    onExit --> exitDec{ExitDecision}
    exitDec -->|Accept| quit["encerra o processo"]
    exitDec -->|Defer| defer["adia (ex.: confirmar 'salvar?')"]
```

**Eventos de runtime** entregues via `on_core_event`: `WindowOpened`, `WindowClosed`,
`WindowFocused`, `LastAppWindowClosed`, `ThemeChanged`, `CommandRejected`, `PlatformError`.
**Atributos de janela:** `WindowRole` (App | Auxiliary), `WindowCardinality` (Single |
Multiple), `WindowMode` (Windowed | Maximized | Fullscreen), `WindowChrome`.

---

## 4. Máquina de estado: `Resource<T>` (request/response)

Valor carregado assincronamente, com *stale-while-revalidate* (retém o valor anterior
enquanto recarrega) e descarte de respostas obsoletas por `RequestId`.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading: begin()
    Loaded --> Loading: begin() (retém valor)
    Failed --> Loading: begin() (retém valor)
    Loading --> Loaded: settle Ok (token atual)
    Loading --> Failed: settle Err (token atual, retém valor)
    Loading --> Loading: settle stale (ignorado)
    Failed --> Loaded: dismiss_error (tinha valor)
    Failed --> Idle: dismiss_error (sem valor)
    Loaded --> Idle: reset
    Loading --> Idle: reset
    Failed --> Idle: reset
```

Açúcar: `Resource::load(future, Msg::Settled)` funde `begin()` + spawn do `Task` e mapeia o
resultado em `Settled<T>` (o token vira plumbing invisível).

---

## 5. Máquina de estado: `Operation<C>` (comando/ação)

Ação assíncrona sem valor de retorno persistente (ex.: salvar, deletar). Em sucesso devolve
o `input` para o chamador; em falha, retém `input` + erro.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Running: begin(input)
    Running --> Idle: settle Ok (devolve input)
    Running --> Failed: settle Err (retém input + erro)
    Failed --> Idle: reset / dismiss_error
    Running --> Idle: reset
    note right of Running
        token RequestId descarta
        respostas obsoletas
    end note
```

Coleções de ações em voo são geridas por `OperationRegistry` (ex.: várias linhas salvando
em paralelo numa tabela).

---

## 6. Fila de Toasts

`ToastState` mantém slots ativos + uma `VecDeque` de pendentes. Cada toast tem severidade
(`ToastTone`) e posição (`ToastPosition`); duração curta/longa expira sozinha.

```mermaid
flowchart LR
    push["push(Toast, now)"] --> cap{slot ativo livre?}
    cap -->|sim| active["ativo (visível)"]
    cap -->|não| queued["fila VecDeque"]
    active -->|expire(now) / dismiss(id)| promote["promove próximo"]
    promote --> active
    queued -.-> promote
```
