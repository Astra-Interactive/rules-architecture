use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleId;

pub(crate) fn build_meta_finding(
    rule_id: &str,
    path: &str,
    line: u32,
    subject: String,
    message: String,
) -> Finding {
    Finding {
        rule_id: RuleId(rule_id.to_owned()),
        location: Location {
            path: path.to_owned(),
            line: Some(line),
        },
        subject,
        message,
    }
}
