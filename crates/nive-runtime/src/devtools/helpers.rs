pub fn join_path(scope: &str, field: &str) -> String {
    if scope.is_empty() {
        field.to_string()
    } else {
        format!("{scope}.{field}")
    }
}

pub(super) fn label_from_field_name(name: &str) -> String {
    let mut words = name.split('_');
    let Some(first) = words.next() else {
        return String::new();
    };

    let mut label = capitalize(first);
    for word in words {
        label.push(' ');
        label.push_str(word);
    }

    label
}

pub(super) fn placeholder_for_field_name(name: &str) -> String {
    match name {
        "color" => "#2563eb".to_string(),
        _ => label_from_field_name(name),
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    let mut output = first.to_ascii_uppercase().to_string();
    output.extend(chars);
    output
}

#[cfg(test)]
mod helpers_tests {
    use super::*;

    #[test]
    fn join_path_keeps_root_scopes_clean() {
        assert_eq!(join_path("", "projects"), "projects");
        assert_eq!(join_path("welcome", "projects"), "welcome.projects");
    }
}
