use std::{borrow::Cow, collections::BTreeSet};

use nive::prelude::ui::{DialogRequest, UserFacingError};
use nive::prelude::*;
use nive::ui::theme::{self, ControlSize, SurfaceRole, ThemeDensity};
use nive::ui::widgets::primitives::text as ntext;
use nive::ui::interaction::{ContextRequest, SelectionMode};
use nive::ui::widgets::{TabCloseRequest, TabDrop, TabTearOff, TreeEvent, TreeState};
use nive::widget::{column, row};

use crate::catalog::{entry_for, matches, PageId, CATALOG};
#[cfg(feature = "devtools")]
use crate::fixtures::DevState;
use crate::{icons, layout, pages};

mod tree_helpers;

use tree_helpers::handle_tree_event;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoTab {
    Overview,
    Details,
    LongLabel,
    PinnedNotes,
    Console,
    Search,
    Preview,
    Logs,
}

impl DemoTab {
    pub fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Details => "Details",
            Self::LongLabel => "Very long tab label",
            Self::PinnedNotes => "Pinned notes",
            Self::Console => "Console",
            Self::Search => "Search",
            Self::Preview => "Preview",
            Self::Logs => "Logs",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackMode {
    Idle,
    Loading,
    Loaded,
    Refreshing,
    Error,
    Empty,
    Running,
}

impl FeedbackMode {
    pub const ALL: [Self; 7] = [
        Self::Idle,
        Self::Loading,
        Self::Loaded,
        Self::Refreshing,
        Self::Error,
        Self::Empty,
        Self::Running,
    ];
}

impl std::fmt::Display for FeedbackMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::Idle => "Idle",
            Self::Loading => "Loading",
            Self::Loaded => "Loaded",
            Self::Refreshing => "Refreshing",
            Self::Error => "Error",
            Self::Empty => "Empty",
            Self::Running => "Running",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Basic,
    Destructive,
    LongContent,
    NestedOverlay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Danger,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverKind {
    Start,
    End,
    Wide,
    Collision,
    MatchAnchor,
    RetainAnchor,
    EdgeToEdge,
    LowHeight,
    FocusFirst,
    Trap,
    Nested,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Typed,
    Persistent,
    CallbackAbsent,
    Nested,
    LongList,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocompleteKind {
    SuggestionsFirst,
    SuggestionsNone,
    Loading,
    Empty,
    Error,
    Validation,
    CallbackAbsent,
    Duplicate,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DemoTreeNode {
    Examples,
    WidgetGallery,
    CargoToml,
    Target,
    Src,
    AppRs,
    Pages,
    LayoutNavRs,
    InputsRs,
    RemotePackages,
    RemoteSchema,
    RemoteCache,
    RemoteConfig,
    Archived,
}

impl DemoTreeNode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Examples => "examples",
            Self::WidgetGallery => "widget-gallery",
            Self::CargoToml => "Cargo.toml",
            Self::Target => "target",
            Self::Src => "src",
            Self::AppRs => "app.rs",
            Self::Pages => "pages",
            Self::LayoutNavRs => "layout_nav.rs",
            Self::InputsRs => "inputs.rs",
            Self::RemotePackages => "remote-packages",
            Self::RemoteSchema => "schema.json",
            Self::RemoteCache => "cache.bin",
            Self::RemoteConfig => "remote-config",
            Self::Archived => "archived",
        }
    }
}

/// Presents a simulated remote-config load failure through the shared
/// `ErrorPresentation` contract, matching how `nive-runtime`'s
/// `UserFacingError` would project a real failure into `TreeNode::branch_failed`.
pub struct DemoTreeLoadError;

impl nive::ui::widgets::ErrorPresentation for DemoTreeLoadError {
    fn summary(&self) -> &str {
        "Failed to load remote-config"
    }

    fn detail(&self) -> &str {
        "Failed to load remote-config: connection reset while fetching config.toml"
    }
}

/// Position and target captured from a Tree `ContextRequested` pointer event,
/// used to host the canonical `Menu` at the request position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TreeContextMenuState {
    pub target: DemoTreeNode,
    pub position: Point,
}

pub struct FormState {
    pub name: String,
    pub email: String,
    pub search: String,
    pub secret: String,
    pub path: String,
    pub checked: CheckboxState,
    pub enabled: bool,
    pub selected_plan: Option<&'static str>,
    pub radio: Option<&'static str>,
    pub segment: &'static str,
    pub color: Color,
}

pub struct OverlayState {
    pub active_dialog: Option<DialogKind>,
    pub active_popover: Option<PopoverKind>,
    pub nested_popover_open: bool,
    pub active_menu: Option<MenuKind>,
    pub menu_pinned: CheckboxState,
    pub menu_mode: Option<&'static str>,
    pub active_autocomplete: Option<AutocompleteKind>,
    pub autocomplete_query: String,
    pub autocomplete_selected: Option<String>,
    pub palette_open: bool,
    pub command_query: String,
    pub selected_command: Option<&'static str>,
}

pub struct LayoutState {
    pub tab: DemoTab,
    pub tab_order: Vec<DemoTab>,
    pub dirty_tab: bool,
    pub selected_card: usize,
    pub selected_item: usize,
    pub split_ratio: f32,
    pub vertical_split_ratio: f32,
    pub expanded_tree_nodes: BTreeSet<DemoTreeNode>,
    pub tree_state: TreeState<DemoTreeNode>,
    pub tree_selection_mode: SelectionMode,
    pub tree_deferred_loaded: bool,
    pub tree_deferred_loading: bool,
    pub tree_config_failed: bool,
    pub tree_config_loading: bool,
    pub tree_event_feedback: String,
    pub tree_context_feedback: String,
    pub tree_clipboard_feedback: String,
    pub tree_drop_feedback: String,
    pub tree_context_menu: Option<TreeContextMenuState>,
    pub tab_feedback: String,
}

pub struct WidgetGallery {
    pub route: PageId,
    pub search: String,
    pub theme: ThemePreference,
    pub density: ThemeDensity,
    pub control_size: ControlSize,
    pub form: FormState,
    pub overlays: OverlayState,
    pub feedback: FeedbackMode,
    pub layout: LayoutState,
    #[cfg(feature = "devtools")]
    pub dev: DevState,
}

#[cfg(test)]
impl WidgetGallery {
    pub(crate) fn test_fixture() -> Self {
        Self {
            route: PageId::Inputs,
            search: String::new(),
            theme: ThemePreference::System,
            density: ThemeDensity::Standard,
            control_size: ControlSize::Sm,
            form: FormState::default(),
            overlays: OverlayState::default(),
            feedback: FeedbackMode::Loaded,
            layout: LayoutState::default(),
            #[cfg(feature = "devtools")]
            dev: DevState::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(PageId),
    SearchChanged(String),
    ThemeChanged(ThemePreference),
    DensityChanged(ThemeDensity),
    ControlSizeChanged(ControlSize),
    NameChanged(String),
    EmailChanged(String),
    InputSearchChanged(String),
    SecretChanged(String),
    PathChanged(String),
    ToggleChecked(CheckboxState),
    ToggleEnabled(bool),
    SelectPlan(&'static str),
    SelectRadio(&'static str),
    SelectSegment(&'static str),
    ColorChanged(Color),
    PickPath,
    FocusInvalidInput,
    FocusProgrammaticInput,
    FocusSelect,
    SelectTab(DemoTab),
    TabCloseRequested(TabCloseRequest<DemoTab>),
    TabContextRequested(ContextRequest<DemoTab>),
    TabReordered(TabDrop<DemoTab>),
    TabTornOff(TabTearOff<DemoTab>),
    ToggleDirtyTab,
    SelectCard(usize),
    SelectItem(usize),
    ToggleTree(DemoTreeNode),
    TreeEvent(TreeEvent<DemoTreeNode>),
    TreeSelectionModeChanged(SelectionMode),
    TreeDeferredLoaded(DemoTreeNode),
    TreeConfigLoadFailed,
    TreeContextMenuAction(&'static str),
    TreeContextMenuDismissed,
    SplitRatioChanged(f32),
    VerticalSplitRatioChanged(f32),
    ShowDialog(DialogKind),
    CloseDialog,
    TogglePopover(PopoverKind),
    ClosePopover,
    ToggleNestedPopover,
    CloseNestedPopover,
    ToggleMenu(MenuKind),
    CloseMenu,
    MenuPinnedChanged(CheckboxState),
    MenuModeChanged(&'static str),
    ToggleAutocomplete(AutocompleteKind),
    CloseAutocomplete,
    AutocompleteQueryChanged(String),
    AutocompleteSelected(String),
    ClearAutocomplete,
    TogglePalette,
    PaletteDismissed,
    CommandQueryChanged(String),
    SelectCommand(&'static str),
    FeedbackModeChanged(FeedbackMode),
    PushToast(ToastKind),
    PushToastBurst,
    PushActionableToast,
    ToastActionAcknowledged,
    PushErrorToast,
    Noop,
}

impl Default for FormState {
    fn default() -> Self {
        Self {
            name: "Ada Lovelace".to_owned(),
            email: "ada@example.com".to_owned(),
            search: String::new(),
            secret: "correct horse battery staple".to_owned(),
            path: "/Users/ada/projects/nive/examples/widget-gallery".to_owned(),
            checked: CheckboxState::Checked,
            enabled: true,
            selected_plan: Some("Pro"),
            radio: None,
            segment: "Preview",
            color: Color::from_rgb8(64, 123, 255),
        }
    }
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            active_dialog: None,
            active_popover: None,
            nested_popover_open: false,
            active_menu: None,
            menu_pinned: CheckboxState::Unchecked,
            menu_mode: Some("standard"),
            active_autocomplete: None,
            autocomplete_query: "niv".to_owned(),
            autocomplete_selected: None,
            palette_open: false,
            command_query: String::new(),
            selected_command: None,
        }
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        let mut tree_state = TreeState::default();
        tree_state.expand(DemoTreeNode::Examples);
        tree_state.expand(DemoTreeNode::WidgetGallery);
        tree_state.expand(DemoTreeNode::Src);
        tree_state.expand(DemoTreeNode::Pages);
        tree_state.expand(DemoTreeNode::Archived);
        tree_state.select_only(DemoTreeNode::AppRs);
        tree_state.select_many([DemoTreeNode::AppRs, DemoTreeNode::LayoutNavRs]);
        tree_state.selection.focused = Some(DemoTreeNode::LayoutNavRs);
        tree_state.selection.anchor = Some(DemoTreeNode::AppRs);

        Self {
            tab: DemoTab::Overview,
            tab_order: vec![
                DemoTab::PinnedNotes,
                DemoTab::Overview,
                DemoTab::Details,
                DemoTab::LongLabel,
                DemoTab::Console,
                DemoTab::Search,
                DemoTab::Preview,
                DemoTab::Logs,
            ],
            dirty_tab: true,
            selected_card: 0,
            selected_item: 0,
            split_ratio: 0.42,
            vertical_split_ratio: 0.35,
            expanded_tree_nodes: [
                DemoTreeNode::Examples,
                DemoTreeNode::WidgetGallery,
                DemoTreeNode::Src,
                DemoTreeNode::Pages,
            ]
            .into_iter()
            .collect(),
            tree_state,
            tree_selection_mode: SelectionMode::Multiple,
            tree_deferred_loaded: false,
            tree_deferred_loading: false,
            tree_config_failed: false,
            tree_config_loading: false,
            tree_event_feedback: "Deferred branch: remote-packages pending".to_owned(),
            tree_context_feedback: "Context: none".to_owned(),
            tree_clipboard_feedback: "Clipboard: none".to_owned(),
            tree_drop_feedback: "Transfer: idle".to_owned(),
            tree_context_menu: None,
            tab_feedback: "TabBar: activation, close, context, reorder, and tear-off intents appear here".to_owned(),
        }
    }
}

impl Application for WidgetGallery {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-widget-gallery")
            .name("Widget Gallery")
            .theme_catalog(app_theme_catalog())
            // Explicit rather than relying on the `BottomEnd` default, so the
            // Gallery also demonstrates a non-default logical position.
            .toast_position(ToastPosition::TopEnd)
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
        (
            Self {
                route: PageId::Actions,
                search: String::new(),
                theme: ThemePreference::System,
                density: ThemeDensity::Standard,
                control_size: ControlSize::Sm,
                form: FormState::default(),
                overlays: OverlayState::default(),
                feedback: FeedbackMode::Loaded,
                layout: LayoutState::default(),
                #[cfg(feature = "devtools")]
                dev: DevState::new(),
            },
            (),
        )
    }

    fn update(
        &mut self,
        _context: Context<'_, Self::Window>,
        _message_context: MessageContext<Self::Window>,
        message: Self::Message,
    ) -> impl Into<Effect<Self::Message, Self::Window>> {
        let mut effect = Effect::none();

        match message {
            Message::Navigate(route) => self.route = route,
            Message::SearchChanged(value) => self.search = value,
            Message::ThemeChanged(theme) => {
                self.theme = theme;
                effect = Effect::theme(theme);
            }
            Message::DensityChanged(density) => self.density = density,
            Message::ControlSizeChanged(size) => self.control_size = size,
            Message::NameChanged(value) => self.form.name = value,
            Message::EmailChanged(value) => self.form.email = value,
            Message::InputSearchChanged(value) => self.form.search = value,
            Message::SecretChanged(value) => self.form.secret = value,
            Message::PathChanged(value) => self.form.path = value,
            Message::ToggleChecked(value) => self.form.checked = value,
            Message::ToggleEnabled(value) => self.form.enabled = value,
            Message::SelectPlan(value) => self.form.selected_plan = Some(value),
            Message::SelectRadio(value) => self.form.radio = Some(value),
            Message::SelectSegment(value) => self.form.segment = value,
            Message::ColorChanged(color) => self.form.color = color,
            Message::PickPath => {
                self.form.path = "/tmp/nive-gallery-selected".to_owned();
            }
            Message::FocusProgrammaticInput => {
                effect = effect.with_task(nive::widget::operation::focus(
                    crate::pages::inputs::programmatic_input_id(),
                ));
            }
            Message::FocusInvalidInput => {
                effect = effect.with_task(nive::widget::operation::focus(
                    crate::pages::inputs::invalid_input_id(),
                ));
            }
            Message::FocusSelect => {
                effect = effect.with_task(nive::widget::operation::focus(
                    crate::pages::inputs::select_focus_id(),
                ));
            }
            Message::SelectTab(tab) => {
                self.layout.tab = tab;
                self.layout.tab_feedback = format!("Activated tab: {}", tab.label());
            }
            Message::TabCloseRequested(request) => {
                let id_label = request.id.label().to_owned();
                self.layout.tab_order.retain(|t| *t != request.id);
                if self.layout.tab == request.id {
                    self.layout.tab = self
                        .layout
                        .tab_order
                        .first()
                        .copied()
                        .unwrap_or(DemoTab::Overview);
                }
                self.layout.tab_feedback = format!("Closed tab: {}", id_label);
            }
            Message::TabContextRequested(request) => {
                self.layout.tab_feedback = format!(
                    "Context request: {:?}, selected {}",
                    request.target,
                    request.selection.selected.len()
                );
            }
            Message::TabReordered(drop) => {
                if let Some(&dragged_id) = drop.payload.ids.first() {
                    #[allow(unreachable_patterns)]
                    let target_pos = match &drop.target {
                        nive::ui::widgets::TabDropTarget::Before(id) => Some((id, 0)),
                        nive::ui::widgets::TabDropTarget::After(id) => Some((id, 1)),
                        _ => None,
                    };
                    if let Some((target_id, offset)) = target_pos {
                        self.layout.tab_order.retain(|t| *t != dragged_id);
                        let mut idx = self
                            .layout
                            .tab_order
                            .iter()
                            .position(|t| *t == *target_id)
                            .map(|p| p + offset)
                            .unwrap_or(self.layout.tab_order.len());
                        if idx > self.layout.tab_order.len() {
                            idx = self.layout.tab_order.len();
                        }
                        self.layout.tab_order.insert(idx, dragged_id);
                    }
                }
                self.layout.tab_feedback = format!(
                    "Reordered tab: {} -> {:?} ({:?})",
                    drop.payload
                        .ids
                        .first()
                        .map(|id| id.label())
                        .unwrap_or("?"),
                    drop.target,
                    drop.operation
                );
            }
            Message::TabTornOff(tear_off) => {
                let id_label = tear_off
                    .payload
                    .ids
                    .first()
                    .map(|id| id.label())
                    .unwrap_or("?")
                    .to_owned();
                self.layout
                    .tab_order
                    .retain(|t| !tear_off.payload.ids.iter().any(|id| id == t));
                if self
                    .layout
                    .tab_order
                    .iter()
                    .all(|t| *t != self.layout.tab)
                {
                    self.layout.tab = self
                        .layout
                        .tab_order
                        .first()
                        .copied()
                        .unwrap_or(DemoTab::Overview);
                }
                self.layout.tab_feedback = format!(
                    "Torn off tab: {} at ({:.0}, {:.0})",
                    id_label, tear_off.position.x, tear_off.position.y
                );
            }
            Message::ToggleDirtyTab => self.layout.dirty_tab = !self.layout.dirty_tab,
            Message::SelectCard(index) => self.layout.selected_card = index,
            Message::SelectItem(index) => self.layout.selected_item = index,
            Message::ToggleTree(node) => {
                if !self.layout.expanded_tree_nodes.remove(&node) {
                    self.layout.expanded_tree_nodes.insert(node);
                }
            }
            Message::TreeEvent(event) => {
                if let Some(task) = handle_tree_event(&mut self.layout, event) {
                    effect = effect.with_task(task);
                }
            }
            Message::TreeSelectionModeChanged(mode) => {
                self.layout.tree_selection_mode = mode;
                if mode == SelectionMode::Single {
                    let focused = self
                        .layout
                        .tree_state
                        .focused()
                        .copied()
                        .or_else(|| self.layout.tree_state.selection.selected.iter().next().copied());
                    if let Some(id) = focused {
                        self.layout.tree_state.select_only(id);
                    } else {
                        self.layout.tree_state.clear_selection();
                    }
                }
                self.layout.tree_event_feedback =
                    format!("Selection mode: {mode:?}");
            }
            Message::TreeDeferredLoaded(id) => {
                if id == DemoTreeNode::RemotePackages {
                    self.layout.tree_deferred_loading = false;
                    self.layout.tree_deferred_loaded = true;
                    self.layout.tree_event_feedback =
                        "Loaded remote-packages with 2 children".to_owned();
                }
            }
            Message::TreeConfigLoadFailed => {
                self.layout.tree_config_loading = false;
                self.layout.tree_config_failed = true;
                self.layout.tree_event_feedback =
                    "remote-config failed to load; retry from the error row".to_owned();
            }
            Message::TreeContextMenuAction(action) => {
                if let Some(menu) = self.layout.tree_context_menu.take() {
                    self.layout.tree_context_feedback =
                        format!("Menu action: {action} on {}", menu.target.label());
                }
            }
            Message::TreeContextMenuDismissed => {
                self.layout.tree_context_menu = None;
            }
            Message::SplitRatioChanged(ratio) => self.layout.split_ratio = ratio,
            Message::VerticalSplitRatioChanged(ratio) => self.layout.vertical_split_ratio = ratio,
            Message::ShowDialog(dialog) => self.overlays.active_dialog = Some(dialog),
            Message::CloseDialog => self.overlays.active_dialog = None,
            Message::TogglePopover(popover) => {
                self.overlays.active_popover =
                    (self.overlays.active_popover != Some(popover)).then_some(popover);
                if self.overlays.active_popover != Some(PopoverKind::Nested) {
                    self.overlays.nested_popover_open = false;
                }
            }
            Message::ClosePopover => {
                self.overlays.active_popover = None;
                self.overlays.nested_popover_open = false;
            }
            Message::ToggleNestedPopover => {
                if self.overlays.active_popover == Some(PopoverKind::Nested) {
                    self.overlays.nested_popover_open = !self.overlays.nested_popover_open;
                }
            }
            Message::CloseNestedPopover => self.overlays.nested_popover_open = false,
            Message::ToggleMenu(menu) => {
                self.overlays.active_menu =
                    (self.overlays.active_menu != Some(menu)).then_some(menu);
            }
            Message::CloseMenu => self.overlays.active_menu = None,
            Message::MenuPinnedChanged(state) => self.overlays.menu_pinned = state,
            Message::MenuModeChanged(mode) => self.overlays.menu_mode = Some(mode),
            Message::ToggleAutocomplete(kind) => {
                self.overlays.active_autocomplete =
                    (self.overlays.active_autocomplete != Some(kind)).then_some(kind);
            }
            Message::CloseAutocomplete => self.overlays.active_autocomplete = None,
            Message::AutocompleteQueryChanged(query) => {
                self.overlays.autocomplete_query = query;
                self.overlays.autocomplete_selected = None;
            }
            Message::AutocompleteSelected(value) => {
                self.overlays.autocomplete_query.clone_from(&value);
                self.overlays.autocomplete_selected = Some(value);
            }
            Message::ClearAutocomplete => {
                self.overlays.autocomplete_query.clear();
                self.overlays.autocomplete_selected = None;
            }
            Message::TogglePalette => {
                self.overlays.palette_open = !self.overlays.palette_open;
                self.overlays.command_query.clear();
            }
            Message::PaletteDismissed => {
                self.overlays.palette_open = false;
                self.overlays.command_query.clear();
            }
            Message::CommandQueryChanged(value) => self.overlays.command_query = value,
            Message::SelectCommand(id) => {
                self.overlays.selected_command = Some(id);
                self.overlays.palette_open = false;
                self.overlays.command_query.clear();
            }
            Message::FeedbackModeChanged(mode) => self.feedback = mode,
            Message::PushToast(kind) => {
                effect = Effect::toast(match kind {
                    ToastKind::Info => Toast::info("Index rebuilt"),
                    ToastKind::Success => Toast::success("Saved"),
                    ToastKind::Warning => Toast::warning("Configuration warning"),
                    ToastKind::Danger => Toast::danger("Sync failed"),
                });
            }
            Message::PushToastBurst => {
                effect = Effect::toast(Toast::info("Notification 1 of 5"))
                    .with_toast(Toast::info("Notification 2 of 5"))
                    .with_toast(Toast::info("Notification 3 of 5"))
                    .with_toast(Toast::info("Notification 4 of 5"))
                    .with_toast(Toast::info("Notification 5 of 5"));
            }
            Message::PushActionableToast => {
                effect = Effect::toast(
                    Toast::info("New version available")
                        .with_body("Restart to apply the update.")
                        .with_action("Restart now", Message::ToastActionAcknowledged),
                );
            }
            Message::ToastActionAcknowledged => {
                effect = Effect::toast(Toast::success("Restarting"));
            }
            Message::PushErrorToast => {
                effect = Effect::toast(Toast::error(UserFacingError::custom(
                    "widget-gallery",
                    "Could not reach the sync service (endpoint: https://example.invalid)",
                )));
            }
            Message::Noop => {}
        }

        effect
    }

    fn view(
        &self,
        context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let resolved = app_theme_catalog_for_density(self.density).resolve(context.theme().mode());
        theme::runtime::set_active(resolved);
        let content = row![self.sidebar(), self.page()]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill);

        let items = pages::overlays::command_palette_items(&self.overlays.command_query);
        let palette_content = CommandPalette::new(content)
            .open(self.overlays.palette_open)
            .query(self.overlays.command_query.as_str())
            .placeholder("Search commands")
            .items(items)
            .on_query_change(Message::CommandQueryChanged)
            .on_dismiss(Message::PaletteDismissed);
        let view = ScreenView::new(palette_content);

        if let Some(dialog) = self.overlays.active_dialog {
            view.dialog(
                DialogRequest::new(pages::overlays::dialog(dialog))
                    .dismiss_on_backdrop(Message::CloseDialog)
                    .dismiss_on_escape(Message::CloseDialog),
            )
        } else {
            view
        }
    }

    fn theme(
        &self,
        _context: Context<'_, Self::Window>,
        _window: Option<WindowContext<Self::Window>>,
    ) -> ThemePreference {
        self.theme
    }

    fn window_title<'a>(
        &'a self,
        _context: Context<'a, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> impl Into<Cow<'a, str>> + 'a {
        Cow::Borrowed("Widget Gallery")
    }
}

fn app_theme_catalog() -> ThemeCatalog {
    app_theme_catalog_for_density(ThemeDensity::Standard)
}

fn app_theme_catalog_for_density(density: ThemeDensity) -> ThemeCatalog {
    ThemeCatalog::new(
        Theme::builder("Widget Gallery Light", ThemeMode::Light)
            .density(density)
            .icons(icons::APP_ICON_CATALOG)
            .build(),
        Theme::builder("Widget Gallery Dark", ThemeMode::Dark)
            .density(density)
            .icons(icons::APP_ICON_CATALOG)
            .build(),
    )
}

impl WidgetGallery {
    fn sidebar(&self) -> Element<'_, Message> {
        let mut entries = column![].spacing(4).width(Length::Fill);

        for entry in CATALOG.iter().filter(|entry| matches(entry, &self.search)) {
            entries = entries.push(layout::sidebar_button(entry, self.route == entry.id));
        }

        let themes = SegmentedControl::new(
            "Theme preference",
            self.theme,
            [
                SegmentedOption::new(ThemePreference::System, "System"),
                SegmentedOption::new(ThemePreference::Light, "Light"),
                SegmentedOption::new(ThemePreference::Dark, "Dark"),
            ],
        )
            .on_select(Message::ThemeChanged)
            .fill_width();

        let sizes = SegmentedControl::new(
            "Control size",
            self.control_size,
            [
                SegmentedOption::new(ControlSize::Xs, "XS"),
                SegmentedOption::new(ControlSize::Sm, "SM"),
                SegmentedOption::new(ControlSize::Md, "MD"),
                SegmentedOption::new(ControlSize::Lg, "LG"),
            ],
        )
            .on_select(Message::ControlSizeChanged)
            .fill_width();

        let densities = SegmentedControl::new(
            "Theme density",
            self.density,
            [
                SegmentedOption::new(ThemeDensity::Compact, "Compact"),
                SegmentedOption::new(ThemeDensity::Standard, "Standard"),
                SegmentedOption::new(ThemeDensity::Comfortable, "Comfortable"),
            ],
        )
            .on_select(Message::DensityChanged)
            .fill_width();

        Panel::new(
            column![
                ntext::title("Widget Gallery"),
                Input::new("Search widgets", &self.search).on_change(Message::SearchChanged),
                ntext::section_label("Theme"),
                themes,
                ntext::section_label("Density"),
                densities,
                ntext::section_label("Control size"),
                sizes,
                scrollable(entries)
                    .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
                    .height(Length::Fill),
            ]
            .spacing(12)
            .height(Length::Fill),
        )
        .role(SurfaceRole::Chrome)
        .body_padding(16)
        .width(300)
        .height(Length::Fill)
        .into()
    }

    fn page(&self) -> Element<'_, Message> {
        let page = match self.route {
            PageId::Actions => pages::actions::view(self),
            PageId::Inputs => pages::inputs::view(self),
            PageId::Display => pages::display::view(self),
            PageId::LayoutNav => pages::layout_nav::view(self),
            PageId::Overlays => pages::overlays::view(self),
            PageId::Feedback => pages::feedback::view(self),
            PageId::Theme => pages::theme::view(self),
            PageId::Icons => pages::icons::view(self),
            PageId::Motion => pages::motion::view(self),
        };

        container(
            scrollable(page)
                .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
                .spacing(16),
        )
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[cfg(feature = "devtools")]
impl nive::DevtoolsApp for WidgetGallery {
    type State = DevState;

    fn devtool_state_mut(&mut self) -> &mut DevState {
        &mut self.dev
    }
}

pub fn page_shell<'a>(
    id: PageId,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    layout::page_shell(entry_for(id), content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_catalog_page_builds_from_the_deterministic_fixture() {
        let mut app = WidgetGallery::test_fixture();

        for route in [
            PageId::Actions,
            PageId::Inputs,
            PageId::Display,
            PageId::LayoutNav,
            PageId::Overlays,
            PageId::Feedback,
            PageId::Theme,
            PageId::Icons,
            PageId::Motion,
        ] {
            app.route = route;
            let _: Element<'_, Message> = app.page();
        }
    }

    #[test]
    fn popup_fixture_state_is_controlled_and_independent() {
        let state = OverlayState::default();

        assert_eq!(state.active_popover, None);
        assert_eq!(state.active_menu, None);
        assert_eq!(state.active_autocomplete, None);
        assert!(!state.nested_popover_open);
        assert_eq!(state.menu_mode, Some("standard"));
        assert_eq!(state.autocomplete_query, "niv");
        assert_eq!(state.autocomplete_selected, None);
    }
}
