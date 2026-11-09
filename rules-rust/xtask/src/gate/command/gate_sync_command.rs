use crate::gate::model::DocumentBlock;
use crate::gate::model::GateError;
use crate::gate::model::SyncChange;
use crate::gate::util::collect_document_names;
use crate::gate::util::extract_blocks;
use crate::gate::util::parse_examples;
use crate::gate::util::parse_gate_arguments;
use crate::gate::util::rewrite_examples;
use crate::gate::util::sync_examples;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::ExitCode;

fn read_text(path: &Path) -> Result<String, GateError> {
    fs::read_to_string(path).map_err(|error| GateError::Read {
        path: path.to_string_lossy().into_owned(),
        source: error.into(),
    })
}

fn list_rust_documents(docs: &Path) -> Result<BTreeSet<String>, GateError> {
    let entries = fs::read_dir(docs).map_err(|error| GateError::Read {
        path: docs.to_string_lossy().into_owned(),
        source: error.into(),
    })?;
    collect_document_names(docs, entries)
}

fn read_blocks(docs: &Path, documents: &BTreeSet<String>) -> Result<Vec<DocumentBlock>, GateError> {
    let mut blocks = Vec::new();
    for document in documents {
        blocks.extend(extract_blocks(document, &read_text(&docs.join(document))?));
    }
    Ok(blocks)
}

fn find_docs_commit(docs: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(docs)
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn format_change(change: &SyncChange) -> String {
    match change {
        SyncChange::Changed {
            document,
            first_line,
        } => format!("changed\t{document}\t{first_line}"),
        SyncChange::Moved {
            document,
            first_line,
        } => format!("moved\t{document}\t{first_line}"),
        SyncChange::Unplaced {
            document,
            heading,
            line,
        } => format!("unplaced\t{document}\t{heading}\t{line}"),
        SyncChange::New {
            document,
            heading,
            first_line,
        } => format!("new\t{document}\t{heading}\t{first_line}"),
    }
}

#[expect(
    clippy::print_stdout,
    reason = "the sync report is this command's product"
)]
fn print_report(lines: &[String]) {
    for line in lines {
        println!("{line}");
    }
}

fn sync_gate(arguments: &[String], workspace_dir: &Path) -> Result<ExitCode, GateError> {
    let arguments = parse_gate_arguments(arguments)?;
    let examples_path = workspace_dir.join("cases/doc/examples.toml");
    let examples_text = read_text(&examples_path)?;
    let examples = parse_examples(&examples_text)?;
    let mut documents = list_rust_documents(&arguments.docs)?;
    documents.extend(examples.entries.iter().map(|entry| entry.document.clone()));
    let blocks = read_blocks(&arguments.docs, &documents)?;
    let result = sync_examples(&examples.entries, &blocks, &documents);
    print_report(
        &result
            .changes
            .iter()
            .map(format_change)
            .collect::<Vec<String>>(),
    );
    if arguments.is_check {
        return Ok(if result.changes.is_empty() {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        });
    }
    let commit = find_docs_commit(&arguments.docs).unwrap_or(examples.commit);
    fs::write(
        &examples_path,
        rewrite_examples(&examples_text, &result.hashes, &commit),
    )
    .map_err(|error| GateError::Write {
        path: examples_path.to_string_lossy().into_owned(),
        source: error.into(),
    })?;
    let needs_hand = result
        .changes
        .iter()
        .any(|change| !matches!(change, SyncChange::Changed { .. }));
    Ok(if needs_hand {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn run_gate_sync(arguments: &[String], workspace_dir: &Path) -> ExitCode {
    match sync_gate(arguments, workspace_dir) {
        Ok(code) => code,
        Err(error) => {
            print_report(&[error.to_string()]);
            ExitCode::from(2)
        }
    }
}

pub(crate) fn run_task(arguments: &[String], workspace_dir: &Path) -> ExitCode {
    match arguments.split_first() {
        Some((task, rest)) if task == "gate-sync" => run_gate_sync(rest, workspace_dir),
        _ => ExitCode::from(2),
    }
}
