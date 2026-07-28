# Runtime Flows and State Machines

`nive-runtime`'s dynamic behaviour: the Elm loop, bootstrap/splash, the window
handshake, and the asynchronous state machines.

---

## 1. The update loop (Elm architecture)

The private *program runner* mediates between Iced and your `Application`. Every
`update` returns an `Effect` combining a `Task` with an ordered list of
`RuntimeCommand`s (internal) that the runtime **drains** after the update.

```mermaid
sequenceDiagram
    actor User
    participant Iced as Iced runtime
    participant Runner as Nive program runner
    participant App as Application (your app)

    User->>Iced: input (click, key, …)
    Iced->>Runner: event
    Runner->>App: update(ctx, message_context, Message)
    App-->>Runner: Effect { task, runtime[] }
    Runner->>Runner: drains RuntimeCommand[]
    Note right of Runner: Toast → ToastState<br/>Window → WindowRegistry<br/>Theme → ThemeController<br/>Exit → shuts down
    Runner->>Iced: Task<Message> (async effects)
    Iced-->>Runner: Message (when the Task resolves)
    Runner->>App: view(ctx, window) → ScreenView
    App-->>User: re-render

    Note over Iced,App: subscription(), shortcuts(), and actions()<br/>emit Messages into the same loop
```

**Key types:** `Effect<M, K = Never>` (what `Application`'s hooks return; no
outcome axis) · `RuntimeCommand<K>` (crate-internal) = `Toast | Window | Theme |
Exit`.

---

## 2. Bootstrap and splash

`BootstrapSpec` wraps the initialisation task (building the product's clients and
services), with a minimum splash duration, stale-result rejection, and retry.
Apps without a splash use `type Bootstrap = ()` and get an instant bootstrap.

```mermaid
sequenceDiagram
    participant Main as fn main()
    participant Runner
    participant Splash as BootstrapView
    participant Task as Bootstrap Task
    participant App

    Main->>Runner: nive::run::<A>()
    Runner->>Runner: A::config() → ApplicationConfig
    alt has a BootstrapSpec
        Runner->>Splash: show the splash (brand + loading)
        Runner->>Task: attempt() → Task<UserFacingResult<B>>
        alt success
            Task-->>Runner: Ok(B)  (respecting the minimum duration)
            Runner->>App: init(ctx, B) → (state, Effect)
            Runner->>App: open the first window → view()
        else failure
            Task-->>Runner: Err(UserFacingError)
            Runner->>Splash: failure screen + retry action
            Splash->>Task: retry → attempt()
        end
    else Bootstrap = ()
        Runner->>App: init(ctx, ()) immediately
    end
```

---

## 3. Window handshake (close / exit)

Multi-window is first class: `WindowSpec` and `WindowRegistry` orchestrate
opening, focus, and closing. The app can intercept both close and exit.

```mermaid
flowchart TD
    closeReq["User closes a window"] --> onClose["on_window_close_requested(ctx, window)"]
    onClose --> decision{CloseDecision}
    decision -->|Close| doClose["closes the window"]
    decision -->|Keep| keep["stays open (cancelled)"]
    decision -->|with Tasks| tasks["runs tasks, then closes"]
    doClose --> last{was it the last<br/>app window?}
    last -->|yes| lastEvt["RuntimeEvent::LastAppWindowClosed"]
    last -->|no| idle["carries on"]

    exitReq["App exit requested"] --> onExit["on_exit_requested(ctx)"]
    onExit --> exitDec{ExitDecision}
    exitDec -->|Exit| quit["ends the process"]
    exitDec -->|Defer| defer["defers (e.g. confirm 'save?')"]
```

**Runtime events** delivered through `on_runtime_event`: `WindowOpened`,
`WindowClosed`, `WindowFocused`, `LastAppWindowClosed`, `ThemeChanged`,
`CommandRejected`, `PlatformError`.
**Window attributes:** `WindowRole` (App | Auxiliary), `WindowCardinality`
(Single | Multiple), `WindowMode` (Windowed | Maximized | Fullscreen),
`WindowChrome`.

---

## 4. Logical focus navigation

Every `Program::view` wraps the window's final element exactly once in
`nive_ui::accessibility::FocusRoot`, outside content, hosts, and overlays. The
coordinator stays local to that window's tree; the application receives no
manager and keeps no focus graph.

```mermaid
sequenceDiagram
    actor User
    participant Root as The window's FocusRoot
    participant Child as A descendant widget/overlay
    participant Sub as keyboard_navigation_subscription
    participant Iced as Native Iced operation
    participant State as Shared FocusState

    User->>Root: Tab or Shift+Tab
    Root->>Root: records a Keyboard origin
    Root->>Child: forwards the event
    Child-->>Root: Ignored (did not capture Tab)
    Root-->>Sub: the ignored event reaches the subscription
    Sub->>Iced: FocusDirection::Next/Previous
    Iced->>State: focus()/unfocus() in the tree's native order
    State-->>User: a new active target + visible indication
```

A primary press or touch on a managed target replaces the anchor and, under the
`Auto` policy, normally hides the indication. A press on empty content clears
active and visible focus but keeps a valid anchor for the next Tab. Composite and
selection state are not transferred to the runtime. The recursive overlay chain
uses the same coordinator; the overlay's policy decides only entry, containment,
and conditional restoration.

---

## 5. State machine: `Resource<T>` (request/response)

An asynchronously loaded value with *stale-while-revalidate* — it keeps the
previous value while reloading — and discards stale responses by `RequestId`.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Loading: begin()
    Loaded --> Loading: begin() (keeps the value)
    Failed --> Loading: begin() (keeps the value)
    Loading --> Loaded: settle Ok (current token)
    Loading --> Failed: settle Err (current token, keeps the value)
    Loading --> Loading: settle stale (ignored)
    Failed --> Loaded: dismiss_error (had a value)
    Failed --> Idle: dismiss_error (no value)
    Loaded --> Idle: reset
    Loading --> Idle: reset
    Failed --> Idle: reset
```

Sugar: `Resource::load(future, Msg::Settled)` fuses `begin()` with spawning the
`Task` and maps the result into `Settled<T>`, leaving the token as invisible
plumbing.

---

## 6. State machine: `Operation<C>` (command/action)

An asynchronous action with no persistent return value — saving, deleting. On
success it hands the `input` back to the caller; on failure it keeps `input` plus
the error.

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> Running: begin(input)
    Running --> Idle: settle Ok (returns input)
    Running --> Failed: settle Err (keeps input + error)
    Failed --> Idle: reset / dismiss_error
    Running --> Idle: reset
    note right of Running
        a RequestId token discards
        stale responses
    end note
```

Collections of in-flight actions are managed by `OperationRegistry` — several
table rows saving in parallel, for instance.

---

## 7. The toast queue

`ToastState` holds active slots plus a `VecDeque` of pending ones. Each toast has
a severity (`ToastTone`) and a position (`ToastPosition`); short and long
durations expire on their own.

```mermaid
flowchart LR
    push["push(Toast, now)"] --> cap{a free active slot?}
    cap -->|yes| active["active (visible)"]
    cap -->|no| queued["VecDeque queue"]
    active -->|expire(now) / dismiss(id)| promote["promotes the next"]
    promote --> active
    queued -.-> promote
```

---

## 8. Dialog hosting (`ScreenView` → `DialogHost`)

`Application::view()` returns a `ScreenView` carrying content plus an optional
`DialogRequest`. `ScreenView::into_element()` — called by the runner, not by the
app — takes the request apart and assembles the `DialogHost` automatically; the
app never instantiates `DialogHost` itself. Dismissal and dialog actions
(backdrop, Escape, footer buttons) are ordinary `Message`s coming back through the
same loop as section 1; there is no second state channel. The app closes the
dialog simply by returning `dialog: None` — or a different `DialogRequest` — from
the next `view()`.

```mermaid
sequenceDiagram
    participant App as Application (your app)
    participant Runner as Nive program runner
    participant SV as ScreenView::into_element
    participant Host as DialogHost (nive-ui)

    App->>Runner: view(ctx, window) → ScreenView { content, dialog: Some(DialogRequest) }
    Runner->>SV: into_element()
    SV->>SV: dialog.into_parts() → (content, dismiss, initial_focus, id)
    SV->>Host: DialogHost::new(content).dialog(…).dialog_id(id?)
    Host-->>Runner: Element (base + Scrim + Dialog, composed)
    Note over Host: modality, focus, Escape/backdrop, and<br/>identity are internal to DialogHost —<br/>see crates/nive-ui/docs/components.md
    Host->>Runner: Message (backdrop/Escape/footer action)
    Runner->>App: update(ctx, message_context, Message)
    App-->>Runner: Effect { … } (typically clearing or swapping the DialogRequest)
    App->>Runner: view(ctx, window) → ScreenView { dialog: None }
    Note right of Host: closing restores the original<br/>invoker's focus as an inactive anchor
```

One `ScreenView` per window structurally enforces at most one modal dialog per
window (`Option<DialogRequest>`); a new `dialog(…)` replaces the previous one
rather than stacking. Hosting a `DialogHost` by hand inside app content, outside
this automatic path, is not supported.
