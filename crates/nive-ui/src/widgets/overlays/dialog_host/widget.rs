use super::initial_focus::{dialog_initial_focus_fn, DialogInitialFocus};
use crate::widgets::overlays::modal_host::{ModalAlignment, ModalHost};
use crate::Element;

/// Modal dialog composition that preserves the prior logical navigation anchor.
///
/// While a dialog is open, base content keeps drawing but is externally
/// inert: it receives no pointer, touch, keyboard, wheel, mouse-interaction,
/// or external widget/focus operations. Only a private invoker-validity
/// probe may still inspect it. Every event the Dialog and its nested
/// overlays leave unhandled is captured, so nothing clicks or types through
/// to base content.
///
/// The backdrop resolves from the theme's `SurfaceRole::Scrim`; there is no
/// raw alpha override. Backdrop dismissal recognizes only a primary
/// (left) mouse press or touch press with a concrete position outside the
/// Dialog frame — never a secondary/middle button, and never
/// `mouse::Cursor::Unavailable`/`Levitating` (including when a nested
/// overlay masks the position). Escape recognizes only a non-repeated
/// keypress, after Dialog-owned nested overlays and descendants have had
/// first priority. An absent route is still captured (never clicks/types
/// through) but publishes no message; programmatic removal of the dialog
/// never publishes a dismissal message either.
///
/// When a dialog closes, a still-valid target captured before opening is
/// restored as an inactive anchor: not actively or visibly focused, and
/// never overwriting a newer programmatic target set outside the dialog.
/// [`dialog_id`](Self::dialog_id) names the current declarative session so a
/// rebuild with the same (or absent) id continues it without recapturing or
/// re-resolving initial focus, while a changed id replaces the workflow step
/// and re-runs initial focus without recapturing the invoker or publishing
/// dismissal.
///
/// Canonical composition draws base content, then the Scrim, then the
/// Dialog frame, then any Dialog-owned nested overlay (Select/Popover/Menu)
/// on top — shadow prominence is independent from this event/paint order. A
/// window hosts at most one canonical modal session; a later
/// [`dialog`](Self::dialog) call replaces rather than stacks. Manually
/// nesting more than one `DialogHost` is unsupported and not detected.
///
/// `DialogHost` is a thin centered typed layer over a private shared
/// modal-hosting kernel (also consumed by `CommandPalette`); this hosting
/// contract is unchanged by that extraction.
///
/// Low-level backdrop-alpha customization and hosting internals are
/// intentionally unavailable to application code:
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::DialogHost;
/// let host: DialogHost<'_, ()> = DialogHost::new(iced::widget::text("Base"));
/// let _ = host.backdrop_alpha(0.5);
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::modal_host::ModalHost;
/// ```
///
/// ```compile_fail
/// use nive_ui::widgets::overlays::dialog_host::initial_focus::FocusFirstSafeTarget;
/// ```
pub struct DialogHost<'a, Message> {
    content: Element<'a, Message>,
    dialog: Option<DialogContent<'a, Message>>,
}

struct DialogContent<'a, Message> {
    content: Element<'a, Message>,
    on_backdrop: Option<Message>,
    on_escape: Option<Message>,
    initial_focus: DialogInitialFocus,
    id: Option<iced::widget::Id>,
}

impl<'a, Message> DialogHost<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(content: impl Into<Element<'a, Message>>) -> Self {
        Self {
            content: content.into(),
            dialog: None,
        }
    }

    pub fn dialog(
        mut self,
        content: impl Into<Element<'a, Message>>,
        on_backdrop: Option<Message>,
        on_escape: Option<Message>,
        initial_focus: DialogInitialFocus,
    ) -> Self {
        self.dialog = Some(DialogContent {
            content: content.into(),
            on_backdrop,
            on_escape,
            initial_focus,
            id: None,
        });
        self
    }

    /// Sets the current dialog's stable declarative session identity.
    /// Rebuilding with the same (or no call at all, leaving it absent)
    /// identity continues the current modal session. A changed identity
    /// replaces the workflow step without recapturing the invoker.
    pub fn dialog_id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        if let Some(dialog) = &mut self.dialog {
            dialog.id = Some(id.into());
        }
        self
    }
}

impl<'a, Message> From<DialogHost<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(host: DialogHost<'a, Message>) -> Self {
        let mut modal_host = ModalHost::new(host.content);

        if let Some(dialog) = host.dialog {
            let initial_focus = dialog_initial_focus_fn(dialog.initial_focus);
            modal_host = modal_host.modal(
                dialog.content,
                dialog.on_backdrop,
                dialog.on_escape,
                initial_focus,
                ModalAlignment::Centered,
            );
            if let Some(id) = dialog.id {
                modal_host = modal_host.modal_id(id);
            }
        }

        modal_host.into()
    }
}
