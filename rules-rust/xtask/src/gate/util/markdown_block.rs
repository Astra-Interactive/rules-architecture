use crate::gate::model::DocumentBlock;

fn find_fence_indent(line: &str) -> Option<usize> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let marker = if trimmed.starts_with('`') {
        '`'
    } else if trimmed.starts_with('~') {
        '~'
    } else {
        return None;
    };
    let length = trimmed
        .chars()
        .take_while(|character| *character == marker)
        .count();
    let has_invalid_info = marker == '`' && trimmed[length..].contains('`');
    (length >= 3 && !has_invalid_info).then(|| line.len() - trimmed.len())
}

fn is_closing_fence(opening: &str, line: &str) -> bool {
    let maximum_indent = opening.len() - opening.trim_start_matches([' ', '\t']).len() + 3;
    let opening = opening.trim_start_matches([' ', '\t']);
    let marker = if opening.starts_with('~') { '~' } else { '`' };
    let required = opening
        .chars()
        .take_while(|character| *character == marker)
        .count();
    let trimmed = line.trim_start_matches([' ', '\t']);
    let length = trimmed
        .chars()
        .take_while(|character| *character == marker)
        .count();
    let has_delimiter = length >= required;
    let has_no_info = trimmed[length..].trim().is_empty();
    let has_closing_indent = line.len() - trimmed.len() <= maximum_indent;
    has_delimiter && has_no_info && has_closing_indent
}

fn strip_indent(line: &str, indent: usize) -> String {
    let leading = line.len() - line.trim_start_matches([' ', '\t']).len();
    line[leading.min(indent)..].to_owned()
}

pub(crate) fn extract_blocks(document: &str, text: &str) -> Vec<DocumentBlock> {
    let mut blocks = Vec::new();
    let mut heading = String::new();
    let mut open: Option<String> = None;
    let mut body: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(opening) = &open {
            if is_closing_fence(opening, line) {
                blocks.push(DocumentBlock {
                    document: document.to_owned(),
                    heading: heading.clone(),
                    first_line: body.first().cloned().unwrap_or_default(),
                    lines: std::mem::take(&mut body),
                });
                open = None;
            } else {
                body.push(strip_indent(
                    line,
                    opening.len() - opening.trim_start_matches([' ', '\t']).len(),
                ));
            }
            continue;
        }
        if find_fence_indent(line).is_some() {
            open = Some(line.to_owned());
        } else if line.starts_with('#') {
            heading = line.trim().to_owned();
        }
    }
    if open.is_some() {
        blocks.push(DocumentBlock {
            document: document.to_owned(),
            heading,
            first_line: body.first().cloned().unwrap_or_default(),
            lines: body,
        });
    }
    blocks
}
