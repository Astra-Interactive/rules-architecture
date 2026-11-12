use crate::check::api::Rule;
use crate::comment::rule::ForbiddenComment;
use crate::test::declaration::util::read_code_context;

fn read_finding_lines(source: &str) -> Vec<Option<u32>> {
    let context = read_code_context(source);
    ForbiddenComment
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect()
}

fn read_sorted_finding_lines(source: &str) -> Vec<Option<u32>> {
    let mut lines = read_finding_lines(source);
    lines.sort_unstable();
    lines
}

#[test]
fn given_one_sided_hash_headers_and_box_drawing_frames_when_checked_then_each_header_is_reported() {
    assert_eq!(
        vec![Some(1), Some(2), Some(3), Some(4)],
        read_sorted_finding_lines(
            "// ### Helpers\n// ─── Constructors ───\n// Helpers ###\n// ─── Transitions\n/// ### Panics\n/// # Panics\n// retry after a pause - then give up\n// see issue #1234\nfn work() {}\n",
        )
    );
}

#[test]
fn given_prose_starting_with_the_word_region_when_checked_then_only_markers_are_reported() {
    assert_eq!(
        vec![Some(4), Some(6)],
        read_sorted_finding_lines(
            "// region, not the country, decides the tax rate.\n// region-based pricing is off by default.\n// region's code is ISO 3166-2.\n// region Helpers\nfn rate() {}\n// endregion\n",
        )
    );
}

#[test]
fn given_editor_fold_mark_and_pragma_region_markers_when_checked_then_each_marker_is_reported() {
    assert_eq!(
        vec![Some(1), Some(7), Some(8), Some(9), Some(10)],
        read_sorted_finding_lines(
            "// <editor-fold desc=\"Predicates\">\nimpl Invoice {\n    pub(crate) fn is_paid(&self) -> bool {\n        self.paid_at.is_some()\n    }\n}\n// </editor-fold>\n// MARK: - Transitions\n// #pragma region Helpers\n// #pragma endregion\n// Mark the row as read.\nstruct Invoice { paid_at: Option<u64> }\n",
        )
    );
}

#[test]
fn given_markdown_emphasis_at_the_start_of_a_comment_when_checked_then_only_headers_are_reported() {
    assert_eq!(
        vec![Some(4), Some(5)],
        read_sorted_finding_lines(
            "/// ***Never*** hold the guard across an await.\npub(crate) fn drain() {}\n// ***Always*** join the thread.\n// *** Helpers\n// Helpers ***\nfn work() {}\n",
        )
    );
}

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

#[test]
fn given_tdd_forbidden_comment_decorative_separator_when_linted_then_reports_comment_line() {
    let context = read_code_context("// ===== | =====\nfn work() {}\n");
    let findings = ForbiddenComment.check(&context);
    assert_eq!(1, findings.len());
    assert_eq!("ForbiddenComment", findings[0].finding.rule_id.0);
    assert_eq!(Some(1), findings[0].finding.location.line);
}

#[test]
fn given_section_header_with_pipe_when_checked_then_reports_header_but_not_table() {
    let lines =
        read_finding_lines("// === Helpers | More ===\n// | Column | Value |\nfn work() {}\n");
    assert_eq!(vec![Some(1)], lines);
}

#[test]
fn given_delimiters_outside_ascii_tables_when_checked_then_reports_each_line() {
    let lines = read_finding_lines(
        "// ─────────────────────────────────────\n// ═════════════════════════════════════\n// .....................................\nfn work() {}\n",
    );
    assert_eq!(vec![Some(1), Some(2), Some(3)], lines);
}

#[test]
fn given_one_sided_header_run_when_checked_then_reports_each_line() {
    let lines = read_finding_lines(
        "// ----- Helpers\n// Helpers -----\n// retry after a pause - then give up\nfn work() {}\n",
    );
    assert_eq!(vec![Some(1), Some(2)], lines);
}

#[test]
fn given_headers_framed_in_other_delimiters_when_checked_then_reports_each_header_not_prose() {
    let lines = read_finding_lines(
        "// ~~~ Helpers ~~~\n// ___ Helpers ___\n// +++ Helpers +++\n// offsets are in bytes (start + length)\nfn work() {}\n",
    );
    assert_eq!(vec![Some(1), Some(2), Some(3)], lines);
}

#[test]
fn given_star_guttered_block_header_when_checked_then_reports_its_line() {
    let lines = read_finding_lines(
        "/*\n * === Helpers ===\n */\nfn trim_name(raw: &str) -> &str {\n    raw.trim()\n}\n// === Other ===\n",
    );
    assert_eq!(vec![Some(2), Some(7)], lines);
}

#[test]
fn given_capitalised_region_prose_when_checked_then_reports_only_the_marker() {
    let lines = read_finding_lines(
        "/// Region code in ISO 3166-2 form, upper case.\nfn work() {}\n// Region codes go upper case: the carrier API rejects lower-case codes.\n// region Helpers\nfn other() {}\n",
    );
    assert_eq!(vec![Some(4)], lines);
}

#[test]
fn given_string_marker_after_closed_comment_when_checked_then_file_is_not_generated() {
    let lines = read_finding_lines(
        "/* handwritten */ const MARKER: &str = \"@generated\";\n// region Hidden\n",
    );
    assert_eq!(vec![Some(2)], lines);
}
