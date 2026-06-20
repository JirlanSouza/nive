//! Proc-macro support for the Nive runtime devtools layer.
//!
//! `nive-runtime-derive` provides the derive and attribute macros used by
//! `nive-runtime`'s optional `devtools` feature. End users normally depend on
//! `nive-runtime` (which re-exports these macros when `devtools` is enabled);
//! this crate is published separately for transparency.
//!
//! # Macros
//!
//! - [`Devtools`] — implements the `nive_runtime::devtools::Devtools` marker.
//! - [`UiErrorProbeCatalog`] — generates a probe catalog from an error enum.
//! - [`runtime_client`] — generates dev probe metadata and key injection for a
//!   client struct.
//! - [`DevtoolStateCatalog`] — collects devtool state fields on a state struct.
//! - [`DevtoolStateHost`] — marks the root state snapshot host.
//! - [`DevtoolOperationContext`] — generates an operation input schema/builder.
//!
//! # Attributes
//!
//! - `#[devtool(fixtures = "path")]` — names a fixture-provider function for a
//!   state field (used with `DevtoolStateCatalog`).
//! - `#[devtool(nested)]` — marks a field whose devtool state is nested.
//! - `#[devtools_path("path::to::devtools")]` — overrides the default
//!   `nive_runtime::devtools` target path for `DevtoolStateCatalog`,
//!   `DevtoolStateHost`, and `DevtoolOperationContext`.
//!
//! # Status
//!
//! Part of Nive **v0.1.0**, a beta release. Generated APIs may change before
//! 1.0. See `nive-runtime/docs/devtools.md` for usage.

#![warn(missing_docs)]

use proc_macro::TokenStream;
use syn::DeriveInput;

mod client;
mod naming;
mod operation;
mod parse;
mod probe;
mod state;

use client::expand_runtime_client;
use operation::expand_operation_context;
use parse::{devtools_path, parse_enum, parse_struct};
use probe::expand_probe_catalog;
use state::{expand_state_catalog, expand_state_host};

const DEFAULT_DEVTOOLS_PATH: &str = "nive_runtime::devtools";

/// Derives the `nive_runtime::devtools::Devtools` marker trait for a type.
///
/// This is a no-op implementation (empty `impl` block) that marks a type as a
/// devtools-capable application. Apply it to your app's state or root message
/// type so the runtime's devtools host can accept it.
///
/// ```ignore
/// #[derive(Devtools)]
/// struct MyApp;
/// ```
#[proc_macro_derive(Devtools)]
pub fn derive_devtools(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = match syn::parse(input) {
        Ok(ast) => ast,
        Err(error) => return error.to_compile_error().into(),
    };
    let ident = ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();

    quote::quote! {
        impl #impl_generics nive_runtime::devtools::Devtools
            for #ident #type_generics #where_clause
        {
        }
    }
    .into()
}

/// Derives a probe catalog from a fieldless error enum.
///
/// Each variant becomes a `ProbeCatalogEntry` keyed by the variant name
/// (snake-cased), with a human-readable summary derived from the variant name.
/// The generated catalog lets the devtools panel inject failures or delays for
/// individual error cases.
///
/// Only fieldless (`unit`) enum variants are supported; variants with fields
/// produce a compile error.
///
/// ```ignore
/// #[derive(UiErrorProbeCatalog)]
/// enum AppError {
///     ProjectNotFound,
///     TagInvalid,
/// }
/// ```
#[proc_macro_derive(UiErrorProbeCatalog)]
pub fn derive_ui_error_probe_catalog(input: TokenStream) -> TokenStream {
    match syn::parse(input)
        .and_then(parse_enum)
        .map(expand_probe_catalog)
    {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Attribute macro that generates dev probe metadata for a client struct.
///
/// Annotate a client `impl` block to generate the static `DEV_PROBES` metadata,
/// probe-key injection on client methods, and explicit app-owned probe labels
/// and scopes. Probe summaries and keys are derived from explicit attributes so
/// no product semantics are hard-coded by the macro.
///
/// ```ignore
/// #[runtime_client]
/// impl RecordCatalogClient {
///     // probe metadata is derived from declared probes
/// }
/// ```
#[proc_macro_attribute]
pub fn runtime_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_runtime_client(attr, item)
}

/// Derives the `DevtoolStateCatalog` trait for a state struct.
///
/// Collects the devtool state fields of a struct (fields holding
/// `AsyncState<T>` or `OperationState<C>`) and generates the
/// `DevtoolStateCatalog` implementation that the devtools panel uses to
/// enumerate, read, and apply fixtures to those fields.
///
/// # Supported attributes
///
/// - `#[devtool(fixtures = "path::to_fn")]` — names a fixture-provider
///   function supplying devtools values for the field.
/// - `#[devtool(nested)]` — marks a field whose devtool state is itself a
///   `DevtoolStateCatalog` and should be flattened into the parent catalog.
/// - `#[devtools_path("path::to::devtools")]` — overrides the default
///   `nive_runtime::devtools` target path.
///
/// ```ignore
/// #[derive(DevtoolStateCatalog)]
/// struct ScreenState {
///     #[devtool(fixtures = project_fixtures)]
///     projects: AsyncState<Vec<Project>>,
/// }
/// ```
#[proc_macro_derive(DevtoolStateCatalog, attributes(devtool, devtools_path))]
pub fn derive_devtool_state_catalog(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = match syn::parse(input) {
        Ok(ast) => ast,
        Err(error) => return error.to_compile_error().into(),
    };
    let path = match devtools_path(&ast.attrs, DEFAULT_DEVTOOLS_PATH) {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };

    match parse_struct(ast).map(|parsed| expand_state_catalog(parsed, &path)) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derives the `DevtoolStateHost` trait for the root state snapshot host.
///
/// Applied to the top-level app state struct, this collects every nested
/// `DevtoolStateCatalog` field and generates the [`DevtoolStateHost`]
/// implementation the devtools panel uses to snapshot and restore the full
/// application state tree.
///
/// # Supported attributes
///
/// - `#[devtools_path("path::to::devtools")]` — overrides the default
///   `nive_runtime::devtools` target path.
///
/// ```ignore
/// #[derive(DevtoolStateHost)]
/// struct AppState {
///     welcome: WelcomeState,
///     projects: ProjectsState,
/// }
/// ```
#[proc_macro_derive(DevtoolStateHost, attributes(devtools_path))]
pub fn derive_devtool_state_host(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = match syn::parse(input) {
        Ok(ast) => ast,
        Err(error) => return error.to_compile_error().into(),
    };
    let path = match devtools_path(&ast.attrs, DEFAULT_DEVTOOLS_PATH) {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };

    match parse_struct(ast).and_then(|parsed| expand_state_host(parsed, &path)) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

/// Derives an operation input schema and builder for a command struct.
///
/// Generates the [`DevtoolOperationContext`] implementation that the devtools
/// panel uses to render an input form for an operation, validate its fields,
/// and build the typed command. Field placeholders and labels are derived from
/// the field names.
///
/// # Supported attributes
///
/// - `#[devtools_path("path::to::devtools")]` — overrides the default
///   `nive_runtime::devtools` target path.
///
/// ```ignore
/// #[derive(DevtoolOperationContext)]
/// struct UpdateLabel {
///     label_id: String,
///     color: String,
/// }
/// ```
#[proc_macro_derive(DevtoolOperationContext, attributes(devtools_path))]
pub fn derive_devtool_operation_context(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = match syn::parse(input) {
        Ok(ast) => ast,
        Err(error) => return error.to_compile_error().into(),
    };
    let path = match devtools_path(&ast.attrs, DEFAULT_DEVTOOLS_PATH) {
        Ok(path) => path,
        Err(error) => return error.to_compile_error().into(),
    };

    match parse_struct(ast).and_then(|parsed| expand_operation_context(parsed, &path)) {
        Ok(output) => output.into(),
        Err(error) => error.to_compile_error().into(),
    }
}
