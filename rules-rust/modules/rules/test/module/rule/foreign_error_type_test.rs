use crate::module::rule::ForeignErrorType;
use crate::test::declaration::util::assert_code_findings;

#[test]
fn given_inner_layer_functions_returning_a_foreign_error_when_checked_then_reports_each_function() {
    assert_code_findings(
        &ForeignErrorType,
        "ForeignErrorType",
        "mod model {\n    pub(crate) fn parse_count(raw: &str) -> Result<u32, std::num::ParseIntError> { raw.parse() }\n    pub(crate) enum CountError { Blank }\n    pub(crate) fn parse_total(raw: &str) -> Result<u32, CountError> { Err(CountError::Blank) }\n}\nmod usecase {\n    pub(crate) struct LoadUseCase;\n    impl LoadUseCase { pub(crate) fn invoke(&self) -> Result<String, std::io::Error> { std::fs::read_to_string(\"x\") } }\n    pub(crate) struct Count(u32);\n    impl std::str::FromStr for Count { type Err = std::num::ParseIntError; fn from_str(raw: &str) -> Result<Self, Self::Err> { raw.parse().map(Count) } }\n}\nmod util { pub(crate) fn read(path: &str) -> Result<String, Box<dyn std::error::Error>> { Ok(path.to_owned()) } }\nmod file { pub(crate) fn read(path: &str) -> Result<String, std::io::Error> { std::fs::read_to_string(path) } }",
        &[2, 8, 12],
    );
}
