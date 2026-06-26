//! Contract test for the `nive` umbrella feature passthrough.
//!
//! Verifies that enabling `devtools` and `file-picker` features on the
//! `nive` umbrella crate exposes the expected surfaces.
//!
//! These tests are gated on the features they test — they only run when
//! the corresponding feature is enabled:
//! ```bash
//! cargo test --package nive --test umbrella_features --features devtools,file-picker
//! ```

#[cfg(feature = "devtools")]
#[test]
fn devtools_feature_exposes_devtools_surface() {
    fn _assert<T>() {}
    // `nive::devtools::DevtoolsConfig` should be available when `devtools`
    // feature is enabled.
    _assert::<nive::devtools::DevtoolsConfig>();
}

#[cfg(feature = "devtools")]
#[test]
fn devtools_feature_supports_umbrella_inspect_derive() {
    fn sample_projects() -> Vec<String> {
        vec!["sample".to_string()]
    }

    fn sample_input() -> String {
        "save".to_string()
    }

    #[derive(nive::Inspect)]
    struct State {
        #[inspect(sample = sample_projects)]
        projects: nive::Resource<Vec<String>>,
        #[inspect(input = sample_input)]
        save: nive::Operation<String>,
    }

    fn _assert<T: nive::runtime::Inspect>() {}
    _assert::<State>();
}

#[cfg(not(feature = "devtools"))]
#[test]
fn inspect_derive_is_available_without_devtools() {
    struct NotInspect;

    #[derive(nive::Inspect)]
    struct State {
        #[inspect(default)]
        #[allow(dead_code)]
        field: NotInspect,
    }

    let _ = std::mem::size_of::<State>();
}

#[cfg(feature = "file-picker")]
#[test]
fn file_picker_feature_exposes_file_picker_surface() {
    fn _assert<T>() {}
    // `nive::FileFilter`, `nive::PickFileParams`, `nive::SaveFileParams`
    // should be available when `file-picker` feature is enabled.
    _assert::<nive::FileFilter>();
    _assert::<nive::PickFileParams>();
    _assert::<nive::SaveFileParams>();
}

#[cfg(all(feature = "devtools", feature = "file-picker"))]
#[test]
fn both_features_together_compile() {
    fn _assert<T>() {}
    // Both features enabled simultaneously should compile without conflicts.
    _assert::<nive::devtools::DevtoolsConfig>();
    _assert::<nive::FileFilter>();
    _assert::<nive::PickFileParams>();
    _assert::<nive::SaveFileParams>();
}
