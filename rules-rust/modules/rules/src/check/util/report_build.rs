use crate::check::model::AcceptedException;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::Report;
use std::cmp::Ordering;
use std::path::PathBuf;

fn compare_locations(left: &Location, right: &Location) -> Ordering {
    left.path
        .cmp(&right.path)
        .then(left.line.unwrap_or(0).cmp(&right.line.unwrap_or(0)))
}

pub(crate) fn build_report(
    mut violations: Vec<Finding>,
    mut accepted: Vec<AcceptedException>,
    target_directory: PathBuf,
) -> Report {
    violations.sort_by(|left, right| {
        compare_locations(&left.location, &right.location)
            .then(left.rule_id.cmp(&right.rule_id))
            .then(left.subject.cmp(&right.subject))
            .then(left.message.cmp(&right.message))
    });
    violations.dedup();
    accepted.sort_by(|left, right| {
        compare_locations(&left.location, &right.location)
            .then(left.rule_id.cmp(&right.rule_id))
            .then(left.subject.cmp(&right.subject))
            .then(left.reason.cmp(&right.reason))
    });
    accepted.dedup();
    Report {
        violations,
        accepted,
        target_directory,
    }
}
