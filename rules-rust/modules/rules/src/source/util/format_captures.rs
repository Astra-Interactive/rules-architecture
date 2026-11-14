fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_')
        && characters.all(|character| character.is_alphanumeric() || character == '_')
}

fn list_specification_captures(specification: &str) -> Vec<String> {
    specification
        .split(|character: char| {
            !(character.is_alphanumeric() || character == '_' || character == '$')
        })
        .filter_map(|part| part.strip_suffix('$'))
        .filter(|name| is_identifier(name))
        .map(str::to_owned)
        .collect()
}

pub(crate) fn list_format_captures(text: &str) -> Vec<String> {
    let mut captures = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        if let Some(escaped) = rest.strip_prefix('{') {
            rest = escaped;
            continue;
        }
        let Some(end) = rest.find('}') else {
            break;
        };
        let (name, specification) = rest[..end].split_once(':').unwrap_or((&rest[..end], ""));
        let name = name.trim();
        if is_identifier(name) {
            captures.push(name.to_owned());
        }
        captures.extend(list_specification_captures(specification));
        rest = &rest[end + 1..];
    }
    captures
}
