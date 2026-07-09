pub mod advanced;
pub mod containers;
pub mod controls;
pub mod display;
pub mod feedback;
pub mod navigation;
pub mod overlays;
pub mod primitives;

// Namespace modules for the iced-style `button::primary(...)` call idiom.
pub use controls::{button, input};
pub use overlays::tooltip;
pub use primitives::{icon, text};

pub use containers::{
    ActionCard, Card, Panel, SectionHeader, SectionHeaderAction, SectionHeaderStatus,
    SelectableCard, SplitPane, SplitPaneConstraints,
};
pub use controls::{
    Autocomplete, AutocompleteMessage, Button, ButtonVariant, Checkbox, ColorInput, ColorPicker,
    Field, FieldError, FieldGroup, FieldHint, FieldLabel, FieldValidation, Input, InputGroup,
    InputGroupVariant, PathInput, RgbHexColor, SegmentedControl, SegmentedItem, Select,
    SelectableItem, Switch, TextInputAppearance,
};
pub use display::{
    reveal, row_height, scroll_offset_to, visible_index_of, AvatarClass, AvatarSize, Badge,
    DataRow, EmptyState, InitialAvatar, KeyValueList, MetadataItem, MetricCard, Tree, TreeChildren,
    TreeDrag, TreeDrop, TreeDropTarget, TreeEvent, TreeEventKind, TreeExpandBehavior, TreeItem,
    TreeNode, TreePasteTarget, TreeState, TreeStateChange, VersionBadge,
};
pub use feedback::{
    ErrorDetailsDialog, ErrorEmptyState, ErrorFeedback, ErrorFeedbackAction,
    ErrorFeedbackActionRow, ErrorFeedbackCommandRole, ErrorPresentation, ErrorStatusLine,
    InlineAlert, OperationActionGroup, OperationStatusLine, OperationStatusPresentation,
    ProgressBar, ResourceStatusLine, ResourceStatusPresentation, Skeleton, SkeletonCard,
    SkeletonControl, Spinner,
};
pub use navigation::{
    command_palette_filter, command_palette_view, ActionGroup, CommandPaletteRow, DropdownMenu,
    DropdownMenuItem, TabBar, TabCloseRequest, TabCloseTrigger, TabDrop, TabDropTarget, TabItem,
    TabTearOff, Toolbar, ToolbarAction, ToolbarGroup,
};
pub use overlays::{
    Dialog, DialogActionFooter, DialogFooter, DialogHeader, DialogHost, Popover, PopoverCollision,
    PopoverPlacement, PopoverWidth, ToastHost, ToastPosition, ToastPresentation, ToastTone,
};
pub use primitives::{space, svg, ColorSwatch, Icon, IconGlyph, IconRole, IconSource, Separator};
