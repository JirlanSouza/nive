use iced::{window, Task};

use crate::application::program::{
    CoreMessage, NiveMessage, ProbeCatalogEntry, Program, RuntimeMessage,
};
use crate::application::Application;
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsHostState, DevtoolsWindowSpec};
#[cfg(feature = "devtools")]
use crate::devtools::{DevtoolsPanelEffect, DevtoolsPanelMessage};

impl<A, P> Program<A, P>
where
    A: Application,
    P: ProbeCatalogEntry,
{
    #[cfg(feature = "devtools")]
    pub(super) fn initialize_devtools(&mut self) -> Task<RuntimeMessage<A, P>> {
        if self.devtools.is_none() {
            return Task::none();
        }

        let config = self.devtools.as_ref().unwrap().config;
        let panel = crate::devtools::DevtoolsPanelState::new().with_config(config);

        let (devtools_opt, app_opt) = (&mut self.devtools, &mut self.app);
        let devtools = devtools_opt.as_mut().unwrap();
        devtools.host = DevtoolsHostState::new(Some(panel));
        if let Some(app) = app_opt.as_mut() {
            let collect = devtools.collect;
            devtools.cached_snapshot = collect(app);
        }

        let start_open = {
            let devtools = self.devtools.as_mut().unwrap();
            let v = devtools.start_open;
            devtools.start_open = false;
            v
        };

        if start_open {
            self.open_devtools_window()
        } else {
            Task::none()
        }
    }

    #[cfg(not(feature = "devtools"))]
    pub(super) fn initialize_devtools(&mut self) -> Task<RuntimeMessage<A, P>> {
        Task::none()
    }

    #[cfg(feature = "devtools")]
    pub(super) fn open_devtools_window(&mut self) -> Task<RuntimeMessage<A, P>> {
        let Some(devtools) = self.devtools.as_mut() else {
            return Task::none();
        };
        if let Some(window_id) = devtools.window_id {
            return window::gain_focus(window_id);
        }

        let spec = DevtoolsWindowSpec::default().window_spec();
        let (window_id, open_task) = window::open(spec.settings(self.core.window_icon.clone()));
        devtools.window_id = Some(window_id);
        open_task.map(|id| NiveMessage::Core(CoreMessage::WindowOpened(id)))
    }

    #[cfg(feature = "devtools")]
    pub(super) fn is_devtools_window(&self, window_id: window::Id) -> bool {
        self.devtools
            .as_ref()
            .is_some_and(|devtools| devtools.window_id == Some(window_id))
    }

    #[cfg(not(feature = "devtools"))]
    pub(super) fn is_devtools_window(&self, _window_id: window::Id) -> bool {
        false
    }

    #[cfg(feature = "devtools")]
    pub(super) fn devtools_view(&self) -> nive_ui::Element<'_, RuntimeMessage<A, P>> {
        use crate::devtools::devtools_window;

        let Some(devtools) = self.devtools.as_ref() else {
            return iced::widget::text("").into();
        };
        let Some(panel) = devtools.host.panel() else {
            return iced::widget::text("").into();
        };

        devtools_window(panel, &devtools.cached_snapshot, NiveMessage::Devtools)
    }

    #[cfg(feature = "devtools")]
    pub(super) fn update_devtools(
        &mut self,
        message: DevtoolsPanelMessage,
    ) -> Task<RuntimeMessage<A, P>> {
        let effect = {
            let Some(devtools) = self.devtools.as_mut() else {
                return Task::none();
            };
            devtools.host.update(message)
        };
        let Some(DevtoolsPanelEffect::Simulate { path, action }) = effect else {
            return Task::none();
        };

        let is_resource = self
            .devtools
            .as_ref()
            .and_then(|d| d.cached_snapshot.entries.iter().find(|e| e.path == path))
            .map(|e| e.is_resource())
            .unwrap_or(true);

        let (devtools_opt, app_opt) = (&mut self.devtools, &mut self.app);
        let (Some(devtools), Some(app)) = (devtools_opt.as_mut(), app_opt.as_mut()) else {
            return Task::none();
        };
        let collect = devtools.collect;
        let apply = devtools.apply;
        let result = apply(app, &path, &action);
        devtools.cached_snapshot = collect(app);
        let new_entries = devtools.cached_snapshot.entries.clone();

        if let Some(panel) = devtools.host.panel_mut() {
            panel.entries = new_entries;
            panel.record_simulate_result(&path, is_resource, result);
        }

        Task::none()
    }
}
