use crate::gate::util::rewrite_examples;

#[test]
fn given_new_hashes_and_commit_when_rewritten_then_only_those_lines_change() {
    let text = "# sha256 = \"kept\"\n[sync]\nconf-ide-commit = \"old\"\n\n[[example]]\nsha256 = \"a\"\n\n[[example]]\nclaim = \"x\"\nsha256 = \"b\"\n";

    let rewritten = rewrite_examples(text, &["first".to_owned(), "second".to_owned()], "new");

    assert_eq!(
        "# sha256 = \"kept\"\n[sync]\nconf-ide-commit = \"new\"\n\n[[example]]\nsha256 = \"first\"\n\n[[example]]\nclaim = \"x\"\nsha256 = \"second\"\n",
        rewritten
    );
}
