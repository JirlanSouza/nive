use syn::{Attribute, Data, DeriveInput, Fields, Generics, Ident, Path, Type};

pub(crate) struct ParsedEnum {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) variants: Vec<ParsedVariant>,
}

pub(crate) struct ParsedVariant {
    pub(crate) ident: Ident,
}

pub(crate) struct ParsedField {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) kind: FieldKind,
}

pub(crate) enum FieldKind {
    Ignored,
    Async { fixtures: Path },
    Operation,
    Nested,
}

pub(crate) struct ParsedStruct {
    pub(crate) ident: Ident,
    pub(crate) generics: Generics,
    pub(crate) fields: StructFields,
}

pub(crate) enum StructFields {
    Unit,
    Named(Vec<ParsedField>),
    Tuple,
}

pub(crate) fn parse_enum(input: DeriveInput) -> syn::Result<ParsedEnum> {
    let Data::Enum(data) = input.data else {
        return Err(syn::Error::new_spanned(input.ident, "expected an enum"));
    };

    let variants = data
        .variants
        .into_iter()
        .map(|variant| {
            if !matches!(variant.fields, Fields::Unit) {
                return Err(syn::Error::new_spanned(
                    variant,
                    "probe catalog variants must be unit variants",
                ));
            }

            Ok(ParsedVariant {
                ident: variant.ident,
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    Ok(ParsedEnum {
        ident: input.ident,
        generics: input.generics,
        variants,
    })
}

pub(crate) fn parse_struct(input: DeriveInput) -> syn::Result<ParsedStruct> {
    let Data::Struct(data) = input.data else {
        return Err(syn::Error::new_spanned(input.ident, "expected a struct"));
    };

    let fields = match data.fields {
        Fields::Unit => StructFields::Unit,
        Fields::Unnamed(_) => StructFields::Tuple,
        Fields::Named(fields) => StructFields::Named(
            fields
                .named
                .into_iter()
                .map(|field| {
                    let ident = field.ident.ok_or_else(|| {
                        syn::Error::new_spanned(&field.ty, "expected named field")
                    })?;
                    let kind = parse_field_kind(&field.attrs, &field.ty)?;

                    Ok(ParsedField {
                        ident,
                        ty: field.ty,
                        kind,
                    })
                })
                .collect::<syn::Result<Vec<_>>>()?,
        ),
    };

    Ok(ParsedStruct {
        ident: input.ident,
        generics: input.generics,
        fields,
    })
}

fn parse_field_kind(attrs: &[Attribute], ty: &Type) -> syn::Result<FieldKind> {
    let mut nested = false;
    let mut fixtures = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("devtool")) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("nested") {
                if nested {
                    return Err(meta.error("duplicate `nested` option"));
                }
                nested = true;
                return Ok(());
            }

            if meta.path.is_ident("fixtures") {
                if fixtures.is_some() {
                    return Err(meta.error("duplicate `fixtures` option"));
                }
                fixtures = Some(meta.value()?.parse::<Path>()?);
                return Ok(());
            }

            Err(meta.error("unsupported devtool field option"))
        })?;
    }

    if nested && fixtures.is_some() {
        return Err(syn::Error::new_spanned(
            ty,
            "`nested` and `fixtures` cannot be combined",
        ));
    }

    if nested {
        return Ok(FieldKind::Nested);
    }

    if type_has_last_segment(ty, "AsyncState") {
        return fixtures
            .map(|fixtures| FieldKind::Async { fixtures })
            .ok_or_else(|| {
                syn::Error::new_spanned(
                    ty,
                    "AsyncState fields require `#[devtool(fixtures = path::to::provider)]`",
                )
            });
    }

    if let Some(fixtures) = fixtures {
        return Err(syn::Error::new_spanned(
            fixtures,
            "`fixtures` is only supported on AsyncState fields",
        ));
    }

    if type_has_last_segment(ty, "OperationState") {
        Ok(FieldKind::Operation)
    } else {
        Ok(FieldKind::Ignored)
    }
}

fn type_has_last_segment(ty: &Type, expected: &str) -> bool {
    let Type::Path(ty) = ty else {
        return false;
    };

    ty.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == expected)
}

pub(crate) fn devtools_path(attrs: &[Attribute], default: &str) -> syn::Result<Path> {
    let paths = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("devtools_path"))
        .map(|attr| {
            let path = attr.parse_args::<syn::LitStr>()?;
            path.parse::<Path>()
        })
        .collect::<syn::Result<Vec<_>>>()?;

    match paths.as_slice() {
        [path] => Ok(path.clone()),
        [path, ..] => Err(syn::Error::new_spanned(
            path,
            "duplicate `devtools_path` attribute",
        )),
        [] => syn::parse_str(default),
    }
}
