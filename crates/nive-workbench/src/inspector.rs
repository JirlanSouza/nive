use std::borrow::Cow;

use nive_ui::theme::ToneRole;
use nive_ui::widgets::EmptyState;
use nive_ui::{Element, IconRole};

use crate::panels::WorkbenchPanel;

/// Generic inspector helper state.
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
            Self::Content(content) => content,
        }
    }
}

/// Builds a generic inspector panel.
pub fn inspector_panel<'a, PanelId, ActionId, Message>(
    panel_id: PanelId,
    state: InspectorState<'a, Message>,
) -> WorkbenchPanel<'a, PanelId, ActionId, Message>
where
    Message: Clone + 'a,
{
    let status = match &state {
        InspectorState::Error { .. } => (ToneRole::Danger, "Inspector error"),
        InspectorState::Loading { .. } => (ToneRole::Accent, "Inspector loading"),
        InspectorState::NoSelection => (ToneRole::Neutral, "No selection"),
        InspectorState::Content(_) => (ToneRole::Neutral, "Inspector ready"),
    };

    WorkbenchPanel::new(panel_id, "Inspector", state.view())
        .icon(IconRole::PreferencesSystem)
        .status_text(status.0, status.1)
}
