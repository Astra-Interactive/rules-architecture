use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::{
    build_shape_item_finding, find_kind_name, list_shape_words, resolve_shape_type,
};
use crate::source::model::CrateIndex;
use syn::Item;

pub(crate) struct UtilShape;

impl UtilShape {
    fn holds_function(index: &CrateIndex, module: usize, file: &str) -> bool {
        index.items.iter().any(|item| {
            item.module == module && item.file == file && matches!(item.syntax, Item::Fn(_))
        })
    }
}

impl Rule for UtilShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                if context.is_shape_generated_file(&item.file) {
                    continue;
                }
                let data = matches!(
                    item.syntax,
                    Item::Struct(_)
                        | Item::Enum(_)
                        | Item::Union(_)
                        | Item::Type(_)
                        | Item::Const(_)
                        | Item::Static(_)
                );
                let file_constant = matches!(item.syntax, Item::Const(_))
                    && Self::holds_function(index, item.module, &item.file);
                let util = find_kind_name(context, ordinal, item.module) == "util";
                if data && !file_constant && util {
                    findings.push(build_shape_item_finding(index, item, "UtilShape", "util holds functions and extension implementations, never data declarations"));
                }
            }
            let extensions = list_shape_words(context, &index.crate_name, "extension-kinds");
            for implementation in &index.implementations {
                if context.is_shape_generated_file(&implementation.file) {
                    continue;
                }
                if implementation.syntax.trait_.is_some() {
                    continue;
                }
                let kind = find_kind_name(context, ordinal, implementation.module);
                if kind == "util" || extensions.iter().any(|word| word == kind) {
                    continue;
                }
                let misplaced = resolve_shape_type(
                    index,
                    implementation.module,
                    &implementation.syntax.self_ty,
                )
                .iter()
                .any(|target| {
                    let item = &index.items[*target];
                    item.file != implementation.file
                        && item.cfg.and(&implementation.cfg).is_satisfiable()
                });
                if misplaced {
                    findings.push(build_node_finding("UtilShape", &implementation.file, implementation.span, index.modules[implementation.module].module.path.join("::"), "an out-of-file inherent impl belongs in util or a configured extension kind".to_owned()));
                }
            }
        }
        findings
    }
}
