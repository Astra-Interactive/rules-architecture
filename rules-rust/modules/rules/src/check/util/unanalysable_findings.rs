use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::source::model::CrateSources;
use crate::source::model::UnanalysableSite;

pub(crate) fn build_unanalysable_findings(crates: &[CrateSources]) -> Vec<RuleFinding> {
    crates
        .iter()
        .flat_map(|sources| {
            sources.unanalysable.iter().map(|unanalysable| {
                let (location, anchor) = match &unanalysable.site {
                    UnanalysableSite::Node { file, span } => (
                        Location {
                            path: file.clone(),
                            line: Some(span.start.line),
                        },
                        Anchor::Node {
                            file: file.clone(),
                            span: *span,
                        },
                    ),
                    UnanalysableSite::Crate { path } => (
                        Location {
                            path: path.clone(),
                            line: None,
                        },
                        Anchor::Crate(sources.crate_name.clone()),
                    ),
                };
                RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("Unanalysable".to_owned()),
                        location,
                        subject: unanalysable.subject.clone(),
                        message: unanalysable.message.clone(),
                    },
                    anchor,
                }
            })
        })
        .collect()
}
