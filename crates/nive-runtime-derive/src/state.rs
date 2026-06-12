use crate::{
    naming::{is_nested_state_type, label_from_snake, normalize_type},
    parse::{ParsedField, ParsedStruct, StructFields},
};

pub(crate) fn expand_state_catalog(parsed: ParsedStruct, devtools_path: &str) -> String {
    let StructFields::Named(fields) = &parsed.fields else {
        return format!(
            r#"
impl {devtools_path}::DevtoolStateCatalog for {name} {{}}
"#,
            devtools_path = devtools_path,
            name = parsed.name,
        );
    };

    let collect_fields = fields
        .iter()
        .filter_map(state_field_kind)
        .map(|(field, kind)| state_collect_expr(field, kind, devtools_path))
        .collect::<Vec<_>>()
        .join("");
    let apply_fields = fields
        .iter()
        .filter_map(state_field_kind)
        .map(|(field, kind)| state_apply_expr(field, kind, devtools_path))
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"
impl {devtools_path}::DevtoolStateCatalog for {name} {{
    fn devtool_collect(&self, scope: &str, snapshot: &mut {devtools_path}::DevtoolStateSnapshot) {{
        {collect_fields}
    }}

    fn devtool_apply(&mut self, scope: &str, command: &{devtools_path}::DevtoolCommand) -> {devtools_path}::DevtoolCommandResult {{
        {apply_fields}
        {devtools_path}::DevtoolCommandResult::not_handled()
    }}
}}
"#,
        devtools_path = devtools_path,
        name = parsed.name,
        collect_fields = collect_fields,
        apply_fields = apply_fields,
    )
}

pub(crate) fn expand_state_host(
    parsed: ParsedStruct,
    devtools_path: &str,
) -> Result<String, String> {
    let StructFields::Named(fields) = &parsed.fields else {
        return Err("DevtoolStateHost requires a named-field struct".to_string());
    };

    let field = fields
        .iter()
        .find(|field| field.name == "state")
        .ok_or_else(|| "DevtoolStateHost expected a field named `state`".to_string())?;

    Ok(format!(
        r#"
impl {devtools_path}::DevtoolStateHost for {name} {{
    type State = {state_ty};

    fn devtool_state(&self) -> &Self::State {{
        &self.{state_field}
    }}

    fn devtool_state_mut(&mut self) -> &mut Self::State {{
        &mut self.{state_field}
    }}
}}
"#,
        devtools_path = devtools_path,
        name = parsed.name,
        state_ty = field.ty,
        state_field = field.name,
    ))
}

#[derive(Debug, Clone, Copy)]
enum StateFieldKind {
    Async,
    Operation,
    Nested,
}

fn state_field_kind(field: &ParsedField) -> Option<(&ParsedField, StateFieldKind)> {
    let ty = normalize_type(&field.ty);

    if ty.starts_with("AsyncState<") {
        Some((field, StateFieldKind::Async))
    } else if ty.starts_with("OperationState<") {
        Some((field, StateFieldKind::Operation))
    } else if is_nested_state_type(&ty) {
        Some((field, StateFieldKind::Nested))
    } else {
        None
    }
}

fn state_collect_expr(field: &ParsedField, kind: StateFieldKind, devtools_path: &str) -> String {
    let path = field_path_expr(&field.name, devtools_path);
    let label = label_from_snake(&field.name);

    match kind {
        StateFieldKind::Async | StateFieldKind::Operation => format!(
            r#"
        let path = {path};
        <{ty} as {devtools_path}::DevtoolStateField>::devtool_collect_field(&self.{name}, &path, "{label}", snapshot);
"#,
            path = path,
            ty = field.ty,
            devtools_path = devtools_path,
            name = field.name,
            label = label,
        ),
        StateFieldKind::Nested => format!(
            r#"
        let path = {path};
        {devtools_path}::DevtoolStateCatalog::devtool_collect(&self.{name}, &path, snapshot);
"#,
            path = path,
            devtools_path = devtools_path,
            name = field.name,
        ),
    }
}

fn state_apply_expr(field: &ParsedField, kind: StateFieldKind, devtools_path: &str) -> String {
    let path = field_path_expr(&field.name, devtools_path);

    match kind {
        StateFieldKind::Async | StateFieldKind::Operation => format!(
            r#"
        let path = {path};
        let result = <{ty} as {devtools_path}::DevtoolStateField>::devtool_apply_field(&mut self.{name}, &path, command);
        if result.handled() {{
            return result;
        }}
"#,
            path = path,
            ty = field.ty,
            devtools_path = devtools_path,
            name = field.name,
        ),
        StateFieldKind::Nested => format!(
            r#"
        let path = {path};
        let result = {devtools_path}::DevtoolStateCatalog::devtool_apply(&mut self.{name}, &path, command);
        if result.handled() {{
            return result;
        }}
"#,
            path = path,
            devtools_path = devtools_path,
            name = field.name,
        ),
    }
}

fn field_path_expr(field_name: &str, devtools_path: &str) -> String {
    format!(r#"{devtools_path}::join_path(scope, "{field_name}")"#)
}
