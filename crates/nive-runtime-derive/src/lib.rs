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

#[proc_macro_attribute]
pub fn runtime_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_runtime_client(attr, item)
}

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
