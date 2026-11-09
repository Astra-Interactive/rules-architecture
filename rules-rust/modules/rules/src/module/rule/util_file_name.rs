use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::check::util::format_module_path;

pub(crate) struct UtilFileName;

impl Rule for UtilFileName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| sources.modules.iter().filter(|module| {
            !sources.files.iter().any(|file| file.module == module.id && file.is_generated)
        })).filter_map(|module| {
            let declaration = module.declaration.as_ref()?;
            let name = module.id.path.last()?;
            if declaration.is_inline || !["utils", "helpers", "extensions"].contains(&name.as_str()) {
                return None;
            }
            Some(RuleFinding {
                finding: Finding {
                    rule_id: RuleId("UtilFileName".to_owned()),
                    location: Location { path: declaration.file.clone(), line: Some(declaration.span.start.line) },
                    subject: format_module_path(&module.id.crate_name, &module.id.path),
                    message: format!("`{name}.rs` does not name what its functions do (arch-package.rs.md, What goes in one file)"),
                },
                anchor: Anchor::Module { module: module.id.clone(), file: declaration.file.clone(), body_file: declaration.body_file.clone(), declaration: declaration.span },
            })
        }).collect()
    }
}
