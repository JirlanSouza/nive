use nive_ui::prelude::*;

#[test]
fn feedback_presentation_contracts_are_reexported_from_nive_core() {
    use nive_ui::widgets::overlays::{ToastPresentation, ToastTone};
    use nive_ui::widgets::{
        ErrorPresentation, OperationStatusPresentation, ResourceStatusPresentation,
    };

    struct NoError;

    impl nive_core::ErrorPresentation for NoError {
        fn summary(&self) -> &str {
            "summary"
        }

        fn detail(&self) -> &str {
            "detail"
        }
    }

    struct NoResource;

    impl nive_core::ResourceStatusPresentation for NoResource {
        fn is_refreshing(&self) -> bool {
            false
        }

        fn has_value(&self) -> bool {
            false
        }

        fn error(&self) -> Option<&dyn nive_core::ErrorPresentation> {
            None
        }
    }

    struct NoOperation;

    impl nive_core::OperationStatusPresentation for NoOperation {
        fn is_running(&self) -> bool {
            false
        }

        fn error(&self) -> Option<&dyn nive_core::ErrorPresentation> {
            None
        }
    }

    struct NoToast;

    impl nive_core::ToastPresentation for NoToast {
        type Id = u64;

        fn id(&self) -> u64 {
            0
        }

        fn title(&self) -> &str {
            "title"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> nive_core::ToastTone {
            nive_core::ToastTone::Info
        }
    }

    // Each assertion only compiles if `nive_ui`'s facade name is the exact
    // same trait/type as `nive_core`'s, not a duplicate local definition.
    fn assert_error<T: ErrorPresentation>() {}
    fn assert_resource_status<T: ResourceStatusPresentation>() {}
    fn assert_operation_status<T: OperationStatusPresentation>() {}
    fn assert_toast<T: ToastPresentation>() {}

    assert_error::<NoError>();
    assert_resource_status::<NoResource>();
    assert_operation_status::<NoOperation>();
    assert_toast::<NoToast>();
    let _: ToastTone = nive_core::ToastTone::Success;
}

#[test]
fn toast_host_public_builder_surface_composes_from_the_prelude() {
    struct FakeToast;

    impl nive_core::ToastPresentation for FakeToast {
        type Id = u64;

        fn id(&self) -> u64 {
            0
        }

        fn title(&self) -> &str {
            "title"
        }

        fn body(&self) -> Option<&str> {
            None
        }

        fn tone(&self) -> nive_core::ToastTone {
            nive_core::ToastTone::Info
        }
    }

    // Exercises the full canonical-host chain a runtime integration drives:
    // position, safe insets, hover pause, focus-within pause, dismiss/action
    // extraction. Only compiles if this builder surface stays public and
    // ergonomic; a breaking signature change here fails at compile time.
    let _: Element<'_, &'static str> = ToastHost::new(text("content"))
        .position(ToastPosition::TopStart)
        .safe_insets(ToastInsets {
            top: 8.0,
            ..ToastInsets::NONE
        })
        .on_hover("pause", "resume")
        .on_focus_within("focus-enter", "focus-exit")
        .toasts(
            std::iter::once(&FakeToast),
            |_id: u64| "dismiss",
            |_toast: &FakeToast| None,
        )
        .into();
}
