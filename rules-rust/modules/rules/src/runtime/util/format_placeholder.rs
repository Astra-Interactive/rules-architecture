pub(crate) fn names_format_argument(format: &str, name: &str) -> bool {
    let mut rest = format;
    while let Some(open) = rest.find('{') {
        rest = &rest[open + 1..];
        if let Some(escaped) = rest.strip_prefix('{') {
            rest = escaped;
            continue;
        }
        let end = rest.find(['}', ':']).unwrap_or(rest.len());
        if rest[..end].trim() == name {
            return true;
        }
    }
    false
}
