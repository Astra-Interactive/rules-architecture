fn escaped(text: &str, offset: usize) -> bool {
    text[..offset]
        .bytes()
        .rev()
        .take_while(|byte| *byte == b'\\')
        .count()
        % 2
        == 1
}

fn linked(line: &str, start: usize, end: usize) -> bool {
    let before = line[..start].trim_end();
    let after = line[end..].trim_start();
    if before.ends_with('[') && !escaped(before, before.len() - 1) && after.starts_with(']') {
        return true;
    }
    let Some(open) = before.rfind('[') else {
        return false;
    };
    if escaped(before, open) || before[open..].contains(']') {
        return false;
    }
    let Some(close) = after.find(']') else {
        return false;
    };
    after[close + 1..].trim_start().starts_with(['(', '['])
}

fn candidates_in_line(line: &str, number: usize, candidates: &mut Vec<(usize, String)>) {
    let bytes = line.as_bytes();
    let mut cursor = 0;
    while cursor < bytes.len() {
        if bytes[cursor] != b'`' || escaped(line, cursor) {
            cursor += 1;
            continue;
        }
        let start = cursor;
        let width = bytes[cursor..]
            .iter()
            .take_while(|byte| **byte == b'`')
            .count();
        cursor += width;
        let content = cursor;
        while cursor < bytes.len() {
            if bytes[cursor] != b'`' {
                cursor += 1;
                continue;
            }
            let close = cursor;
            let closing = bytes[cursor..]
                .iter()
                .take_while(|byte| **byte == b'`')
                .count();
            cursor += closing;
            if closing != width {
                continue;
            }
            let code = line[content..close].trim();
            let name = code.strip_suffix("()").unwrap_or(code);
            let valid_characters = !name.is_empty()
                && name.chars().all(|character| {
                    character.is_alphanumeric() || ['_', ':', '#'].contains(&character)
                });
            let path = valid_characters
                .then(|| syn::parse_str::<syn::Path>(name).ok())
                .flatten();
            if path.is_some() && !linked(line, start, cursor) {
                candidates.push((number, code.to_owned()));
            }
            break;
        }
    }
}

pub(crate) fn doc_inline_candidates(text: &str) -> Vec<(usize, String)> {
    let mut candidates = Vec::new();
    let mut fence = None;
    let indent = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.bytes().take_while(|byte| *byte == b' ').count())
        .min()
        .unwrap_or(0);
    for (number, original) in text.lines().enumerate() {
        let line = original.get(indent..).unwrap_or("");
        let trimmed = line.trim_start();
        let marker = trimmed.as_bytes().first().copied();
        let width = marker.map_or(0, |marker| {
            trimmed.bytes().take_while(|byte| *byte == marker).count()
        });
        let valid_info = marker != Some(b'`') || !trimmed[width..].contains('`');
        if matches!(marker, Some(b'`' | b'~'))
            && width >= 3
            && line.len() - trimmed.len() <= 3
            && (fence.is_some() || valid_info)
        {
            match fence {
                None => fence = marker.map(|marker| (marker, width)),
                Some((open, length))
                    if marker == Some(open)
                        && width >= length
                        && trimmed[width..].trim().is_empty() =>
                {
                    fence = None
                }
                _ => {}
            }
            continue;
        }
        if fence.is_some() || line.starts_with("    ") || line.starts_with('\t') {
            continue;
        }
        candidates_in_line(line, number, &mut candidates);
    }
    candidates
}
