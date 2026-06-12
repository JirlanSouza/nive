use crate::parse::{ParsedStruct, StructFields};

pub(crate) fn expand_operation_context(parsed: ParsedStruct, devtools_path: &str) -> String {
    match &parsed.fields {
        StructFields::Unit => format!(
            r#"
impl {devtools_path}::DevtoolOperationContext for {name} {{
    fn devtool_fields() -> ::std::vec::Vec<{devtools_path}::DevtoolFieldSchema> {{
        ::std::vec::Vec::new()
    }}

    fn devtool_build(_inputs: &{devtools_path}::DevtoolInputValues<'_>) -> ::std::result::Result<Self, ::std::string::String> {{
        Ok(Self)
    }}
}}
"#,
            devtools_path = devtools_path,
            name = parsed.name,
        ),
        StructFields::Named(fields) => {
            let schemas = fields
                .iter()
                .map(|field| {
                    format!(
                        r#"<{ty} as {devtools_path}::DevtoolInputField>::devtool_schema("{name}")"#,
                        ty = field.ty,
                        name = field.name,
                        devtools_path = devtools_path,
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let build = fields
                .iter()
                .map(|field| {
                    format!(
                        r#"{name}: <{ty} as {devtools_path}::DevtoolInputField>::devtool_build("{name}", inputs)?"#,
                        ty = field.ty,
                        name = field.name,
                        devtools_path = devtools_path,
                    )
                })
                .collect::<Vec<_>>()
                .join(",");

            format!(
                r#"
impl {devtools_path}::DevtoolOperationContext for {name} {{
    fn devtool_fields() -> ::std::vec::Vec<{devtools_path}::DevtoolFieldSchema> {{
        ::std::vec![{schemas}]
    }}

    fn devtool_build(inputs: &{devtools_path}::DevtoolInputValues<'_>) -> ::std::result::Result<Self, ::std::string::String> {{
        Ok(Self {{ {build} }})
    }}
}}
"#,
                devtools_path = devtools_path,
                name = parsed.name,
                schemas = schemas,
                build = build,
            )
        }
        StructFields::Tuple => {
            "compile_error!(\"DevtoolOperationContext supports unit or named-field structs only\");"
                .to_string()
        }
    }
}
