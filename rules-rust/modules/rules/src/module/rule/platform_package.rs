use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::format_module_path;
use crate::module::rule::PlatformAttributeVisitor;
use crate::source::util::list_manifest_target_values;
use std::collections::BTreeSet;
use syn::visit::Visit;

pub(crate) struct PlatformPackage;

impl PlatformPackage {
    fn collect_manifest_targets(context: &RuleContext, crate_name: &str) -> BTreeSet<String> {
        context
            .workspace
            .members
            .iter()
            .find(|member| member.name == crate_name)
            .and_then(|member| member.manifest.root.find(&["target"]))
            .into_iter()
            .flat_map(crate::workspace::model::TomlValue::entries)
            .filter(|entry| entry.value.find(&["dependencies"]).is_some())
            .flat_map(|entry| list_manifest_target_values(&entry.key))
            .collect()
    }

    const TARGET_WORDS: [&str; 21] = [
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
        "dragonfly",
        "solaris",
        "illumos",
        "fuchsia",
        "redox",
        "haiku",
        "wasi",
        "emscripten",
        "tvos",
        "watchos",
        "visionos",
    ];
}

impl Rule for PlatformPackage {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            let mut visitor = PlatformAttributeVisitor {
                values: BTreeSet::new(),
            };
            for file in &sources.files {
                visitor.visit_file(&file.syntax);
            }
            visitor
                .values
                .extend(Self::collect_manifest_targets(context, &sources.crate_name));
            if visitor.values.is_empty() {
                continue;
            }
            visitor
                .values
                .extend(Self::TARGET_WORDS.iter().map(|word| (*word).to_owned()));
            for module in &sources.modules {
                let Some(declaration) = &module.declaration else {
                    continue;
                };
                let is_platform_name = module
                    .id
                    .path
                    .last()
                    .is_some_and(|name| visitor.values.contains(name));
                if !is_platform_name {
                    continue;
                }
                findings.push(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("PlatformPackage".to_owned()),
                        location: Location { path: declaration.file.clone(), line: Some(declaration.span.start.line) },
                        subject: format_module_path(&module.id.crate_name, &module.id.path),
                        message: "platform code uses a target cfg at the same module path, never a platform module (arch-package.rs.md, Modules and directories)".to_owned(),
                    },
                    anchor: Anchor::Module { module: module.id.clone(), file: declaration.file.clone(), body_file: declaration.body_file.clone(), declaration: declaration.span },
                });
            }
        }
        findings
    }
}
