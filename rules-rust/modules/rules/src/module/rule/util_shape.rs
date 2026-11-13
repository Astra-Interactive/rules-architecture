use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::UtilContractParameterVisitor;
use crate::module::util::{
    build_shape_item_finding, declares_type_elsewhere, find_kind_name, format_shape_subject,
    list_shape_words,
};
use crate::source::model::{CfgPredicate, CrateIndex, IndexedImpl, SourceTree};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, Item, Type};

pub(crate) struct UtilShape;

impl UtilShape {
    fn holds_function(index: &CrateIndex, module: usize, file: &str) -> bool {
        index.items.iter().any(|item| {
            item.module == module && item.file == file && matches!(item.syntax, Item::Fn(_))
        })
    }
}

impl UtilShape {
    fn names_api_trait<'ast>(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        cfg: CfgPredicate,
        generics: &[&'ast syn::Generics],
        inputs: impl Iterator<Item = &'ast FnArg>,
    ) -> bool {
        inputs
            .filter_map(|input| match input {
                FnArg::Typed(parameter) => Some(parameter.ty.as_ref()),
                FnArg::Receiver(_) => None,
            })
            .any(|typed: &'ast Type| {
                UtilContractParameterVisitor::new(context, ordinal, module, cfg.clone(), generics)
                    .names_api_trait(typed)
            })
    }

    fn check_function_parameters(context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let Item::Fn(function) = &item.syntax else {
                    continue;
                };
                if index.modules[item.module].tree != SourceTree::Src
                    || context.is_shape_generated_file(&item.file)
                    || find_kind_name(context, ordinal, item.module) != "util"
                {
                    continue;
                }
                let names_trait = Self::names_api_trait(
                    context,
                    ordinal,
                    item.module,
                    item.cfg.in_scope(ordinal),
                    &[&function.sig.generics],
                    function.sig.inputs.iter(),
                );
                if names_trait {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "UtilShape",
                        Self::CONTRACT_PARAMETER_MESSAGE,
                    ));
                }
            }
        }
        findings
    }

    fn check_extension_methods(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let is_extension = implementation.syntax.trait_.is_none()
            && index.modules[implementation.module].tree == SourceTree::Src
            && !context.is_shape_generated_file(&implementation.file)
            && find_kind_name(context, ordinal, implementation.module) == "util"
            && declares_type_elsewhere(index, implementation);
        if !is_extension {
            return Vec::new();
        }
        implementation
            .syntax
            .items
            .iter()
            .filter_map(|member| match member {
                ImplItem::Fn(method) => Some(method),
                _ => None,
            })
            .filter(|method| {
                Self::names_api_trait(
                    context,
                    ordinal,
                    implementation.module,
                    implementation.cfg.in_scope(ordinal),
                    &[&implementation.syntax.generics, &method.sig.generics],
                    method.sig.inputs.iter(),
                )
            })
            .map(|method| {
                build_node_finding(
                    "UtilShape",
                    &implementation.file,
                    find_node_span(method.span()),
                    format_shape_subject(
                        index,
                        implementation.module,
                        &method.sig.ident.to_string(),
                    ),
                    Self::CONTRACT_PARAMETER_MESSAGE.to_owned(),
                )
            })
            .collect()
    }

    fn check_method_parameters(context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .implementations
                    .iter()
                    .flat_map(move |implementation| {
                        Self::check_extension_methods(context, ordinal, implementation)
                    })
            })
            .collect()
    }

    const CONTRACT_PARAMETER_MESSAGE: &'static str = "a util function takes no contract of the project; a use case or an implementation calls it";
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
                let file_constant = matches!(item.syntax, Item::Const(_) | Item::Static(_))
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
                if declares_type_elsewhere(index, implementation) {
                    findings.push(build_node_finding("UtilShape", &implementation.file, implementation.span, index.modules[implementation.module].module.path.join("::"), "an out-of-file inherent impl belongs in util or a configured extension kind".to_owned()));
                }
            }
        }
        findings.extend(Self::check_function_parameters(context));
        findings.extend(Self::check_method_parameters(context));
        findings
    }
}
