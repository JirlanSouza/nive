use iced::{
    advanced::{
        layout, mouse, overlay, renderer,
        widget::{operation, tree, Tree},
        Clipboard, Layout, Renderer as _, Shell, Widget,
    },
    keyboard::{self, key::Named},
    touch, window, Alignment, Background, Border, Color, Event, Length, Point, Rectangle, Shadow,
    Size, Vector,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use crate::{
    advanced::focus::{FocusState, FocusVisibility},
    theme::{
        self,
        choice::{self, ChoicePersistentState, ChoiceStateInput},
        BorderRole, ControlRole, FieldValidation, TypographyRole,
    },
    widgets::display::measured_text::measure_width,
    widgets::overlays::anchored_overlay::{
        scroll::EnsureVisibleHandle, translated_bounds, AnchoredOverlay, OverlayIdentity,
        OverlayNodeState, PopoverCollision, PopoverPlacement, PopoverWidth,
    },
    Element,
};

use super::{
    MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_MAX_WIDTH, MENU_MIN_WIDTH,
    MENU_ROW_HEIGHT, MENU_ROW_PADDING_H, MENU_ROW_RADIUS, MENU_SEPARATOR_MARGIN,
};

const SEPARATOR_HEIGHT: f32 = 1.0 + MENU_SEPARATOR_MARGIN * 2.0;
const TYPEAHEAD_TIMEOUT: Duration = Duration::from_millis(700);
const SUBMENU_OPEN_DELAY: Duration = Duration::from_millis(200);
const SUBMENU_TRANSFER_GRACE: Duration = Duration::from_millis(300);

#[derive(Debug, Clone)]
pub(super) struct MenuBranchHandle {
    open: Rc<Cell<bool>>,
    pointer_inside: Rc<Cell<bool>>,
    child_bounds: Rc<Cell<Rectangle>>,
    identity: Rc<RefCell<OverlayIdentity>>,
}

impl MenuBranchHandle {
    pub(super) fn new() -> Self {
        Self {
            open: Rc::new(Cell::new(false)),
            pointer_inside: Rc::new(Cell::new(false)),
            child_bounds: Rc::new(Cell::new(Rectangle::default())),
            identity: Rc::new(RefCell::new(OverlayIdentity::root())),
        }
    }

    pub(super) fn open(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.open)
    }

    pub(super) fn pointer_inside(&self) -> Rc<Cell<bool>> {
        Rc::clone(&self.pointer_inside)
    }
}

#[derive(Debug, Clone)]
pub(super) struct MenuSlot<Message> {
    pub(super) eligible: bool,
    pub(super) separator: bool,
    activation: Option<Message>,
    label: Option<String>,
    trailing: Option<MenuTrailingMeasure>,
    persistent: ChoicePersistentState,
    disabled: bool,
    logical_focus: Option<Rc<Cell<bool>>>,
    branch: Option<MenuBranchHandle>,
}

#[derive(Debug, Clone)]
pub(super) enum MenuTrailingMeasure {
    Text(String, TypographyRole),
    Icon,
}

impl<Message> MenuSlot<Message> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn row(
        eligible: bool,
        activation: Option<Message>,
        label: impl Into<String>,
        trailing: Option<MenuTrailingMeasure>,
        persistent: ChoicePersistentState,
        disabled: bool,
        logical_focus: Rc<Cell<bool>>,
        branch: Option<MenuBranchHandle>,
    ) -> Self {
        Self {
            eligible,
            separator: false,
            activation,
            label: Some(label.into()),
            trailing,
            persistent,
            disabled,
            logical_focus: Some(logical_focus),
            branch,
        }
    }

    pub(super) fn separator() -> Self {
        Self {
            eligible: false,
            separator: true,
            activation: None,
            label: None,
            trailing: None,
            persistent: ChoicePersistentState::Unselected,
            disabled: false,
            logical_focus: None,
            branch: None,
        }
    }

    fn height(&self) -> f32 {
        if self.separator {
            SEPARATOR_HEIGHT
        } else {
            MENU_ROW_HEIGHT
        }
    }
}

pub(super) struct MenuList<'a, Message> {
    content: Element<'a, Message>,
    slots: Vec<MenuSlot<Message>>,
    reserve_choice: bool,
    reserve_icon: bool,
    trailing_width: Rc<Cell<f32>>,
    root: bool,
    shared_focus_visible: Rc<Cell<bool>>,
    level_open: Option<Rc<Cell<bool>>>,
    level_pointer_inside: Option<Rc<Cell<bool>>>,
    ensure_visible: EnsureVisibleHandle,
}

impl<'a, Message> MenuList<'a, Message> {
    pub(super) fn new(
        content: impl Into<Element<'a, Message>>,
        slots: Vec<MenuSlot<Message>>,
        reserve_choice: bool,
        reserve_icon: bool,
        trailing_width: Rc<Cell<f32>>,
        context: MenuLevelContext,
    ) -> Self {
        Self {
            content: content.into(),
            slots,
            reserve_choice,
            reserve_icon,
            trailing_width,
            root: context.root,
            shared_focus_visible: context.shared_focus_visible,
            level_open: context.level_open,
            level_pointer_inside: context.level_pointer_inside,
            ensure_visible: context.ensure_visible,
        }
    }

    fn level_active(&self, state: &MenuListState) -> bool {
        if self.root {
            state.focus.as_ref().is_some_and(FocusState::is_active)
        } else {
            self.level_open.as_ref().is_some_and(|open| open.get())
        }
    }

    fn focus_visible(&self, state: &MenuListState) -> bool {
        if self.root {
            state
                .focus
                .as_ref()
                .is_some_and(FocusState::is_focus_visible)
        } else {
            self.shared_focus_visible.get()
        }
    }

    fn request_highlight_visible(&self, state: &MenuListState, layout: Layout<'_>) {
        if let Some(target) = state
            .highlight
            .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
        {
            self.ensure_visible.request(target);
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct MenuLevelContext {
    root: bool,
    shared_focus_visible: Rc<Cell<bool>>,
    level_open: Option<Rc<Cell<bool>>>,
    level_pointer_inside: Option<Rc<Cell<bool>>>,
    ensure_visible: EnsureVisibleHandle,
}

impl MenuLevelContext {
    pub(super) fn root() -> Self {
        Self {
            root: true,
            shared_focus_visible: Rc::new(Cell::new(false)),
            level_open: None,
            level_pointer_inside: None,
            ensure_visible: EnsureVisibleHandle::new(),
        }
    }

    pub(super) fn child(&self, branch: &MenuBranchHandle) -> Self {
        Self {
            root: false,
            shared_focus_visible: Rc::clone(&self.shared_focus_visible),
            level_open: Some(branch.open()),
            level_pointer_inside: Some(branch.pointer_inside()),
            ensure_visible: EnsureVisibleHandle::new(),
        }
    }

    pub(super) fn ensure_visible(&self) -> EnsureVisibleHandle {
        self.ensure_visible.clone()
    }
}

#[derive(Debug)]
pub(super) struct MenuListState {
    focus: Option<FocusState>,
    pub(super) highlight: Option<usize>,
    highlighted_label: Option<String>,
    pressed: Option<usize>,
    typeahead: String,
    typeahead_deadline: Option<iced::time::Instant>,
    now: Option<iced::time::Instant>,
    pub(super) open_submenu: Option<usize>,
    open_submenu_label: Option<String>,
    submenu_intent: Option<(usize, iced::time::Instant)>,
    transfer_deadline: Option<iced::time::Instant>,
    overlay_nodes: Vec<OverlayNodeState>,
    last_pointer: Option<Point>,
}

impl Default for MenuListState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl MenuListState {
    fn new(root: bool) -> Self {
        Self {
            focus: root.then(|| FocusState::new(FocusVisibility::Auto)),
            highlight: None,
            highlighted_label: None,
            pressed: None,
            typeahead: String::new(),
            typeahead_deadline: None,
            now: None,
            open_submenu: None,
            open_submenu_label: None,
            submenu_intent: None,
            transfer_deadline: None,
            overlay_nodes: Vec::new(),
            last_pointer: None,
        }
    }
}

impl<'a, Message> Widget<Message, crate::theme::Theme, iced::Renderer> for MenuList<'a, Message>
where
    Message: Clone + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuListState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuListState::new(self.root))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
        let state = tree.state.downcast_mut::<MenuListState>();
        let reconciled = state
            .highlighted_label
            .as_deref()
            .and_then(|label| {
                self.slots.iter().position(|slot| {
                    slot.eligible
                        && slot
                            .label
                            .as_deref()
                            .is_some_and(|current| current == label)
                })
            })
            .or_else(|| {
                state
                    .highlight
                    .filter(|index| self.slots.get(*index).is_some_and(|slot| slot.eligible))
            })
            .or_else(|| first_eligible(&self.slots));
        set_highlight(&self.slots, state, reconciled);
        reconcile_open_submenu(&self.slots, state);
        ensure_overlay_nodes(&self.slots, state);
        sync_logical_focus(&self.slots, state, self.focus_visible(state));
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    #[allow(clippy::manual_clamp)]
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let maximum = limits.max().width.max(0.0).min(MENU_MAX_WIDTH);
        let minimum = MENU_MIN_WIDTH.min(maximum);
        self.trailing_width
            .set(max_trailing_width(renderer, &self.slots));
        let width = natural_width(
            renderer,
            &self.slots,
            self.reserve_choice,
            self.reserve_icon,
        )
        .clamp(minimum, maximum);
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &limits.width(width))
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        let state = tree.state.downcast_mut::<MenuListState>();
        if self.root {
            let focus = state.focus.as_mut().expect("root Menu focus state");
            if self.slots.iter().any(|slot| slot.eligible) {
                focus.register(operation, None, layout.bounds());
            } else {
                focus.clear();
            }
            self.shared_focus_visible.set(focus.is_focus_visible());
        }
        let entered = self.level_active(state) && state.highlight.is_none();
        if entered {
            set_highlight(&self.slots, state, first_eligible(&self.slots));
            self.request_highlight_visible(state, layout);
        }
        ensure_overlay_nodes(&self.slots, state);
        for ((index, slot), node) in self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.branch.is_some())
            .zip(state.overlay_nodes.iter_mut())
        {
            operation.custom(
                None,
                slot_bounds(&self.slots, layout.bounds(), index).unwrap_or_default(),
                node,
            );
            if let Some(branch) = &slot.branch {
                *branch.identity.borrow_mut() = node.identity().clone();
            }
        }
        sync_logical_focus(&self.slots, state, self.focus_visible(state));
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        {
            let state = tree.state.downcast_mut::<MenuListState>();
            sync_logical_focus(&self.slots, state, self.focus_visible(state));
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<MenuListState>();
        reconcile_closed_submenu(&self.slots, state);
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.now = Some(*now);
            if state
                .typeahead_deadline
                .is_some_and(|deadline| *now > deadline)
            {
                state.typeahead.clear();
                state.typeahead_deadline = None;
            }
            if state
                .submenu_intent
                .is_some_and(|(_, deadline)| *now >= deadline)
            {
                if let Some((index, _)) = state.submenu_intent.take() {
                    open_submenu(&self.slots, state, index);
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
            if state
                .transfer_deadline
                .is_some_and(|deadline| *now >= deadline)
            {
                let child_contains_pointer = state
                    .open_submenu
                    .and_then(|index| self.slots.get(index))
                    .and_then(|slot| slot.branch.as_ref())
                    .is_some_and(|branch| {
                        branch.pointer_inside.get()
                            || state
                                .last_pointer
                                .is_some_and(|point| branch.child_bounds.get().contains(point))
                    });
                if child_contains_pointer {
                    state.transfer_deadline = None;
                } else {
                    close_submenu(&self.slots, state);
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
            }
        }
        if let Some(position) = pointer_highlight_position(event, cursor) {
            state.last_pointer = position;
            if let Some(pointer_inside) = &self.level_pointer_inside {
                pointer_inside.set(position.is_some_and(|point| layout.bounds().contains(point)));
            }
            let highlight = position
                .and_then(|position| slot_at(&self.slots, layout.bounds(), position))
                .filter(|index| self.slots[*index].eligible);
            set_highlight(&self.slots, state, highlight);
            update_submenu_pointer_intent(&self.slots, state, highlight, shell);
            sync_logical_focus(&self.slots, state, self.focus_visible(state));
        }
        if is_primary_press(event)
            && primary_press_position(event, cursor)
                .is_some_and(|point| layout.bounds().contains(point))
        {
            if self.root {
                let focus = state.focus.as_mut().expect("root Menu focus state");
                focus.focus_from_pointer();
                self.shared_focus_visible.set(focus.is_focus_visible());
            }
            state.pressed = state.highlight;
            if matches!(event, Event::Touch(touch::Event::FingerPressed { .. }))
                && state.pressed.is_some()
            {
                shell.capture_event();
            }
            shell.request_redraw();
        }
        let released = release_position(event, cursor)
            .and_then(|position| slot_at(&self.slots, layout.bounds(), position))
            .filter(|index| Some(*index) == state.pressed);
        if let Some(index) = released {
            if self.slots[index].branch.is_some() {
                open_submenu(&self.slots, state, index);
                shell.capture_event();
                shell.invalidate_layout();
            } else if matches!(event, Event::Touch(touch::Event::FingerLifted { .. })) {
                if let Some(message) = self.slots[index].activation.clone() {
                    shell.publish(message);
                    shell.capture_event();
                }
            }
        }
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. })
        ) {
            state.pressed = None;
            shell.request_redraw();
        }

        if self.level_active(state) {
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowLeft),
                    ..
                })
            ) && !self.root
            {
                if let Some(open) = &self.level_open {
                    open.set(false);
                }
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowRight),
                    ..
                })
            ) {
                if let Some(index) = state
                    .highlight
                    .filter(|index| self.slots[*index].branch.is_some())
                {
                    open_submenu(&self.slots, state, index);
                    shell.capture_event();
                    shell.invalidate_layout();
                    shell.request_redraw();
                }
                return;
            }
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                text: Some(text), ..
            }) = event
            {
                if !text.is_empty() && text.chars().all(|character| !character.is_control()) {
                    let now = state.now.unwrap_or_else(iced::time::Instant::now);
                    if state
                        .typeahead_deadline
                        .is_none_or(|deadline| now > deadline)
                    {
                        state.typeahead.clear();
                    }
                    state.typeahead.push_str(text);
                    state.typeahead_deadline = Some(now + TYPEAHEAD_TIMEOUT);
                    if let Some(index) =
                        typeahead_match(&self.slots, state.highlight, state.typeahead.as_str())
                    {
                        set_highlight(&self.slots, state, Some(index));
                        self.request_highlight_visible(state, layout);
                        sync_logical_focus(&self.slots, state, self.focus_visible(state));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            if matches!(
                event,
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Enter | Named::Space),
                    ..
                })
            ) {
                if let Some(index) = state.highlight {
                    if self.slots[index].branch.is_some() {
                        open_submenu(&self.slots, state, index);
                        shell.capture_event();
                        shell.invalidate_layout();
                        shell.request_redraw();
                    } else if let Some(message) = self.slots[index].activation.clone() {
                        shell.publish(message);
                        shell.capture_event();
                    }
                }
                return;
            }
            let moved = match event {
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowDown),
                    ..
                }) => move_highlight(&self.slots, state.highlight, 1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::ArrowUp),
                    ..
                }) => move_highlight(&self.slots, state.highlight, -1),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::Home),
                    ..
                }) => first_eligible(&self.slots),
                Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(Named::End),
                    ..
                }) => last_eligible(&self.slots),
                _ => return,
            };
            if moved != state.highlight {
                set_highlight(&self.slots, state, moved);
                self.request_highlight_visible(state, layout);
                sync_logical_focus(&self.slots, state, self.focus_visible(state));
                shell.request_redraw();
            }
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuListState>();
        let focus_visible = self.focus_visible(state);
        for (index, slot) in self
            .slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| !slot.separator)
        {
            let Some(bounds) = slot_bounds(&self.slots, layout.bounds(), index) else {
                continue;
            };
            let resolved = choice::resolve_state(ChoiceStateInput {
                persistent: slot.persistent,
                validation: FieldValidation::Valid,
                callback_present: slot.eligible,
                disabled: slot.disabled,
                hovered: state.highlight == Some(index),
                pressed: state.pressed == Some(index),
                focused: focus_visible && state.highlight == Some(index),
            });
            let control = theme.control(ControlRole::Selectable, resolved.control);
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border::default().rounded(MENU_ROW_RADIUS),
                    shadow: Shadow::default(),
                    snap: true,
                },
                control.background,
            );
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if focus_visible {
            if let Some(bounds) = state
                .highlight
                .and_then(|index| slot_bounds(&self.slots, layout.bounds(), index))
            {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + 1.0,
                            y: bounds.y + 1.0,
                            width: (bounds.width - 2.0).max(0.0),
                            height: (bounds.height - 2.0).max(0.0),
                        },
                        border: Border {
                            color: theme.border(BorderRole::Focus).color,
                            width: 1.0,
                            radius: MENU_ROW_RADIUS.into(),
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::theme::Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<MenuList<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(list: MenuList<'a, Message>) -> Self {
        Element::new(list)
    }
}

#[derive(Debug, Clone)]
enum BranchEvent<Message> {
    Content(Message),
    Close,
}

pub(super) struct MenuBranch<'a, Message> {
    anchor: Element<'a, Message>,
    content: Element<'a, BranchEvent<Message>>,
    handle: MenuBranchHandle,
    ensure_visible: EnsureVisibleHandle,
}

impl<'a, Message> MenuBranch<'a, Message>
where
    Message: 'a,
{
    pub(super) fn new(
        anchor: impl Into<Element<'a, Message>>,
        content: Element<'a, Message>,
        handle: MenuBranchHandle,
        ensure_visible: EnsureVisibleHandle,
    ) -> Self {
        Self {
            anchor: anchor.into(),
            content: content.map(BranchEvent::Content),
            handle,
            ensure_visible,
        }
    }
}

impl<'menu, Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for MenuBranch<'menu, Message>
where
    Message: Clone + 'menu,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.anchor), Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() != 2 {
            tree.children = self.children();
            return;
        }
        tree.children[0].diff(self.anchor.as_widget());
        tree.children[1].diff(self.content.as_widget());
    }

    fn size(&self) -> Size<Length> {
        self.anchor.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.anchor.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.anchor
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        self.anchor
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.anchor.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.anchor.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        let (anchor_tree, content_tree) = tree.children.split_at_mut(1);
        let anchor = self.anchor.as_widget_mut().overlay(
            &mut anchor_tree[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        let branch = self.handle.open.get().then(|| {
            let open = self.handle.open();
            let pointer_inside = self.handle.pointer_inside();
            overlay::Element::new(Box::new(
                AnchoredOverlay::new(
                    translated_bounds(layout.bounds(), translation),
                    &mut self.content,
                    &mut content_tree[0],
                    PopoverPlacement::RightStart,
                    PopoverWidth::Content,
                    PopoverCollision::FlipAndShift,
                    0.0,
                    Some(BranchEvent::Close),
                    move |event, shell: &mut Shell<'_, Message>| match event {
                        BranchEvent::Content(message) => shell.publish(message),
                        BranchEvent::Close => {
                            open.set(false);
                            pointer_inside.set(false);
                            shell.invalidate_layout();
                            shell.request_redraw();
                        }
                    },
                )
                .identity(self.handle.identity.borrow().clone())
                .report_bounds(self.handle.child_bounds.as_ref())
                .ensure_visible(self.ensure_visible.clone())
                .with_nested_overlay_map(unwrap_branch_event::<Message>),
            ))
        });

        match (anchor, branch) {
            (Some(anchor), Some(branch)) => {
                Some(overlay::Group::with_children(vec![anchor, branch]).overlay())
            }
            (Some(anchor), None) => Some(anchor),
            (None, Some(branch)) => Some(branch),
            (None, None) => None,
        }
    }
}

fn unwrap_branch_event<Message>(event: BranchEvent<Message>) -> Message {
    match event {
        BranchEvent::Content(message) => message,
        BranchEvent::Close => unreachable!("nested branch close is handled by its owner"),
    }
}

impl<'a, Message> From<MenuBranch<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(branch: MenuBranch<'a, Message>) -> Self {
        Element::new(branch)
    }
}

pub(super) struct MenuTrailingTrack<'a, Message> {
    content: Element<'a, Message>,
    width: Rc<Cell<f32>>,
}

impl<'a, Message> MenuTrailingTrack<'a, Message> {
    pub(super) fn new(content: impl Into<Element<'a, Message>>, width: Rc<Cell<f32>>) -> Self {
        Self {
            content: content.into(),
            width,
        }
    }
}

impl<Message> Widget<Message, crate::theme::Theme, iced::Renderer>
    for MenuTrailingTrack<'_, Message>
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = self.width.get().min(limits.max().width).max(0.0);
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, limits.max().height)),
        );
        let size = Size::new(width, child.size().height);
        layout::Node::with_children(
            size,
            vec![child.align(Alignment::End, Alignment::Center, size)],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        if let Some(layout) = layout.children().next() {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout,
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Some(layout) = layout.children().next() {
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        layout
            .children()
            .next()
            .map_or(mouse::Interaction::None, |layout| {
                self.content.as_widget().mouse_interaction(
                    &tree.children[0],
                    layout,
                    cursor,
                    viewport,
                    renderer,
                )
            })
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &crate::theme::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(layout) = layout.children().next() {
            self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout,
                cursor,
                viewport,
            );
        }
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, crate::theme::Theme, iced::Renderer>> {
        let layout = layout.children().next()?;
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message> From<MenuTrailingTrack<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(track: MenuTrailingTrack<'a, Message>) -> Self {
        Element::new(track)
    }
}

fn first_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().position(|slot| slot.eligible)
}

fn last_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().rposition(|slot| slot.eligible)
}

fn ensure_overlay_nodes<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState) {
    let branch_count = slots.iter().filter(|slot| slot.branch.is_some()).count();
    state
        .overlay_nodes
        .resize_with(branch_count, OverlayNodeState::default);
}

fn reconcile_open_submenu<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState) {
    let target = state
        .open_submenu_label
        .as_deref()
        .and_then(|label| {
            slots.iter().position(|slot| {
                slot.branch.is_some()
                    && slot
                        .label
                        .as_deref()
                        .is_some_and(|current| current == label)
            })
        })
        .or_else(|| {
            state.open_submenu.filter(|index| {
                slots
                    .get(*index)
                    .and_then(|slot| slot.branch.as_ref())
                    .is_some_and(|branch| branch.open.get())
            })
        });
    if let Some(index) = target {
        open_submenu(slots, state, index);
    } else {
        close_submenu(slots, state);
    }
}

fn reconcile_closed_submenu<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState) {
    let closed = state
        .open_submenu
        .and_then(|index| slots.get(index))
        .and_then(|slot| slot.branch.as_ref())
        .is_some_and(|branch| !branch.open.get());
    if closed {
        state.open_submenu = None;
        state.open_submenu_label = None;
        state.submenu_intent = None;
        state.transfer_deadline = None;
    }
}

fn open_submenu<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState, index: usize) {
    for slot in slots {
        if let Some(branch) = &slot.branch {
            branch.open.set(false);
            branch.pointer_inside.set(false);
        }
    }
    let Some(slot) = slots.get(index).filter(|slot| slot.eligible) else {
        close_submenu(slots, state);
        return;
    };
    let Some(branch) = &slot.branch else {
        close_submenu(slots, state);
        return;
    };
    branch.open.set(true);
    state.open_submenu = Some(index);
    state.open_submenu_label = slot.label.clone();
    state.submenu_intent = None;
    state.transfer_deadline = None;
}

fn close_submenu<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState) {
    for slot in slots {
        if let Some(branch) = &slot.branch {
            branch.open.set(false);
            branch.pointer_inside.set(false);
        }
    }
    state.open_submenu = None;
    state.open_submenu_label = None;
    state.submenu_intent = None;
    state.transfer_deadline = None;
}

fn update_submenu_pointer_intent<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
    highlight: Option<usize>,
    shell: &mut Shell<'_, Message>,
) {
    let now = state.now.unwrap_or_else(iced::time::Instant::now);
    match highlight {
        Some(index) if slots[index].branch.is_some() => {
            state.transfer_deadline = None;
            if state.open_submenu == Some(index) {
                state.submenu_intent = None;
            } else {
                close_submenu(slots, state);
                let deadline = now + SUBMENU_OPEN_DELAY;
                state.submenu_intent = Some((index, deadline));
                shell.request_redraw_at(deadline);
            }
        }
        Some(_) => close_submenu(slots, state),
        None => {
            state.submenu_intent = None;
            if state.open_submenu.is_some() && state.transfer_deadline.is_none() {
                let deadline = now + SUBMENU_TRANSFER_GRACE;
                state.transfer_deadline = Some(deadline);
                shell.request_redraw_at(deadline);
            }
        }
    }
}

fn set_highlight<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
    highlight: Option<usize>,
) {
    state.highlight = highlight;
    state.highlighted_label = highlight
        .and_then(|index| slots.get(index))
        .and_then(|slot| slot.label.clone());
}

fn move_highlight<Message>(
    slots: &[MenuSlot<Message>],
    current: Option<usize>,
    direction: isize,
) -> Option<usize> {
    let start = current.or_else(|| {
        if direction > 0 {
            first_eligible(slots)
        } else {
            last_eligible(slots)
        }
    })? as isize;
    let mut index = start + direction;
    while index >= 0 && (index as usize) < slots.len() {
        if slots[index as usize].eligible {
            return Some(index as usize);
        }
        index += direction;
    }
    current.or(Some(start as usize))
}

fn typeahead_match<Message>(
    slots: &[MenuSlot<Message>],
    current: Option<usize>,
    prefix: &str,
) -> Option<usize> {
    if slots.is_empty() {
        return None;
    }
    let prefix = prefix.to_lowercase();
    let start = current.unwrap_or(slots.len() - 1);
    (1..=slots.len())
        .map(|offset| (start + offset) % slots.len())
        .find(|index| {
            let slot = &slots[*index];
            slot.eligible
                && slot
                    .label
                    .as_deref()
                    .is_some_and(|label| label.to_lowercase().starts_with(&prefix))
        })
}

fn sync_logical_focus<Message>(
    slots: &[MenuSlot<Message>],
    state: &MenuListState,
    focus_visible: bool,
) {
    let highlighted = focus_visible.then_some(state.highlight).flatten();
    for (index, slot) in slots.iter().enumerate() {
        if let Some(focused) = &slot.logical_focus {
            focused.set(highlighted == Some(index));
        }
    }
}

fn max_trailing_width<Message>(renderer: &iced::Renderer, slots: &[MenuSlot<Message>]) -> f32 {
    slots
        .iter()
        .filter_map(|slot| slot.trailing.as_ref())
        .map(|trailing| match trailing {
            MenuTrailingMeasure::Text(text, role) => {
                measure_width(renderer, text, theme::typography(*role))
            }
            MenuTrailingMeasure::Icon => MENU_ICON_SIZE,
        })
        .fold(0.0, f32::max)
}

fn natural_width<Message>(
    renderer: &iced::Renderer,
    slots: &[MenuSlot<Message>],
    reserve_choice: bool,
    reserve_icon: bool,
) -> f32 {
    let label_style = theme::typography(TypographyRole::Control);
    let label = slots
        .iter()
        .filter_map(|slot| slot.label.as_deref())
        .map(|label| measure_width(renderer, label, label_style))
        .fold(0.0, f32::max);
    let trailing = max_trailing_width(renderer, slots);
    let mut tracks = 1usize;
    let mut width = label;
    if reserve_choice {
        tracks += 1;
        width += MENU_ICON_SIZE;
    }
    if reserve_icon {
        tracks += 1;
        width += MENU_ICON_SIZE;
    }
    if trailing > 0.0 {
        tracks += 1;
        width += trailing;
    }

    MENU_LIST_INSET * 2.0
        + MENU_ROW_PADDING_H * 2.0
        + width
        + MENU_COLUMN_GAP * (tracks.saturating_sub(1) as f32)
}

fn slot_at<Message>(slots: &[MenuSlot<Message>], bounds: Rectangle, point: Point) -> Option<usize> {
    slots.iter().enumerate().find_map(|(index, _)| {
        slot_bounds(slots, bounds, index)
            .is_some_and(|bounds| bounds.contains(point))
            .then_some(index)
    })
}

fn slot_bounds<Message>(
    slots: &[MenuSlot<Message>],
    bounds: Rectangle,
    target: usize,
) -> Option<Rectangle> {
    let slot = slots.get(target)?;
    let y = bounds.y
        + MENU_LIST_INSET
        + slots[..target]
            .iter()
            .map(|slot| slot.height())
            .sum::<f32>();
    Some(Rectangle::new(
        Point::new(bounds.x + MENU_LIST_INSET, y),
        Size::new(
            (bounds.width - MENU_LIST_INSET * 2.0).max(0.0),
            slot.height(),
        ),
    ))
}

fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

fn pointer_highlight_position(event: &Event, cursor: mouse::Cursor) -> Option<Option<Point>> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. })
        | Event::Touch(touch::Event::FingerMoved { position, .. })
        | Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(Some(*position)),
        Event::Touch(touch::Event::FingerLost { .. }) | Event::Mouse(mouse::Event::CursorLeft) => {
            Some(None)
        }
        Event::Mouse(
            mouse::Event::CursorEntered
            | mouse::Event::CursorMoved { .. }
            | mouse::Event::ButtonPressed(mouse::Button::Left)
            | mouse::Event::ButtonReleased(mouse::Button::Left),
        ) => Some(cursor.position()),
        _ => None,
    }
}

fn primary_press_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        _ => None,
    }
}

fn release_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => cursor.position(),
        _ => None,
    }
}
