use crate::comment::util::strip_doc_code;

fn is_rust_fence(info: &str) -> bool {
    info.trim().is_empty()
        || info
            .split(|character: char| character == ',' || character.is_whitespace())
            .any(|flag| {
                matches!(
                    flag,
                    "rust"
                        | "ignore"
                        | "no_run"
                        | "compile_fail"
                        | "should_panic"
                        | "standalone_crate"
                ) || flag.starts_with("edition")
                    || flag.starts_with("ignore-")
            })
}

fn has_disabled_execution(info: &str) -> bool {
    info.split(|character: char| character == ',' || character.is_whitespace())
        .any(|flag| matches!(flag, "ignore" | "no_run") || flag.starts_with("ignore-"))
}

fn is_indented_code(line: &str) -> bool {
    line.bytes()
        .take_while(|byte| matches!(byte, b' ' | b'\t'))
        .fold(0, |width, byte| match byte {
            b'\t' => width + 4 - width % 4,
            _ => width + 1,
        })
        >= 4
}

fn has_invalid_section(section: &str) -> bool {
    let mut fence_marker = None;
    let mut fence_width = 0;
    let mut executable = false;
    let mut content = false;
    let mut runnable = false;
    let mut indented_start = true;
    for line in section.lines() {
        let trimmed = line.trim_start();
        let marker = trimmed.as_bytes().first().copied();
        let width = marker.map_or(0, |marker| {
            trimmed.bytes().take_while(|byte| *byte == marker).count()
        });
        let delimiter =
            matches!(marker, Some(b'`' | b'~')) && width >= 3 && line.len() - trimmed.len() <= 3;
        if let Some(opening) = fence_marker {
            let closes = delimiter
                && marker == Some(opening)
                && width >= fence_width
                && trimmed[width..].trim().is_empty();
            if closes {
                runnable |= executable && content;
                fence_marker = None;
                indented_start = true;
            } else {
                content |= !trimmed.is_empty();
            }
            continue;
        }
        if !delimiter {
            let indented = is_indented_code(line);
            runnable |= indented_start && indented && !trimmed.is_empty();
            indented_start = trimmed.is_empty() || indented_start && indented;
            continue;
        }
        let info = trimmed[width..].trim();
        if marker == Some(b'`') && info.contains('`') {
            continue;
        }
        let rust = is_rust_fence(info);
        if rust && has_disabled_execution(info) {
            return true;
        }
        executable = rust
            && !info
                .split(|character: char| character == ',' || character.is_whitespace())
                .any(|flag| flag == "compile_fail");
        content = false;
        fence_marker = marker;
        fence_width = width;
    }
    !(runnable || (fence_marker.is_some() && executable && content))
}

pub(crate) fn collect_invalid_example_sections(text: &str) -> Vec<usize> {
    let indentation = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.bytes().take_while(|byte| *byte == b' ').count())
        .min()
        .unwrap_or(0);
    let lines: Vec<_> = text
        .lines()
        .map(|line| line.get(indentation..).unwrap_or(line))
        .collect();
    let normalized = lines.join("\n");
    let mut section: Option<ExamplesSection> = None;
    let mut invalid = Vec::new();
    for (number, line) in strip_doc_code(&normalized).lines().enumerate() {
        let heading = line.trim_start();
        let level = heading.bytes().take_while(|byte| *byte == b'#').count();
        let invalid_depth = level == 0 || level > 6;
        let indented = line.len() - heading.len() > 3;
        let spaced = heading[level..].starts_with(char::is_whitespace);
        if invalid_depth || indented || !spaced {
            continue;
        }
        let examples = heading[level..].trim().trim_end_matches('#').trim_end() == "Examples";
        let finished = section.filter(|open| level <= open.depth || examples);
        if let Some(finished_section) = finished {
            let invalid_section =
                has_invalid_section(&lines[finished_section.start_line..number].join("\n"));
            if invalid_section {
                invalid.push(finished_section.heading_line);
            }
            section = None;
        }
        if examples {
            section = Some(ExamplesSection {
                heading_line: number,
                start_line: number + 1,
                depth: level,
            });
        }
    }
    let Some(last_section) = section else {
        return invalid;
    };
    let invalid_section = has_invalid_section(&lines[last_section.start_line..].join("\n"));
    if invalid_section {
        invalid.push(last_section.heading_line);
    }
    invalid
}

#[derive(Clone, Copy)]
struct ExamplesSection {
    heading_line: usize,
    start_line: usize,
    depth: usize,
}
