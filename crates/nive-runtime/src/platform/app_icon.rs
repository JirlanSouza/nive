#[cfg(target_os = "macos")]
pub fn install(icon_png: &[u8]) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(main_thread) = MainThreadMarker::new() else {
        return;
    };

    let app = NSApplication::sharedApplication(main_thread);
    let data = NSData::with_bytes(icon_png);

    if let Some(app_icon) = NSImage::initWithData(NSImage::alloc(), &data) {
        unsafe {
            app.setApplicationIconImage(Some(&app_icon));
        }
    }
}

#[cfg(target_os = "linux")]
pub fn install(icon_png: &[u8]) {
    install_desktop_entry(icon_png, env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub fn install(_icon_png: &[u8]) {}

/// Embed the app icon into the Windows executable's resources at build time.
///
/// Call this from a user app's `build.rs`:
///
/// ```ignore
/// fn main() {
///     #[cfg(target_os = "windows")]
///     nive_runtime::platform::app_icon::install_app_icon_at_build(
///         "assets/icons/app.ico",
///     );
/// }
/// ```
///
/// The icon path is resolved relative to the crate's manifest directory.
/// If the file does not exist, a warning is emitted and the build continues
/// (the executable will use the default OS icon).
#[cfg(target_os = "windows")]
pub fn install_app_icon_at_build(icon_path: &str) {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR is set by cargo during build");
    let icon_path = std::path::Path::new(&manifest_dir).join(icon_path);

    if !icon_path.exists() {
        eprintln!(
            "cargo:warning=App icon not found at {}; the executable will use the default OS icon.",
            icon_path.display()
        );
        return;
    }

    let mut res = winres::WindowsResource::new();
    res.set_icon(icon_path.to_str().expect("icon path is not valid UTF-8"));
    res.compile().expect("Failed to compile Windows resources");
}

#[cfg(not(target_os = "windows"))]
pub fn install_app_icon_at_build(_icon_path: &str) {}

/// Returns the recommended `WM_CLASS` (X11) / `app_id` (Wayland) for the
/// given application id.
///
/// On Linux, winit sets the window's `WM_CLASS` from the window title by
/// default. The ideal behavior is to set it to the `ApplicationConfig::id()`
/// value so desktop environments can match the running window to the
/// installed `.desktop` entry.
///
/// **Limitation:** iced 0.14 does not expose winit's
/// `WindowBuilderExtUnix::with_name` through its public API. Until this is
/// upstreamed, the `.desktop` file's `Name` field provides partial mitigation
/// — most desktop environments use the `.desktop` entry for app association
/// rather than `WM_CLASS` lookup.
#[cfg(target_os = "linux")]
pub fn recommended_wm_class(app_id: &str) -> String {
    app_id.to_owned()
}

#[cfg(not(target_os = "linux"))]
pub fn recommended_wm_class(app_id: &str) -> String {
    app_id.to_owned()
}

#[cfg(target_os = "linux")]
fn xdg_data_home() -> std::path::PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME").expect("HOME is always set on Linux");
            std::path::PathBuf::from(home).join(".local/share")
        })
}

#[cfg(target_os = "linux")]
fn install_desktop_entry(icon_png: &[u8], app_id: &str, version: &str) {
    use std::fs;

    let data_home = xdg_data_home();
    let apps_dir = data_home.join("applications");
    let icons_dir = data_home.join("icons/hicolor/256x256/apps");

    if let Err(e) = fs::create_dir_all(&apps_dir) {
        eprintln!("nive: failed to create applications dir: {e}");
        return;
    }
    if let Err(e) = fs::create_dir_all(&icons_dir) {
        eprintln!("nive: failed to create icons dir: {e}");
        return;
    }

    let desktop_path = apps_dir.join(format!("{app_id}.desktop"));
    let icon_path = icons_dir.join(format!("{app_id}.png"));

    if desktop_path.exists() {
        if let Ok(content) = fs::read_to_string(&desktop_path) {
            if content.contains(version) {
                return;
            }
        }
    }

    let exe = std::env::current_exe().expect("current exe is available");
    let desktop_content = format!(
        "[Desktop Entry]\nType=Application\nName={app_id}\nExec={}\nIcon={app_id}\nTerminal=false\nCategories=Utility;\n# nive-version={version}\n",
        exe.display()
    );

    if let Err(e) = fs::write(&desktop_path, &desktop_content) {
        eprintln!("nive: failed to write .desktop file: {e}");
        return;
    }
    if let Err(e) = fs::write(&icon_path, icon_png) {
        eprintln!("nive: failed to write icon PNG: {e}");
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn install_app_icon_at_build_accepts_path() {
        super::install_app_icon_at_build("nonexistent.ico");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_install_is_idempotent() {
        use std::fs;
        let dir = tempfile::tempdir().unwrap();
        let data_home = dir.path();
        let apps_dir = data_home.join("applications");
        let icons_dir = data_home.join("icons/hicolor/256x256/apps");
        fs::create_dir_all(&apps_dir).unwrap();
        fs::create_dir_all(&icons_dir).unwrap();

        let icon = b"fake-png-data";
        super::install_desktop_entry(icon, "test-app", "0.1.0");

        let desktop = fs::read_to_string(apps_dir.join("test-app.desktop")).unwrap();
        assert!(desktop.contains("0.1.0"));
        assert!(icons_dir.join("test-app.png").exists());

        let meta_before = fs::metadata(icons_dir.join("test-app.png")).unwrap();
        super::install_desktop_entry(icon, "test-app", "0.1.0");
        let meta_after = fs::metadata(icons_dir.join("test-app.png")).unwrap();
        assert_eq!(
            meta_before.modified().unwrap(),
            meta_after.modified().unwrap()
        );

        super::install_desktop_entry(icon, "test-app", "0.2.0");
        let desktop = fs::read_to_string(apps_dir.join("test-app.desktop")).unwrap();
        assert!(desktop.contains("0.2.0"));
        assert!(!desktop.contains("0.1.0"));
    }
}
