//! Contract tests for the two-tier prelude (Spec 9).
//!
//! `nive::prelude::*` is the minimal template-stable surface; `nive::prelude::ui::*`
//! is the extended surface for app code that uses toasts, async state, dialogs,
//! theming, shortcuts, or window-handle types. The tests below only type-check
//! the relevant `use` forms — they don't run anything.

// Minimal tier: a counter-template-shaped `Application` must compile using
// only `nive::prelude::*`.
mod minimal_tier_counter {
    use nive::prelude::*;

    pub struct CounterApp {
        counter: i32,
    }

    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code)]
    pub enum CounterMessage {
        Increment,
        Decrement,
    }

    impl Application for CounterApp {
        type Message = CounterMessage;
        type Window = ();
        type Bootstrap = ();

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("counter")
        }

        fn init(
            _context: Context<'_, Self::Window>,
            _bootstrap: Self::Bootstrap,
        ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
            (Self { counter: 0 }, AppUpdate::none())
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            message: Self::Message,
        ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
            match message {
                CounterMessage::Increment => self.counter += 1,
                CounterMessage::Decrement => self.counter -= 1,
            }
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            ScreenView::new(text(self.counter.to_string()))
        }

        fn window_title<'a>(
            &'a self,
            _context: Context<'a, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> impl Into<std::borrow::Cow<'a, str>> + 'a {
            "Counter"
        }
    }

    pub(super) fn _assert_application_compiles_with_only_minimal_prelude() {
        fn _assert<A: Application>() {}
        _assert::<CounterApp>();
    }
}

// Extended tier: an app-shaped `Application` that uses `Resource`, `Toast`,
// `DialogRequest`, `OperationId`, `OperationDescriptor`, `OperationRegistry`,
// `ThemeBuilder`, `WindowRegistry`, `WindowHandle`, etc. must compile using
// only `nive::prelude::ui::*`.
mod extended_tier_dashboard {
    use nive::prelude::ui::*;

    pub struct DashboardApp {
        projects: Resource<Vec<String>>,
        #[allow(dead_code)]
        registry: OperationRegistry,
    }

    #[derive(Debug, Clone)]
    #[allow(dead_code)]
    pub enum DashboardMessage {
        Refresh,
        DismissDialog,
    }

    impl Application for DashboardApp {
        type Message = DashboardMessage;
        type Window = ();
        type Bootstrap = ();

        fn config() -> ApplicationConfig<Self::Window, Self::Bootstrap> {
            ApplicationConfig::new("dashboard")
        }

        fn init(
            _context: Context<'_, Self::Window>,
            _bootstrap: Self::Bootstrap,
        ) -> (Self, impl Into<AppUpdate<Self::Message, Self::Window>>) {
            (
                Self {
                    projects: Resource::idle(),
                    registry: OperationRegistry::new(),
                },
                AppUpdate::none(),
            )
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _window: Option<WindowContext<Self::Window>>,
            message: Self::Message,
        ) -> impl Into<AppUpdate<Self::Message, Self::Window>> {
            match message {
                DashboardMessage::Refresh => {
                    self.projects.begin();
                    AppUpdate::none().toast(Toast::info("Refreshing"))
                }
                DashboardMessage::DismissDialog => AppUpdate::none(),
            }
        }

        fn view(
            &self,
            _context: Context<'_, Self::Window>,
            _window: WindowContext<Self::Window>,
        ) -> ScreenView<'_, Self::Message> {
            ScreenView::new(text("dashboard"))
        }
    }

    pub(super) fn _assert_application_compiles_with_extended_prelude() {
        fn _assert<A: Application>() {}
        _assert::<DashboardApp>();
    }
}

#[test]
fn minimal_prelude_compiles_counter_template() {
    minimal_tier_counter::_assert_application_compiles_with_only_minimal_prelude();
}

#[test]
fn extended_prelude_compiles_dashboard_template() {
    extended_tier_dashboard::_assert_application_compiles_with_extended_prelude();
}
