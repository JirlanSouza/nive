use proc_macro::{Delimiter, TokenStream, TokenTree};

#[derive(Debug)]
pub(crate) struct ParsedEnum {
    pub(crate) name: String,
    pub(crate) variants: Vec<ParsedVariant>,
}

#[derive(Debug)]
pub(crate) struct ParsedVariant {
    pub(crate) name: String,
}

#[derive(Debug)]
pub(crate) struct ParsedField {
    pub(crate) name: String,
    pub(crate) ty: String,
}

#[derive(Debug)]
pub(crate) struct ParsedStruct {
    pub(crate) name: String,
    pub(crate) fields: StructFields,
}

#[derive(Debug)]
pub(crate) enum StructFields {
    Unit,
    Named(Vec<ParsedField>),
    Tuple,
}

pub(crate) fn parse_enum(input: TokenStream) -> Result<ParsedEnum, String> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let enum_index = tokens
        .iter()
        .position(|token| matches!(token, TokenTree::Ident(ident) if ident.to_string() == "enum"))
        .ok_or_else(|| "expected an enum".to_string())?;

    let name = tokens
        .iter()
        .skip(enum_index + 1)
        .find_map(|token| match token {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .ok_or_else(|| "expected enum name".to_string())?;

    let body = tokens
        .iter()
        .skip(enum_index + 1)
        .find_map(|token| match token {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                Some(group.stream())
            }
            _ => None,
        })
        .ok_or_else(|| "expected enum body".to_string())?;

    Ok(ParsedEnum {
        name,
        variants: parse_variants(body)?,
    })
}

pub(crate) fn parse_struct(input: TokenStream) -> Result<ParsedStruct, String> {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let struct_index = tokens
        .iter()
        .position(|token| matches!(token, TokenTree::Ident(ident) if ident.to_string() == "struct"))
        .ok_or_else(|| "expected a struct".to_string())?;

    let name = tokens
        .iter()
        .skip(struct_index + 1)
        .find_map(|token| match token {
            TokenTree::Ident(ident) => Some(ident.to_string()),
            _ => None,
        })
        .ok_or_else(|| "expected struct name".to_string())?;

    let fields = tokens
        .iter()
        .skip(struct_index + 1)
        .find_map(|token| match token {
            TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {
                Some(parse_named_fields(group.stream()).map(StructFields::Named))
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {
                Some(Ok(StructFields::Tuple))
            }
            _ => None,
        })
        .transpose()?
        .unwrap_or(StructFields::Unit);

    Ok(ParsedStruct { name, fields })
}

fn parse_variants(body: TokenStream) -> Result<Vec<ParsedVariant>, String> {
    split_top_level_commas(body.into_iter().collect())
        .into_iter()
        .filter(|tokens| !tokens.is_empty())
        .map(parse_variant)
        .collect()
}

fn parse_variant(tokens: Vec<TokenTree>) -> Result<ParsedVariant, String> {
    let name_index = tokens
        .iter()
        .position(|token| matches!(token, TokenTree::Ident(_)))
        .ok_or_else(|| "expected variant name".to_string())?;
    let name = match &tokens[name_index] {
        TokenTree::Ident(ident) => ident.to_string(),
        _ => unreachable!(),
    };

    Ok(ParsedVariant { name })
}

fn parse_named_fields(stream: TokenStream) -> Result<Vec<ParsedField>, String> {
    split_top_level_commas(stream.into_iter().collect())
        .into_iter()
        .filter(|tokens| !tokens.is_empty())
        .map(|tokens| {
            let colon_index = tokens
                .iter()
                .position(
                    |token| matches!(token, TokenTree::Punct(punct) if punct.as_char() == ':'),
                )
                .ok_or_else(|| "expected ':' in named field".to_string())?;
            let name = tokens[..colon_index]
                .iter()
                .rev()
                .find_map(|token| match token {
                    TokenTree::Ident(ident) => Some(ident.to_string()),
                    _ => None,
                })
                .ok_or_else(|| "expected named field ident".to_string())?;
            let ty = tokens_to_string(&tokens[colon_index + 1..]);

            Ok(ParsedField { name, ty })
        })
        .collect()
}

fn split_top_level_commas(tokens: Vec<TokenTree>) -> Vec<Vec<TokenTree>> {
    let mut parts = Vec::new();
    let mut current = Vec::new();
    let mut angle_depth = 0i32;

    for token in tokens {
        match &token {
            TokenTree::Punct(punct) if punct.as_char() == '<' => {
                angle_depth += 1;
                current.push(token);
            }
            TokenTree::Punct(punct) if punct.as_char() == '>' && angle_depth > 0 => {
                angle_depth -= 1;
                current.push(token);
            }
            TokenTree::Punct(punct) if punct.as_char() == ',' && angle_depth == 0 => {
                parts.push(current);
                current = Vec::new();
            }
            _ => current.push(token),
        }
    }

    if !current.is_empty() {
        parts.push(current);
    }

    parts
}

fn tokens_to_string(tokens: &[TokenTree]) -> String {
    tokens
        .iter()
        .map(TokenTree::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}
