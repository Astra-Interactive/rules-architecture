const SEPARATOR_CHARACTERS: [char; 9] = ['-', '=', '*', '#', '~', '_', '+', '/', '|'];
const HEADER_CHARACTERS: [char; 8] = ['=', '-', '*', '#', '~', '_', '+', '/'];
const ONE_SIDED_HEADER_CHARACTERS: [char; 3] = ['=', '-', '*'];
const BREAK_CHARACTERS: [char; 4] = ['-', '*', '_', '~'];

fn is_region_marker(text: &str) -> bool {
    let trimmed = text.trim();
    let without_hash = trimmed.strip_prefix('#').unwrap_or(trimmed).trim_start();
    let rest = without_hash.strip_prefix("end").unwrap_or(without_hash);
    let Some(after) = rest.strip_prefix("region") else {
        return false;
    };
    after
        .chars()
        .next()
        .is_none_or(|character| !character.is_alphanumeric() && character != '_')
}

fn is_separator(text: &str) -> bool {
    let compact: Vec<char> = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let Some(first) = compact.first() else {
        return false;
    };
    let is_known_run = compact
        .iter()
        .all(|character| SEPARATOR_CHARACTERS.contains(character));
    let is_repeated_delimiter =
        !first.is_alphanumeric() && compact.iter().all(|character| character == first);
    compact.len() >= 3 && (is_known_run || is_repeated_delimiter)
}

fn count_run(characters: impl Iterator<Item = char>, markers: &[char]) -> usize {
    characters
        .take_while(|character| markers.contains(character))
        .count()
}

fn is_section_header(text: &str) -> bool {
    let trimmed = text.trim();
    if !trimmed.chars().any(char::is_alphanumeric) {
        return false;
    }
    let is_framed = count_run(trimmed.chars(), &HEADER_CHARACTERS) >= 3
        && count_run(trimmed.chars().rev(), &HEADER_CHARACTERS) >= 3;
    let is_one_sided = count_run(trimmed.chars(), &ONE_SIDED_HEADER_CHARACTERS) >= 3
        || count_run(trimmed.chars().rev(), &ONE_SIDED_HEADER_CHARACTERS) >= 3;
    is_framed || is_one_sided
}

fn is_doc_fence_or_break(text: &str) -> bool {
    let compact: String = text
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    let is_fence = compact.starts_with("```") || compact.starts_with("~~~");
    let is_break = compact.len() >= 3
        && BREAK_CHARACTERS
            .iter()
            .any(|marker| compact.chars().all(|character| character == *marker));
    is_fence || is_break
}

pub(crate) fn find_forbidden_shape(text: &str, is_doc: bool) -> Option<&'static str> {
    if is_region_marker(text) {
        return Some("a region marker");
    }
    if text.contains('|') && !is_separator(text) {
        return is_section_header(text).then_some("a section header");
    }
    let is_table = text.contains('|')
        && text.trim().trim_matches('|').split('|').all(|cell| {
            let markers = cell.trim().trim_matches(':');
            !markers.is_empty() && markers.chars().all(|marker| marker == '-')
        });
    if is_table {
        return None;
    }
    if is_doc && is_doc_fence_or_break(text) {
        return None;
    }
    if is_separator(text) {
        return Some("a separator line");
    }
    if is_section_header(text) {
        return Some("a section header");
    }
    None
}
