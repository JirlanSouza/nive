use iced::{
    advanced::{mouse, Shell},
    touch, Event, Point, Rectangle, Size,
};

use super::{
    MenuListState, MenuSlot, MenuTrailingMeasure, SUBMENU_OPEN_DELAY, SUBMENU_TRANSFER_GRACE,
};
use crate::widgets::navigation::menu::{
    MENU_COLUMN_GAP, MENU_ICON_SIZE, MENU_LIST_INSET, MENU_ROW_PADDING_H,
};
use crate::{
    theme::{self, TypographyRole},
    widgets::display::measured_text::measure_width,
    widgets::overlays::anchored_overlay::OverlayNodeState,
};

pub(super) fn first_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().position(|slot| slot.eligible)
}

pub(super) fn last_eligible<Message>(slots: &[MenuSlot<Message>]) -> Option<usize> {
    slots.iter().rposition(|slot| slot.eligible)
}

pub(super) fn ensure_overlay_nodes<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
) {
    let branch_count = slots.iter().filter(|slot| slot.branch.is_some()).count();
    state
        .overlay_nodes
        .resize_with(branch_count, OverlayNodeState::default);
}

pub(super) fn reconcile_open_submenu<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
) {
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

pub(super) fn reconcile_closed_submenu<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
) {
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

pub(super) fn open_submenu<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
    index: usize,
) {
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

pub(super) fn close_submenu<Message>(slots: &[MenuSlot<Message>], state: &mut MenuListState) {
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

pub(super) fn update_submenu_pointer_intent<Message>(
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

pub(super) fn set_highlight<Message>(
    slots: &[MenuSlot<Message>],
    state: &mut MenuListState,
    highlight: Option<usize>,
) {
    state.highlight = highlight;
    state.highlighted_label = highlight
        .and_then(|index| slots.get(index))
        .and_then(|slot| slot.label.clone());
}

pub(super) fn move_highlight<Message>(
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

pub(super) fn typeahead_match<Message>(
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

pub(super) fn sync_logical_focus<Message>(
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

pub(super) fn max_trailing_width<Message>(
    renderer: &iced::Renderer,
    slots: &[MenuSlot<Message>],
) -> f32 {
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

pub(super) fn natural_width<Message>(
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

pub(super) fn slot_at<Message>(
    slots: &[MenuSlot<Message>],
    bounds: Rectangle,
    point: Point,
) -> Option<usize> {
    slots.iter().enumerate().find_map(|(index, _)| {
        slot_bounds(slots, bounds, index)
            .is_some_and(|bounds| bounds.contains(point))
            .then_some(index)
    })
}

pub(super) fn slot_bounds<Message>(
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

pub(super) fn is_primary_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

pub(super) fn pointer_highlight_position(
    event: &Event,
    cursor: mouse::Cursor,
) -> Option<Option<Point>> {
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

pub(super) fn primary_press_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerPressed { position, .. }) => Some(*position),
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => cursor.position(),
        _ => None,
    }
}

pub(super) fn release_position(event: &Event, cursor: mouse::Cursor) -> Option<Point> {
    match event {
        Event::Touch(touch::Event::FingerLifted { position, .. }) => Some(*position),
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => cursor.position(),
        _ => None,
    }
}
