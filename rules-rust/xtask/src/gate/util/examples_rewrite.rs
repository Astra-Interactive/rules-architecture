pub(crate) fn rewrite_examples(text: &str, hashes: &[String], commit: &str) -> String {
    let mut example_index: Option<usize> = None;
    let mut rewritten = String::with_capacity(text.len());
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed == "[[example]]" {
            example_index = Some(example_index.map_or(0, |index| index + 1));
        }
        let hash = example_index.and_then(|index| hashes.get(index));
        let key = trimmed.split_once('=').map(|(key, _)| key.trim_end());
        let replacement = match hash {
            Some(hash) if key == Some("sha256") => format!("sha256 = \"{hash}\""),
            _ if key == Some("conf-ide-commit") => {
                format!("conf-ide-commit = \"{commit}\"")
            }
            _ => line.to_owned(),
        };
        rewritten.push_str(&replacement);
        rewritten.push('\n');
    }
    rewritten
}
