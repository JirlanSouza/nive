mod generated;
pub mod lucide;

/// A semantic icon role resolved through the active theme catalog.
///
/// Directional roles such as [`Self::GoNext`], [`Self::GoPrevious`], and the
/// disclosure roles resolve left-to-right today. They are the locale-aware
/// extension point for future direction-aware catalogs; callers should use
/// them instead of rotating a physical arrow.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IconRole {
    DialogError,
    DialogWarning,
    DialogSuccess,
    DialogInformation,
    ActionConfirm,
    EditCopy,
    EditDelete,
    EditFind,
    EditModify,
    Folder,
    GoNext,
    GoPrevious,
    Identity,
    ListAdd,
    ListRemove,
    MailInbox,
    NotificationAlert,
    NiveDisclosureDown,
    NiveDisclosureLeft,
    NiveDisclosureRight,
    NiveDisclosureUp,
    OpenMenu,
    PreferencesSystem,
    TabPinned,
    ValidationError,
    ViewActivity,
    ViewConceal,
    ViewMaximize,
    ViewMore,
    ViewRefresh,
    ViewRestore,
    ViewReveal,
    ViewTheme,
    WindowClose,
}

impl IconRole {
    pub const ALL: &'static [Self] = &[
        Self::DialogError,
        Self::DialogWarning,
        Self::DialogSuccess,
        Self::DialogInformation,
        Self::ActionConfirm,
        Self::EditCopy,
        Self::EditDelete,
        Self::EditFind,
        Self::EditModify,
        Self::Folder,
        Self::GoNext,
        Self::GoPrevious,
        Self::Identity,
        Self::ListAdd,
        Self::ListRemove,
        Self::MailInbox,
        Self::NotificationAlert,
        Self::NiveDisclosureDown,
        Self::NiveDisclosureLeft,
        Self::NiveDisclosureRight,
        Self::NiveDisclosureUp,
        Self::OpenMenu,
        Self::PreferencesSystem,
        Self::TabPinned,
        Self::ValidationError,
        Self::ViewActivity,
        Self::ViewConceal,
        Self::ViewMaximize,
        Self::ViewMore,
        Self::ViewRefresh,
        Self::ViewRestore,
        Self::ViewReveal,
        Self::ViewTheme,
        Self::WindowClose,
    ];

    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::DialogError => "dialog-error",
            Self::DialogWarning => "dialog-warning",
            Self::DialogSuccess => "dialog-success",
            Self::DialogInformation => "dialog-information",
            Self::ActionConfirm => "action-confirm",
            Self::EditCopy => "edit-copy",
            Self::EditDelete => "edit-delete",
            Self::EditFind => "edit-find",
            Self::EditModify => "edit-modify",
            Self::Folder => "folder",
            Self::GoNext => "go-next",
            Self::GoPrevious => "go-previous",
            Self::Identity => "identity",
            Self::ListAdd => "list-add",
            Self::ListRemove => "list-remove",
            Self::MailInbox => "mail-inbox",
            Self::NotificationAlert => "notification-alert",
            Self::NiveDisclosureDown => "nive-disclosure-down",
            Self::NiveDisclosureLeft => "nive-disclosure-left",
            Self::NiveDisclosureRight => "nive-disclosure-right",
            Self::NiveDisclosureUp => "nive-disclosure-up",
            Self::OpenMenu => "open-menu",
            Self::PreferencesSystem => "preferences-system",
            Self::TabPinned => "tab-pinned",
            Self::ValidationError => "validation-error",
            Self::ViewActivity => "view-activity",
            Self::ViewConceal => "view-conceal",
            Self::ViewMaximize => "view-maximize",
            Self::ViewMore => "view-more",
            Self::ViewRefresh => "view-refresh",
            Self::ViewRestore => "view-restore",
            Self::ViewReveal => "view-reveal",
            Self::ViewTheme => "view-theme",
            Self::WindowClose => "window-close",
        }
    }

    pub fn from_canonical_name(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|role| role.canonical_name() == name)
    }
}

/// An icon a widget renders: a framework role or an application-owned glyph.
///
/// Widget icon slots accept anything convertible into this, so a framework role
/// and an icon generated into an application keep the same call shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconRef {
    /// A semantic role resolved through the active theme catalog.
    Role(IconRole),
    /// Ready-to-render SVG bytes, typically from an app's generated symbols.
    Glyph(IconGlyph),
}

impl IconRef {
    /// Captures any [`IconSource`], including an application's generated symbols.
    pub fn from_source<S>(source: S) -> Self
    where
        S: IconSource,
    {
        Self::Glyph(IconGlyph::new(source.svg_bytes(), source.provider_slug()))
    }
}

impl From<IconRole> for IconRef {
    fn from(role: IconRole) -> Self {
        Self::Role(role)
    }
}

impl From<IconGlyph> for IconRef {
    fn from(glyph: IconGlyph) -> Self {
        Self::Glyph(glyph)
    }
}

/// Static SVG bytes ready to render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconGlyph {
    svg_bytes: &'static [u8],
    provider_slug: &'static str,
}

impl IconGlyph {
    pub const fn new(svg_bytes: &'static [u8], provider_slug: &'static str) -> Self {
        Self {
            svg_bytes,
            provider_slug,
        }
    }

    pub const fn svg_bytes(self) -> &'static [u8] {
        self.svg_bytes
    }

    pub const fn provider_slug(self) -> &'static str {
        self.provider_slug
    }

    pub fn from_source(source: impl IconSource) -> Self {
        Self::new(source.svg_bytes(), source.provider_slug())
    }
}

/// Concrete app or framework icon data that can be rendered directly.
///
/// Custom sources must use a 24×24 view box, stroke width 2, rounded line caps
/// and joins, consistent optical margins, and monochrome `currentColor`
/// rendering. The `nive-cli` authoring/codegen workflow vets this geometry;
/// the runtime primitive trusts generated sources.
pub trait IconSource: Copy + 'static {
    fn svg_bytes(self) -> &'static [u8];

    fn provider_slug(self) -> &'static str;
}

impl IconSource for IconGlyph {
    fn svg_bytes(self) -> &'static [u8] {
        self.svg_bytes()
    }

    fn provider_slug(self) -> &'static str {
        self.provider_slug()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconCatalogEntry {
    pub role: IconRole,
    pub glyph: IconGlyph,
}

impl IconCatalogEntry {
    pub const fn new(role: IconRole, glyph: IconGlyph) -> Self {
        Self { role, glyph }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconCatalog {
    entries: &'static [IconCatalogEntry],
}

impl IconCatalog {
    pub const fn new(entries: &'static [IconCatalogEntry]) -> Self {
        Self { entries }
    }

    pub const fn empty() -> Self {
        Self { entries: &[] }
    }

    pub const fn entries(self) -> &'static [IconCatalogEntry] {
        self.entries
    }

    pub fn glyph(self, role: IconRole) -> Option<IconGlyph> {
        self.entries
            .iter()
            .find(|entry| entry.role == role)
            .map(|entry| entry.glyph)
    }

    pub fn covers(self, role: IconRole) -> bool {
        self.glyph(role).is_some()
    }
}

impl Default for IconCatalog {
    fn default() -> Self {
        default_catalog()
    }
}

pub const fn default_catalog() -> IconCatalog {
    generated::catalog::APP_ICON_CATALOG
}

pub fn default_glyph_for(role: IconRole) -> IconGlyph {
    default_catalog()
        .glyph(role)
        .expect("default icon catalog must cover every IconRole")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_ref_carries_roles_and_app_glyphs() {
        const GLYPH: IconGlyph = IconGlyph::new(b"<svg/>", "custom:brand");

        assert_eq!(
            IconRef::from(IconRole::Folder),
            IconRef::Role(IconRole::Folder)
        );
        assert_eq!(IconRef::from(GLYPH), IconRef::Glyph(GLYPH));
        // Any source, including an app's generated symbols, resolves to a glyph.
        assert_eq!(IconRef::from_source(GLYPH), IconRef::Glyph(GLYPH));
    }

    #[test]
    fn role_canonical_names_round_trip() {
        for role in IconRole::ALL {
            assert_eq!(
                IconRole::from_canonical_name(role.canonical_name()),
                Some(*role)
            );
        }
    }

    #[test]
    fn identity_uses_the_provider_neutral_canonical_name() {
        assert_eq!(IconRole::Identity.canonical_name(), "identity");
        assert_eq!(
            IconRole::from_canonical_name("identity"),
            Some(IconRole::Identity)
        );
    }

    #[test]
    fn custom_catalog_can_supply_identity_and_missing_catalog_is_detectable() {
        const GLYPH: IconGlyph = IconGlyph::new(
            br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
            "custom:identity",
        );
        const CATALOG: IconCatalog =
            IconCatalog::new(&[IconCatalogEntry::new(IconRole::Identity, GLYPH)]);

        assert_eq!(CATALOG.glyph(IconRole::Identity), Some(GLYPH));
        assert!(!IconCatalog::empty().covers(IconRole::Identity));
    }

    #[test]
    fn validation_error_uses_the_provider_neutral_canonical_name() {
        assert_eq!(
            IconRole::ValidationError.canonical_name(),
            "validation-error"
        );
        assert_eq!(
            IconRole::from_canonical_name("validation-error"),
            Some(IconRole::ValidationError)
        );
    }

    #[test]
    fn custom_and_empty_catalogs_expose_validation_error_coverage() {
        const GLYPH: IconGlyph = IconGlyph::new(
            br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
            "custom:validation-error",
        );
        const CATALOG: IconCatalog =
            IconCatalog::new(&[IconCatalogEntry::new(IconRole::ValidationError, GLYPH)]);

        assert_eq!(CATALOG.glyph(IconRole::ValidationError), Some(GLYPH));
        assert!(!IconCatalog::empty().covers(IconRole::ValidationError));
        assert!(IconRole::ALL.contains(&IconRole::Identity));
    }
}
