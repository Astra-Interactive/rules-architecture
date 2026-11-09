use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::format_module_path;
use crate::source::model::{DeclaredVisibility, ModuleDeclaration, SourceModule, SourceTree};

pub(crate) struct ModuleVisibility;

impl ModuleVisibility {
    fn select_expected_visibility(
        module: &SourceModule,
        declaration: &ModuleDeclaration,
        modules: &[SourceModule],
    ) -> DeclaredVisibility {
        if !declaration.is_inline {
            return DeclaredVisibility::Private;
        }
        if matches!(module.tree, SourceTree::Test | SourceTree::TestsIt) {
            return if declaration
                .use_visibilities
                .contains(&DeclaredVisibility::Crate)
            {
                DeclaredVisibility::Crate
            } else {
                DeclaredVisibility::Private
            };
        }
        let exports_public = modules.iter().any(|child| {
            child.id.path.starts_with(&module.id.path)
                && child.declaration.as_ref().is_some_and(|inner| {
                    inner.use_visibilities.contains(&DeclaredVisibility::Public)
                })
        });
        if exports_public {
            DeclaredVisibility::Public
        } else {
            DeclaredVisibility::Crate
        }
    }
}

impl Rule for ModuleVisibility {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for sources in &context.crates {
            for module in &sources.modules {
                let Some(declaration) = &module.declaration else {
                    continue;
                };
                let expected =
                    Self::select_expected_visibility(module, declaration, &sources.modules);
                if declaration.visibility == expected {
                    continue;
                }
                findings.push(RuleFinding {
                    finding: Finding {
                        rule_id: RuleId("ModuleVisibility".to_owned()),
                        location: Location { path: declaration.file.clone(), line: Some(declaration.span.start.line) },
                        subject: format_module_path(&module.id.crate_name, &module.id.path),
                        message: format!("module visibility must be {expected:?} (arch-package.rs.md, Modules and directories)"),
                    },
                    anchor: Anchor::Module { module: module.id.clone(), file: declaration.file.clone(), body_file: declaration.body_file.clone(), declaration: declaration.span },
                });
            }
        }
        findings
    }
}
