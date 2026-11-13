pub(crate) fn strip_doc_code(text: &str) -> String {
    let mut result = String::new();
    let mut fence: Option<(char, usize)> = None;
    let mut inline = 0;
    for line in text.lines() {
        let trimmed = line.trim_start();
        let marker = trimmed.chars().next();
        let width = marker.map_or(0, |marker| {
            trimmed
                .chars()
                .take_while(|character| *character == marker)
                .count()
        });
        if matches!(marker, Some('`' | '~')) && width >= 3 {
            match fence {
                None => fence = marker.map(|marker| (marker, width)),
                Some((opener, length)) if marker == Some(opener) && width >= length => fence = None,
                _ => {}
            }
            result.push('\n');
            continue;
        }
        if fence.is_some() {
            result.push('\n');
            continue;
        }
        let mut characters = line.chars().peekable();
        while let Some(character) = characters.next() {
            if character == '`' {
                let mut length = 1;
                while characters.next_if_eq(&'`').is_some() {
                    length += 1;
                }
                if inline == 0 {
                    inline = length;
                } else if inline == length {
                    inline = 0;
                }
                result.push(' ');
            } else if inline == 0 {
                result.push(character);
            }
        }
        result.push('\n');
    }
    result
}
