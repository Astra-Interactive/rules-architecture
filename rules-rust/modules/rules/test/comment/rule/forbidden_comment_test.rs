use crate::check::api::Rule;
use crate::comment::rule::ForbiddenComment;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_region_markers_with_table_delimiters_when_checked_then_regions_still_report() {
    let context = read_code_context(
        "// region Pipe | Header\n// | Column | Value |\n// endregion Pipe | Header\nfn work() {}\n",
    );
    let lines: Vec<_> = ForbiddenComment
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(1), Some(3)], lines);
}
