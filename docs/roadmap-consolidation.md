# Markdown Roadmap Consolidation

> **Status:** migration record, audited and applied on 2026-08-03. This file is
> not the operational roadmap. Current priority and delivery state live only in
> [Nive Framework — GitHub Project #1](https://github.com/users/JirlanSouza/projects/1)
> (`PVT_kwHOBCbfJc4BfMpL`).

## Outcome

The old Markdown roadmaps mix four different kinds of information: delivered
history, current contracts, active structural work, and uncommitted ideas. They
must not be copied wholesale into GitHub Project.

The applied migration keeps:

- the eight existing GitHub Issues instead of creating duplicates;
- one Project item for each OpenSpec change that still represents deliverable
  work;
- focused Issues or initiatives for confirmed gaps and reproducible defects;
- speculative components as ideas only, without pretending they are committed
  roadmap work.

Completed and superseded material remains searchable in Git/OpenSpec history.
It does not become an open Project item.

## Applied result

| Result | Applied state |
| --- | --- |
| Project | `Nive Framework` #1, private and open |
| Fields | 19 total; `Roadmap`, `Work type`, and `Planning` added |
| Items | 43 total: 32 GitHub Issues and 11 Project drafts |
| Issues | Existing #1–#8 retained; consolidated #9–#32 created |
| Milestones | The existing Alpha.2, Alpha.3, and Alpha.4 milestones retained |
| Validation | All 43 items checked; no duplicate Issues and no field mismatches |

Beta and 1.0 milestones were deliberately not created. Issue
[#8](https://github.com/JirlanSouza/nive/issues/8) defines the compatibility and
release policy that must authorize those milestones first.

## Audit coverage

The audit covered every Markdown file under the repository except `.git/` and
build output. The 562 files divide as follows:

| Surface | Files | Treatment |
| --- | ---: | --- |
| Public repository documentation | 53 | Read for current gaps, promises, and stale roadmap language |
| Local `.turns/` planning and handoffs | 30 | Read as historical planning evidence; only unresolved findings survive |
| Active OpenSpec changes | 73 across 14 changes | Triaged change by change |
| Archived OpenSpec changes | 282 | Historical by default; unchecked tasks and known-issue files were audited |
| Consolidated OpenSpec specs | 65 | Treated as current contracts, not backlog |
| Generated agent integrations | 28 | Excluded from product planning |
| Vendored `.opencode/node_modules` docs | 27 | Excluded as third-party material |
| Other OpenSpec support docs | 4 | Workflow/configuration only |

The current code was checked where a recommendation could already have been
implemented or invalidated. GitHub Issues, milestones, Project fields, and
Project items were reconciled before the explicitly authorized migration. A
final read confirmed the applied item count, classifications, and absence of
duplicate Issues.

## Existing GitHub Issues

These Issues already provided the public contract. They were retained in the
Project and classified without recreating them from Markdown.

| Issue | Area | Recommended horizon | Relationship to Markdown findings |
| --- | --- | --- | --- |
| [#1 Manage the complete application development loop](https://github.com/JirlanSouza/nive/issues/1) | CLI/tooling | Next | Absorbs generated `justfile` concerns, project discovery, process supervision, and most CLI workflow follow-ups |
| [#2 Validate Alpha.2 contracts against a real consumer](https://github.com/JirlanSouza/nive/issues/2) | Validation | Next, blocked | Replaces the old “first real app” roadmap phase and the repeated consumer-validation recommendations |
| [#3 Pin the MSRV and locked dependency resolution](https://github.com/JirlanSouza/nive/issues/3) | CI/release | Now | Covers the reproducibility gap in release documentation |
| [#4 Make generated Rust deterministic and rustfmt-compatible](https://github.com/JirlanSouza/nive/issues/4) | CLI/tooling | Now | Covers icon-generation correctness and scaffold formatting |
| [#5 Use one canonical local/CI readiness gate](https://github.com/JirlanSouza/nive/issues/5) | CI/release | Now | Absorbs package-order coverage and the proposed `unsafe`/readiness policy gate |
| [#6 Return explicit Resource/Operation settlement outcomes](https://github.com/JirlanSouza/nive/issues/6) | Runtime | Now | Replaces broader async-state vocabulary reconsideration with a verified consumer problem |
| [#7 Promote ScreenEffect into application Effect](https://github.com/JirlanSouza/nive/issues/7) | Runtime | Now | Covers repeated screen-composition boilerplate without introducing a second state model |
| [#8 Define the Alpha compatibility and release policy](https://github.com/JirlanSouza/nive/issues/8) | Release/docs | Now | Owns Alpha/Beta/1.0 compatibility, migration guidance, experimental API policy, and release gates |

## Active OpenSpec triage

Every retained active change is now represented by a GitHub Issue and a Project
item. The Issue states the public objective; the existing OpenSpec change
remains the detailed contract.

| Change | Observed state | Project action |
| --- | --- | --- |
| `standardize-toast-host` | 24/27; code is largely complete, but modal-active detection is broken and visual validation remains | [#9](https://github.com/JirlanSouza/nive/issues/9), blocked by bug [#18](https://github.com/JirlanSouza/nive/issues/18) |
| `reach-the-whole-theme-builder` | 8/14; implementation exists, verification remains, plus two genuine follow-ups | [#10](https://github.com/JirlanSouza/nive/issues/10); runtime theme ownership extracted to [#23](https://github.com/JirlanSouza/nive/issues/23) |
| `flatten-workbench-split-axis` | 56/57; only interactive sign-off remains | [#11](https://github.com/JirlanSouza/nive/issues/11), kept in progress rather than duplicated |
| `give-popup-rows-one-render-owner` | 0/17 | [#12](https://github.com/JirlanSouza/nive/issues/12), `Planning = OpenSpec` |
| `refine-section-header-priority` | 0/19 | [#13](https://github.com/JirlanSouza/nive/issues/13), `Planning = OpenSpec` |
| `standardize-specialized-inputs` | 0/24 | [#14](https://github.com/JirlanSouza/nive/issues/14), `Planning = OpenSpec` |
| `establish-motion-preference-plumbing` | 0/32 | [#15](https://github.com/JirlanSouza/nive/issues/15), `Planning = OpenSpec` |
| `add-github-alpha-distribution` | 30/33; `v0.1.0-alpha.1` exists, but no GitHub Release was visible and clean-install verification remains | [#16](https://github.com/JirlanSouza/nive/issues/16), without repeating completed tag work |
| `add-distribution-readiness` | 30/36; remaining steps are irreversible crates.io publication work | [#17](https://github.com/JirlanSouza/nive/issues/17), blocked by policy and consumer validation |
| `ground-embedded-control-fills-in-their-host` | 20/20 | Archive; no open Project item |
| `harden-interaction-affordance-contracts` | Functional scope complete; three unchecked lines are follow-up notes | Follow-up extracted to [#24](https://github.com/JirlanSouza/nive/issues/24); archive the change |
| `make-app-icon-manifests-additive` | Functional scope complete; two unchecked lines are future questions | `nive icons roles` retained as an uncommitted Project draft; archive the change |
| `open-the-cli-to-existing-projects` | Functional scope complete; three unchecked lines are explicitly out of scope | Workflow absorbed into #1 and feature inference retained as a draft; archive the change |
| `demonstrate-workbench-session-persistence` | Empty directory with only `.openspec.yaml` and no artifacts/tasks | Remove or explicitly scope it; do not create a Project item for an empty placeholder |

## New confirmed Project candidates

These items survived deduplication against code, active OpenSpec work, archived
history, and existing Issues. Priority and horizon are recommendations for
initial triage, not committed delivery promises.

### Correctness and current contract gaps

| Proposed item | Type / planning | Initial field suggestion | Evidence and scope |
| --- | --- | --- | --- |
| Fix modal-active detection through the real DialogHost/ModalHost tree | Bug / Issue | P0 · Now · UI/runtime | A real Dialog remains invisible to `FocusRoot` modal detection, so Toast expiry does not pause. The active Toast change has an end-to-end reproduction in `KNOWN_BUGS.md`. |
| Close anchored-overlay residuals | Bug initiative / Issue | P1 · Now · UI | Popover tall-content scrolling and nested inset containment remain broken; Select popup width exceeds its anchor. Keep the Forms low-viewport scroll defect as a small child Issue. |
| Close Tree Gallery selection and focus residuals | Bug / Issue | P1 · Now · UI/examples | Additive Cmd/Ctrl selection, Shift range selection, collapsed-parent focus recovery, and the `SelectionMode::None` Gallery path remain unresolved. |
| Make TabBar rebuilt content structurally shape-stable | Bug risk / Issue | P1 · Next · UI | Several immutable widget methods rebuild content against an existing tree. Current state-driven shapes were repaired, but the invariant remains latent for future decorations. |
| Finish the local/error feedback family | Initiative / OpenSpec recommendation | P1 · Next · UI/runtime | Spinner, ProgressBar, Skeleton variants, EmptyState, InlineAlert, operation/resource status, and error feedback remain the unstarted visual-review unit. Split into loading, runtime status, and error recovery slices before implementation. |
| Make runtime-created themes bounded and audit public builders | API/debt / OpenSpec recommendation | P1 · Next · UI | `Theme::custom` leaks through `Box::leak` when used repeatedly; the ThemeBuilder review also found no general audit of builder-shaped public APIs. |
| Finish interaction-affordance regression coverage | Test/debt / Issue | P2 · Next · UI | Add a FocusRoot-capable pointer-origin harness and reconcile local disabled-foreground alpha scaling with the shared control-state contract. |

### Foundations

| Proposed item | Type / planning | Initial field suggestion | Evidence and scope |
| --- | --- | --- | --- |
| Introduce OKLCH-backed palette generation and high-contrast profiles | Feature / OpenSpec recommendation | P1 · Next · UI/theme | The current token layer remains hex/RGB. This is a token/contrast contract, not merely a color conversion helper. |
| Establish native accessibility-tree emission | Initiative / OpenSpec recommendation | P1 · Later/Blocked · Cross-cutting | Names, roles, relations, active descendants, announcements, form errors/groups, and live regions are currently preparatory metadata only. Track toolkit limitations explicitly and adopt by widget family. |
| Establish logical layout direction and RTL resolution | Initiative / OpenSpec recommendation | P2 · Later · Cross-cutting | Start/End vocabulary exists, but rendering and submenu/action ordering remain physical LTR. Required before any RTL support claim. |
| Complete reduced-motion support by widget family | Initiative / OpenSpec recommendation | P1 · Next · Cross-cutting | Depends on `establish-motion-preference-plumbing`. Create child items for selection, overlays, loading feedback, scrollbar, Dialog, CommandPalette, Tree, Toast, specialized inputs, and examples. |

### Product capabilities

| Proposed item | Type / planning | Initial field suggestion | Evidence and scope |
| --- | --- | --- | --- |
| Add a typed `NumericInputField<T>` | Feature / OpenSpec recommendation | P1 · Next · UI | Validate drafts, Min/Max/Step, scientific notation, units, keyboard behavior, and accessibility without conflating it with the active Path/Color input change. |
| Add localized number/date/duration formatting | Feature / Issue | P1 · Next · Core/UI | Valuable independently of translation dictionaries; keep it separate from full Fluent integration. |
| Add continuous stream-to-Subscription helpers and mock streams | Feature / OpenSpec recommendation | P1 · Next · Runtime/devtools | Cover channel/stream ingestion, cancellation/lifetime, backpressure expectations, and high-frequency simulation. |
| Add modular application-state composition | Feature / OpenSpec recommendation | P2 · Later · Runtime | A trait/macro for nested State/Message/Update needs consumer evidence and must not compete with ScreenEffect or create a second runtime. |
| Add virtualized dense tables | Initiative / OpenSpec recommendation | P1 · Later · UI | Opt-in `tables` feature: viewport culling, multi-column sort, predicate filters, compact cells, and stress examples. Keep Tree row virtualization as a related but separate child. |
| Add high-performance time-series charts | Initiative / OpenSpec recommendation | P2 · Later · UI | Opt-in `charts` feature: evaluate `plotters` integration first, then define downsampling and rendering contracts. |
| Add a notification history/center | Feature / OpenSpec recommendation | P2 · Later · Runtime/UI | Build on or distinguish the existing `DiagnosticEventLog`; define persistence, timestamping, alert versus diagnostic ownership, and multi-window behavior. |
| Evolve runtime session persistence | Initiative / OpenSpec recommendation | P2 · Later · Runtime | Group default config-path helper, deterministic duplicate keys, per-instance keys, display mode, most-recent app window, and user-facing load/save diagnostics. |
| Add full i18n dictionaries with Project Fluent | Initiative / OpenSpec recommendation | P3 · Later · Cross-cutting | Follow localized formatting; do not couple full translation runtime to the minimal core by default. |

### Developer experience and maintenance

| Proposed item | Type / planning | Initial field suggestion | Evidence and scope |
| --- | --- | --- | --- |
| Define the Devtools Pro initiative | Initiative / OpenSpec recommendation | P2 · Later · Runtime/devtools | Decompose real-state serde snapshot/restore, layout/overflow inspection, bounded Message capture, and optional theme-resource reload. Do not revive unlimited time travel. |
| Revisit the Devtools macro surface after early-consumer feedback | API/debt / OpenSpec recommendation | P2 · Later · Runtime/derive | The archived `simplify-devtools` change intentionally parked collapsing derives and removing marker ceremony for a later redesign. Revalidate against current `Inspect` APIs first. |
| Complete public rustdoc and remove scheduled compatibility bridges | Docs/release / Issue | P1 · Next · Cross-cutting | Finish the `missing_docs` long tail and coordinate removal of deprecated form/selection aliases with #8's compatibility policy. |
| Add `nive icons roles` | Feature / Issue | P3 · Later · CLI | Let applications inspect semantic roles declared by their Nive dependency; current `icons list` shows only the app manifest. |
| Infer Nive feature flags during `nive init` | Feature / Issue or #1 child | P3 · Later · CLI | Keep as a child of #1 if it participates in project discovery; do not turn Iced-to-Nive source migration into implicit scaffolding. |
| Reduce justified advanced-widget duplication | Debt / Issue | P3 · Later · UI | Evaluate `MinWidth` + `MinHeight` consolidation and ColorPicker canvas programs after specialized-input behavior stabilizes. Do not remove the custom ToastStack: later evidence proved Iced keyed-column state leakage on same-length swaps. |

## Applied migration index

Active OpenSpec delivery is tracked by Issues
[#9](https://github.com/JirlanSouza/nive/issues/9)–[#17](https://github.com/JirlanSouza/nive/issues/17).
Confirmed corrections and capabilities are tracked by Issues
[#18](https://github.com/JirlanSouza/nive/issues/18)–[#32](https://github.com/JirlanSouza/nive/issues/32).

The following lower-definition recommendations were preserved as Project drafts,
not public Issues:

- logical direction and RTL resolution;
- modular application-state composition;
- high-performance time-series charts;
- notification history and center;
- evolved runtime session persistence;
- full Project Fluent dictionaries;
- Devtools Pro;
- a later Devtools macro-surface review;
- `nive icons roles`;
- feature inference during `nive init`;
- advanced-widget duplication cleanup.

## Absorbed work: no new item

The following recommendations should become acceptance criteria or child work
inside another item instead of independent Project entries:

- large-data dashboard examples belong to the table, chart, and stream items;
- the `unsafe` occurrence policy belongs to the canonical readiness/release gate
  in #5;
- OperationRegistry presentation belongs to the local/runtime feedback unit;
- real-consumer architectural validation belongs to #2;
- generated development recipes and `nive init` whole-project opinions belong
  to #1;
- compatibility aliases and migration-note policy belong to #8;
- motion adoption belongs to the motion initiative after the active plumbing
  change;
- crates.io publication remains the execution tail of
  `add-distribution-readiness`, not a second stable-release roadmap;
- manual visual matrices are validation criteria on their owning work item,
  not permanent roadmap cards.

## Do not migrate as active work

These sources were evaluated and should not become open Issues now:

- completed phases for UI taxonomy, `nive-core`, Workbench MVP, command palette,
  focus navigation, Rust module splitting, widget-impl slimming, and Workbench
  polish;
- archived unchecked manual-review boxes that are explicitly retained as
  historical evidence and have no current reproduced defect;
- the proposal to replace the local ToastStack with Iced `keyed_column`, which
  was invalidated by a later reproducible same-length swap bug;
- the TreeFocus shape-stability warning whose implementation now re-diffs the
  active child during layout;
- TabBar directional chevron option B, because option A was selected and works;
- runtime icon stroke-width control and unlimited time travel, both explicitly
  rejected/re-scoped;
- generic Canvas, Ribbon, docking, PropertyGrid, native menu bar, general image
  Avatar, dedicated Hyperlink, and letter-spacing rendering until a concrete
  consumer problem justifies them;
- Iced-to-Nive automatic source migration and generic hot reload as committed
  work. If retained, keep them as Project `Idea` drafts with no priority or
  target date.

## Applied Project model

Use a small field set and keep technical detail in Issues/OpenSpec:

| Field | Applied values |
| --- | --- |
| `Status` | Todo · In Progress · Done |
| `Priority` | P0 · P1 · P2; `Idea` items may remain unprioritized |
| `Roadmap` | Now · Next · Later · Idea |
| `Milestone` | Alpha.2 correctness/release · Alpha.3 managed development loop · Alpha.4 structured async lifecycle |
| `Area` | Runtime · Async · UI · Devtools · CLI · Tooling · Testing · Platform · Release |
| `Work type` | Bug · Feature · Debt · Docs · Validation · Release |
| `Planning` | Direct · Issue · OpenSpec |
| `Size` | S · M · L; decompose L before implementation |

Applied views:

1. **Triage:** table filtered by `status:Todo`.
2. **Delivery:** board filtered by `roadmap:Now,Next -status:Done`.
3. **Roadmap:** timeline filtered by
   `roadmap:Now,Next,Later -status:Done`.
4. **OpenSpec:** table filtered by `planning:OpenSpec -status:Done`.
5. **Quality:** table filtered to bugs, debt, validation, and documentation
   follow-ups that are not done.
6. **Ideas:** table filtered by `roadmap:Idea`, excluded from committed delivery
   views.

## Ongoing governance

1. Use [Nive Framework Project #1](https://github.com/users/JirlanSouza/projects/1)
   as the single operational roadmap; do not maintain status in this file.
2. Close Issues and archive active OpenSpec changes only after their validation
   evidence is complete.
3. Promote a draft to an Issue only when its problem, boundary, and acceptance
   criteria are ready.
4. Let Issue #8 define Beta and 1.0 compatibility before creating those
   milestones.
5. Keep exactly one Project item for each Issue and leave historical Markdown
   and archived OpenSpec material out of the active roadmap.
