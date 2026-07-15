# nive-workbench

`nive-workbench` provides a fixed-region professional desktop shell built from
`nive-ui` primitives. It covers document tabs, generic panel hosts, side rails,
bottom header tabs, layout/session state, diagnostics/status surfaces, and
command palette hosting.

The crate stores shell/view state only. Application domain state, side effects,
persistence location, runtime/window behavior, resources, operations, and final
message routing remain app-owned.

Runtime adapters are available behind the optional `runtime` feature.

## Shell composition

Use `DocumentHeader` inside `document_content` for a principal 16 px document
title; it is transparent content, not another shell slot. `SectionHeader`
remains the compact section/panel tier.

`StatusBar` has explicit `leading` and `trailing` lanes. Long leading text
clips before protected operational content; optional context is muted and
semantic tone remains concentrated in dots/icons. `PanelHeaderBar` follows the
same title-first priority, forwards panel title tooltips, and orders app actions
before restore, maximize, collapse, and close controls.

The shell keeps one owner per structural concern: Toolbar and StatusBar own
Chrome surfaces and outer seams, Panel owns its internal header/body seam,
SplitPane owns region seams, side/bottom hosts use Sidebar, document content
uses Canvas, and shell wrappers remain transparent without compensating inset.

`WorkbenchPaneConstraints` configures non-persisted expanded-region minima.
Defaults are 160/240/160 logical pixels for left/center/right and 160/96 for
upper/bottom. Layout clamps current rendering without rewriting app-owned or
serialized split ratios; collapse remains the semantic zero-size path.
