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

#[cfg(not(target_os = "macos"))]
pub fn install(_icon_png: &[u8]) {}
