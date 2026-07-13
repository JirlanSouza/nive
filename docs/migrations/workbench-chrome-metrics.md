# Workbench Chrome Metrics Migration

`WorkbenchShell` now owns one shared chrome scale and accepts typed toolbar
and status values. This is an intentional breaking change: keeping those
values typed until shell rendering lets the workbench enforce one coherent
metric contract across its managed chrome.

## Typed toolbar and status inputs

`WorkbenchShell::toolbar` now accepts only `nive_ui::widgets::Toolbar`, and
the arbitrary `status_bar(impl Into<Element<_>>)` builder has been removed.
Use the typed `StatusBar` builder instead.

```rust
use nive_ui::widgets::Toolbar;
use nive_workbench::prelude::*;

let shell = WorkbenchShell::new(state, Message::Workbench)
    .toolbar(Toolbar::new())
    .status(StatusBar::new());
```

Direct `nive-workbench` consumers must add `nive-ui` as a direct dependency to
import `Toolbar`; transitive dependencies are not available to application
code. `StatusBar` remains a workbench type.

## One shell chrome size

Configure one local scale for managed workbench chrome:

```rust
let shell = WorkbenchShell::new(state, Message::Workbench)
    .chrome_size(ControlSize::Sm)
    .toolbar(Toolbar::new().lg())
    .status(StatusBar::new());
```

`ControlSize::Sm` is the default. The shell applies its final size to the
typed toolbar and status during rendering, so `chrome_size` takes precedence
over a size previously selected on `Toolbar`, regardless of builder order.
The same size reaches document tabs, side rails, panel headers, bottom
selectors, and split panes. `ThemeDensity` remains the independent global
theme-density setting; it changes the metrics resolved for the selected local
control size.

Standalone `DocumentArea`, `PanelRail`, `PanelHeaderBar`, `panel_host`, and
`StatusBar::view()` keep their existing signatures and use `ControlSize::Sm`.
`StatusBar` has no independent public size builder.

## SplitPane sizing

`SplitPane` now uses the same public control-size vocabulary:

```rust
use nive_ui::{theme::ControlSize, widgets::SplitPane};

let split = SplitPane::new(leading, trailing).size(ControlSize::Md);
let compact = SplitPane::new(leading, trailing).sm();
```

It defaults to `ControlSize::Sm`; `xs`, `sm`, `md`, and `lg` are equivalent
builders. These builders do not expose raw divider metrics: the visual and
layout seam stays one logical pixel, while the centered resize target and grip
length derive from the selected control size.
