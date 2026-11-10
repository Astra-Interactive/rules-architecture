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

fn ends_initialism(characters: &[char], index: usize) -> bool {
    let start = characters[..index]
        .iter()
        .rposition(|character| character.is_whitespace())
        .map_or(0, |space| space + 1);
    let word = &characters[start..=index];
    let (groups, rest) = word.as_chunks::<2>();
    word.len() >= 4
        && rest.is_empty()
        && groups
            .iter()
            .all(|group| group[0].is_alphabetic() && group[1] == '.')
}

fn is_sentence_end(characters: &[char], index: usize) -> bool {
    let character = characters[index];
    let is_terminal = matches!(character, '.' | '!' | '?');
    let next = characters.get(index + 1);
    is_terminal
        && next.is_none_or(|next| next.is_whitespace())
        && !(character == '.' && ends_initialism(characters, index))
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
    let ends: Vec<usize> = (0..characters.len())
        .filter(|index| is_sentence_end(&characters, *index))
        .collect();
    let tail_start = ends.last().map_or(0, |last| last + 1);
    let has_tail = characters[tail_start..]
        .iter()
        .any(|character| !character.is_whitespace());
    ends.len() + usize::from(has_tail)
}
