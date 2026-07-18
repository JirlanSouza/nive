pub mod advanced;
pub mod containers;
pub mod controls;
pub mod display;
pub mod feedback;
pub mod navigation;
pub mod overlays;
pub mod primitives;
pub mod scrollable;

// Namespace modules for the iced-style `button::primary(...)` call idiom.
pub use controls::{button, input};
pub use overlays::tooltip;
pub use primitives::{icon, text};

pub use containers::{
    ActionCard, Card, CardVariant, Panel, SectionHeader, SectionHeaderAction, SectionHeaderStatus,
    SelectableCard, SplitPane, SplitPaneConstraints,
};
#[allow(deprecated)]
pub use controls::{
    ActionGroup, Autocomplete, AutocompleteMessage, Button, ButtonIntent, ButtonVariant, Checkbox,
    CheckboxState, ColorInput, ColorPicker, ContentAction, Field, FieldControl, FieldError,
    FieldGroup, FieldGroupLayout, FieldHint, FieldLabel, FieldRequirement, FieldValidation, Input,
    InputGroup, InputGroupVariant, LegacySegmentedControl, PathInput, RadioGroup, RadioGroupLayout,
    RadioOption, RgbHexColor, SegmentedControl, SegmentedControlVariant, SegmentedItem,
    SegmentedOption, Select, SelectableItem, Switch, TextInputAppearance,
};
#[allow(deprecated)]
pub use display::{
    reveal, row_height, scroll_offset_to, visible_index_of, AvatarClass, AvatarKind, AvatarSize,
    AvatarStatus, Badge, BadgeContent, BadgeKind, DataRow, EmptyState, InitialAvatar, KeyValueList,
    MetadataItem, MetadataTag, MetricCard, Tree, TreeChildren, TreeDrag, TreeDrop, TreeDropTarget,
    TreeEvent, TreeEventKind, TreeExpandBehavior, TreeItem, TreeNode, TreePasteTarget, TreeState,
    TreeStateChange, VersionBadge,
};
pub use feedback::{
    ErrorDetailsDialog, ErrorEmptyState, ErrorFeedback, ErrorFeedbackAction,
    ErrorFeedbackActionRow, ErrorFeedbackCommandRole, ErrorPresentation, ErrorStatusLine,
    InlineAlert, OperationActionGroup, OperationStatusLine, OperationStatusPresentation,
    ProgressBar, ResourceStatusLine, ResourceStatusPresentation, Skeleton, SkeletonCard,
    SkeletonControl, Spinner,
};
pub use navigation::{
    command_palette_filter, command_palette_view, CommandPaletteRow, Menu, MenuCheckbox,
    MenuCommand, MenuDismissPolicy, MenuRadioGroup, MenuRadioOption, MenuSubmenu, RailSide, TabBar,
    TabCloseRequest, TabCloseTrigger, TabDrop, TabDropTarget, TabItem, TabTearOff, Toolbar,
    ToolbarAction, ToolbarGroup, VerticalRail, VerticalRailBadge, VerticalRailItem,
};
pub use overlays::{
    Dialog, DialogActionFooter, DialogFooter, DialogHeader, DialogHost, Popover, PopoverCollision,
    PopoverFocusPolicy, PopoverInset, PopoverPlacement, PopoverWidth, ToastHost, ToastPosition,
    ToastPresentation, ToastTone, Tooltip, TooltipPlacement, TooltipScope,
};
pub use primitives::{
    space, svg, ColorSwatch, Icon, IconGlyph, IconRole, IconSource, Separator, SeparatorExtent,
    SeparatorStrength, StatusIndicator, ToneDot,
};
pub use scrollable::overlay_scrollbar;
