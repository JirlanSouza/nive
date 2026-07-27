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
                PanelActionId::ClearSelection => self.clear_selection(),
            },
            WorkbenchEvent::Panel(WorkbenchPanelEvent::CloseRequested { region, .. }) => {
                self.layout.collapse_region(region);
            }
            WorkbenchEvent::Layout(_) | WorkbenchEvent::Document(_) | WorkbenchEvent::Panel(_) => {}
            _ => {}
        }
    }

    pub(super) fn close_palette(&mut self) {
        self.palette_open = false;
        self.palette_query.clear();
    }

    pub(super) fn run_command(&mut self, command: AppCommand) {
        self.close_palette();
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
        }
    }

    pub(super) fn select(&mut self, selection: Selection) {
        self.selected = selection;
        self.inspector_loading_until = Some(self.model.tick + 2);
    }

    pub(super) fn clear_selection(&mut self) {
        self.selected = Selection::None;
        self.inspector_loading_until = None;
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

        app.palette_open = true;
        app.palette_query = query.into();

        assert!(app.palette_open);
        assert_eq!(app.palette_query, query);
        assert_eq!(app.documents, documents);

        app.close_palette();
        assert!(!app.palette_open);
        assert!(app.palette_query.is_empty());
        assert_eq!(app.documents, documents);
    }

    #[test]
    fn clearing_selection_removes_pending_inspector_loading_state() {
        let mut app = WorkbenchMonitor::seeded();

        app.select(Selection::Host("data-01"));
        assert!(app.inspector_loading_until.is_some());

        app.clear_selection();

        assert_eq!(app.selected, Selection::None);
        assert_eq!(app.inspector_loading_until, None);
    }

    #[test]
    fn inspector_clear_action_does_not_clear_the_events_panel() {
        let mut app = WorkbenchMonitor::seeded();
        let events = app.model.events.clone();

        app.apply_workbench_event(WorkbenchEvent::Panel(WorkbenchPanelEvent::Action {
            region: WorkbenchRegion::Right,
            panel_id: "inspector",
            action_id: PanelActionId::ClearSelection,
        }));

        assert_eq!(app.selected, Selection::None);
        assert_eq!(app.inspector_loading_until, None);
        assert_eq!(app.model.events, events);
    }
}
