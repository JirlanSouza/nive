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
    ListAdd,
    ListRemove,
    MailInbox,
    NiveDisclosureDown,
    NiveDisclosureLeft,
    NiveDisclosureRight,
    NiveDisclosureUp,
    OpenMenu,
    PreferencesSystem,
    TabPinned,
    ViewConceal,
    ViewMore,
    ViewRefresh,
    ViewReveal,
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
        Self::ListAdd,
        Self::ListRemove,
        Self::MailInbox,
        Self::NiveDisclosureDown,
        Self::NiveDisclosureLeft,
        Self::NiveDisclosureRight,
        Self::NiveDisclosureUp,
        Self::OpenMenu,
        Self::PreferencesSystem,
        Self::TabPinned,
        Self::ViewConceal,
        Self::ViewMore,
        Self::ViewRefresh,
        Self::ViewReveal,
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
            Self::ListAdd => "list-add",
            Self::ListRemove => "list-remove",
            Self::MailInbox => "mail-inbox",
            Self::NiveDisclosureDown => "nive-disclosure-down",
            Self::NiveDisclosureLeft => "nive-disclosure-left",
            Self::NiveDisclosureRight => "nive-disclosure-right",
            Self::NiveDisclosureUp => "nive-disclosure-up",
            Self::OpenMenu => "open-menu",
            Self::PreferencesSystem => "preferences-system",
            Self::TabPinned => "tab-pinned",
            Self::ViewConceal => "view-conceal",
            Self::ViewMore => "view-more",
            Self::ViewRefresh => "view-refresh",
            Self::ViewReveal => "view-reveal",
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
    fn role_canonical_names_round_trip() {
        for role in IconRole::ALL {
            assert_eq!(
                IconRole::from_canonical_name(role.canonical_name()),
                Some(*role)
            );
        }
    }
}
