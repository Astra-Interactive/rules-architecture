const SEPARATOR_CHARACTERS: [char; 8] = ['-', '=', '*', '#', '~', '_', '+', '/'];
const HEADER_CHARACTERS: [char; 4] = ['=', '-', '*', '#'];
const BREAK_CHARACTERS: [char; 4] = ['-', '*', '_', '~'];

fn is_region_marker(text: &str) -> bool {
    let lowercase = text.trim().to_lowercase();
    let without_hash = lowercase
        .strip_prefix('#')
        .unwrap_or(&lowercase)
        .trim_start();
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
    compact.len() >= 3
        && compact
            .iter()
            .all(|character| SEPARATOR_CHARACTERS.contains(character))
}

fn count_run(characters: impl Iterator<Item = char>) -> usize {
    characters
        .take_while(|character| HEADER_CHARACTERS.contains(character))
        .count()
}

fn is_section_header(text: &str) -> bool {
    let trimmed = text.trim();
    let leading = count_run(trimmed.chars());
    let trailing = count_run(trimmed.chars().rev());
    let has_word = trimmed.chars().any(char::is_alphanumeric);
    leading >= 3 && trailing >= 3 && has_word && leading + trailing < trimmed.chars().count()
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
    if text.contains('|') {
        return None;
    }
    if is_region_marker(text) {
        return Some("a region marker");
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
