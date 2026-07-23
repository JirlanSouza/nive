use super::footer_widget::action_button;
use super::{
    DialogAction, DialogActionFooter, DialogActionFooterError, DialogActionFooterWidget,
    DialogActionRole, DialogTerminalAction, TerminalActionMarker,
};
use crate::Element;

impl<'a, Message> DialogActionFooter<'a, Message>
where
    Message: Clone + 'a,
{
    pub fn new(terminal: DialogTerminalAction<'a, Message>) -> Self {
        Self {
            status: None,
            preceding: Vec::new(),
            terminal: terminal.0,
        }
    }

    pub fn with_one(
        preceding: DialogAction<'a, Message>,
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Self {
        Self {
            status: None,
            preceding: vec![preceding],
            terminal: terminal.0,
        }
    }

    pub fn with_two(
        preceding: [DialogAction<'a, Message>; 2],
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Self {
        Self {
            status: None,
            preceding: preceding.into(),
            terminal: terminal.0,
        }
    }

    /// Builds a footer from a dynamically sized preceding-action collection.
    /// Returns a typed error instead of truncating when the collection is
    /// invalid.
    pub fn try_from_parts(
        preceding: Vec<DialogAction<'a, Message>>,
        terminal: DialogTerminalAction<'a, Message>,
    ) -> Result<Self, DialogActionFooterError> {
        if preceding.len() > 2 {
            return Err(DialogActionFooterError::TooManyPrecedingActions(
                preceding.len(),
            ));
        }

        for action in &preceding {
            if !action.is_safe_preceding_role() {
                return Err(DialogActionFooterError::InvalidPrecedingRole(action.role));
            }
        }

        Ok(Self {
            status: None,
            preceding,
            terminal: terminal.0,
        })
    }

    pub fn status(mut self, status: impl Into<Element<'a, Message>>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// The terminal action's message, published on unconsumed non-repeated
    /// Enter when the terminal action is `Primary` and enabled. Never
    /// exposed for a `Destructive` terminal action.
    pub(crate) fn enter_default_message(&self) -> Option<&Message> {
        if self.terminal.disabled || self.terminal.role != DialogActionRole::Primary {
            return None;
        }

        Some(&self.terminal.message)
    }

    #[cfg(test)]
    pub(super) fn all_actions(&self) -> impl Iterator<Item = &DialogAction<'a, Message>> {
        self.preceding.iter().chain(std::iter::once(&self.terminal))
    }

    pub(super) fn into_element(self) -> Element<'a, Message> {
        let enter_default = self.enter_default_message().cloned();
        let mut actions: Vec<Element<'a, Message>> =
            self.preceding.into_iter().map(action_button).collect();
        // Tagged so a modal host's initial-focus resolution can recognize
        // and skip the terminal action (`DialogInitialFocus::First` must
        // never land on it, Primary or Destructive) without needing to know
        // Dialog's internal anatomy.
        actions.push(TerminalActionMarker::wrap(action_button(self.terminal)));

        DialogActionFooterWidget {
            status: self.status,
            actions,
            enter_default,
        }
        .into()
    }
}
