use crate::{
    naming::split_words,
    parse::{ParsedEnum, ParsedVariant},
};

pub(crate) fn expand_probe_catalog(parsed: ParsedEnum) -> String {
    let all = parsed
        .variants
        .iter()
        .map(|variant| format!("Self::{}", variant.name))
        .collect::<Vec<_>>()
        .join(",");
    let arms = parsed
        .variants
        .iter()
        .map(probe_meta_arm)
        .collect::<Vec<_>>()
        .join(",");

    format!(
        r#"
impl {enum_name} {{
    pub(super) const ALL: &'static [Self] = <Self as nive_runtime::ProbeCatalogEntry>::ALL;

    pub(super) fn meta(self) -> nive_runtime::ProbeMeta {{
        <Self as nive_runtime::ProbeCatalogEntry>::meta(self)
    }}
}}

impl nive_runtime::ProbeCatalogEntry for {enum_name} {{
    const ALL: &'static [Self] = &[{all}];

    fn meta(self) -> nive_runtime::ProbeMeta {{
        match self {{
            {arms}
        }}
    }}
}}
"#,
        enum_name = parsed.name,
        all = all,
        arms = arms,
    )
}

fn probe_meta_arm(variant: &ParsedVariant) -> String {
    let words = split_words(&variant.name);
    let meta = probe_meta(&words);

    format!(
        r#"
Self::{variant} => nive_runtime::ProbeMeta::new(
    "{key}",
    "{short_key}",
    "{summary}",
    nive_runtime::ProbeErrorScope::{kind},
)
"#,
        variant = variant.name,
        key = meta.key,
        short_key = meta.short_key,
        summary = meta.summary,
        kind = meta.kind,
    )
}

pub(crate) struct ProbeMetaParts {
    pub(crate) key: String,
    pub(crate) short_key: String,
    pub(crate) summary: String,
    pub(crate) kind: &'static str,
}

fn probe_meta(words: &[String]) -> ProbeMetaParts {
    if words == ["bootstrap"] {
        return ProbeMetaParts {
            key: "bootstrap".to_string(),
            short_key: "bootstrap".to_string(),
            summary: "Couldn't initialize the app".to_string(),
            kind: "Bootstrap",
        };
    }

    if words.starts_with(&["project".to_string(), "catalog".to_string()]) {
        let operation = &words[2..];
        return ProbeMetaParts {
            key: format!("project_catalog.{}", operation.join("_")),
            short_key: project_catalog_short_key(operation),
            summary: project_catalog_summary(operation),
            kind: "ProjectCatalog",
        };
    }

    if words.starts_with(&["tag".to_string()]) {
        let operation = &words[1..];
        return ProbeMetaParts {
            key: format!("tag.{}", operation.join("_")),
            short_key: tag_short_key(operation),
            summary: tag_summary(operation),
            kind: "Tag",
        };
    }

    ProbeMetaParts {
        key: words.join("_"),
        short_key: words.join("_"),
        summary: format!("Couldn't run {}", words.join(" ")),
        kind: "ProjectCatalog",
    }
}

pub(crate) fn probe_meta_from_client_method(
    client_type: &str,
    method_name: &str,
    key_override: Option<&str>,
) -> ProbeMetaParts {
    let scope = client_scope(client_type);
    let default_key = format!("{scope}.{method_name}");
    let operation = operation_words(method_name);
    let mut meta = probe_meta_from_scope_operation(&scope, &operation);

    meta.key = key_override.map(str::to_string).unwrap_or(default_key);

    meta
}

pub(crate) fn client_scope(client_type: &str) -> String {
    let stem = client_type.strip_suffix("Client").unwrap_or(client_type);
    split_words(stem).join("_")
}

fn operation_words(method_name: &str) -> Vec<String> {
    let words = method_name
        .split('_')
        .filter(|word| !word.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();

    match words.as_slice() {
        [first, rest @ ..] if first == "get" && !rest.is_empty() => rest.to_vec(),
        _ => words,
    }
}

fn probe_meta_from_scope_operation(scope: &str, operation: &[String]) -> ProbeMetaParts {
    match scope {
        "project_catalog" => ProbeMetaParts {
            key: format!("project_catalog.{}", operation.join("_")),
            short_key: project_catalog_short_key(operation),
            summary: project_catalog_summary(operation),
            kind: "ProjectCatalog",
        },
        "tag" => ProbeMetaParts {
            key: format!("tag.{}", operation.join("_")),
            short_key: tag_short_key(operation),
            summary: tag_summary(operation),
            kind: "Tag",
        },
        _ => {
            let label = scope.replace('_', " ");
            ProbeMetaParts {
                key: format!("{}.{}", scope, operation.join("_")),
                short_key: operation.join("_"),
                summary: format!("Couldn't run {} {}", label, operation.join(" ")),
                kind: "ProjectCatalog",
            }
        }
    }
}

fn project_catalog_short_key(operation: &[String]) -> String {
    match operation {
        [word] if word == "list" => "list_projects".to_string(),
        [word] if word == "summary" => "project_summary".to_string(),
        [word] if word == "create" => "create_project".to_string(),
        [word] if word == "delete" => "delete_project".to_string(),
        [word] if word == "open" => "open_project".to_string(),
        _ => operation.join("_"),
    }
}

fn tag_short_key(operation: &[String]) -> String {
    match operation {
        [word] if word == "list" => "list_tags".to_string(),
        [first, second] if first == "update" && second == "color" => "update_tag_color".to_string(),
        _ => operation.join("_"),
    }
}

fn project_catalog_summary(operation: &[String]) -> String {
    match operation {
        [word] if word == "list" => "Couldn't load projects".to_string(),
        [word] if word == "summary" => "Couldn't load project summary".to_string(),
        [word] if word == "create" => "Couldn't create project".to_string(),
        [word] if word == "delete" => "Couldn't delete project".to_string(),
        [word] if word == "open" => "Couldn't open project".to_string(),
        _ => format!("Couldn't run project catalog {}", operation.join(" ")),
    }
}

fn tag_summary(operation: &[String]) -> String {
    match operation {
        [word] if word == "list" => "Couldn't load tags".to_string(),
        [first, second] if first == "update" && second == "color" => {
            "Couldn't update tag color".to_string()
        }
        _ => format!("Couldn't run tag {}", operation.join(" ")),
    }
}
