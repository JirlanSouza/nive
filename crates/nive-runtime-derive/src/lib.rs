//! Proc-macro support for the Nive runtime devtools layer.
//!
//! Provides `#[derive(Inspect)]` for recursive state inspection. All generated
//! code is gated behind the `devtools` feature: with the feature disabled the
//! macro expands to nothing, adding no runtime cost.

use proc_macro::TokenStream;

#[cfg(feature = "devtools")]
mod inspect;

/// Derives the `nive_runtime::Inspect` trait for a struct.
///
/// When the `devtools` feature is enabled, generates an `Inspect` implementation
/// that recurses into each field: it pushes the field name onto the `InspectPath`,
/// calls `Inspect::inspect` on the field, then pops the name. Fields annotated
/// with `#[inspect(skip)]` are excluded and their types are not required to
/// implement `Inspect`.
///
/// When the `devtools` feature is disabled, the macro expands to nothing.
///
/// # Example
///
/// ```ignore
/// #[derive(Inspect)]
/// struct AppState {
///     projects: Resource<Vec<Project>>,
///     #[inspect(skip)]
///     cache: SomeNonInspectType,
/// }
/// ```
#[proc_macro_derive(Inspect, attributes(inspect))]
pub fn derive_inspect(input: TokenStream) -> TokenStream {
    #[cfg(feature = "devtools")]
    {
        inspect::expand(input)
    }
    #[cfg(not(feature = "devtools"))]
    {
        let _ = input;
        TokenStream::new()
    }
}
