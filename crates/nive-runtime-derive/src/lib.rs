use proc_macro::TokenStream;
use syn::{DeriveInput, Result};

mod client;
mod naming;
mod operation;
mod parse;
mod probe;
mod state;

use client::expand_runtime_client;
use operation::expand_operation_context;
use parse::{parse_enum, parse_struct};
use probe::expand_probe_catalog;
use state::{expand_state_catalog, expand_state_host};

const DEFAULT_DEVTOOLS_PATH: &str = "nive_runtime::devtools";

fn extract_devtools_path(input: TokenStream) -> Result<(TokenStream, String)> {
    let ast: DeriveInput = syn::parse(input)?;
    let mut path = DEFAULT_DEVTOOLS_PATH.to_string();

    for attr in &ast.attrs {
        if attr.path().is_ident("devtools_path") {
            let lit_str: syn::LitStr = attr.parse_args()?;
            path = lit_str.value();
        }
    }

    let item_tokens = quote::quote! { #ast };
    Ok((item_tokens.into(), path))
}

#[proc_macro_derive(UiErrorProbeCatalog)]
pub fn derive_ui_error_probe_catalog(input: TokenStream) -> TokenStream {
    match parse_enum(input).map(expand_probe_catalog) {
        Ok(output) => output
            .parse()
            .unwrap_or_else(|_| compile_error("failed to emit probe catalog")),
        Err(error) => compile_error(&error),
    }
}

#[proc_macro_attribute]
pub fn runtime_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_runtime_client(attr, item)
}

#[proc_macro_derive(DevtoolStateCatalog, attributes(devtools_path))]
pub fn derive_devtool_state_catalog(input: TokenStream) -> TokenStream {
    let (filtered, devtools_path) = match extract_devtools_path(input) {
        Ok(result) => result,
        Err(err) => return err.to_compile_error().into(),
    };
    match parse_struct(filtered).map(|parsed| expand_state_catalog(parsed, &devtools_path)) {
        Ok(output) => output
            .parse()
            .unwrap_or_else(|_| compile_error("failed to emit state catalog")),
        Err(error) => compile_error(&error),
    }
}

#[proc_macro_derive(DevtoolStateHost, attributes(devtools_path))]
pub fn derive_devtool_state_host(input: TokenStream) -> TokenStream {
    let (filtered, devtools_path) = match extract_devtools_path(input) {
        Ok(result) => result,
        Err(err) => return err.to_compile_error().into(),
    };
    match parse_struct(filtered).and_then(|parsed| expand_state_host(parsed, &devtools_path)) {
        Ok(output) => output
            .parse()
            .unwrap_or_else(|_| compile_error("failed to emit state host")),
        Err(error) => compile_error(&error),
    }
}

#[proc_macro_derive(DevtoolOperationContext, attributes(devtools_path))]
pub fn derive_devtool_operation_context(input: TokenStream) -> TokenStream {
    let (filtered, devtools_path) = match extract_devtools_path(input) {
        Ok(result) => result,
        Err(err) => return err.to_compile_error().into(),
    };
    match parse_struct(filtered).map(|parsed| expand_operation_context(parsed, &devtools_path)) {
        Ok(output) => output
            .parse()
            .unwrap_or_else(|_| compile_error("failed to emit operation context")),
        Err(error) => compile_error(&error),
    }
}

fn compile_error(message: &str) -> TokenStream {
    format!("compile_error!({message:?});")
        .parse()
        .expect("compile_error should parse")
}
