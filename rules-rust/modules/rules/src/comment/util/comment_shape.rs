const SEPARATOR_CHARACTERS: [char; 9] = ['-', '=', '*', '#', '~', '_', '+', '/', '|'];
const HEADER_CHARACTERS: [char; 8] = ['=', '-', '*', '#', '~', '_', '+', '/'];
const ONE_SIDED_HEADER_CHARACTERS: [char; 3] = ['=', '-', '*'];
const ONE_SIDED_PLAIN_HEADER_CHARACTERS: [char; 7] = ['=', '-', '*', '#', '~', '_', '+'];
const BREAK_CHARACTERS: [char; 4] = ['-', '*', '_', '~'];
const EDITOR_MARKER_PREFIXES: [&str; 3] = ["<editor-fold", "</editor-fold>", "MARK:"];

fn is_region_marker(text: &str) -> bool {
    let trimmed = text.trim();
    if EDITOR_MARKER_PREFIXES
        .iter()
        .any(|prefix| trimmed.starts_with(prefix))
    {
        return true;
    }
    let without_hash = trimmed.strip_prefix('#').unwrap_or(trimmed).trim_start();
    let without_pragma = without_hash
        .strip_prefix("pragma ")
        .map_or(without_hash, str::trim_start);
    let rest = without_pragma.strip_prefix("end").unwrap_or(without_pragma);
    let Some(after) = rest.strip_prefix("region") else {
        return false;
    };
    after
        .chars()
        .next()
        .is_none_or(|character| character.is_whitespace() || character == ':')
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

fn count_symbol_run(mut characters: impl Iterator<Item = char>) -> usize {
    let Some(first) = characters.next() else {
        return 0;
    };
    if first.is_ascii() || first.is_alphanumeric() || first.is_whitespace() {
        return 0;
    }
    1 + characters
        .take_while(|character| *character == first)
        .count()
}

fn is_emphasis_run(characters: &[char]) -> bool {
    let Some(&marker) = characters.first() else {
        return false;
    };
    if marker != '*' && marker != '_' {
        return false;
    }
    let length = characters
        .iter()
        .take_while(|character| **character == marker)
        .count();
    let touches_word = characters
        .get(length)
        .is_some_and(|character| !character.is_whitespace());
    let run: String = std::iter::repeat_n(marker, length).collect();
    let rest: String = characters[length..].iter().collect();
    touches_word && rest.contains(&run)
}

fn has_header_run(characters: &[char], markers: &[char], allows_symbols: bool) -> bool {
    let is_marker_run = count_run(characters.iter().copied(), markers) >= 3;
    let is_symbol_run = allows_symbols && count_symbol_run(characters.iter().copied()) >= 3;
    (is_marker_run || is_symbol_run) && !is_emphasis_run(characters)
}

fn is_section_header(text: &str, is_doc: bool) -> bool {
    let trimmed = text.trim();
    if !trimmed.chars().any(char::is_alphanumeric) {
        return false;
    }
    let forward: Vec<char> = trimmed.chars().collect();
    let backward: Vec<char> = trimmed.chars().rev().collect();
    let is_framed = has_header_run(&forward, &HEADER_CHARACTERS, true)
        && has_header_run(&backward, &HEADER_CHARACTERS, true);
    let one_sided_markers: &[char] = if is_doc {
        &ONE_SIDED_HEADER_CHARACTERS
    } else {
        &ONE_SIDED_PLAIN_HEADER_CHARACTERS
    };
    let is_one_sided = has_header_run(&forward, one_sided_markers, !is_doc)
        || has_header_run(&backward, one_sided_markers, !is_doc);
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
        return is_section_header(text, is_doc).then_some("a section header");
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
    if is_section_header(text, is_doc) {
        return Some("a section header");
    }
    None
}
