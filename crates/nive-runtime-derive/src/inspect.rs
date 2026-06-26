use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::{format_ident, quote};
use syn::{parse_quote, Data, DeriveInput, Expr, Field, Fields, Index, Meta, Path, Token};

pub(crate) fn expand(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = match syn::parse(input) {
        Ok(ast) => ast,
        Err(e) => return e.to_compile_error().into(),
    };

    match expand_derive(ast) {
        Ok(output) => output.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_derive(ast: DeriveInput) -> syn::Result<TokenStream2> {
    let ident = &ast.ident;
    let (impl_generics, type_generics, where_clause) = ast.generics.split_for_impl();
    let runtime = runtime_path();

    let fields = match &ast.data {
        Data::Struct(s) => &s.fields,
        _ => {
            return Err(syn::Error::new_spanned(
                ident,
                "#[derive(Inspect)] is only supported on structs",
            ))
        }
    };

    let field_calls = field_inspect_calls(fields, &runtime)?;

    Ok(quote! {
        impl #impl_generics #runtime::__inspect::Inspect for #ident #type_generics #where_clause {
            fn inspect(
                &mut self,
                path: &mut #runtime::__inspect::InspectPath,
                sink: &mut dyn #runtime::__inspect::InspectSink,
            ) {
                #field_calls
            }
        }
    })
}

fn runtime_path() -> Path {
    if let Some(path) = dependency_path("nive-runtime") {
        return path;
    }
    if let Some(path) = dependency_path("nive") {
        return path;
    }
    parse_quote!(::nive_runtime)
}

fn dependency_path(package: &str) -> Option<Path> {
    match crate_name(package).ok()? {
        FoundCrate::Itself => Some(parse_quote!(crate)),
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            Some(parse_quote!(::#ident))
        }
    }
}

#[derive(Default)]
struct InspectAttrs {
    skip: bool,
    default: bool,
    sample: Option<Expr>,
    input: Option<Expr>,
}

fn inspect_attrs(field: &Field) -> syn::Result<InspectAttrs> {
    let mut parsed = InspectAttrs::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("inspect") {
            continue;
        }

        let metas =
            attr.parse_args_with(syn::punctuated::Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in metas {
            match meta {
                Meta::Path(path) if path.is_ident("skip") => parsed.skip = true,
                Meta::Path(path) if path.is_ident("default") => parsed.default = true,
                Meta::NameValue(nv) if nv.path.is_ident("sample") => parsed.sample = Some(nv.value),
                Meta::NameValue(nv) if nv.path.is_ident("input") => parsed.input = Some(nv.value),
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "unsupported #[inspect(...)] attribute; expected skip, default, sample = path, or input = path",
                    ));
                }
            }
        }
    }

    if parsed.skip && (parsed.default || parsed.sample.is_some() || parsed.input.is_some()) {
        return Err(syn::Error::new_spanned(
            field,
            "#[inspect(skip)] cannot be combined with simulator capability attributes",
        ));
    }
    if parsed.input.is_some() && (parsed.default || parsed.sample.is_some()) {
        return Err(syn::Error::new_spanned(
            field,
            "#[inspect(input = ...)] cannot be combined with resource capability attributes",
        ));
    }

    Ok(parsed)
}

fn field_inspect_calls(fields: &Fields, runtime: &Path) -> syn::Result<TokenStream2> {
    let mut calls = TokenStream2::new();

    for (idx, field) in fields.iter().enumerate() {
        let attrs = inspect_attrs(field)?;
        if attrs.skip {
            continue;
        }

        let accessor = match &field.ident {
            Some(ident) => quote!(self.#ident),
            None => {
                let index = Index::from(idx);
                quote!(self.#index)
            }
        };
        let field_name = match &field.ident {
            Some(ident) => ident.to_string(),
            None => idx.to_string(),
        };

        let inspect_call = if attrs.default || attrs.sample.is_some() {
            let default_call = attrs
                .default
                .then(|| quote!(simulator = simulator.with_default();));
            let sample_call = attrs
                .sample
                .as_ref()
                .map(|sample| quote!(simulator = simulator.with_sample(#sample);));
            quote! {
                let mut simulator = #runtime::__inspect::ResourceSimulator::new(&mut #accessor);
                #default_call
                #sample_call
                sink.register(&path.as_str(), &mut simulator);
            }
        } else if let Some(input) = attrs.input.as_ref() {
            quote! {
                let mut simulator = #runtime::__inspect::OperationSimulator::new(&mut #accessor)
                    .with_input(#input);
                sink.register(&path.as_str(), &mut simulator);
            }
        } else {
            quote! {
                #runtime::__inspect::Inspect::inspect(&mut #accessor, path, sink);
            }
        };

        let call = match &field.ident {
            Some(ident) => {
                quote! {
                    let _ = &self.#ident;
                    path.push(#field_name);
                    #inspect_call
                    path.pop();
                }
            }
            None => {
                let index = Index::from(idx);
                quote! {
                    let _ = &self.#index;
                    path.push(#field_name);
                    #inspect_call
                    path.pop();
                }
            }
        };

        calls.extend(call);
    }

    Ok(calls)
}
