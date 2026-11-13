use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::format_module_path;
use crate::declaration::rule::InitializerVisitor;

pub(crate) struct InitializerOnDeclarationLine;

impl Rule for InitializerOnDeclarationLine {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .filter(|file| !file.is_generated)
            .flat_map(|file| {
                let module_path = format_module_path(&file.module.crate_name, &file.module.path);
                InitializerVisitor::new(file.path.clone(), module_path)
                    .collect_findings(&file.syntax)
            })
            .collect()
    }
}
