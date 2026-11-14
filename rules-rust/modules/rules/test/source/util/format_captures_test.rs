use crate::source::util::list_format_captures;

#[test]
fn given_format_string_with_inline_arguments_when_listed_then_each_captured_name_is_returned() {
    assert_eq!(
        list_format_captures("limit {LIMIT} of {max:>4} and {:>width$} {0} {}"),
        ["LIMIT", "max", "width"]
    );
}

#[test]
fn given_escaped_braces_and_unclosed_brace_when_listed_then_nothing_is_captured() {
    assert!(list_format_captures("{{LIMIT}} {{ {").is_empty());
}
