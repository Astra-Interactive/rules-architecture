use crate::check::model::RuleFinding;
use std::collections::BTreeSet;

const MIRROR_BLOCKS: [&str; 2] = ["test", "fixture"];

fn build_key(subject: &str, message: &str) -> String {
    format!("{subject}\n{message}")
}

fn find_source_twin(subject: &str) -> Option<String> {
    let mut segments = subject.split("::");
    let crate_name = segments.next()?;
    let block = segments.next()?;
    if !MIRROR_BLOCKS.contains(&block) {
        return None;
    }
    let path: Vec<&str> = segments.collect();
    if path.is_empty() {
        return None;
    }
    Some(format!("{crate_name}::{}", path.join("::")))
}

pub(crate) fn drop_mirrored_findings(findings: Vec<RuleFinding>) -> Vec<RuleFinding> {
    let reported: BTreeSet<String> = findings
        .iter()
        .map(|finding| build_key(&finding.finding.subject, &finding.finding.message))
        .collect();
    findings
        .into_iter()
        .filter(|finding| {
            let Some(twin) = find_source_twin(&finding.finding.subject) else {
                return true;
            };
            !reported.contains(&build_key(&twin, &finding.finding.message))
        })
        .collect()
}
