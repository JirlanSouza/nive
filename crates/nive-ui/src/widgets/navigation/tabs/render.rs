use std::rc::Rc;

use iced::{
    mouse,
    widget::{container, mouse_area, rule, stack, Row, Space},
    Alignment, Length, Padding,
};

use super::geometry::label_budget;
use super::style as theme_tabs;
use super::{
    AllTabsMenuEntry, DisplayedTab, MenuMessage, TabBar, TabBarState, TabItem,
    HIDDEN_AFFORDANCE_WIDTH,
};
use crate::theme::TypographyRole;
use crate::widgets::controls::button::{self, GroupedItemKind, GroupedItemSpec, SelectionChrome};
use crate::widgets::display::measured_text::{EllipsisStrategy, MeasuredText};
use crate::widgets::display::min_width::MinWidth;
use crate::widgets::navigation::menu::{
    relay::MessageRelay, Menu, MenuRadioGroup, MenuRadioOption,
};
use crate::widgets::navigation::overflow::ClipViewport;
use crate::widgets::overlays::tooltip as tooltip_widget;
use crate::widgets::overlays::TooltipScope;
use crate::widgets::overlays::{PopoverCollision, PopoverPlacement};
use crate::widgets::primitives::{icon as icon_widget, IconRole};
use crate::Element;

impl<'a, Id, Message> TabBar<'a, Id, Message>
where
    Id: Clone + Eq + 'static,
    Message: Clone + 'a,
{
    pub(super) fn displayed_tabs<'b>(&'b self) -> Vec<DisplayedTab<'b, 'a, Id>> {
        let mut pinned = Vec::new();
        let mut unpinned = Vec::new();

        for (original_index, item) in self.tabs.iter().enumerate() {
            let displayed = DisplayedTab {
                original_index,
                item,
            };

            if item.pinned {
                pinned.push(displayed);
            } else {
                unpinned.push(displayed);
            }
        }

        pinned.extend(unpinned);
        pinned
    }

    pub(super) fn content_element(&self, state: &TabBarState<Id>) -> Element<'a, Message> {
        let metrics = theme_tabs::metrics(self.size);
        let visible = self.displayed_tabs();
        let close_enabled = self.on_close_request.is_some();

        // Always reserve fixed slots in the outer row for chevrons and the
        // all-tabs trigger so the layout tree keeps a stable index for the
        // scrollable strip. Hidden affordances become zero-width spacers.
        let left_chevron = self.chevron_button(
            metrics,
            IconRole::NiveDisclosureLeft,
            state.overflow.show_start_chevron(),
            state.has_overflow,
            "Scroll tabs toward start",
        );
        let right_chevron = self.chevron_button(
            metrics,
            IconRole::NiveDisclosureRight,
            state.overflow.show_end_chevron(),
            state.has_overflow,
            "Scroll tabs toward end",
        );
        let all_tabs_button = self.all_tabs_menu(metrics, state);

        let mut tabs_row = Row::new()
            .spacing(metrics.tab_gap)
            .align_y(Alignment::Center)
            .width(Length::Shrink)
            .height(Length::Fixed(metrics.tab_height));
        for displayed in visible {
            tabs_row = tabs_row.push(self.tab_element(displayed, metrics, close_enabled, state));
        }

        // Wrap the tab strip in a horizontal clip so the scroll offset exposes
        // only the visible viewport. The translation is applied during
        // `Widget::layout` once the natural tab bounds are known.
        let strip = ClipViewport::horizontal(tabs_row)
            .width(Length::Fill)
            .height(Length::Fixed(metrics.tab_height));

        let bar_row = Row::new()
            .spacing(0.0)
            .align_y(Alignment::Center)
            .push(left_chevron)
            .push(strip)
            .push(right_chevron)
            .push(all_tabs_button);

        let mut bar = container(bar_row)
            .style(theme_tabs::bar_style(self.role))
            .padding(
                Padding::ZERO
                    .vertical(metrics.bar_padding_v)
                    .horizontal(metrics.bar_padding_h),
            )
            .height(Length::Fixed(metrics.height));

        if let Some(width) = self.width {
            bar = bar.width(width);
        }

        TooltipScope::new(bar).into()
    }

    pub(super) fn chevron_button(
        &self,
        metrics: theme_tabs::TabBarMetrics,
        role: IconRole,
        actionable: bool,
        reserve: bool,
        tooltip: &'static str,
    ) -> Element<'a, Message> {
        let chevron: Element<'a, Message> = button::Button::custom(
            icon_widget::role(role)
                // While the slot is reserved the glyph is inherited: full when it
                // can scroll, dimmed by the disabled embedded style when it sits
                // at that end so the boundary reads without a layout shift. It is
                // only hidden when there is no overflow and the slot collapses.
                .custom_size(metrics.icon_size)
                .color_maybe((!reserve).then_some(iced::Color::TRANSPARENT))
                .into(),
        )
        .disabled(!actionable)
        .tooltip(tooltip)
        .width(Length::Fixed(if reserve {
            metrics.close_side
        } else {
            HIDDEN_AFFORDANCE_WIDTH
        }))
        .into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: metrics.tab_height,
            padding_h: 0.0,
            selected: false,
            selection: SelectionChrome::Outlined,
            destructive: false,
            kind: GroupedItemKind::Embedded,
        });

        // The bar handles the scroll click, so the button carries no `on_press`
        // and would show the default cursor. A `mouse_area` supplies the pointer
        // like an enabled tab does. It wraps unconditionally — `actionable`
        // flips as the strip reaches its ends, and a shape that changed with it
        // would desync `tree.children[0]`, which `layout_impl` reuses without a
        // re-diff.
        mouse_area(chevron)
            .interaction(if actionable {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::None
            })
            .into()
    }

    pub(super) fn all_tabs_menu(
        &self,
        metrics: theme_tabs::TabBarMetrics,
        state: &TabBarState<Id>,
    ) -> Element<'a, Message> {
        let actionable = state.has_overflow;
        let trigger = button::Button::custom(
            icon_widget::role(IconRole::ViewMore)
                .custom_size(metrics.icon_size)
                .color_maybe((!actionable).then_some(iced::Color::TRANSPARENT))
                .into(),
        )
        .disabled(!actionable)
        .tooltip("Show all tabs")
        .on_press_maybe(actionable.then_some(MenuMessage::Open))
        .width(Length::Fixed(if state.has_overflow {
            metrics.close_side
        } else {
            HIDDEN_AFFORDANCE_WIDTH
        }))
        .into_grouped_item(GroupedItemSpec {
            size: metrics.size,
            radius: metrics.radius.into(),
            height: metrics.tab_height,
            padding_h: 0.0,
            selected: false,
            selection: SelectionChrome::Outlined,
            destructive: false,
            kind: GroupedItemKind::Embedded,
        });
        let menu = self.build_menu(trigger, state.menu_open.get());
        let menu_open = Rc::clone(&state.menu_open);
        let on_select = self.on_select.clone();

        MessageRelay::new(menu, move |message, shell| {
            match message {
                MenuMessage::Open => menu_open.set(true),
                MenuMessage::Select(id) => {
                    menu_open.set(false);
                    if let Some(on_select) = &on_select {
                        shell.publish(on_select(id));
                    }
                }
                MenuMessage::Dismiss => menu_open.set(false),
            }
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        })
        .into()
    }

    pub(super) fn tab_element(
        &self,
        displayed: DisplayedTab<'_, 'a, Id>,
        metrics: theme_tabs::TabBarMetrics,
        close_enabled: bool,
        state: &TabBarState<Id>,
    ) -> Element<'a, Message> {
        let tab = displayed.item;
        let selected = self.active.as_ref().is_some_and(|active| active == &tab.id);
        let has_close = tab.closable && close_enabled;
        let show_close = has_close
            && (selected
                || state.hovered_id.as_ref().is_some_and(|id| id == &tab.id)
                || state.focused_id.as_ref().is_some_and(|id| id == &tab.id));
        let status_side = if has_close {
            metrics.close_side
        } else {
            metrics.status_side
        };
        let mut content = self.main_content(tab, metrics, status_side);

        let close = container(
            icon_widget::role(IconRole::WindowClose)
                .custom_size(metrics.close_icon_size)
                .color_maybe((!show_close).then_some(iced::Color::TRANSPARENT)),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill);
        let dirty = theme_tabs::unsaved_indicator(metrics.dirty_size, tab.dirty && !show_close);
        let status: Element<'_, Message> = container(stack![dirty, close])
            .width(Length::Fixed(status_side))
            .height(Length::Fixed(metrics.tab_height))
            .into();
        content = content.push(status);

        let matches = |id: &Option<Id>| id.as_ref().is_some_and(|current| current == &tab.id);
        let content: Element<'_, Message> = container(content)
            .style(theme_tabs::tab_style(theme_tabs::TabChrome {
                active_role: self.active_role,
                radius: metrics.radius,
                selected,
                hovered: matches(&state.hovered_id),
                pressed: matches(&state.pressed_id),
                disabled: tab.disabled,
                focused: state.focus.is_focus_visible() && matches(&state.focused_id),
            }))
            .padding(Padding::ZERO.horizontal(metrics.padding_h))
            .height(Length::Fixed(metrics.tab_height))
            .clip(true)
            .into();

        // The floor belongs in layout: the tab paints itself now, so a width
        // imposed after the fact would leave its fill and indicator short.
        let content: Element<'a, Message> = MinWidth::new(content, metrics.min_tab_width).into();
        let content = self.tab_decorations(content, metrics, selected, matches(&state.dragged_id));

        // The tab owns its cursor too, like the overflow chevrons own theirs.
        // A clipped tab still reports Pointer only over its visible part: the
        // strip's `ClipViewport` hides the cursor outside its window.
        let content: Element<'a, Message> = if tab.disabled {
            content
        } else {
            mouse_area(content)
                .interaction(mouse::Interaction::Pointer)
                .into()
        };

        match tab.tooltip.clone() {
            Some(label) => tooltip_widget::Tooltip::new(content, label).into(),
            None => content,
        }
    }

    /// Marks the tab is responsible for: the active indicator along its top
    /// edge, and the veil covering it while its copy is dragged.
    ///
    /// Both layers stay in the stack unconditionally and toggle by colour.
    /// `selected` and `dragged` are interaction state; a shape that appeared
    /// with them would desync `tree.children[0]`, which the runtime relayouts
    /// and redraws without a diff to reconcile.
    fn tab_decorations(
        &self,
        content: Element<'a, Message>,
        metrics: theme_tabs::TabBarMetrics,
        selected: bool,
        dragged: bool,
    ) -> Element<'a, Message> {
        let indicator = container(
            rule::horizontal(metrics.indicator_width)
                .style(theme_tabs::active_indicator_style(selected)),
        )
        .height(Length::Fill)
        .align_y(Alignment::Start);
        let veil = container(Space::new())
            .style(theme_tabs::dragged_veil_style(self.role, dragged))
            .width(Length::Fill)
            .height(Length::Fill);

        stack![content, indicator, veil]
            .height(Length::Fixed(metrics.tab_height))
            .into()
    }

    pub(super) fn main_content(
        &self,
        tab: &TabItem<'a, Id>,
        metrics: theme_tabs::TabBarMetrics,
        status_side: f32,
    ) -> Row<'a, Message, crate::theme::Theme, iced::Renderer> {
        let leading_icons = usize::from(tab.icon.is_some()) + usize::from(tab.pinned);
        let label = MeasuredText::new_inherited(
            tab.label.clone(),
            EllipsisStrategy::End,
            TypographyRole::Control,
        )
        .max_width(label_budget(metrics, leading_icons, status_side));
        let mut content = Row::new()
            .spacing(metrics.gap)
            .align_y(Alignment::Center)
            .height(Length::Fill)
            .width(Length::Shrink);

        if let Some(icon) = tab.icon {
            content = content.push(icon_widget::reference(icon).custom_size(metrics.icon_size));
        }

        if tab.pinned {
            content =
                content.push(icon_widget::role(IconRole::TabPinned).custom_size(metrics.icon_size));
        }

        content = content.push(label);
        content
    }

    pub(super) fn menu_entries(&self) -> Vec<AllTabsMenuEntry<'a, Id>> {
        self.displayed_tabs()
            .into_iter()
            .map(|displayed| {
                let tab = displayed.item;
                AllTabsMenuEntry {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                    icon: tab.icon,
                    active: self.active.as_ref().is_some_and(|active| active == &tab.id),
                    dirty: tab.dirty,
                    pinned: tab.pinned,
                    disabled: tab.disabled,
                }
            })
            .collect()
    }

    /// Returns the canonical anchored Menu for the all-tabs overflow.
    pub(super) fn build_menu(
        &self,
        trigger: impl Into<Element<'a, MenuMessage<Id>>>,
        open: bool,
    ) -> Element<'a, MenuMessage<Id>> {
        let mut group = MenuRadioGroup::new(self.active.clone());
        if self.on_select.is_some() {
            group = group.on_select(MenuMessage::Select);
        }
        for entry in self.menu_entries() {
            let annotation = match (entry.pinned, entry.dirty) {
                (true, true) => Some("Pinned · Unsaved"),
                (true, false) => Some("Pinned"),
                (false, true) => Some("Unsaved"),
                (false, false) => None,
            };
            let mut option = MenuRadioOption::new(entry.id, entry.label).disabled(entry.disabled);
            if let Some(icon) = entry
                .icon
                .or(entry.pinned.then_some(IconRole::TabPinned.into()))
            {
                option = option.icon(icon);
            }
            if let Some(annotation) = annotation {
                option = option.annotation(annotation);
            }
            group = group.option(option);
        }

        Menu::new(trigger)
            .open(open)
            .on_dismiss(MenuMessage::Dismiss)
            .placement(PopoverPlacement::BottomStart)
            .collision(PopoverCollision::FlipAndShift)
            .radio_group(group)
            .into()
    }
}
