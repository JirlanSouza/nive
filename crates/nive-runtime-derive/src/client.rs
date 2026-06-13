use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Ident, ImplItem, ItemImpl, LitStr, Token, Type,
};

use crate::probe::probe_meta_from_client_method;

pub(crate) fn expand_runtime_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as RuntimeClientArgs);
    let mut item_impl = parse_macro_input!(item as ItemImpl);
    let client_type = match client_type_name(&item_impl.self_ty) {
        Ok(client_type) => client_type,
        Err(error) => return error.to_compile_error().into(),
    };
    let mut matched = Vec::new();

    for item in &mut item_impl.items {
        let ImplItem::Fn(method) = item else {
            continue;
        };

        let method_name = method.sig.ident.to_string();
        let Some(probe) = args.probe_for(&method_name) else {
            continue;
        };
        matched.push(method_name.clone());

        let key = probe
            .key
            .clone()
            .unwrap_or_else(|| default_probe_key(&client_type, &method_name));
        let key = LitStr::new(&key, method.sig.ident.span());
        let block = &method.block;

        method.block = parse_quote!({
            const CLIENT_PROBE_KEY: &str = #key;
            #block
        });
    }

    if let Some(unmatched) = args
        .probes
        .iter()
        .find(|probe| !matched.contains(&probe.method))
    {
        return syn::Error::new_spanned(
            &item_impl.self_ty,
            format!(
                "runtime_client probe method `{}` was not found",
                unmatched.method
            ),
        )
        .to_compile_error()
        .into();
    }

    if !args.probes.is_empty() {
        item_impl
            .items
            .insert(0, client_probe_const(&client_type, &args));
    }

    quote!(#item_impl).into()
}

#[derive(Default)]
struct RuntimeClientArgs {
    probes: Vec<ClientProbeSpec>,
}

struct ClientProbeSpec {
    method: String,
    key: Option<String>,
    short_key: Option<String>,
    summary: Option<String>,
    scope: Option<String>,
}

impl RuntimeClientArgs {
    fn probe_for(&self, method: &str) -> Option<&ClientProbeSpec> {
        self.probes.iter().find(|probe| probe.method == method)
    }
}

impl Parse for RuntimeClientArgs {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut probes = Vec::new();

        while !input.is_empty() {
            let kind: Ident = input.parse()?;
            if kind != "probe" {
                return Err(syn::Error::new(kind.span(), "expected probe(...)"));
            }

            let content;
            parenthesized!(content in input);
            probes.push(parse_probe_spec(&content)?);

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { probes })
    }
}

fn parse_probe_spec(input: ParseStream<'_>) -> syn::Result<ClientProbeSpec> {
    let method: Ident = input.parse()?;
    let mut spec = ClientProbeSpec {
        method: method.to_string(),
        key: None,
        short_key: None,
        summary: None,
        scope: None,
    };

    while input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
        if input.is_empty() {
            break;
        }

        let option: Ident = input.parse()?;
        if option != "key" && option != "short_key" && option != "summary" && option != "scope" {
            return Err(syn::Error::new(option.span(), "unsupported probe option"));
        }

        input.parse::<Token![=]>()?;
        let value = input.parse::<LitStr>()?.value();
        match option.to_string().as_str() {
            "key" => spec.key = Some(value),
            "short_key" => spec.short_key = Some(value),
            "summary" => spec.summary = Some(value),
            "scope" => spec.scope = Some(value),
            _ => unreachable!("unsupported probe option should be rejected"),
        }
    }

    Ok(spec)
}

fn client_type_name(self_ty: &Type) -> syn::Result<String> {
    let Type::Path(path) = self_ty else {
        return Err(syn::Error::new_spanned(
            self_ty,
            "runtime_client requires an impl for a concrete client type",
        ));
    };

    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
        .ok_or_else(|| {
            syn::Error::new_spanned(
                self_ty,
                "runtime_client requires an impl for a concrete client type",
            )
        })
}

fn client_probe_const(client_type: &str, args: &RuntimeClientArgs) -> ImplItem {
    let metas = args.probes.iter().map(|probe| {
        let meta = probe_meta_from_client_method(client_type, &probe.method, probe.key.as_deref());
        let key = LitStr::new(&meta.key, proc_macro2::Span::call_site());
        let short_key = LitStr::new(
            probe.short_key.as_deref().unwrap_or(&meta.short_key),
            proc_macro2::Span::call_site(),
        );
        let summary = LitStr::new(
            probe.summary.as_deref().unwrap_or(&meta.summary),
            proc_macro2::Span::call_site(),
        );
        let kind = probe
            .scope
            .as_deref()
            .map(|scope| format!("Custom({scope:?})"))
            .unwrap_or(meta.kind);
        let scope: proc_macro2::TokenStream =
            kind.parse().expect("valid ProbeErrorScope expression");

        quote! {
            nive_runtime::ProbeMeta::new(
                #key,
                #short_key,
                #summary,
                nive_runtime::ProbeErrorScope::#scope,
            )
        }
    });

    ImplItem::Const(parse_quote! {
        pub(crate) const DEV_PROBES: &'static [nive_runtime::ProbeMeta] = &[#(#metas),*];
    })
}

fn default_probe_key(client_type: &str, method_name: &str) -> String {
    probe_meta_from_client_method(client_type, method_name, None).key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_scope_strips_suffix_and_snake_cases_type() {
        assert_eq!(
            crate::probe::client_scope("ProjectCatalogClient"),
            "project_catalog"
        );
        assert_eq!(crate::probe::client_scope("TagClient"), "tag");
    }

    #[test]
    fn default_probe_key_combines_client_scope_and_method() {
        assert_eq!(
            default_probe_key("ProjectCatalogClient", "get_summary"),
            "project_catalog.get_summary"
        );
    }

    #[test]
    fn client_probe_meta_preserves_key_override() {
        let meta = probe_meta_from_client_method(
            "ProjectCatalogClient",
            "get_summary",
            Some("project_catalog.summary"),
        );

        assert_eq!(meta.key, "project_catalog.summary");
        assert_eq!(meta.short_key, "summary");
        assert_eq!(meta.summary, "Couldn't run project catalog summary");
        assert_eq!(meta.kind, r#"Custom("project_catalog")"#);
    }

    #[test]
    fn generated_client_probe_meta_qualifies_custom_scope() {
        let args = RuntimeClientArgs {
            probes: vec![ClientProbeSpec {
                method: "list".to_string(),
                key: None,
                short_key: None,
                summary: None,
                scope: None,
            }],
        };
        let item = client_probe_const("ProjectCatalogClient", &args);

        let generated = quote::quote!(#item);

        assert!(generated
            .to_string()
            .contains("nive_runtime :: ProbeErrorScope :: Custom"));
    }

    #[test]
    fn client_probe_meta_uses_generic_defaults() {
        let meta = probe_meta_from_client_method("ProjectCatalogClient", "get_summary", None);

        assert_eq!(meta.key, "project_catalog.get_summary");
        assert_eq!(meta.short_key, "summary");
        assert_eq!(meta.summary, "Couldn't run project catalog summary");
        assert_eq!(meta.kind, r#"Custom("project_catalog")"#);
    }

    #[test]
    fn parses_explicit_probe_metadata() {
        let args = syn::parse_str::<RuntimeClientArgs>(
            r#"probe(
                list,
                key = "catalog.list",
                short_key = "list_items",
                summary = "Couldn't load items",
                scope = "catalog"
            )"#,
        )
        .expect("runtime client args should parse");
        let probe = &args.probes[0];

        assert_eq!(probe.method, "list");
        assert_eq!(probe.key.as_deref(), Some("catalog.list"));
        assert_eq!(probe.short_key.as_deref(), Some("list_items"));
        assert_eq!(probe.summary.as_deref(), Some("Couldn't load items"));
        assert_eq!(probe.scope.as_deref(), Some("catalog"));
    }
}
