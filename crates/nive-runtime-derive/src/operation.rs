use proc_macro2::TokenStream;
use quote::quote;
use syn::{LitStr, Path};

use crate::parse::{ParsedStruct, StructFields};

pub(crate) fn expand_operation_context(
    parsed: ParsedStruct,
    devtools_path: &Path,
) -> syn::Result<TokenStream> {
    let ParsedStruct {
        ident,
        generics,
        fields,
    } = parsed;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    match fields {
        StructFields::Unit => Ok(quote! {
            impl #impl_generics #devtools_path::DevtoolOperationContext
                for #ident #type_generics #where_clause
            {
                fn devtool_fields() -> ::std::vec::Vec<#devtools_path::DevtoolFieldSchema> {
                    ::std::vec::Vec::new()
                }

                fn devtool_build(
                    _inputs: &#devtools_path::DevtoolInputValues<'_>,
                ) -> ::std::result::Result<Self, ::std::string::String> {
                    Ok(Self)
                }
            }
        }),
        StructFields::Named(fields) => {
            let schemas = fields.iter().map(|field| {
                let ty = &field.ty;
                let name = LitStr::new(&field.ident.to_string(), field.ident.span());
                quote! {
                    <#ty as #devtools_path::DevtoolInputField>::devtool_schema(#name)
                }
            });
            let build = fields.iter().map(|field| {
                let ident = &field.ident;
                let ty = &field.ty;
                let name = LitStr::new(&ident.to_string(), ident.span());
                quote! {
                    #ident: <#ty as #devtools_path::DevtoolInputField>::devtool_build(
                        #name,
                        inputs,
                    )?
                }
            });

            Ok(quote! {
                impl #impl_generics #devtools_path::DevtoolOperationContext
                    for #ident #type_generics #where_clause
                {
                    fn devtool_fields() -> ::std::vec::Vec<#devtools_path::DevtoolFieldSchema> {
                        ::std::vec![#(#schemas),*]
                    }

                    fn devtool_build(
                        inputs: &#devtools_path::DevtoolInputValues<'_>,
                    ) -> ::std::result::Result<Self, ::std::string::String> {
                        Ok(Self { #(#build),* })
                    }
                }
            })
        }
        StructFields::Tuple => Err(syn::Error::new_spanned(
            ident,
            "DevtoolOperationContext supports unit or named-field structs only",
        )),
    }
}
