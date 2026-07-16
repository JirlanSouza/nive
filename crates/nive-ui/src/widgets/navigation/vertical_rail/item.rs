use std::borrow::Cow;

use crate::theme::ToneRole;
use crate::widgets::primitives::IconRole;
use crate::widgets::{BadgeContent, BadgeKind};

/// Compact rail badge metadata.
///
/// The badge owns the visible label, semantic tone, and optional description.
/// The description is not a separate tooltip target; the owning rail item uses
/// it when composing its single item tooltip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerticalRailBadge<'a> {
    pub(super) content: BadgeContent<'a>,
    pub(super) label: Cow<'a, str>,
    pub(super) tone: ToneRole,
    pub(super) description: Option<Cow<'a, str>>,
}

impl<'a> VerticalRailBadge<'a> {
    /// Builds a neutral badge with visible text.
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            content: BadgeContent::Status(label.into()),
            label: Cow::Borrowed(""),
            tone: ToneRole::Neutral,
            description: None,
        }
        .cache_label()
    }

    pub fn count(count: u64) -> Self {
        Self {
            content: BadgeContent::Count(count),
            label: Cow::Owned(if count > 99 {
                "99+".into()
            } else {
                count.to_string()
            }),
            tone: ToneRole::Neutral,
            description: None,
        }
    }

    pub fn from_content(content: BadgeContent<'a>) -> Self {
        Self {
            content,
            label: Cow::Borrowed(""),
            tone: ToneRole::Neutral,
            description: None,
        }
        .cache_label()
    }

    fn cache_label(mut self) -> Self {
        self.label = match &self.content {
            BadgeContent::Count(count) => Cow::Owned(if *count > 99 {
                "99+".into()
            } else {
                count.to_string()
            }),
            BadgeContent::Status(label) => label.clone(),
        };
        self
    }

    /// Returns the visible badge label.
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn content(&self) -> &BadgeContent<'a> {
        &self.content
    }

    pub fn kind(&self) -> BadgeKind {
        self.content.kind()
    }

    /// Returns the semantic badge tone.
    pub fn tone_role(&self) -> ToneRole {
        self.tone
    }

    /// Returns the semantic badge description.
    pub fn description_text(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Sets the semantic badge tone.
    pub fn tone(mut self, tone: ToneRole) -> Self {
        self.tone = tone;
        self
    }

    /// Uses the neutral badge tone.
    pub fn neutral(self) -> Self {
        self.tone(ToneRole::Neutral)
    }

    /// Uses the accent badge tone.
    pub fn accent(self) -> Self {
        self.tone(ToneRole::Accent)
    }

    /// Uses the info badge tone.
    pub fn info(self) -> Self {
        self.tone(ToneRole::Info)
    }

    /// Uses the success badge tone.
    pub fn success(self) -> Self {
        self.tone(ToneRole::Success)
    }

    /// Uses the warning badge tone.
    pub fn warning(self) -> Self {
        self.tone(ToneRole::Warning)
    }

    /// Uses the danger badge tone.
    pub fn danger(self) -> Self {
        self.tone(ToneRole::Danger)
    }

    /// Sets the semantic description used by the owning item tooltip.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

impl<'a> From<Cow<'a, str>> for VerticalRailBadge<'a> {
    fn from(label: Cow<'a, str>) -> Self {
        Self::new(label)
    }
}

impl<'a> From<&'a str> for VerticalRailBadge<'a> {
    fn from(label: &'a str) -> Self {
        Self::new(label)
    }
}

impl<'a> From<String> for VerticalRailBadge<'a> {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

/// Data for one vertical rail entry.
///
/// `VerticalRailItem` carries item identity and metadata only. Activation is
/// configured on [`super::VerticalRail`] with `on_select` or `on_select_maybe`.
/// The label is the visible rotated label and accessible label. Optional icon,
/// badge metadata renders upright while only the label rotates.
#[derive(Debug, Clone)]
pub struct VerticalRailItem<'a, Id> {
    pub(super) id: Id,
    pub(super) label: Cow<'a, str>,
    pub(super) icon: Option<IconRole>,
    pub(super) selected: bool,
    pub(super) disabled: bool,
    pub(super) badge: Option<VerticalRailBadge<'a>>,
    pub(super) tooltip: Option<Cow<'a, str>>,
}

impl<'a, Id> VerticalRailItem<'a, Id> {
    /// Builds an item with mandatory identity and visible/accessibility label.
    pub fn new(id: Id, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            icon: None,
            selected: false,
            disabled: false,
            badge: None,
            tooltip: None,
        }
    }

    /// Returns the item identity.
    pub fn id(&self) -> &Id {
        &self.id
    }

    /// Returns the visible and accessible item label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Sets the upright icon rendered at the icon end of the item.
    pub fn icon(mut self, icon: IconRole) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the selected visual state. Selection remains app-owned.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Disables pointer and keyboard activation.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets the upright badge metadata.
    pub fn badge(mut self, badge: impl Into<VerticalRailBadge<'a>>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Sets the tooltip shown for this item.
    ///
    /// Explicit tooltips override badge-description and full-label fallbacks.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }
}
