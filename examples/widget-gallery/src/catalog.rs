#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Actions,
    Inputs,
    Display,
    LayoutNav,
    Overlays,
    Feedback,
    Theme,
    Icons,
    Motion,
}

#[derive(Debug, Clone, Copy)]
pub struct CatalogEntry {
    pub id: PageId,
    pub family: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub terms: &'static [&'static str],
}

pub const CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: PageId::Actions,
        family: "Controls",
        title: "Controls",
        summary: "Buttons, segmented controls, grouped actions, and command triggers.",
        terms: &[
            "controls",
            "button",
            "action group",
            "segmented",
            "toolbar",
            "dropdown",
            "menu",
            "action card",
        ],
    },
    CatalogEntry {
        id: PageId::Inputs,
        family: "Composite",
        title: "Composite Inputs",
        summary: "Fields, input groups, path inputs, and composed control rows.",
        terms: &[
            "composite",
            "input",
            "field",
            "field group",
            "input group",
            "checkbox",
            "switch",
            "select",
            "segmented",
            "autocomplete",
            "color",
            "path",
        ],
    },
    CatalogEntry {
        id: PageId::Display,
        family: "Primitives",
        title: "Primitives / Display",
        summary: "Text, icons, separators, badges, metadata, avatars, and display cards.",
        terms: &[
            "primitive",
            "display",
            "text",
            "icon",
            "badge",
            "card",
            "panel",
            "metric",
            "metadata",
            "avatar",
            "version",
            "separator",
        ],
    },
    CatalogEntry {
        id: PageId::LayoutNav,
        family: "Containers",
        title: "Containers / Navigation",
        summary: "Panels, split panes, tabs, trees, section headers, and selectable rows.",
        terms: &[
            "container",
            "navigation",
            "panel",
            "tab",
            "split pane",
            "tree",
            "section header",
            "selectable",
        ],
    },
    CatalogEntry {
        id: PageId::Overlays,
        family: "Overlays",
        title: "Overlays",
        summary: "Dialogs, popovers, tooltips, command palette rows, and overlay hosts.",
        terms: &["dialog", "popover", "tooltip", "command palette", "overlay"],
    },
    CatalogEntry {
        id: PageId::Feedback,
        family: "Feedback",
        title: "Feedback",
        summary: "Alerts, loading, progress, empty/error states, and status lines.",
        terms: &[
            "alert",
            "callout",
            "progress",
            "spinner",
            "loading",
            "skeleton",
            "empty",
            "error",
            "resource",
            "operation",
        ],
    },
    CatalogEntry {
        id: PageId::Theme,
        family: "Theme",
        title: "Theme",
        summary: "Theme roles, typography, spacing, shapes, and control sizes.",
        terms: &["theme", "role", "typography", "spacing", "shape", "surface"],
    },
    CatalogEntry {
        id: PageId::Icons,
        family: "Graphics",
        title: "Graphics / Icons",
        summary: "Framework icons across size, tone, and dense-grid scenarios.",
        terms: &["graphics", "icon", "lucide", "asset"],
    },
    CatalogEntry {
        id: PageId::Motion,
        family: "Motion",
        title: "Motion",
        summary: "Animation primitives and time-driven visual/layout examples.",
        terms: &["animation", "motion", "animated", "easing", "pulse"],
    },
];

pub fn entry_for(id: PageId) -> &'static CatalogEntry {
    CATALOG
        .iter()
        .find(|entry| entry.id == id)
        .expect("page id is present in static catalog")
}

pub fn matches(entry: &CatalogEntry, query: &str) -> bool {
    let query = query.trim().to_ascii_lowercase();
    query.is_empty()
        || entry.title.to_ascii_lowercase().contains(&query)
        || entry.family.to_ascii_lowercase().contains(&query)
        || entry.summary.to_ascii_lowercase().contains(&query)
        || entry
            .terms
            .iter()
            .any(|term| term.to_ascii_lowercase().contains(&query))
}
