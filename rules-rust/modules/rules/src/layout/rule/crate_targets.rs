use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::workspace::model::{Member, Target};

pub(crate) struct CrateTargets;

impl CrateTargets {
    fn check_target(
        member: &Member,
        target: &Target,
        has_lib: bool,
        bins: usize,
    ) -> Option<RuleFinding> {
        let path = if member.directory.is_empty() {
            target.src_path.as_str()
        } else {
            target
                .src_path
                .strip_prefix(&format!("{}/", member.directory))
                .unwrap_or(&target.src_path)
        };
        let mut reasons = Vec::new();
        if target.is_lib_target() && path != "src/lib.rs" {
            reasons.push("a library target starts at src/lib.rs");
        }
        if target.is_bin() {
            if path != "src/main.rs" {
                reasons.push("a binary target starts at src/main.rs");
            }
            if has_lib {
                reasons.push("a crate is a library or a binary, never both");
            }
            if bins > 1 {
                reasons.push("several binaries are separate entry-point crates");
            }
        }
        if target.kinds.iter().any(|kind| kind == "test") && !path.starts_with("tests/") {
            reasons.push("a test target stays in the tests/ directory that TestMirror checks");
        }
        if path.starts_with("src/bin/") {
            reasons.push("src/bin/ is not used");
        }
        if reasons.is_empty() {
            return None;
        }
        Some(RuleFinding {
            finding: Finding {
                rule_id: RuleId("CrateTargets".to_owned()),
                location: Location {
                    path: target.src_path.clone(),
                    line: None,
                },
                subject: format!("{} {}", member.name, target.name),
                message: reasons.join("; "),
            },
            anchor: Anchor::Crate(member.name.clone()),
        })
    }
}

impl Rule for CrateTargets {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .members
            .iter()
            .flat_map(|member| {
                let has_lib = member.targets.iter().any(Target::is_lib_target);
                let bins = member
                    .targets
                    .iter()
                    .filter(|target| target.is_bin())
                    .count();
                member
                    .targets
                    .iter()
                    .filter_map(move |target| Self::check_target(member, target, has_lib, bins))
            })
            .collect()
    }
}
