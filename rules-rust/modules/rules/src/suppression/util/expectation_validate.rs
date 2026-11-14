use crate::catalogue::model::CatalogueEntry;
use crate::catalogue::util::find_catalogue_entry;
use crate::catalogue::util::find_retired_rule;
use crate::check::model::Finding;
use crate::source::model::Expectation;
use crate::source::model::ExpectationPosition;
use crate::suppression::model::ExpectationSlot;
use crate::suppression::model::ValidatedExpectations;
use crate::suppression::util::build_meta_finding;
use std::collections::BTreeSet;

fn is_position_allowed(entry: &CatalogueEntry, position: &ExpectationPosition) -> bool {
    match position {
        ExpectationPosition::Node(_) => entry.anchors.node,
        ExpectationPosition::ModuleDeclaration { .. } => entry.anchors.node || entry.anchors.module,
        ExpectationPosition::ModuleInner(_)
        | ExpectationPosition::InlineModuleInner { .. }
        | ExpectationPosition::File => entry.anchors.module || entry.anchors.file,
    }
}

fn check_rule_id(expectation: &Expectation, rule_id: &str) -> Result<(), Finding> {
    let meta = |meta_rule: &str, message: String| {
        build_meta_finding(
            meta_rule,
            &expectation.file,
            expectation.line,
            rule_id.to_owned(),
            message,
        )
    };
    if let Some(retired) = find_retired_rule(rule_id) {
        let message = format!(
            "`{rule_id}` is retired in klibs-rules: use `{}`",
            retired.replacement
        );
        return Err(meta("RetiredRule", message));
    }
    let Some(entry) = find_catalogue_entry(rule_id) else {
        let message = format!("klibs-rules knows no rule `{rule_id}`");
        return Err(meta("UnknownRule", message));
    };
    if !is_position_allowed(&entry, &expectation.position) {
        let message =
            format!("an expectation at this position cannot hold any finding of `{rule_id}`");
        return Err(meta("MalformedExpectation", message));
    }
    Ok(())
}

pub(crate) fn validate_expectations(expectations: &[Expectation]) -> ValidatedExpectations {
    let mut sorted: Vec<&Expectation> = expectations.iter().collect();
    sorted.sort_by(|left, right| left.file.cmp(&right.file).then(left.line.cmp(&right.line)));
    let mut slots: Vec<ExpectationSlot> = Vec::new();
    let mut findings = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    for expectation in sorted {
        if let Some(malformation) = &expectation.malformation {
            let subject = expectation.rule_ids.join(", ");
            findings.push(build_meta_finding(
                "MalformedExpectation",
                &expectation.file,
                expectation.line,
                subject,
                malformation.clone(),
            ));
            continue;
        }
        for rule_id in &expectation.rule_ids {
            if let Err(finding) = check_rule_id(expectation, rule_id) {
                findings.push(finding);
                continue;
            }
            let key = format!(
                "{}\u{0}{:?}\u{0}{rule_id}",
                expectation.file, expectation.position
            );
            if !seen.insert(key) {
                findings.push(build_meta_finding(
                    "MalformedExpectation",
                    &expectation.file,
                    expectation.line,
                    rule_id.clone(),
                    format!("a second expectation of `{rule_id}` on the same node fulfils nothing"),
                ));
                continue;
            }
            slots.push(ExpectationSlot {
                rule_id: rule_id.clone(),
                file: expectation.file.clone(),
                line: expectation.line,
                position: expectation.position.clone(),
                reason: expectation.reason.clone(),
            });
        }
    }
    ValidatedExpectations { slots, findings }
}
