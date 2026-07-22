use std::borrow::Cow;

use iced::widget::{container, scrollable};
use iced::Length;
use nive_ui::theme::{padding, PaddingRole, ToneRole};
use nive_ui::widgets::{overlay_scrollbar, EmptyState};
use nive_ui::{Element, IconRole};

use crate::panels::WorkbenchPanel;

/// Generic inspector helper state.
///
/// The helper owns body inset and scrolling in every state, including
/// `Content`: applications supply chrome-free content and the helper insets
/// and, when the content exceeds the panel body, scrolls it. The inset is
/// applied to the scroll content rather than as outer panel padding, so the
/// scrollbar stays at the panel edge.
pub enum InspectorState<'a, Message> {
    /// No object is selected.
    NoSelection,
    /// Selection content is loading.
    Loading {
        /// Visible label.
        label: Cow<'a, str>,
    },
    /// Inspector failed to load/render.
    Error {
        /// Visible error message.
        message: Cow<'a, str>,
    },
    /// App-provided selected-object content.
    Content(Element<'a, Message>),
}

impl<'a, Message> InspectorState<'a, Message>
where
    Message: Clone + 'a,
{
    /// Renders inspector state content.
    pub fn view(self) -> Element<'a, Message> {
        match self {
            Self::NoSelection => EmptyState::new("No selection")
                .description("Select an object to inspect its details.")
                .icon(IconRole::PreferencesSystem)
                .into(),
            Self::Loading { label } => EmptyState::new(label)
                .description("Inspector content is loading.")
                .icon(IconRole::ViewRefresh)
                .loading(true)
                .into(),
            Self::Error { message } => EmptyState::new("Inspector unavailable")
                .description(message)
                .icon(IconRole::DialogError)
                .into(),
            Self::Content(content) => scrollable(
                container(content)
                    .padding(padding(PaddingRole::Panel))
                    .width(Length::Fill),
            )
            .direction(scrollable::Direction::Vertical(overlay_scrollbar()))
            .into(),
        }
    }
}

/// Builds a generic inspector panel.
///
/// `InspectorState::Content` contributes no panel status: the body already
/// shows the content, so a status label would only restate what is plainly
/// visible. `NoSelection`, `Loading`, and `Error` keep their status because
/// the label is the only thing in the header stating why the body looks the
/// way it does.
pub fn inspector_panel<'a, PanelId, ActionId, Message>(
    panel_id: PanelId,
    state: InspectorState<'a, Message>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message>
where
    Message: Clone + 'a,
{
    let status = match &state {
        InspectorState::Error { .. } => Some((ToneRole::Danger, "Inspector error")),
        InspectorState::Loading { .. } => Some((ToneRole::Accent, "Inspector loading")),
        InspectorState::NoSelection => Some((ToneRole::Neutral, "No selection")),
        InspectorState::Content(_) => None,
    };

    let panel =
        WorkbenchPanel::new(panel_id, "Inspector", state.view()).icon(IconRole::PreferencesSystem);
    match status {
        Some((tone, label)) => panel.status_text(tone, label),
        None => panel,
    }
}

#[cfg(test)]
mod panel_status_tests {
    use super::*;

    #[test]
    fn content_state_contributes_no_panel_status() {
        let panel = inspector_panel::<&str, &str, ()>(
            "inspector",
            InspectorState::Content(iced::widget::Space::new().into()),
        );

        assert!(panel.status_indicator_value().is_none());
    }

    #[test]
    fn non_content_states_keep_their_labelled_status() {
        let error = inspector_panel::<&str, &str, ()>(
            "inspector",
            InspectorState::Error {
                message: "boom".into(),
            },
        );
        let status = error.status_indicator_value().expect("error status");
        assert_eq!(status.tone(), ToneRole::Danger);
        assert!(!status.is_empty());

        let loading = inspector_panel::<&str, &str, ()>(
            "inspector",
            InspectorState::Loading {
                label: "Loading…".into(),
            },
        );
        let status = loading.status_indicator_value().expect("loading status");
        assert_eq!(status.tone(), ToneRole::Accent);
        assert!(!status.is_empty());

        let no_selection =
            inspector_panel::<&str, &str, ()>("inspector", InspectorState::NoSelection);
        let status = no_selection
            .status_indicator_value()
            .expect("no-selection status");
        assert_eq!(status.tone(), ToneRole::Neutral);
        assert!(!status.is_empty());
    }
}

#[cfg(test)]
mod body_tests {
    use iced::advanced::{
        layout::{Layout, Limits},
        mouse,
        widget::{operation, Tree},
    };
    use iced::widget::{container as iced_container, Space};
    use iced::{Font, Pixels, Point, Rectangle, Size, Vector};

    use crate::layout_probe;

    use super::*;

    fn renderer() -> iced::Renderer {
        iced_renderer::fallback::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::default(),
            Pixels(14.0),
        ))
    }

    #[derive(Default)]
    struct ScrollProbe {
        found: bool,
        bounds: Rectangle,
        content_bounds: Rectangle,
    }

    impl operation::Operation for ScrollProbe {
        fn scrollable(
            &mut self,
            _id: Option<&iced::advanced::widget::Id>,
            bounds: Rectangle,
            content_bounds: Rectangle,
            _translation: Vector,
            _state: &mut dyn operation::Scrollable,
        ) {
            self.found = true;
            self.bounds = bounds;
            self.content_bounds = content_bounds;
        }

        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn operation::Operation)) {
            operate(self);
        }
    }

    fn probe_content(content: Element<'static, ()>, body: Size) -> ScrollProbe {
        layout_probe::clear();
        let mut element = InspectorState::<'static, ()>::Content(content).view();
        let mut tree = Tree::new(&element);
        let renderer = renderer();
        let node =
            element
                .as_widget_mut()
                .layout(&mut tree, &renderer, &Limits::new(Size::ZERO, body));
        let viewport = Rectangle::new(Point::ORIGIN, body);
        let _ = element.as_widget().mouse_interaction(
            &tree,
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
            &renderer,
        );

        let mut probe = ScrollProbe::default();
        element
            .as_widget_mut()
            .operate(&mut tree, Layout::new(&node), &renderer, &mut probe);
        probe
    }

    #[test]
    fn content_state_is_wrapped_in_a_scrollable_body() {
        let content: Element<'static, ()> = Space::new().width(40).height(40).into();
        let probe = probe_content(content, Size::new(240.0, 200.0));

        assert!(probe.found, "Content body should carry a Scrollable");
    }

    #[test]
    fn content_taller_than_the_body_overflows_into_the_scroll_region() {
        let content: Element<'static, ()> =
            iced_container(Space::new().width(40).height(4000)).into();
        let probe = probe_content(content, Size::new(240.0, 200.0));

        assert!(probe.found);
        assert!(
            probe.content_bounds.height > probe.bounds.height,
            "tall content should overflow the visible scroll body"
        );
    }

    #[test]
    fn the_inset_wraps_the_content_inside_the_scroll_region_not_as_outer_panel_padding() {
        let inner = layout_probe::probe("inner-content", Space::new().width(40).height(40));
        let probe = probe_content(inner, Size::new(240.0, 200.0));

        assert!(probe.found);
        // No outer panel padding: the scrollable itself starts at the
        // element's own origin.
        assert_eq!(probe.bounds.x, 0.0);
        assert_eq!(probe.bounds.y, 0.0);

        let inner_bounds = layout_probe::snapshot()
            .get("inner-content")
            .copied()
            .expect("inner content probe recorded");
        let inset = padding(PaddingRole::Panel);

        // The inset sits inside the scroll content, offsetting the inner
        // content away from the scroll region's own edge.
        assert!((inner_bounds.x - probe.content_bounds.x - inset.left).abs() < 0.5);
        assert!((inner_bounds.y - probe.content_bounds.y - inset.top).abs() < 0.5);
    }
}
