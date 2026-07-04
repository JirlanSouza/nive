use std::{borrow::Cow, collections::BTreeSet};

use nive::prelude::ui::DialogRequest;
use nive::prelude::*;
use nive::ui::theme::{ControlSize, SurfaceRole};
use nive::ui::widgets::{text as ntext, TreeEvent, TreeState};
use nive::ui::SelectionMode;
use nive::widget::{column, row};

use crate::catalog::{entry_for, matches, PageId, CATALOG};
#[cfg(feature = "devtools")]
use crate::fixtures::DevState;
use crate::{layout, pages};

mod tree_helpers;

use tree_helpers::{handle_tree_event, load_deferred_tree_branch};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoTab {
    Overview,
    Details,
    LongLabel,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Basic,
    Destructive,
    LongContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverKind {
    Start,
    End,
    Wide,
    Collision,
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
        }
    }
}

pub struct FormState {
    pub name: String,
    pub email: String,
    pub search: String,
    pub secret: String,
    pub path: String,
    pub checked: bool,
    pub enabled: bool,
    pub selected_plan: Option<&'static str>,
    pub segment: &'static str,
    pub color: Color,
}

#[derive(Default)]
pub struct OverlayState {
    pub active_dialog: Option<DialogKind>,
    pub active_popover: Option<PopoverKind>,
    pub command_query: String,
    pub selected_command: Option<&'static str>,
}

pub struct LayoutState {
    pub tab: DemoTab,
    pub dirty_tab: bool,
    pub selected_card: usize,
    pub selected_item: usize,
    pub split_ratio: f32,
    pub expanded_tree_nodes: BTreeSet<DemoTreeNode>,
    pub tree_state: TreeState<DemoTreeNode>,
    pub tree_selection_mode: SelectionMode,
    pub tree_deferred_loaded: bool,
    pub tree_deferred_loading: bool,
    pub tree_event_feedback: String,
    pub tree_context_feedback: String,
    pub tree_clipboard_feedback: String,
    pub tree_drop_feedback: String,
}

pub struct WidgetGallery {
    pub route: PageId,
    pub search: String,
    pub theme: ThemePreference,
    pub control_size: ControlSize,
    pub form: FormState,
    pub overlays: OverlayState,
    pub feedback: FeedbackMode,
    pub layout: LayoutState,
    #[cfg(feature = "devtools")]
    pub dev: DevState,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(PageId),
    SearchChanged(String),
    ThemeChanged(ThemePreference),
    ControlSizeChanged(ControlSize),
    NameChanged(String),
    EmailChanged(String),
    InputSearchChanged(String),
    SecretChanged(String),
    PathChanged(String),
    ToggleChecked(bool),
    ToggleEnabled(bool),
    SelectPlan(&'static str),
    SelectSegment(&'static str),
    ColorChanged(Color),
    PickPath,
    SelectTab(DemoTab),
    ToggleDirtyTab,
    SelectCard(usize),
    SelectItem(usize),
    ToggleTree(DemoTreeNode),
    TreeEvent(TreeEvent<DemoTreeNode>),
    TreeSelectionModeChanged(SelectionMode),
    TreeDeferredLoaded(DemoTreeNode),
    SplitRatioChanged(f32),
    ShowDialog(DialogKind),
    CloseDialog,
    TogglePopover(PopoverKind),
    ClosePopover,
    CommandQueryChanged(String),
    SelectCommand(&'static str),
    FeedbackModeChanged(FeedbackMode),
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
            checked: true,
            enabled: true,
            selected_plan: Some("Pro"),
            segment: "Preview",
            color: Color::from_rgb8(64, 123, 255),
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
        tree_state.select_only(DemoTreeNode::AppRs);
        tree_state.select_many([DemoTreeNode::AppRs, DemoTreeNode::LayoutNavRs]);
        tree_state.selection.focused = Some(DemoTreeNode::LayoutNavRs);
        tree_state.selection.anchor = Some(DemoTreeNode::AppRs);

        Self {
            tab: DemoTab::Overview,
            dirty_tab: true,
            selected_card: 0,
            selected_item: 0,
            split_ratio: 0.42,
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
            tree_event_feedback: "Deferred branch: remote-packages pending".to_owned(),
            tree_context_feedback: "Context: none".to_owned(),
            tree_clipboard_feedback: "Clipboard: none".to_owned(),
            tree_drop_feedback: "Transfer: idle".to_owned(),
        }
    }
}

impl Application for WidgetGallery {
    type Message = Message;
    type Window = ();
    type Bootstrap = ();

    fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
        ApplicationConfig::new("nive-example-widget-gallery").name("Widget Gallery")
    }

    fn init(
        _context: Context<'_, Self::Window>,
        _bootstrap: Self::Bootstrap,
    ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
        (
            Self {
                route: PageId::Actions,
                search: String::new(),
                theme: ThemePreference::System,
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
        _window: Option<WindowContext<Self::Window>>,
        message: Self::Message,
    ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
        let mut update = AppUpdate::none();

        match message {
            Message::Navigate(route) => self.route = route,
            Message::SearchChanged(value) => self.search = value,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ControlSizeChanged(size) => self.control_size = size,
            Message::NameChanged(value) => self.form.name = value,
            Message::EmailChanged(value) => self.form.email = value,
            Message::InputSearchChanged(value) => self.form.search = value,
            Message::SecretChanged(value) => self.form.secret = value,
            Message::PathChanged(value) => self.form.path = value,
            Message::ToggleChecked(value) => self.form.checked = value,
            Message::ToggleEnabled(value) => self.form.enabled = value,
            Message::SelectPlan(value) => self.form.selected_plan = Some(value),
            Message::SelectSegment(value) => self.form.segment = value,
            Message::ColorChanged(color) => self.form.color = color,
            Message::PickPath => {
                self.form.path = "/tmp/nive-gallery-selected".to_owned();
            }
            Message::SelectTab(tab) => self.layout.tab = tab,
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
                    update = update.task(task);
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
            Message::SplitRatioChanged(ratio) => self.layout.split_ratio = ratio,
            Message::ShowDialog(dialog) => self.overlays.active_dialog = Some(dialog),
            Message::CloseDialog => self.overlays.active_dialog = None,
            Message::TogglePopover(popover) => {
                self.overlays.active_popover =
                    (self.overlays.active_popover != Some(popover)).then_some(popover);
            }
            Message::ClosePopover => self.overlays.active_popover = None,
            Message::CommandQueryChanged(value) => self.overlays.command_query = value,
            Message::SelectCommand(id) => self.overlays.selected_command = Some(id),
            Message::FeedbackModeChanged(mode) => self.feedback = mode,
            Message::Noop => {}
        }

        update.theme(self.theme)
    }

    fn view(
        &self,
        _context: Context<'_, Self::Window>,
        _window: WindowContext<Self::Window>,
    ) -> ScreenView<'_, Self::Message> {
        let content = row![self.sidebar(), self.page()]
            .spacing(0)
            .height(Length::Fill)
            .width(Length::Fill);
        let view = ScreenView::new(content);

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

impl WidgetGallery {
    fn sidebar(&self) -> Element<'_, Message> {
        let mut entries = column![].spacing(4).width(Length::Fill);

        for entry in CATALOG.iter().filter(|entry| matches(entry, &self.search)) {
            entries = entries.push(layout::sidebar_button(entry, self.route == entry.id));
        }

        let themes = SegmentedControl::new()
            .item(
                SegmentedItem::new("System")
                    .selected(self.theme == ThemePreference::System)
                    .on_press(Message::ThemeChanged(ThemePreference::System)),
            )
            .item(
                SegmentedItem::new("Light")
                    .selected(self.theme == ThemePreference::Light)
                    .on_press(Message::ThemeChanged(ThemePreference::Light)),
            )
            .item(
                SegmentedItem::new("Dark")
                    .selected(self.theme == ThemePreference::Dark)
                    .on_press(Message::ThemeChanged(ThemePreference::Dark)),
            )
            .fill();

        let sizes = SegmentedControl::new()
            .item(size_item("XS", ControlSize::Xs, self.control_size))
            .item(size_item("SM", ControlSize::Sm, self.control_size))
            .item(size_item("MD", ControlSize::Md, self.control_size))
            .item(size_item("LG", ControlSize::Lg, self.control_size))
            .fill();

        Panel::new(
            column![
                ntext::title("Widget Gallery"),
                Input::new("Search widgets", &self.search).on_input(Message::SearchChanged),
                ntext::section_label("Theme"),
                themes,
                ntext::section_label("Control size"),
                sizes,
                scrollable(entries).height(Length::Fill),
            ]
            .spacing(12)
            .height(Length::Fill),
        )
        .role(SurfaceRole::Chrome)
        .padding(16)
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

        container(scrollable(page).spacing(16))
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

fn size_item(
    label: &'static str,
    size: ControlSize,
    active: ControlSize,
) -> SegmentedItem<'static, Message> {
    SegmentedItem::new(label)
        .selected(active == size)
        .on_press(Message::ControlSizeChanged(size))
}
