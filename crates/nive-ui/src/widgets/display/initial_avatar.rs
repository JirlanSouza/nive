use std::{borrow::Cow, marker::PhantomData};

use iced::{
    widget::{container, stack, text, Space},
    Alignment, Background, Border, Color, Length, Padding, Shadow,
};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    theme::{
        self, BorderRole, ShapeSize, SurfaceRole, TextRole, TextStyle, ToneRole, TypographyRole,
    },
    tokens::typography as token_typography,
    widgets::Icon,
    Element, IconRole,
};

/// Visual treatment for neutral or explicitly accent-owned identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvatarClass {
    /// Neutral identity treatment on an ordinary surface.
    Surface,
    /// Explicit product/accent identity treatment.
    Accent,
}

/// Identity-specific fixed geometry, independent from ControlSize and density.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
    /// 24 px, one initial.
    Xs,
    /// 32 px default.
    #[default]
    Sm,
    /// 40 px.
    Md,
    /// 56 px.
    Lg,
}

/// Person-circle or entity rounded-square semantics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AvatarKind {
    /// Circular human identity.
    Person,
    /// Shared rounded-square service, product, or workspace identity.
    #[default]
    Entity,
}

/// Stable avatar status with an explicit marker-outline source.
///
/// Use [`Self::on_surface`] or [`Self::with_outline`] only on static host
/// backgrounds. Controls whose fill changes with interaction use
/// [`Self::on_interactive`] so the Strong separator remains stable.
#[derive(Debug, Clone, Copy, PartialEq)]
enum StatusOutline {
    Surface(SurfaceRole),
    Color(Color),
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AvatarStatus {
    tone: ToneRole,
    outline: StatusOutline,
}

impl AvatarStatus {
    /// Uses a known static semantic surface as the marker cutout source.
    pub const fn on_surface(tone: ToneRole, surface: SurfaceRole) -> Self {
        Self {
            tone,
            outline: StatusOutline::Surface(surface),
        }
    }

    /// Uses an explicit static host color as the marker cutout source.
    pub const fn with_outline(tone: ToneRole, color: Color) -> Self {
        Self {
            tone,
            outline: StatusOutline::Color(color),
        }
    }

    /// Uses the stable Strong separator for changing interactive fills.
    pub const fn on_interactive(tone: ToneRole) -> Self {
        Self {
            tone,
            outline: StatusOutline::Interactive,
        }
    }

    /// Returns the semantic status tone.
    pub const fn tone(self) -> ToneRole {
        self.tone
    }
}

/// Textual identity fallback with fixed identity-specific geometry.
///
/// Initials are selected by Unicode grapheme and invalid/empty identities use
/// the active catalog's `IconRole::Identity`. Custom catalogs must map the
/// canonical `identity` role and regenerate checked-in icon artifacts. A
/// status marker never changes the avatar footprint; hosts still own complete
/// visible identity and status labels because the avatar has no tooltip-only
/// accessibility contract.
pub struct InitialAvatar<'a, Message> {
    label: Cow<'a, str>,
    class: AvatarClass,
    size: AvatarSize,
    kind: AvatarKind,
    status: Option<AvatarStatus>,
    _marker: PhantomData<Message>,
}

impl<'a, Message> InitialAvatar<'a, Message>
where
    Message: 'a,
{
    pub fn new(label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            label: label.into(),
            class: AvatarClass::Surface,
            size: AvatarSize::Sm,
            kind: AvatarKind::Entity,
            status: None,
            _marker: PhantomData,
        }
    }

    /// Selects neutral Surface or explicit Accent identity treatment.
    pub fn class(mut self, class: AvatarClass) -> Self {
        self.class = class;
        self
    }

    pub fn accent(self) -> Self {
        self.class(AvatarClass::Accent)
    }

    /// Selects identity-specific geometry independently from ControlSize.
    pub fn size(mut self, size: AvatarSize) -> Self {
        self.size = size;
        self
    }

    pub fn xs(self) -> Self {
        self.size(AvatarSize::Xs)
    }

    pub fn sm(self) -> Self {
        self.size(AvatarSize::Sm)
    }

    pub fn md(self) -> Self {
        self.size(AvatarSize::Md)
    }

    pub fn lg(self) -> Self {
        self.size(AvatarSize::Lg)
    }

    /// Selects person-circle or entity-rounded identity shape.
    pub fn kind(mut self, kind: AvatarKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn person(self) -> Self {
        self.kind(AvatarKind::Person)
    }

    pub fn entity(self) -> Self {
        self.kind(AvatarKind::Entity)
    }

    /// Adds a layout-neutral lower-trailing status marker.
    pub fn status(mut self, status: AvatarStatus) -> Self {
        self.status = Some(status);
        self
    }

    fn into_element(self) -> Element<'a, Message> {
        let metrics = metrics(self.size, self.kind);
        let initials = initials(self.label.as_ref(), self.size);
        let content: Element<'a, Message> = initials.map_or_else(
            || {
                Icon::role(IconRole::Identity)
                    .custom_size(metrics.icon_size)
                    .into()
            },
            |initials| {
                text(initials)
                    .font(metrics.text_style.font)
                    .size(metrics.text_style.size)
                    .line_height(text::LineHeight::Relative(metrics.text_style.line_height))
                    .center()
                    .into()
            },
        );

        let avatar: Element<'a, Message> = container(content)
            .style(avatar_style(self.class, metrics.radius))
            .width(Length::Fixed(metrics.dimension))
            .height(Length::Fixed(metrics.dimension))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into();

        let Some(status) = self.status else {
            return avatar;
        };

        let marker = status_marker(status, metrics.status_core);
        let overlay = container(marker)
            .width(Length::Fixed(metrics.dimension))
            .height(Length::Fixed(metrics.dimension))
            .align_x(Alignment::End)
            .align_y(Alignment::End);

        stack![avatar, overlay]
            .width(Length::Fixed(metrics.dimension))
            .height(Length::Fixed(metrics.dimension))
            .into()
    }
}

impl<'a, Message> From<InitialAvatar<'a, Message>> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(avatar: InitialAvatar<'a, Message>) -> Self {
        avatar.into_element()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Metrics {
    dimension: f32,
    text_style: TextStyle,
    icon_size: f32,
    radius: f32,
    status_core: f32,
}

fn metrics(size: AvatarSize, kind: AvatarKind) -> Metrics {
    let (dimension, text_style, icon_size, status_core) = match size {
        AvatarSize::Xs => (
            24.0,
            TextStyle {
                font: token_typography::UI.semibold(),
                size: 10.0,
                line_height: token_typography::LEADING_SNUG,
                letter_spacing: 0.0,
            },
            12.0,
            6.0,
        ),
        AvatarSize::Sm => (
            32.0,
            theme::typography(TypographyRole::SectionLabel),
            16.0,
            6.0,
        ),
        AvatarSize::Md => (
            40.0,
            theme::typography(TypographyRole::BodyStrong),
            20.0,
            8.0,
        ),
        AvatarSize::Lg => (56.0, theme::typography(TypographyRole::Title), 28.0, 8.0),
    };
    let radius = match kind {
        AvatarKind::Person => dimension / 2.0,
        AvatarKind::Entity => theme::active().shape(ShapeSize::Md).radius_value(),
    };

    Metrics {
        dimension,
        text_style,
        icon_size,
        radius,
        status_core,
    }
}

fn initials(label: &str, size: AvatarSize) -> Option<String> {
    let terms: Vec<&str> = label
        .split_whitespace()
        .filter(|term| meaningful_grapheme(term).is_some())
        .collect();
    let first = meaningful_grapheme(terms.first()?)?;

    if matches!(size, AvatarSize::Xs) || terms.len() == 1 {
        return Some(first.to_string());
    }

    let last = meaningful_grapheme(terms.last()?)?;
    Some(format!("{first}{last}"))
}

fn meaningful_grapheme(term: &str) -> Option<&str> {
    term.graphemes(true)
        .find(|grapheme| grapheme.chars().any(char::is_alphanumeric))
}

fn avatar_style(
    class: AvatarClass,
    radius: f32,
) -> impl Fn(&crate::theme::Theme) -> container::Style {
    move |theme| {
        let (background, text_color) = match class {
            AvatarClass::Surface => {
                let neutral = theme.tone(ToneRole::Neutral);
                (neutral.container, theme.text(TextRole::Secondary).color)
            }
            AvatarClass::Accent => {
                let accent = theme.tone(ToneRole::Accent);
                (accent.color, accent.on_color)
            }
        };
        let border = theme.border(BorderRole::Subtle);

        container::Style {
            text_color: Some(text_color),
            background: Some(Background::Color(background)),
            border: Border {
                color: border.color,
                width: 1.0,
                radius: radius.into(),
            },
            shadow: Shadow::default(),
            ..container::Style::default()
        }
    }
}

fn status_marker<'a, Message>(status: AvatarStatus, core: f32) -> Element<'a, Message>
where
    Message: 'a,
{
    let outer = core + 4.0;
    let inner = container(Space::new())
        .style(move |theme: &crate::theme::Theme| container::Style {
            background: Some(Background::Color(theme.tone(status.tone).color)),
            border: Border {
                radius: (core / 2.0).into(),
                ..Border::default()
            },
            ..container::Style::default()
        })
        .width(Length::Fixed(core))
        .height(Length::Fixed(core));

    container(inner)
        .style(move |theme: &crate::theme::Theme| {
            let outline = match status.outline {
                StatusOutline::Surface(surface) => theme.surface(surface).background,
                StatusOutline::Color(color) => color,
                StatusOutline::Interactive => theme.border(BorderRole::Strong).color,
            };
            container::Style {
                background: Some(Background::Color(outline)),
                border: Border {
                    radius: (outer / 2.0).into(),
                    ..Border::default()
                },
                ..container::Style::default()
            }
        })
        .padding(Padding::new(2.0))
        .width(Length::Fixed(outer))
        .height(Length::Fixed(outer))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;

    #[test]
    fn avatar_sizes_and_kinds_have_fixed_geometry() {
        for (size, dimension, font, core) in [
            (AvatarSize::Xs, 24.0, 10.0, 6.0),
            (AvatarSize::Sm, 32.0, 12.0, 6.0),
            (AvatarSize::Md, 40.0, 14.0, 8.0),
            (AvatarSize::Lg, 56.0, 20.0, 8.0),
        ] {
            let entity = metrics(size, AvatarKind::Entity);
            let person = metrics(size, AvatarKind::Person);
            assert_eq!(entity.dimension, dimension);
            assert_eq!(entity.text_style.size, font);
            assert_eq!(entity.status_core, core);
            assert_eq!(person.radius, dimension / 2.0);
        }
    }

    #[test]
    fn initials_are_unicode_grapheme_safe_and_size_specific() {
        assert_eq!(
            initials("  Widget   Gallery ", AvatarSize::Xs).as_deref(),
            Some("W")
        );
        assert_eq!(
            initials("  Widget   Gallery ", AvatarSize::Sm).as_deref(),
            Some("WG")
        );
        assert_eq!(initials("Élodie", AvatarSize::Lg).as_deref(), Some("É"));
        assert_eq!(
            initials("e\u{301}clair studio", AvatarSize::Md).as_deref(),
            Some("e\u{301}s")
        );
        assert_eq!(initials("界 面", AvatarSize::Md).as_deref(), Some("界面"));
    }

    #[test]
    fn invalid_identity_clusters_use_icon_fallback() {
        for label in ["", "   ", "!!!", "👩‍💻", "\u{301}", "\u{200d}"] {
            assert_eq!(initials(label, AvatarSize::Sm), None, "{label:?}");
        }
        assert_eq!(initials("1️⃣", AvatarSize::Sm).as_deref(), Some("1️⃣"));
    }

    #[test]
    fn status_metrics_include_outward_two_pixel_outline() {
        assert_eq!(
            metrics(AvatarSize::Sm, AvatarKind::Entity).status_core + 4.0,
            10.0
        );
        assert_eq!(
            metrics(AvatarSize::Lg, AvatarKind::Entity).status_core + 4.0,
            12.0
        );
    }

    #[test]
    fn status_requires_explicit_static_or_interactive_outline_source() {
        assert!(matches!(
            AvatarStatus::on_surface(ToneRole::Success, SurfaceRole::Panel).outline,
            StatusOutline::Surface(SurfaceRole::Panel)
        ));
        assert!(matches!(
            AvatarStatus::with_outline(ToneRole::Warning, Color::BLACK).outline,
            StatusOutline::Color(Color::BLACK)
        ));
        assert!(matches!(
            AvatarStatus::on_interactive(ToneRole::Info).outline,
            StatusOutline::Interactive
        ));
    }

    #[test]
    fn real_layout_keeps_fixed_square_and_status_overlay_footprint() {
        for (size, dimension) in [
            (AvatarSize::Xs, 24.0),
            (AvatarSize::Sm, 32.0),
            (AvatarSize::Md, 40.0),
            (AvatarSize::Lg, 56.0),
        ] {
            let plain = crate::test_support::layout(
                InitialAvatar::<()>::new("Ada Lovelace").size(size).into(),
                Size::new(100.0, 100.0),
            );
            let status = crate::test_support::layout(
                InitialAvatar::<()>::new("Ada Lovelace")
                    .size(size)
                    .status(AvatarStatus::on_surface(
                        ToneRole::Success,
                        SurfaceRole::Panel,
                    ))
                    .into(),
                Size::new(100.0, 100.0),
            );
            assert_eq!(plain.size(), Size::new(dimension, dimension));
            assert_eq!(status.size(), plain.size());
            assert!(status
                .children()
                .iter()
                .all(|child| status.bounds().contains(child.bounds().center())));
        }
    }

    #[test]
    fn real_layout_icon_fallback_keeps_identity_geometry() {
        let node = crate::test_support::layout(
            InitialAvatar::<()>::new("👩‍💻").person().into(),
            Size::new(100.0, 100.0),
        );
        assert_eq!(node.size(), Size::new(32.0, 32.0));
    }
}
