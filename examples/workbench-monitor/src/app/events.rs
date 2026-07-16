use nive::prelude::*;

use super::{AppCommand, DocumentId, PanelActionId, Selection, WorkbenchMonitor, WorkbenchMsg};

impl WorkbenchMonitor {
    pub(super) fn apply_workbench_event(&mut self, event: WorkbenchMsg) {
        event.apply_to(&mut self.layout);

        match event {
            WorkbenchEvent::Document(WorkbenchDocumentEvent::CloseRequested { id, .. }) => {
                self.close_document(id);
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::Reorder { dragged, target }) => {
                self.reorder_document(dragged, target);
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::TearOffRequested {
                dragged, ..
            }) => {
                self.model
                    .events
                    .push(format!("Tear-off requested for {:?}", dragged));
            }
            WorkbenchEvent::Document(WorkbenchDocumentEvent::ContextRequested { id, .. }) => {
                self.model
                    .events
                    .push(format!("Document context requested for {:?}", id));
            }
            WorkbenchEvent::Panel(WorkbenchPanelEvent::Action { action_id, .. }) => match action_id
            {
                PanelActionId::Refresh => self.model.events.push("Panel refresh requested".into()),
                PanelActionId::RunHealth => self.model.run_health_check(),
                PanelActionId::Clear => self.model.events.clear(),
            },
            WorkbenchEvent::Panel(WorkbenchPanelEvent::CloseRequested { region, .. }) => {
                self.layout.collapse_region(region);
            }
            WorkbenchEvent::Layout(_) | WorkbenchEvent::Document(_) | WorkbenchEvent::Panel(_) => {}
            _ => {}
        }
    }

    pub(super) fn apply_palette_event(&mut self, event: WorkbenchCommandPaletteEvent<AppCommand>) {
        match event {
            WorkbenchCommandPaletteEvent::QueryChanged(query) => self.palette.set_query(query),
            WorkbenchCommandPaletteEvent::Highlighted(index) => self.palette.highlighted = index,
            WorkbenchCommandPaletteEvent::Submitted(command) => {
                self.palette.close();
                self.run_command(command);
            }
            WorkbenchCommandPaletteEvent::Dismissed => self.palette.close(),
            _ => {}
        }
    }

    pub(super) fn run_command(&mut self, command: AppCommand) {
        match command {
            AppCommand::OpenApi => self.open_document(DocumentId::Service("api")),
            AppCommand::OpenBilling => self.open_document(DocumentId::Service("billing")),
            AppCommand::AcknowledgeFirstAlert => {
                let first_alert = self.model.active_alerts().next().map(|alert| alert.id);
                if let Some(id) = first_alert {
                    self.model.acknowledge_alert(id);
                }
            }
            AppCommand::RunHealthCheck => self.model.run_health_check(),
            AppCommand::SwitchEnvironment => self.model.toggle_environment(),
            AppCommand::ToggleLeft => self.toggle_region(WorkbenchRegion::Left),
            AppCommand::ToggleBottom => self.toggle_region(WorkbenchRegion::Bottom),
            AppCommand::ToggleTheme => {
                self.theme = match self.theme {
                    ThemePreference::Dark => ThemePreference::Light,
                    _ => ThemePreference::Dark,
                };
            }
        }
    }

    pub(super) fn select(&mut self, selection: Selection) {
        self.selected = selection;
        self.inspector_loading_until = Some(self.model.tick + 2);
    }

    pub(super) fn open_document(&mut self, id: DocumentId) {
        if !self.documents.contains(&id) {
            self.documents.push(id);
        }
        self.layout.set_active_document(Some(id));
    }

    fn close_document(&mut self, id: DocumentId) {
        self.documents.retain(|document| *document != id);
        if self.layout.active_document() == Some(&id) {
            self.layout
                .set_active_document(self.documents.first().copied());
        }
    }

    fn reorder_document(
        &mut self,
        dragged: Vec<DocumentId>,
        target: WorkbenchDocumentDropTarget<DocumentId>,
    ) {
        let Some(document) = dragged.first().copied() else {
            return;
        };

        self.documents.retain(|existing| *existing != document);
        let index = match target {
            WorkbenchDocumentDropTarget::Before(target) => self
                .documents
                .iter()
                .position(|existing| *existing == target)
                .unwrap_or(0),
            WorkbenchDocumentDropTarget::After(target) => self
                .documents
                .iter()
                .position(|existing| *existing == target)
                .map(|index| index + 1)
                .unwrap_or(self.documents.len()),
            WorkbenchDocumentDropTarget::Unknown => self.documents.len(),
            _ => self.documents.len(),
        };

        self.documents
            .insert(index.min(self.documents.len()), document);
    }

    fn toggle_region(&mut self, region: WorkbenchRegion) {
        if self.layout.is_collapsed(region) {
            self.layout.restore_region(region);
        } else {
            self.layout.collapse_region(region);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrated_palette_preserves_long_query_and_shell_state_until_dismissal() {
        let mut app = WorkbenchMonitor::seeded();
        let documents = app.documents.clone();
        let query = "open the production billing service with a deliberately long query";

        app.palette.open();
        app.apply_palette_event(WorkbenchCommandPaletteEvent::QueryChanged(query.into()));

        assert!(app.palette.open);
        assert_eq!(app.palette.query, query);
        assert_eq!(app.palette.highlighted, Some(0));
        assert_eq!(app.documents, documents);

        app.apply_palette_event(WorkbenchCommandPaletteEvent::Dismissed);
        assert!(!app.palette.open);
        assert!(app.palette.query.is_empty());
        assert_eq!(app.documents, documents);
    }
}
