use std::borrow::Cow;
use std::rc::Rc;

use nive_ui::widgets::{command_palette_filter, command_palette_view, CommandPaletteRow};
use nive_ui::Element;

/// Command palette host state owned by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandPaletteState {
    /// Whether the palette is open.
    pub open: bool,
    /// Current query text.
    pub query: String,
    /// Highlighted row index in the filtered list.
    pub highlighted: Option<usize>,
}

/// Workbench command metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkbenchCommand<'a, CommandId> {
    /// Stable command id.
    pub id: CommandId,
    /// Visible label.
    pub label: Cow<'a, str>,
    /// Optional description.
    pub description: Option<Cow<'a, str>>,
    /// Optional shortcut label.
    pub shortcut_label: Option<Cow<'a, str>>,
    /// Whether the command can be submitted.
    pub enabled: bool,
}

/// Command palette host events.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum WorkbenchCommandPaletteEvent<CommandId> {
    /// Query changed.
    QueryChanged(String),
    /// Highlighted row changed.
    Highlighted(Option<usize>),
    /// Command was submitted.
    Submitted(CommandId),
    /// Palette was dismissed.
    Dismissed,
}

/// Command palette host view helper.
pub struct WorkbenchCommandPalette<'a, CommandId, Message> {
    state: &'a CommandPaletteState,
    commands: &'a [WorkbenchCommand<'a, CommandId>],
    on_event: Rc<dyn Fn(WorkbenchCommandPaletteEvent<CommandId>) -> Message + 'a>,
}

impl Default for CommandPaletteState {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandPaletteState {
    /// Builds a closed command palette state.
    pub fn new() -> Self {
        Self {
            open: false,
            query: String::new(),
            highlighted: None,
        }
    }

    /// Opens the palette.
    pub fn open(&mut self) {
        self.open = true;
        self.highlighted = Some(0);
    }

    /// Closes the palette.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
        self.highlighted = None;
    }

    /// Updates query text and resets highlight.
    pub fn set_query(&mut self, query: impl Into<String>) {
        self.query = query.into();
        self.highlighted = Some(0);
    }

    /// Moves the highlighted row within the current result length.
    pub fn move_highlight(&mut self, delta: isize, row_count: usize) {
        if row_count == 0 {
            self.highlighted = None;
            return;
        }

        let current = self.highlighted.unwrap_or(0) as isize;
        let next = (current + delta).rem_euclid(row_count as isize);
        self.highlighted = Some(next as usize);
    }

    /// Returns a submit event for the highlighted command.
    pub fn submit<CommandId: Clone>(
        &self,
        filtered: &[WorkbenchCommand<'_, CommandId>],
    ) -> Option<WorkbenchCommandPaletteEvent<CommandId>> {
        let index = self.highlighted?;
        let command = filtered.get(index)?;
        command
            .enabled
            .then(|| WorkbenchCommandPaletteEvent::Submitted(command.id.clone()))
    }

    /// Returns the dismiss event.
    pub const fn dismiss<CommandId>(&self) -> WorkbenchCommandPaletteEvent<CommandId> {
        WorkbenchCommandPaletteEvent::Dismissed
    }
}

impl<'a, CommandId> WorkbenchCommand<'a, CommandId> {
    /// Builds a command.
    pub fn new(id: CommandId, label: impl Into<Cow<'a, str>>) -> Self {
        Self {
            id,
            label: label.into(),
            description: None,
            shortcut_label: None,
            enabled: true,
        }
    }

    /// Sets description.
    pub fn description(mut self, description: impl Into<Cow<'a, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets shortcut label.
    pub fn shortcut_label(mut self, shortcut_label: impl Into<Cow<'a, str>>) -> Self {
        self.shortcut_label = Some(shortcut_label.into());
        self
    }

    /// Sets disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.enabled = !disabled;
        self
    }
}

impl<'a, CommandId, Message> WorkbenchCommandPalette<'a, CommandId, Message>
where
    CommandId: Clone + 'a,
    Message: Clone + 'a,
{
    /// Builds a command palette host.
    pub fn new(
        state: &'a CommandPaletteState,
        commands: &'a [WorkbenchCommand<'a, CommandId>],
        on_event: impl Fn(WorkbenchCommandPaletteEvent<CommandId>) -> Message + 'a,
    ) -> Self {
        Self {
            state,
            commands,
            on_event: Rc::new(on_event),
        }
    }

    /// Renders the command palette content.
    pub fn view(self) -> Element<'a, Message> {
        let rows = command_rows(self.commands, self.on_event.as_ref());
        let visible = command_palette_filter(&self.state.query, &rows);
        let filtered: Vec<CommandPaletteRow<'a, Message>> =
            visible.iter().map(|index| rows[*index].clone()).collect();
        let on_query_change = {
            let on_event = self.on_event.clone();
            move |query| on_event(WorkbenchCommandPaletteEvent::QueryChanged(query))
        };
        let submit = filtered
            .get(self.state.highlighted.unwrap_or(0))
            .and_then(|row| row.activated())
            .cloned();

        command_palette_view(
            "Search commands",
            &self.state.query,
            filtered,
            self.state.highlighted,
            on_query_change,
            submit,
        )
    }
}

fn command_rows<'a, CommandId, Message>(
    commands: &'a [WorkbenchCommand<'a, CommandId>],
    on_event: &dyn Fn(WorkbenchCommandPaletteEvent<CommandId>) -> Message,
) -> Vec<CommandPaletteRow<'a, Message>>
where
    CommandId: Clone + 'a,
    Message: Clone + 'a,
{
    commands
        .iter()
        .map(|command| {
            let mut row = CommandPaletteRow::new(
                command.label.as_ref(),
                command.label.as_ref(),
                on_event(WorkbenchCommandPaletteEvent::Submitted(command.id.clone())),
            );
            if let Some(description) = command.description.as_deref() {
                row = row.description(description);
            }
            if let Some(shortcut_label) = command.shortcut_label.clone() {
                row = row.shortcut_label(shortcut_label);
            }
            row.disabled(!command.enabled)
        })
        .collect()
}

#[cfg(test)]
mod tests;
