use crate::check::api::Rule;
use crate::test::check::usecase::read_case_context;

pub(crate) fn assert_resolution_cases(rule: &dyn Rule, id: &str, expected: &str) {
    for (case, expected) in [("fail-resolution", expected), ("pass-resolution", "")] {
        let context = read_case_context(id, case);
        assert!(
            context
                .indexes
                .iter()
                .all(|index| index.unanalysable.is_empty()),
            "{id}/{case} must be fully resolvable"
        );
        let actual = rule
            .check(&context)
            .iter()
            .map(|finding| {
                format!(
                    "{}:{}\t{}\n",
                    finding.finding.location.path,
                    finding.finding.location.line.unwrap_or(0),
                    finding.finding.subject
                )
            })
            .collect::<String>();
        assert_eq!(actual, expected, "{id}/{case}");
    }
}
