use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::format_module_path;
use crate::tuple::rule::TupleTypeVisitor;

pub(crate) struct AnonymousTuple;

impl Rule for AnonymousTuple {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<_> = context
            .crates
            .iter()
            .flat_map(|sources| &sources.files)
            .flat_map(|file| {
                let module_path = format_module_path(&file.module.crate_name, &file.module.path);
                TupleTypeVisitor::new(file.path.clone(), module_path).collect_findings(&file.syntax)
            })
            .collect();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                for span in
                    crate::tuple::rule::TupleBindingVisitor::new(context, ordinal, item.module)
                        .collect_item(&item.syntax)
                {
                    findings.push(crate::check::util::build_node_finding("AnonymousTuple", &item.file, span, format_module_path(&index.crate_name, &index.modules[item.module].module.path), "destructure a tuple where it appears instead of binding anonymous domain data to one name".to_owned()));
                }
            }
            for implementation in &index.implementations {
                for span in crate::tuple::rule::TupleBindingVisitor::new(
                    context,
                    ordinal,
                    implementation.module,
                )
                .collect_impl(&implementation.syntax)
                {
                    findings.push(crate::check::util::build_node_finding("AnonymousTuple", &implementation.file, span, format_module_path(&index.crate_name, &index.modules[implementation.module].module.path), "destructure a tuple where it appears instead of binding anonymous domain data to one name".to_owned()));
                }
            }
        }
        findings
    }
}
