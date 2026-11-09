use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};

pub(crate) struct FakePlacement;

impl Rule for FakePlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.workspace.members.iter().filter(|member| {
            member.directory.rsplit('/').next() == Some("fake")
                || member.name.rsplit('-').next() == Some("fake")
        }).map(|member| RuleFinding {
            finding: Finding {
                rule_id: RuleId("FakePlacement".to_owned()),
                location: Location { path: member.manifest.path.clone(), line: None },
                subject: member.name.clone(),
                message: "shared fakes belong to the owning crate's fixture tree behind its fake feature, never a separate crate (arch-package.rs.md, Tests)".to_owned(),
            },
            anchor: Anchor::Crate(member.name.clone()),
        }).collect()
    }
}
