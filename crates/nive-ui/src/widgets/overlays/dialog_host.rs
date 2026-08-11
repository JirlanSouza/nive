mod initial_focus;
mod widget;

pub use initial_focus::DialogInitialFocus;
pub use widget::DialogHost;

#[cfg(test)]
mod dialog_boundary_tests;
#[cfg(test)]
mod dialog_focus_tests;
#[cfg(test)]
mod dialog_identity_tests;
#[cfg(test)]
mod dialog_initial_focus_tests;
#[cfg(test)]
mod dialog_modal_activity_tests;
/// A real Dialog-owned nested `Popover`
/// (not a synthetic rectangle) hosted inside a `Dialog` body inside
/// `DialogHost`, exercised through the same `overlay::Nested`
/// infrastructure the popover-in-popover and Menu-submenu tests already
/// use — proving Dialog composes with the shared nested-overlay and focus
/// foundations rather than bypassing them.
#[cfg(test)]
mod nested_overlay_tests;
