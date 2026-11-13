use crate::gate::command::run_task;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

const EXAMPLES: &str = "[sync]\nconf-ide-commit = \"old\"\n\n[[example]]\ndocument = \"rule.rs.md\"\nanchor = \"# Rule\"\nfirst-line = \"fn first() {}\"\nwrap = \"file\"\nsha256 = \"\"\n";

fn write_workspace(root: &Path, document: &str) {
    fs::create_dir_all(root.join("cases/doc")).expect("cases/doc");
    fs::create_dir_all(root.join("docs")).expect("docs");
    fs::write(root.join("cases/doc/examples.toml"), EXAMPLES).expect("examples.toml");
    fs::write(root.join("docs/rule.rs.md"), document).expect("the document");
}

fn invoke_gate_sync(arguments: &[String], root: &Path) -> ExitCode {
    let mut task = vec!["gate-sync".to_owned()];
    task.extend_from_slice(arguments);
    run_task(&task, root)
}

fn run(root: &Path, extra: &[&str]) -> ExitCode {
    let mut arguments = vec![
        "--docs".to_owned(),
        root.join("docs").to_string_lossy().into_owned(),
    ];
    arguments.extend(extra.iter().map(|word| (*word).to_owned()));
    invoke_gate_sync(&arguments, root)
}

#[test]
fn given_stale_hash_when_synced_then_rewrites_it_and_the_check_passes_afterwards() {
    let root = tempfile::tempdir().expect("a temporary directory");
    write_workspace(root.path(), "# Rule\n```rust\nfn first() {}\n```\n");

    let check_before = run(root.path(), &["--check"]);
    let sync = run(root.path(), &[]);
    let check_after = run(root.path(), &["--check"]);

    let examples =
        fs::read_to_string(root.path().join("cases/doc/examples.toml")).expect("examples.toml");
    assert_eq!(ExitCode::from(1), check_before);
    assert_eq!(ExitCode::SUCCESS, sync);
    assert_eq!(ExitCode::SUCCESS, check_after);
    assert!(!examples.contains("sha256 = \"\""));
    assert!(examples.contains("conf-ide-commit = \"old\""));
}

#[test]
fn given_new_example_when_synced_then_fails_for_a_hand_edit() {
    let root = tempfile::tempdir().expect("a temporary directory");
    write_workspace(
        root.path(),
        "# Rule\n```rust\nfn first() {}\n```\n```rust\nfn second() {}\n```\n",
    );

    assert_eq!(ExitCode::from(1), run(root.path(), &[]));
}

#[test]
fn given_missing_docs_directory_when_synced_then_exits_with_two() {
    let root = tempfile::tempdir().expect("a temporary directory");
    fs::create_dir_all(root.path().join("cases/doc")).expect("cases/doc");
    fs::write(root.path().join("cases/doc/examples.toml"), EXAMPLES).expect("examples.toml");

    assert_eq!(ExitCode::from(2), run(root.path(), &[]));
}

#[test]
fn given_read_only_examples_when_synced_then_exits_with_two() {
    let root = tempfile::tempdir().expect("a temporary directory");
    write_workspace(root.path(), "# Rule\n```rust\nfn first() {}\n```\n");
    let examples = root.path().join("cases/doc/examples.toml");
    let mut permissions = fs::metadata(&examples).expect("metadata").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&examples, permissions).expect("read-only");

    assert_eq!(ExitCode::from(2), run(root.path(), &[]));
}

#[test]
fn given_no_arguments_when_synced_then_exits_with_two() {
    let root = tempfile::tempdir().expect("a temporary directory");

    assert_eq!(ExitCode::from(2), invoke_gate_sync(&[], root.path()));
}

#[test]
fn given_new_tilde_example_when_checked_then_gate_reports_drift() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path(), "# Rule\n```rust\nfn first() {}\n```\n");
    assert_eq!(run(root.path(), &[]), ExitCode::SUCCESS);
    fs::write(
        root.path().join("docs/rule.rs.md"),
        "# Rule\n```rust\nfn first() {}\n```\n~~~rust\nfn new_example() {}\n~~~\n",
    )
    .expect("append tilde example");
    assert_eq!(run(root.path(), &["--check"]), ExitCode::from(1));
}

#[test]
fn given_dispatch_arguments_when_run_then_known_tasks_dispatch_and_unknown_tasks_fail() {
    let root = tempfile::tempdir().expect("temporary directory");
    for arguments in [
        vec![],
        vec!["unknown".to_owned()],
        vec!["gate-sync".to_owned()],
    ] {
        assert_eq!(run_task(&arguments, root.path()), ExitCode::from(2));
    }
}

#[test]
fn given_missing_referenced_document_when_synced_then_read_failure_is_reported() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path(), "# Rule\n```rust\nfn first() {}\n```\n");
    fs::remove_file(root.path().join("docs/rule.rs.md")).expect("remove temporary document");
    assert_eq!(run(root.path(), &[]), ExitCode::from(2));
}

#[test]
fn given_moved_block_and_unplaced_line_when_synced_then_manual_review_is_required() {
    let root = tempfile::tempdir().expect("temporary directory");
    write_workspace(root.path(), "# Rule\n```rust\nfn first() {}\n```\n");
    assert_eq!(run(root.path(), &[]), ExitCode::SUCCESS);
    fs::write(
        root.path().join("docs/rule.rs.md"),
        "# Moved\n```rust\nfn first() {}\nextra();\n```\n",
    )
    .expect("changed document");
    assert_eq!(run(root.path(), &[]), ExitCode::from(1));
    fs::write(
        root.path().join("cases/doc/examples.toml"),
        format!("{EXAMPLES}\n[[example.fragment]]\nblock-lines = \"1-1\"\n"),
    )
    .expect("fragment placement");
    fs::write(
        root.path().join("docs/rule.rs.md"),
        "# Rule\n```rust\nfn first() {}\nextra();\n```\n",
    )
    .expect("unplaced line");
    assert_eq!(run(root.path(), &[]), ExitCode::from(1));
}

#[test]
fn given_documents_inside_checkout_when_synced_then_records_the_git_commit() {
    let root = tempfile::tempdir().expect("temporary directory");
    fs::create_dir_all(root.path().join("cases/doc")).expect("cases/doc");
    fs::write(
        root.path().join("cases/doc/examples.toml"),
        "[sync]\nconf-ide-commit=\"old\"\n",
    )
    .expect("manifest");
    let arguments = vec!["--docs".to_owned(), env!("CARGO_MANIFEST_DIR").to_owned()];
    assert_eq!(invoke_gate_sync(&arguments, root.path()), ExitCode::SUCCESS);
    let examples = fs::read_to_string(root.path().join("cases/doc/examples.toml"))
        .expect("rewritten manifest");
    assert!(!examples.contains("\"old\""));
}
