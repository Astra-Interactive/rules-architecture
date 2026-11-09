use crate::gate::model::DocumentBlock;
use crate::gate::model::ExampleEntry;
use crate::gate::model::SyncChange;
use crate::gate::model::SyncResult;
use crate::gate::util::compute_example_hash;
use crate::gate::util::is_marker_line;
use std::collections::BTreeSet;

fn find_block<'blocks>(
    entry: &ExampleEntry,
    blocks: &'blocks [DocumentBlock],
) -> Option<&'blocks DocumentBlock> {
    blocks
        .iter()
        .filter(|block| {
            block.document == entry.document
                && block.heading == entry.anchor
                && block.first_line == entry.first_line
        })
        .nth(entry.occurrence.saturating_sub(1))
}

fn is_same_block(entry: &ExampleEntry, block: &DocumentBlock, blocks: &[DocumentBlock]) -> bool {
    find_block(entry, blocks).is_some_and(|found| std::ptr::eq(found, block))
}

fn find_covered_lines(block: &DocumentBlock, entries: &[&ExampleEntry]) -> BTreeSet<usize> {
    let is_whole = entries
        .iter()
        .any(|entry| entry.wrap == "none" || entry.fragments.is_empty());
    if is_whole {
        return (1..=block.lines.len()).collect();
    }
    entries
        .iter()
        .flat_map(|entry| &entry.fragments)
        .flat_map(|fragment| fragment.first_block_line..=fragment.last_block_line)
        .collect()
}

fn find_placement_changes(
    entries: &[ExampleEntry],
    blocks: &[DocumentBlock],
    documents: &BTreeSet<String>,
) -> Vec<SyncChange> {
    let mut changes = Vec::new();
    for block in blocks
        .iter()
        .filter(|block| documents.contains(&block.document))
    {
        let matching: Vec<&ExampleEntry> = entries
            .iter()
            .filter(|entry| is_same_block(entry, block, blocks))
            .collect();
        if matching.is_empty() {
            changes.push(SyncChange::New {
                document: block.document.clone(),
                heading: block.heading.clone(),
                first_line: block.first_line.clone(),
            });
            continue;
        }
        let covered = find_covered_lines(block, &matching);
        for (index, line) in block.lines.iter().enumerate() {
            let is_placed =
                line.trim().is_empty() || is_marker_line(line) || covered.contains(&(index + 1));
            if !is_placed {
                changes.push(SyncChange::Unplaced {
                    document: block.document.clone(),
                    heading: block.heading.clone(),
                    line: line.trim().to_owned(),
                });
            }
        }
    }
    changes
}

pub(crate) fn sync_examples(
    entries: &[ExampleEntry],
    blocks: &[DocumentBlock],
    documents: &BTreeSet<String>,
) -> SyncResult {
    let mut changes = Vec::new();
    let mut hashes = Vec::new();
    for entry in entries {
        let hash = find_block(entry, blocks).and_then(|block| compute_example_hash(entry, block));
        let Some(hash) = hash else {
            changes.push(SyncChange::Moved {
                document: entry.document.clone(),
                first_line: entry.first_line.clone(),
            });
            hashes.push(entry.sha256.clone());
            continue;
        };
        if hash != entry.sha256 {
            changes.push(SyncChange::Changed {
                document: entry.document.clone(),
                first_line: entry.first_line.clone(),
            });
        }
        hashes.push(hash);
    }
    changes.extend(find_placement_changes(entries, blocks, documents));
    SyncResult { changes, hashes }
}
