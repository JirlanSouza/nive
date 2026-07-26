use std::borrow::Cow;

use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Shell, Widget,
    },
    widget::{column, container, row, scrollable, text, Space},
    Alignment, Background, Border, Event, Length, Point, Rectangle, Shadow, Size, Vector,
};

use super::item::CommandPaletteItem;
use crate::theme::{
    self, spacing::SpacingScale, text as theme_text, BorderRole, ControlRole, ControlState,
    SurfaceRole, TextRole, TypographyRole,
};
use crate::widgets::overlays::modal_host::{InitialFocusFn, ModalAlignment, ModalHost};
use crate::widgets::{icon, scrollable::overlay_scrollbar, Input};
use crate::{Element, Renderer, Theme};

const PALETTE_WIDTH: f32 = 480.0;
const TOP_OFFSET: f32 = 96.0;
const MAX_BODY_HEIGHT: f32 = 360.0;
const FRAME_RADIUS: f32 = 8.0;
const SEARCH_INPUT_ID: &str = "nive-command-palette-search";
const LIST_SCROLLABLE_ID: &str = "nive-command-palette-list";

/// A self-contained typed composite command palette.
///
/// `CommandPalette` owns its viewport-centered top placement, single
/// search-input focus target, filtered keyboard navigation, highlight,
/// ensure-visible scrolling, empty state, and visual frame. Hosts supply
/// only [`open`](Self::open), the controlled query, filtered typed
/// [`CommandPaletteItem`]s, [`on_query_change`](Self::on_query_change), and
/// [`on_dismiss`](Self::on_dismiss) — mirroring `Menu::open`.
///
/// The search query is application-controlled: the application owns the
/// query text and runs its own filtering (with the provided
/// [`super::command_palette_filter`] default matcher, or its own). The
/// widget owns only transient state — the highlight index,
/// `ArrowUp`/`ArrowDown`/`Enter`/`Escape` result navigation, and
/// ensure-visible scrolling. The single focus target is the native search
/// `Input`, so `Home`/`End`/`Left`/`Right` and text stay its own caret and
/// editing controls.
///
/// `CommandPalette` shares a private modal-hosting kernel with
/// [`super::super::super::overlays::DialogHost`]: base content stays drawn
/// but externally inert while the palette is open, `Escape` and an outside
/// primary press each publish [`on_dismiss`](Self::on_dismiss) exactly once,
/// and a window hosts at most one canonical modal session — a later
/// `CommandPalette` or `Dialog` mount replaces rather than stacks.
///
/// It exposes no generic `Length` and no raw width/height/padding/radius
/// builders; placement and geometry are fixed and viewport-clamped:
///
/// ```compile_fail
/// use nive_ui::widgets::CommandPalette;
/// let palette: CommandPalette<'_, ()> = CommandPalette::new(iced::widget::text("Base"));
/// let _ = palette.width(iced::Length::Fill);
/// ```
///
/// The shared modal kernel and the palette's own highlight/navigation/scroll
/// state are private; the removed `command_palette_view` render helper and
/// `CommandPaletteRow` type (renamed to [`super::CommandPaletteItem`]) are
/// gone with no compatibility facade:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::modal_host::ModalHost;
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::command_palette_view;
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::CommandPaletteRow;
/// ```
pub struct CommandPalette<'a, Message> {
    content: Element<'a, Message>,
    open: bool,
    query: Cow<'a, str>,
    placeholder: Cow<'a, str>,
    items: Vec<CommandPaletteItem<'a, Message>>,
    on_query_change: Option<Box<dyn Fn(String) -> Message + 'a>>,
    on_dismiss: Option<Message>,
}

impl<'a, Message> CommandPalette<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            open: false,
            query: Cow::Borrowed(""),
            placeholder: Cow::Borrowed("Search commands"),
            items: Vec::new(),
            on_query_change: None,
            on_dismiss: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn query(mut self, query: impl Into<Cow<'a, str>>) -> Self {
        self.query = query.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<Cow<'a, str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn items(
        mut self,
        items: impl IntoIterator<Item = CommandPaletteItem<'a, Message>>,
    ) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn on_query_change(mut self, f: impl Fn(String) -> Message + 'a) -> Self {
        self.on_query_change = Some(Box::new(f));
        self
    }

    pub fn on_dismiss(mut self, message: Message) -> Self {
        self.on_dismiss = Some(message);
        self
    }
}

impl<'a, Message> From<CommandPalette<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(palette: CommandPalette<'a, Message>) -> Self {
        let mut modal_host = ModalHost::new(palette.content);

        if palette.open {
            let search_id = iced::widget::Id::new(SEARCH_INPUT_ID);

            let mut input = Input::new(palette.placeholder, palette.query.clone())
                .id(search_id.clone())
                .semantic_name("Command search")
                .md();
            if let Some(on_query_change) = palette.on_query_change {
                input = input.on_change_maybe(Some(on_query_change));
            }
            let input_element: Element<'a, Message> = input.into();

            let (list_element, rows) = build_list(&palette.items, palette.query.as_ref());
            let body = PaletteBody {
                input: input_element,
                list: list_element,
                rows,
            };

            let initial_focus: InitialFocusFn<'a, Message> =
                Box::new(move |content, tree, layout, renderer| {
                    let mut op = operation::focusable::focus::<()>(search_id.clone());
                    content
                        .as_widget_mut()
                        .operate(tree, layout, renderer, &mut op);
                });

            modal_host = modal_host.modal(
                body,
                palette.on_dismiss.clone(),
                palette.on_dismiss,
                initial_focus,
                ModalAlignment::TopCentered(TOP_OFFSET),
            );
        }

        modal_host.into()
    }
}

struct PaletteRowMeta<Message> {
    id: String,
    message: Option<Message>,
}

/// Builds the rows list content plus its parallel navigation metadata, or
/// the distinct empty state for an empty query versus a non-matching query.
fn build_list<'a, Message>(
    items: &[CommandPaletteItem<'a, Message>],
    query: &str,
) -> (Element<'a, Message>, Vec<PaletteRowMeta<Message>>)
where
    Message: Clone + 'a,
{
    let spacing = theme::spacing();

    if items.is_empty() {
        return (empty_state(query, spacing), Vec::new());
    }

    let rows = items
        .iter()
        .map(|item| PaletteRowMeta {
            id: item.id.to_owned(),
            message: item.activated().cloned(),
        })
        .collect();

    let mut list = iced::widget::Column::new().spacing(0.0).width(Length::Fill);
    for item in items {
        list = list.push(row_element(item, spacing));
    }

    let list_element: Element<'a, Message> = scrollable(list)
        .id(iced::widget::Id::new(LIST_SCROLLABLE_ID))
        .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
        .width(Length::Fill)
        .height(Length::Shrink)
        .into();

    (list_element, rows)
}

fn row_element<'a, Message>(
    item: &CommandPaletteItem<'a, Message>,
    spacing: SpacingScale,
) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let label_color = if item.enabled {
        TextRole::Primary
    } else {
        TextRole::Muted
    };

    let label = text(item.label)
        .size(theme::typography(TypographyRole::Body).size)
        .style(theme_text::style(label_color));

    let label_column: Element<'a, Message> = if let Some(description) = item.description {
        column![
            label,
            text(description)
                .size(theme::typography(TypographyRole::Caption).size)
                .style(theme_text::style(TextRole::Muted)),
        ]
        .spacing(spacing.xxs)
        .width(Length::Fill)
        .into()
    } else {
        label.width(Length::Fill).into()
    };

    let leading: Element<'a, Message> = match item.icon {
        Some(role) => icon::reference(role).md().into(),
        None => Space::new().width(Length::Shrink).into(),
    };

    let trailing: Element<'a, Message> = match item.shortcut_label.clone() {
        Some(shortcut) => text(shortcut)
            .size(theme::typography(TypographyRole::CodeSmall).size)
            .font(theme::typography(TypographyRole::CodeSmall).font)
            .style(theme_text::style(TextRole::Muted))
            .into(),
        None => Space::new().width(Length::Shrink).into(),
    };

    let content = row![leading, label_column, trailing]
        .align_y(Alignment::Center)
        .spacing(spacing.md)
        .width(Length::Fill)
        .padding([spacing.xs, spacing.md]);

    container(content).width(Length::Fill).into()
}

fn empty_state<'a, Message>(query: &str, spacing: SpacingScale) -> Element<'a, Message>
where
    Message: Clone + 'a,
{
    let message: &str = if query.trim().is_empty() {
        "Type to search commands"
    } else {
        "No commands match this search"
    };

    let content = text(message)
        .size(theme::typography(TypographyRole::Body).size)
        .style(theme_text::style(TextRole::Muted));

    container(content)
        .width(Length::Fill)
        .padding([spacing.lg, spacing.xl])
        .center_x(Length::Fill)
        .into()
}

/// Survives across rebuilds via `Tree::diff`: the widget-owned transient
/// highlight index (stored by item identity, not position, so it reconciles
/// across query changes and reorderings the same way Menu reconciles its
/// highlight by label after a rebuild).
#[derive(Debug, Default, Clone)]
struct PaletteBodyState {
    highlighted_id: Option<String>,
}

/// Private composite content handed to the shared modal kernel: the search
/// `Input` plus its filtered result list, owning transient highlight
/// navigation, activation, and the fixed clamped visual frame.
struct PaletteBody<'a, Message> {
    input: Element<'a, Message>,
    list: Element<'a, Message>,
    rows: Vec<PaletteRowMeta<Message>>,
}

impl<'a, Message> PaletteBody<'a, Message>
where
    Message: Clone + 'a,
{
    fn eligible_ids(&self) -> Vec<&str> {
        self.rows
            .iter()
            .filter(|row| row.message.is_some())
            .map(|row| row.id.as_str())
            .collect()
    }

    fn message_for(&self, id: &str) -> Option<Message> {
        self.rows
            .iter()
            .find(|row| row.id == id)
            .and_then(|row| row.message.clone())
    }

    fn row_index(&self, id: &str) -> Option<usize> {
        self.rows.iter().position(|row| row.id == id)
    }
}

impl<'a, Message> Widget<Message, Theme, Renderer> for PaletteBody<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PaletteBodyState>()
    }

    fn state(&self) -> tree::State {
        // `diff()` only reconciles the highlight on a *rebuild*; the very
        // first render for a freshly opened session never calls `diff()`
        // (the kernel builds this tree fresh via `Tree::new()`), so the
        // initial highlight must be resolved here too.
        let highlighted_id = self.eligible_ids().into_iter().next().map(str::to_owned);
        tree::State::new(PaletteBodyState { highlighted_id })
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.input), Tree::new(&self.list)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.input.as_widget(), self.list.as_widget()]);

        let eligible = self.eligible_ids();
        let state = tree.state.downcast_mut::<PaletteBodyState>();
        let still_eligible = state
            .highlighted_id
            .as_deref()
            .is_some_and(|id| eligible.contains(&id));
        if !still_eligible {
            state.highlighted_id = eligible.first().map(|id| (*id).to_owned());
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn size_hint(&self) -> Size<Length> {
        self.size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let spacing = theme::spacing();
        let pad = spacing.sm;
        let gap = spacing.xs;

        let max = limits.max();
        let width = PALETTE_WIDTH.min(max.width).max(0.0);
        let inner_width = (width - 2.0 * pad).max(0.0);

        let input_limits = layout::Limits::new(Size::ZERO, Size::new(inner_width, f32::INFINITY));
        let input_node =
            self.input
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &input_limits);
        let input_height = input_node.size().height;

        let max_body_height = MAX_BODY_HEIGHT.min((max.height - 2.0 * pad).max(0.0));
        let list_max_height = (max_body_height - input_height - gap).max(0.0);
        let list_limits = layout::Limits::new(Size::ZERO, Size::new(inner_width, list_max_height));
        let list_node =
            self.list
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &list_limits);
        let list_height = list_node.size().height.min(list_max_height);

        let input_positioned = input_node.move_to(Point::new(pad, pad));
        let list_positioned = list_node.move_to(Point::new(pad, pad + input_height + gap));

        let total_height = input_height + gap + list_height + 2.0 * pad;

        layout::Node::with_children(
            Size::new(width, total_height),
            vec![input_positioned, list_positioned],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let mut children = layout.children();
        if let (Some(input_layout), Some(list_layout)) = (children.next(), children.next()) {
            self.input.as_widget_mut().operate(
                &mut tree.children[0],
                input_layout,
                renderer,
                operation,
            );
            self.list.as_widget_mut().operate(
                &mut tree.children[1],
                list_layout,
                renderer,
                operation,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(navigation) = navigation_key(event) {
            self.handle_navigation(navigation, tree, layout, renderer, shell);
            shell.capture_event();
            return;
        }

        // `Escape` must reach the shared modal kernel's own dismissal
        // handling (`on_dismiss`), but Iced's built-in `text_input` widget
        // (the search Input's inner implementation) unconditionally
        // captures `Escape` itself to blur — never delegate it to children,
        // or the kernel would never see an uncaptured event to dismiss on.
        if is_non_repeated_escape(event) {
            return;
        }

        let mut children = layout.children();
        let (Some(input_layout), Some(list_layout)) = (children.next(), children.next()) else {
            return;
        };

        self.input.as_widget_mut().update(
            &mut tree.children[0],
            event,
            input_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if shell.is_event_captured() {
            return;
        }

        self.list.as_widget_mut().update(
            &mut tree.children[1],
            event,
            list_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let mut children = layout.children();
        let (Some(input_layout), Some(list_layout)) = (children.next(), children.next()) else {
            return mouse::Interaction::Idle;
        };

        let input_interaction = self.input.as_widget().mouse_interaction(
            &tree.children[0],
            input_layout,
            cursor,
            viewport,
            renderer,
        );
        let list_interaction = self.list.as_widget().mouse_interaction(
            &tree.children[1],
            list_layout,
            cursor,
            viewport,
            renderer,
        );

        input_interaction.max(list_interaction)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        inherited_style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = frame_style(theme);
        container::draw_background(renderer, &style, bounds);

        let mut children = layout.children();
        let (Some(input_layout), Some(list_layout)) = (children.next(), children.next()) else {
            return;
        };

        let state = tree.state.downcast_ref::<PaletteBodyState>();
        if let Some(highlighted_id) = state.highlighted_id.as_deref() {
            if let Some(index) = self.row_index(highlighted_id) {
                if let Some(row_bounds) = self.row_bounds(list_layout, index) {
                    draw_highlight(renderer, theme, row_bounds);
                }
            }
        }

        self.input.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            inherited_style,
            input_layout,
            cursor,
            viewport,
        );
        self.list.as_widget().draw(
            &tree.children[1],
            renderer,
            theme,
            inherited_style,
            list_layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();
        let (Some(input_layout), Some(list_layout)) = (children.next(), children.next()) else {
            return None;
        };

        let (input_state, list_state) = tree.children.split_at_mut(1);
        let mut overlays = Vec::with_capacity(2);
        overlays.extend(self.input.as_widget_mut().overlay(
            &mut input_state[0],
            input_layout,
            renderer,
            viewport,
            translation,
        ));
        overlays.extend(self.list.as_widget_mut().overlay(
            &mut list_state[0],
            list_layout,
            renderer,
            viewport,
            translation,
        ));

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message> From<PaletteBody<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(body: PaletteBody<'a, Message>) -> Self {
        Element::new(body)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NavigationKey {
    Up,
    Down,
    Enter,
}

fn is_non_repeated_escape(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
            repeat: false,
            ..
        })
    )
}

fn navigation_key(event: &Event) -> Option<NavigationKey> {
    let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event else {
        return None;
    };

    match key {
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(NavigationKey::Up),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => {
            Some(NavigationKey::Down)
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => Some(NavigationKey::Enter),
        _ => None,
    }
}

impl<'a, Message> PaletteBody<'a, Message>
where
    Message: Clone + 'a,
{
    fn handle_navigation(
        &mut self,
        navigation: NavigationKey,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        let eligible: Vec<String> = self.eligible_ids().into_iter().map(str::to_owned).collect();

        match navigation {
            NavigationKey::Enter => {
                let state = tree.state.downcast_ref::<PaletteBodyState>();
                if let Some(id) = state.highlighted_id.clone() {
                    if let Some(message) = self.message_for(&id) {
                        shell.publish(message);
                    }
                }
            }
            NavigationKey::Up | NavigationKey::Down => {
                if eligible.is_empty() {
                    return;
                }

                let state = tree.state.downcast_mut::<PaletteBodyState>();
                let current_position = state
                    .highlighted_id
                    .as_deref()
                    .and_then(|id| eligible.iter().position(|candidate| candidate == id));

                let delta: isize = if navigation == NavigationKey::Down {
                    1
                } else {
                    -1
                };
                let next_position = match current_position {
                    None => 0,
                    Some(position) => {
                        (position as isize + delta).rem_euclid(eligible.len() as isize) as usize
                    }
                };

                state.highlighted_id = Some(eligible[next_position].clone());
                let new_id = eligible[next_position].clone();

                if let Some(index) = self.row_index(&new_id) {
                    let mut children = layout.children();
                    if let (Some(_), Some(list_layout)) = (children.next(), children.next()) {
                        if let Some(row_bounds) = self.row_bounds(list_layout, index) {
                            let scrollable_id = self.scrollable_id();
                            if let Some(scrollable_id) = scrollable_id {
                                let mut op = crate::widgets::overlays::anchored_overlay::scroll::ensure_visible(
                                    scrollable_id,
                                    row_bounds,
                                );
                                self.list.as_widget_mut().operate(
                                    &mut tree.children[1],
                                    list_layout,
                                    renderer,
                                    &mut op,
                                );
                            }
                        }
                    }
                }

                shell.invalidate_layout();
                shell.request_redraw();
            }
        }
    }

    /// Reads the currently rendered row bounds from the persistent layout
    /// tree: `list_layout` is the scrollable's own layout, whose single
    /// child is the content (row column) node — already positioned
    /// according to the scrollable's current scroll offset, matching the
    /// coordinate space `scroll::ensure_visible` expects.
    fn row_bounds(&self, list_layout: Layout<'_>, index: usize) -> Option<Rectangle> {
        let content_layout = list_layout.children().next()?;
        content_layout.children().nth(index).map(|row| row.bounds())
    }

    fn scrollable_id(&self) -> Option<iced::widget::Id> {
        // The list content is only a real scrollable while there are rows to
        // navigate; the empty-state list has no rows and no scrollable id to
        // target.
        (!self.rows.is_empty()).then(|| iced::widget::Id::new(LIST_SCROLLABLE_ID))
    }
}

fn draw_highlight(renderer: &mut Renderer, theme: &Theme, bounds: Rectangle) {
    use iced::advanced::Renderer as _;

    let control = theme.control(ControlRole::Selectable, ControlState::SELECTED);

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border::default(),
            shadow: Shadow::default(),
            snap: true,
        },
        Background::Color(control.background),
    );
}

fn frame_style(theme: &Theme) -> container::Style {
    let surface = theme.surface(SurfaceRole::Popover);
    let perimeter = theme.border(BorderRole::Subtle);

    container::Style {
        text_color: Some(surface.foreground),
        background: Some(Background::Color(surface.background)),
        border: Border {
            color: perimeter.color,
            width: 1.0,
            radius: iced::border::Radius::new(FRAME_RADIUS),
        },
        shadow: surface.shadow,
        ..container::Style::default()
    }
}
