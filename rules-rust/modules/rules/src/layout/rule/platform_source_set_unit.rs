use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::config::model::Role;
use crate::module::rule::PlatformAttributeVisitor;
use crate::source::util::list_manifest_target_values;
use crate::workspace::model::TomlValue;
use std::collections::BTreeSet;
use syn::visit::Visit;

pub(crate) struct PlatformSourceSetUnit;

impl PlatformSourceSetUnit {
    fn is_target_word(word: &str, own_cfg_words: &BTreeSet<String>) -> bool {
        Self::TARGET_WORDS.contains(&word)
            || (Self::DOMAIN_WORDS.contains(&word) && own_cfg_words.contains(word))
    }

    const TARGET_WORDS: [&str; 17] = [
        "unix",
        "windows",
        "wasm",
        "linux",
        "macos",
        "ios",
        "android",
        "freebsd",
        "netbsd",
        "openbsd",
        "illumos",
        "redox",
        "wasi",
        "emscripten",
        "tvos",
        "watchos",
        "visionos",
    ];

    const DOMAIN_WORDS: [&str; 4] = ["dragonfly", "solaris", "fuchsia", "haiku"];
}

impl Rule for PlatformSourceSetUnit {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for member in &context.workspace.members {
            let entry_point =
                context.project_config.crates.iter().any(|config| {
                    config.name == member.name && config.role == Some(Role::EntryPoint)
                });
            if entry_point {
                continue;
            }
            let mut visitor = PlatformAttributeVisitor {
                values: BTreeSet::new(),
            };
            for sources in context
                .crates
                .iter()
                .filter(|sources| sources.crate_name == member.name)
            {
                for file in &sources.files {
                    visitor.visit_file(&file.syntax);
                }
            }
            visitor.values.extend(
                member
                    .manifest
                    .root
                    .find(&["target"])
                    .into_iter()
                    .flat_map(TomlValue::entries)
                    .filter(|entry| entry.value.find(&["dependencies"]).is_some())
                    .flat_map(|entry| list_manifest_target_values(&entry.key)),
            );
            if visitor.values.is_empty() {
                continue;
            }
            let suffix = member.directory.rsplit('/').next().unwrap_or_default();
            if !Self::is_target_word(suffix, &visitor.values) {
                continue;
            }
            findings.push(RuleFinding {
                finding: Finding {
                    rule_id: RuleId("PlatformSourceSetUnit".to_owned()),
                    location: Location { path: member.manifest.path.clone(), line: None },
                    subject: member.name.clone(),
                    message: "platform code belongs under target cfgs in its owning crate, never in a target-named crate".to_owned(),
                },
                anchor: Anchor::Crate(member.name.clone()),
            });
        }
        findings
    }
}
