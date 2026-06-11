#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppIcon {
    Add,
    CircleAlert,
    Folder,
    Grid,
    Inbox,
    Info,
    RefreshCw,
    Search,
    X,
}

impl AppIcon {
    fn svg_bytes(&self) -> &'static [u8] {
        match self {
            Self::Add => include_bytes!("../../assets/icons/lucide/plus.svg"),
            Self::CircleAlert => include_bytes!("../../assets/icons/lucide/circle-alert.svg"),
            Self::Folder => include_bytes!("../../assets/icons/lucide/folder.svg"),
            Self::Grid => include_bytes!("../../assets/icons/lucide/layout-grid.svg"),
            Self::Inbox => include_bytes!("../../assets/icons/lucide/inbox.svg"),
            Self::Info => include_bytes!("../../assets/icons/lucide/info.svg"),
            Self::RefreshCw => include_bytes!("../../assets/icons/lucide/refresh-cw.svg"),
            Self::Search => include_bytes!("../../assets/icons/lucide/search.svg"),
            Self::X => include_bytes!("../../assets/icons/lucide/x.svg"),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn provider_slug(&self) -> &'static str {
        match self {
            Self::Add => "plus",
            Self::CircleAlert => "circle-alert",
            Self::Folder => "folder",
            Self::Grid => "layout-grid",
            Self::Inbox => "inbox",
            Self::Info => "info",
            Self::RefreshCw => "refresh-cw",
            Self::Search => "search",
            Self::X => "x",
        }
    }
}
