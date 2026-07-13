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
        ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
            (Self { counter: 0 }, Effect::none())
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _message_context: MessageContext<Self::Window>,
            message: Self::Message,
        ) -> impl Into<Effect<Self::Message, Self::Window>> {
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
        ) -> (Self, impl Into<Effect<Self::Message, Self::Window>>) {
            (
                Self {
                    projects: Resource::idle(),
                    registry: OperationRegistry::new(),
                },
                Effect::none(),
            )
        }

        fn update(
            &mut self,
            _context: Context<'_, Self::Window>,
            _message_context: MessageContext<Self::Window>,
            message: Self::Message,
        ) -> impl Into<Effect<Self::Message, Self::Window>> {
            match message {
                DashboardMessage::Refresh => {
                    self.projects.begin();
                    Effect::none().with_toast(Toast::info("Refreshing"))
                }
                DashboardMessage::DismissDialog => Effect::none(),
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

        let _operation = Operation::<()>::idle();
        let _descriptor = OperationDescriptor::new("sync", "Sync");
        let _registry = OperationRegistry::new();
        let _dialog: Option<DialogRequest<'static, DashboardMessage>> = None;
        let _theme = ThemeBuilder::new("contract", ThemeMode::Light).build();
        let _shortcuts = ShortcutMap::<DashboardMessage>::new();
        let _windows = WindowRegistry::<()>::default();
        let _source = MessageSource::Action;
        let _event: RuntimeEvent<()> = RuntimeEvent::LastAppWindowClosed;
        let _effect: Effect<DashboardMessage, ()> = Effect::toast(Toast::info("Dashboard ready"));
        let _screen_effect: ScreenEffect<DashboardMessage, &'static str> =
            ScreenEffect::output("done").with_toast(Toast::success("Saved"));
        let _close_all: WindowCommand<()> = WindowCommand::CloseAllKind(());
        let _replace: WindowCommand<()> = WindowCommand::Replace {
            current: window::Id::unique(),
            next: (),
        };
        let _query_acceptor: for<'a> fn(WindowQuery<'a, ()>) = |_| {};

        #[cfg(feature = "file-picker")]
        {
            let _filter = FileFilter {
                name: "Rust",
                extensions: &["rs"],
            };
            let _pick = PickFileParams {
                filters: Vec::new(),
                start_dir: None,
            };
            let _save = SaveFileParams {
                filters: Vec::new(),
                start_dir: None,
                default_name: None,
            };
        }
    }
}

mod app_icon_contract {
    use nive::prelude::*;

    const SHIELD_GLYPH: IconGlyph = IconGlyph::new(
        br#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#,
        "custom:shield",
    );
    const APP_ICON_CATALOG: IconCatalog =
        IconCatalog::new(&[IconCatalogEntry::new(IconRole::WindowClose, SHIELD_GLYPH)]);

    #[derive(Debug, Clone, Copy)]
    enum IconSymbol {
        Shield,
    }

    impl IconSource for IconSymbol {
        fn svg_bytes(self) -> &'static [u8] {
            SHIELD_GLYPH.svg_bytes()
        }

        fn provider_slug(self) -> &'static str {
            match self {
                Self::Shield => "shield",
            }
        }
    }

    pub(super) fn _assert_app_icon_source_compiles_with_icon_widget() {
        let _role: IconRole = IconRole::WindowClose;
        let _role_element: Element<'static, ()> = Icon::role(_role).md().into();
        let _symbol_element: Element<'static, ()> = Icon::symbol(IconSymbol::Shield).md().into();
        let _glyph_element: Element<'static, ()> =
            Icon::glyph(IconGlyph::from_source(IconSymbol::Shield))
                .md()
                .into();

        let _theme = ThemeBuilder::new("contract", ThemeMode::Light)
            .icons(APP_ICON_CATALOG)
            .build();
        let _density_theme = ThemeBuilder::new("density", ThemeMode::Dark)
            .density(ThemeDensity::Compact)
            .build();
    }
}

mod workbench_chrome_prelude_contract {
    use nive::workbench::prelude::*;

    fn map_workbench_event(_: WorkbenchEvent<&'static str, &'static str, &'static str>) {}

    pub(super) fn _assert_workbench_prelude_exposes_chrome_size() {
        let state = WorkbenchLayoutState::<&str, &str>::default();
        let _shell = WorkbenchShell::new(state, map_workbench_event).chrome_size(ControlSize::Lg);
    }
}

mod umbrella_workbench_chrome_prelude_contract {
    use nive::prelude::*;

    fn map_workbench_event(_: WorkbenchEvent<&'static str, &'static str, &'static str>) {}

    pub(super) fn _assert_umbrella_prelude_exposes_chrome_size() {
        let state = WorkbenchLayoutState::<&str, &str>::default();
        let _shell = WorkbenchShell::new(state, map_workbench_event).chrome_size(ControlSize::Md);
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

#[test]
fn generated_app_icon_source_compiles_with_icon_widget() {
    app_icon_contract::_assert_app_icon_source_compiles_with_icon_widget();
}

#[test]
fn workbench_prelude_exposes_chrome_size() {
    workbench_chrome_prelude_contract::_assert_workbench_prelude_exposes_chrome_size();
}

#[test]
fn umbrella_prelude_exposes_chrome_size() {
    umbrella_workbench_chrome_prelude_contract::_assert_umbrella_prelude_exposes_chrome_size();
}
