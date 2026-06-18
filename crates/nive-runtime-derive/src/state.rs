use proc_macro2::TokenStream;
use quote::quote;
use syn::{LitStr, Path};

use crate::{
    naming::label_from_snake,
    parse::{FieldKind, ParsedField, ParsedStruct, StructFields},
};

pub(crate) fn expand_state_catalog(parsed: ParsedStruct, devtools_path: &Path) -> TokenStream {
    let ParsedStruct {
        ident,
        generics,
        fields,
    } = parsed;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    let StructFields::Named(fields) = fields else {
        return quote! {
            impl #impl_generics #devtools_path::DevtoolStateCatalog
                for #ident #type_generics #where_clause
            {}
        };
    };

    let collect_fields = fields
        .iter()
        .filter(|field| !matches!(field.kind, FieldKind::Ignored))
        .map(|field| state_collect_expr(field, devtools_path));
    let apply_fields = fields
        .iter()
        .filter(|field| !matches!(field.kind, FieldKind::Ignored))
        .map(|field| state_apply_expr(field, devtools_path));

    quote! {
        impl #impl_generics #devtools_path::DevtoolStateCatalog
            for #ident #type_generics #where_clause
        {
            fn devtool_collect(
                &self,
                scope: &str,
                snapshot: &mut #devtools_path::DevtoolStateSnapshot,
            ) {
                #(#collect_fields)*
            }

            fn devtool_apply(
                &mut self,
                scope: &str,
                command: &#devtools_path::DevtoolCommand,
            ) -> #devtools_path::DevtoolCommandResult {
                #(#apply_fields)*
                #devtools_path::DevtoolCommandResult::not_handled()
            }
        }
    }
}

pub(crate) fn expand_state_host(
    parsed: ParsedStruct,
    devtools_path: &Path,
) -> syn::Result<TokenStream> {
    let ParsedStruct {
        ident,
        generics,
        fields,
    } = parsed;
    let StructFields::Named(fields) = fields else {
        return Err(syn::Error::new_spanned(
            ident,
            "DevtoolStateHost requires a named-field struct",
        ));
    };
    let field = fields
        .iter()
        .find(|field| field.ident == "state")
        .ok_or_else(|| {
            syn::Error::new_spanned(&ident, "DevtoolStateHost expected a field named `state`")
        })?;
    let state_ty = &field.ty;
    let state_field = &field.ident;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #devtools_path::DevtoolStateHost
            for #ident #type_generics #where_clause
        {
            type State = #state_ty;

            fn devtool_state(&self) -> &Self::State {
                &self.#state_field
            }

            fn devtool_state_mut(&mut self) -> &mut Self::State {
                &mut self.#state_field
            }
        }
    })
}

fn state_collect_expr(field: &ParsedField, devtools_path: &Path) -> TokenStream {
    let name = &field.ident;
    let field_name = LitStr::new(&name.to_string(), name.span());
    let label = LitStr::new(&label_from_snake(&name.to_string()), name.span());
    let path = quote!(#devtools_path::join_path(scope, #field_name));

    match &field.kind {
        FieldKind::Async { fixtures } => quote! {
            let path = #path;
            #devtools_path::collect_async_state_field(
                &self.#name,
                &path,
                #label,
                snapshot,
                #fixtures(&path),
            );
        },
        FieldKind::Operation => quote! {
            let path = #path;
            #devtools_path::collect_operation_state_field(
                &self.#name,
                &path,
                #label,
                snapshot,
            );
        },
        FieldKind::Nested => quote! {
            let path = #path;
            #devtools_path::DevtoolStateCatalog::devtool_collect(
                &self.#name,
                &path,
                snapshot,
            );
        },
        FieldKind::Ignored => TokenStream::new(),
    }
}

fn state_apply_expr(field: &ParsedField, devtools_path: &Path) -> TokenStream {
    let name = &field.ident;
    let field_name = LitStr::new(&name.to_string(), name.span());
    let path = quote!(#devtools_path::join_path(scope, #field_name));

    let apply = match &field.kind {
        FieldKind::Async { fixtures } => quote! {
            #devtools_path::apply_async_state_field(
                &mut self.#name,
                &path,
                command,
                || #fixtures(&path),
            )
        },
        FieldKind::Operation => quote! {
            #devtools_path::apply_operation_state_field(&mut self.#name, &path, command)
        },
        FieldKind::Nested => quote! {
            #devtools_path::DevtoolStateCatalog::devtool_apply(
                &mut self.#name,
                &path,
                command,
            )
        },
        FieldKind::Ignored => return TokenStream::new(),
    };

    quote! {
        let path = #path;
        let result = #apply;
        if result.handled() {
            return result;
        }
    }
}
