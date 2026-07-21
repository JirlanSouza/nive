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
pub use primitives::{icon, text};

// Negative compile contracts: removed deprecated surface is absent.
//
// Each removed method, type, and export must fail to resolve. If any of these
// lines compiles, the removal is incomplete and the build must fail.

/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Card::<()>::new(iced::widget::Space::new()).bordered();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Panel::<()>::new(iced::widget::Space::new()).padding(0);
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = SectionHeaderStatus::icon(IconRole::DialogInfo, theme::roles::ToneRole::Info, "text");
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = InputGroup::<()>::new(Input::new("Label", "")).leading_text("prefix");
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = InputGroup::<()>::new(Input::new("Label", "")).trailing_text("unit");
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Switch::new(false).label("Label");
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = SegmentedControl::<_, ()>::new("Mode", 1, []).flat();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::new("text");
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::status("text").size(ControlSize::Sm);
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::status("text").xs();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::status("text").sm();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::status("text").md();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = Badge::<()>::status("text").lg();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").value(iced::widget::text("custom"));
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").tone(theme::roles::ToneRole::Info);
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").neutral();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").accent();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").info();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").success();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").warning();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = MetadataItem::<()>::new("Label", "value").danger();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// let _ = ToolbarGroup::<()>::new().separator();
/// ```
///
/// ```compile_fail
/// use nive_ui::prelude::*;
/// use nive_workbench::status::StatusBar;
/// let _ = StatusBar::new().item(nive_workbench::status::StatusItem::text("Ready"));
/// ```
pub use containers::{
    ActionCard, Card, CardVariant, Panel, SectionHeader, SectionHeaderAction, SectionHeaderStatus,
    SelectableCard, SplitPane, SplitPaneConstraints,
};
pub use controls::{
    ActionGroup, Autocomplete, AutocompleteHighlight, AutocompleteResults, AutocompleteSuggestion,
    Button, ButtonIntent, ButtonVariant, Checkbox, CheckboxState, ColorInput, ColorPicker,
    ContentAction, Field, FieldControl, FieldError, FieldGroup, FieldGroupLayout, FieldHint,
    FieldLabel, FieldRequirement, FieldValidation, Input, InputGroup, InputGroupVariant, PathInput,
    RadioGroup, RadioGroupLayout, RadioOption, RgbHexColor, SegmentedControl,
    SegmentedControlVariant, SegmentedOption, Select, SelectOption, SelectableItem, Switch,
    TextInputAppearance,
};
pub use display::{
    reveal, row_height, scroll_offset_to, visible_index_of, AvatarClass, AvatarKind, AvatarSize,
    AvatarStatus, Badge, BadgeContent, BadgeKind, DataRow, EmptyState, InitialAvatar, KeyValueList,
    MetadataItem, MetadataTag, MetricCard, Tree, TreeChildren, TreeDrag, TreeDrop, TreeDropTarget,
    TreeEvent, TreeEventKind, TreeExpandBehavior, TreeItem, TreeItemDropEdge, TreeNode,
    TreePasteTarget, TreeState, TreeStateChange,
};
pub use feedback::{
    ErrorDetailsDialog, ErrorEmptyState, ErrorFeedback, ErrorFeedbackAction,
    ErrorFeedbackActionRow, ErrorFeedbackCommandRole, ErrorPresentation, ErrorStatusLine,
    InlineAlert, OperationActionGroup, OperationStatusLine, OperationStatusPresentation,
    ProgressBar, ResourceStatusLine, ResourceStatusPresentation, Skeleton, SkeletonCard,
    SkeletonControl, Spinner,
};
pub use navigation::{
    command_palette_filter, CommandPalette, CommandPaletteItem, Menu, MenuCheckbox, MenuCommand,
    MenuDismissPolicy, MenuRadioGroup, MenuRadioOption, MenuSubmenu, RailSide, TabBar,
    TabCloseRequest, TabCloseTrigger, TabDrop, TabDropTarget, TabItem, TabTearOff, Toolbar,
    ToolbarAction, ToolbarGroup, VerticalRail, VerticalRailBadge, VerticalRailItem,
};
pub use overlays::{
    AnnouncementPoliteness, Dialog, DialogAction, DialogActionFooter, DialogActionFooterError,
    DialogActionRole, DialogFooter, DialogHeader, DialogHost, DialogInitialFocus, DialogSize,
    DialogTerminalAction, Popover, PopoverCollision, PopoverFocusPolicy, PopoverInset,
    PopoverPlacement, PopoverWidth, ToastHost, ToastInsets, ToastPosition, ToastPresentation,
    ToastTone, Tooltip, TooltipPlacement, TooltipScope,
};
pub use primitives::{
    space, svg, ColorSwatch, Icon, IconGlyph, IconRole, IconSource, Separator, SeparatorExtent,
    SeparatorStrength, StatusIndicator, ToneDot,
};
pub use scrollable::overlay_scrollbar;
