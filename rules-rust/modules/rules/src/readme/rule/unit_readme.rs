use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::readme::util::count_sentences;
use crate::workspace::model::Member;
use crate::workspace::model::TomlKind;
use crate::workspace::model::TomlValue;
use crate::workspace::model::Workspace;
use std::fs;

pub(crate) struct UnitReadme;

impl UnitReadme {
    fn build_finding(member: &Member, location: Location, message: String) -> RuleFinding {
        RuleFinding {
            finding: Finding {
                rule_id: RuleId(Self::RULE.to_owned()),
                location,
                subject: member.name.clone(),
                message,
            },
            anchor: Anchor::Crate(member.name.clone()),
        }
    }

    fn find_readme_value<'workspace>(
        workspace: &'workspace Workspace,
        member: &'workspace Member,
    ) -> Option<&'workspace TomlValue> {
        let value = member.manifest.root.find(&["package", "readme"])?;
        let inherits = value
            .find(&["workspace"])
            .and_then(TomlValue::as_bool)
            .unwrap_or(false);
        if inherits {
            return workspace
                .root_manifest
                .root
                .find(&["workspace", "package", "readme"]);
        }
        Some(value)
    }

    fn check_readme_key(workspace: &Workspace, member: &Member) -> Option<RuleFinding> {
        let value = Self::find_readme_value(workspace, member)?;
        let described = match &value.kind {
            TomlKind::String(path) if path == Self::README => return None,
            TomlKind::String(path) => format!("\"{path}\""),
            TomlKind::Boolean(true) => return None,
            _ => "false".to_owned(),
        };
        let location = Location {
            path: member.manifest.path.clone(),
            line: Some(value.line),
        };
        let message = format!(
            "`readme` is {described}: Cargo takes README.md as the package readme, so leave the key out or set it to \"README.md\""
        );
        Some(Self::build_finding(member, location, message))
    }

    fn read_inherited_readme(workspace: &Workspace, member: &Member) -> Option<String> {
        let declared = member.manifest.root.find(&["package", "readme"])?;
        let inherits = declared
            .find(&["workspace"])
            .and_then(TomlValue::as_bool)
            .unwrap_or(false);
        if !inherits {
            return None;
        }
        let shared = workspace
            .root_manifest
            .root
            .find(&["workspace", "package", "readme"])?;
        let path = shared.as_str()?;
        fs::read_to_string(workspace.root.join(path)).ok()
    }

    fn check_readme_text(workspace: &Workspace, member: &Member) -> Option<RuleFinding> {
        let Some(own_readme) = member.readme.as_ref() else {
            let location = Location {
                path: member.directory.clone(),
                line: None,
            };
            let message = format!(
                "Add a README.md of one sentence that states the responsibility of {}.",
                member.directory
            );
            return Some(Self::build_finding(member, location, message));
        };
        let inherited_readme = Self::read_inherited_readme(workspace, member);
        let sentences = count_sentences(inherited_readme.as_ref().unwrap_or(own_readme));
        if sentences == 1 {
            return None;
        }
        let location = Location {
            path: format!("{}/{}", member.directory, Self::README),
            line: None,
        };
        let message = format!(
            "README.md of {} has {sentences} sentences, expected one: a responsibility that does not fit one sentence is the signal to split the unit.",
            member.directory
        );
        Some(Self::build_finding(member, location, message))
    }

    const RULE: &str = "UnitReadme";
    const README: &str = "README.md";
}

impl Rule for UnitReadme {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .workspace
            .members
            .iter()
            .filter(|member| !member.directory.is_empty())
            .flat_map(|member| {
                [
                    Self::check_readme_key(&context.workspace, member),
                    Self::check_readme_text(&context.workspace, member),
                ]
            })
            .flatten()
            .collect()
    }
}
