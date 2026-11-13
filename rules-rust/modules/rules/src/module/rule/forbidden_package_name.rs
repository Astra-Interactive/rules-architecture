use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::check::model::Finding;
use crate::check::model::Location;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::model::RuleId;
use crate::check::util::format_module_path;
use crate::source::model::ModuleDeclaration;
use crate::source::model::SourceModule;
use crate::source::model::SourceTree;

pub(crate) struct ForbiddenPackageName;

impl ForbiddenPackageName {
    fn find_reason(module: &SourceModule, declaration: &ModuleDeclaration) -> Option<String> {
        let name = module.id.path.last()?.as_str();
        let depth = module.id.path.len();
        if declaration.is_raw_identifier {
            return Some(format!(
                "`r#{name}` is a keyword: a module is named in the project's words"
            ));
        }
        if declaration.is_inline && Self::FORBIDDEN.contains(&name) {
            return Some(format!(
                "`{name}` is never a package name (arch-package.md, Concepts and kinds 10)"
            ));
        }
        if name == "prelude" {
            return Some("`prelude` is never a module name (arch-package.rs.md)".to_owned());
        }
        if Self::STANDARD_CRATES.contains(&name) {
            return Some(format!(
                "`{name}` hides the standard crate from the `use` lines of its parent (E0432)"
            ));
        }
        let is_test_block = depth == 1 && name == "test" && module.tree == SourceTree::Test;
        let is_fixture_block =
            depth == 1 && name == "fixture" && module.tree == SourceTree::Fixture;
        let is_block_name = name == "test" || name == "fixture";
        if is_block_name && !is_test_block && !is_fixture_block {
            return Some(format!("`{name}` is the name of the {name} block alone"));
        }
        let is_bin_under_src = depth == 1 && name == "bin" && module.tree == SourceTree::Src;
        if is_bin_under_src {
            return Some("`bin` directly under `src/` is a binary to Cargo".to_owned());
        }
        None
    }

    fn check_module(context: &RuleContext, module: &SourceModule) -> Option<RuleFinding> {
        let declaration = module.declaration.as_ref()?;
        let role = context
            .find_role_config(&module.id.crate_name)
            .and_then(|config| config.role);
        let technology_only = module
            .id
            .path
            .last()
            .is_some_and(|name| ["table", "entity", "database"].contains(&name.as_str()))
            && !matches!(
                role,
                Some(
                    crate::config::model::Role::Technology
                        | crate::config::model::Role::Platform
                        | crate::config::model::Role::CorePlatform
                )
            );
        let reason = Self::find_reason(module, declaration).or_else(|| technology_only.then(|| "table, entity and database belong only to a technology, platform or core-platform crate (arch-package.md)".to_owned()))?;
        Some(RuleFinding {
            finding: Finding {
                rule_id: RuleId(Self::RULE.to_owned()),
                location: Location {
                    path: declaration.file.clone(),
                    line: Some(declaration.span.start.line),
                },
                subject: format_module_path(&module.id.crate_name, &module.id.path),
                message: reason,
            },
            anchor: Anchor::Module {
                module: module.id.clone(),
                file: declaration.file.clone(),
                body_file: declaration.body_file.clone(),
                declaration: declaration.span,
            },
        })
    }

    const RULE: &str = "ForbiddenPackageName";
    const STANDARD_CRATES: [&str; 3] = ["std", "core", "alloc"];
    const FORBIDDEN: [&str; 30] = [
        "domain",
        "application",
        "infrastructure",
        "presentation",
        "ui",
        "controller",
        "entrypoint",
        "internal",
        "repository",
        "valueobject",
        "aggregate",
        "factory",
        "exception",
        "dto",
        "dao",
        "common",
        "shared",
        "helpers",
        "misc",
        "port",
        "adapter",
        "out",
        "spi",
        "inbound",
        "outbound",
        "validation",
        "prelude",
        "composable",
        "theme",
        "viewmodel",
    ];
}

impl Rule for ForbiddenPackageName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.modules)
            .filter_map(|module| Self::check_module(context, module))
            .collect()
    }
}
