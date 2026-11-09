const ABBREVIATIONS: [&str; 4] = ["e.g.", "i.e.", "etc.", "vs."];

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn remove_abbreviations(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut skip_until = 0;
    let mut previous: Option<char> = None;
    for (index, character) in text.char_indices() {
        let follows_word = previous.is_some_and(is_word_character);
        previous = Some(character);
        if index < skip_until {
            continue;
        }
        let rest = &text[index..];
        let abbreviation = ABBREVIATIONS.iter().find(|abbreviation| {
            let head = rest.get(..abbreviation.len());
            !follows_word && head.is_some_and(|head| head.eq_ignore_ascii_case(abbreviation))
        });
        match abbreviation {
            Some(abbreviation) => skip_until = index + abbreviation.len(),
            None => kept.push(character),
        }
    }
    kept
}

pub(crate) fn count_sentences(text: &str) -> usize {
    let joined = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect::<Vec<&str>>()
        .join(" ");
    let cleaned = remove_abbreviations(&joined);
    if cleaned.trim().is_empty() {
        return 0;
    }
    let characters: Vec<char> = cleaned.chars().collect();
    let ends = characters
        .iter()
        .enumerate()
        .filter(|(index, character)| {
            let is_terminal = matches!(character, '.' | '!' | '?');
            let next = characters.get(index + 1);
            is_terminal && next.is_none_or(|next| next.is_whitespace())
        })
        .count();
    ends.max(1)
}
