use crate::gate::model::DocumentBlock;
use crate::gate::model::ExampleEntry;
use crate::gate::model::ExampleFragment;
use sha2::Digest;
use sha2::Sha256;

const MARKERS: [&str; 4] = ["Bad", "Good", "Deleted", "Kept"];

pub(crate) fn is_marker_line(line: &str) -> bool {
    let Some(comment) = line.trim().strip_prefix("//") else {
        return false;
    };
    let word: String = comment
        .trim_start()
        .chars()
        .take_while(|character| character.is_alphabetic())
        .collect();
    MARKERS.contains(&word.as_str())
}

fn hash_normalised_lines(lines: &[String]) -> String {
    let joined = lines
        .iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<&str>>()
        .join("\n");
    Sha256::digest(joined.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn render_fragment(fragment: &ExampleFragment, block: &DocumentBlock) -> Option<Vec<String>> {
    let first = fragment.first_block_line.checked_sub(1)?;
    let lines = block.lines.get(first..fragment.last_block_line)?;
    let mut rendered: Vec<String> = lines
        .iter()
        .map(|line| {
            fragment
                .translations
                .iter()
                .fold(line.clone(), |text, translation| {
                    text.replace(&translation.from, &translation.to)
                })
        })
        .collect();
    if let Some(last) = rendered.last_mut() {
        last.push_str(&fragment.suffix);
    }
    Some(rendered)
}

pub(crate) fn compute_example_hash(entry: &ExampleEntry, block: &DocumentBlock) -> Option<String> {
    if entry.fragments.is_empty() {
        return Some(hash_normalised_lines(&block.lines));
    }
    let mut lines = Vec::new();
    for fragment in &entry.fragments {
        lines.extend(render_fragment(fragment, block)?);
    }
    Some(hash_normalised_lines(&lines))
}
