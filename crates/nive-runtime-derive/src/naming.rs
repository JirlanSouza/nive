pub(crate) fn label_from_snake(snake: &str) -> String {
    let mut words = snake.split('_');
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

pub(crate) fn split_words(name: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous_lower_or_digit = false;

    for ch in name.chars() {
        if ch.is_ascii_uppercase() && previous_lower_or_digit && !current.is_empty() {
            words.push(current);
            current = String::new();
        }

        current.push(ch.to_ascii_lowercase());
        previous_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
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
