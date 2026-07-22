use std::sync::Mutex;

#[cfg(feature = "devtools")]
use crate::application::program::DevtoolsRuntime;
use crate::application::program::{ProbeCatalogEntry, Program, RuntimeTask};
use crate::application::{Application, ApplicationConfig, Error, Result, WindowRegistration};
use crate::WindowSpec;

/// Auto-registers a single [`WindowSpec::app()`] when `A::Window = ()` and
/// `ApplicationConfig` has no explicit `.window(...)` calls. Apps with custom
/// `type Window` enums must declare windows in `Application::config()`.
///
/// Detection uses `TypeId` so that the explicit `()` defaulting flow stays
/// generic-friendly without unstable specialization.
pub(super) fn auto_register_default_window<A>(
    config: &mut ApplicationConfig<A::Window, A::Bootstrap>,
) where
    A: Application,
{
    use std::any::TypeId;

    if !config.windows.is_empty() || !config.initial_windows.is_empty() {
        return;
    }

    if TypeId::of::<A::Window>() != TypeId::of::<()>() {
        return;
    }

    // SAFETY: We only enter this branch when `A::Window = ()`, verified above by
    // `TypeId` equality. `()` is a zero-sized type, so `transmute_copy` reads no
    // bytes and the resulting value is the canonical `()`.
    let default_kind = unsafe { std::mem::transmute_copy::<(), A::Window>(&()) };
    config.windows.push(WindowRegistration {
        kind: default_kind,
        spec: WindowSpec::app(),
    });
    config.initial_windows.push(default_kind);
}

#[cfg(feature = "devtools")]
pub(super) fn run_inner<A, P>(devtools: Option<DevtoolsRuntime<A>>) -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let config = A::config();
    let fonts = config.fonts.clone();
    let default_font = config.default_font;
    let (program, initial_task) = Program::<A, P>::new(config, devtools)?;
    run_program::<A, P>(program, initial_task, fonts, default_font)
}

#[cfg(not(feature = "devtools"))]
pub(super) fn run_inner<A, P>() -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let config = A::config();
    let fonts = config.fonts.clone();
    let default_font = config.default_font;
    let (program, initial_task) = Program::<A, P>::new(config)?;
    run_program::<A, P>(program, initial_task, fonts, default_font)
}

pub(super) fn run_program<A, P>(
    program: Program<A, P>,
    initial_task: RuntimeTask<A, P>,
    fonts: Vec<std::borrow::Cow<'static, [u8]>>,
    default_font: iced::Font,
) -> Result
where
    A: Application,
    P: ProbeCatalogEntry,
{
    let boot = Mutex::new(Some((program, initial_task)));

    let mut daemon = iced::daemon(
        move || {
            boot.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take()
                .unwrap_or_else(|| panic!("Nive program booted more than once"))
        },
        Program::<A, P>::update,
        Program::<A, P>::view,
    )
    .title(Program::<A, P>::title)
    .theme(Program::<A, P>::theme)
    .subscription(Program::<A, P>::subscription)
    .default_font(default_font);

    for font in nive_ui::fonts::bundled() {
        daemon = daemon.font(*font);
    }

    for font in fonts {
        daemon = daemon.font(font);
    }

    daemon.run().map_err(Error::from)
}
