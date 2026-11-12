use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::ErrorPlacement;
use crate::module::util::find_kind_name;
use crate::source::model::{CrateIndex, NodeSpan, SourceTree};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) struct ForeignErrorType;

impl ForeignErrorType {
    fn is_inner_layer(context: &RuleContext, ordinal: usize, module: usize) -> bool {
        let index = &context.indexes[ordinal];
        let kind = find_kind_name(context, ordinal, module);
        index.modules[module].tree == SourceTree::Src && Self::INNER_KINDS.contains(&kind)
    }

    fn returns_foreign_error(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        output: &syn::ReturnType,
    ) -> bool {
        let Some(error) = ErrorPlacement::find_result_error(context, ordinal, module, output)
        else {
            return false;
        };
        context
            .resolve_workspace_type(ordinal, module, &error)
            .is_empty()
            && !context
                .resolve_outer_external_type_paths(ordinal, module, &error)
                .is_empty()
    }

    fn build_finding(
        index: &CrateIndex,
        module: usize,
        file: &str,
        anchor: NodeSpan,
        name: &syn::Ident,
    ) -> RuleFinding {
        let mut path = index.modules[module].module.path.clone();
        if index.modules[module].is_file_module {
            path.pop();
        }
        path.push(name.to_string());
        build_node_finding(
            "ForeignErrorType",
            file,
            anchor,
            format!("{}::{}", index.crate_name.replace('-', "_"), path.join("::")),
            "inner layers return the concept's own error family, not a foreign error type (error-handling.rs.md)"
                .to_owned(),
        )
    }

    fn check_functions(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        index
            .items
            .iter()
            .filter(|item| Self::is_inner_layer(context, ordinal, item.module))
            .filter_map(|item| match &item.syntax {
                syn::Item::Fn(function)
                    if Self::returns_foreign_error(
                        context,
                        ordinal,
                        item.module,
                        &function.sig.output,
                    ) =>
                {
                    let ident = &function.sig.ident;
                    Some(Self::build_finding(
                        index,
                        item.module,
                        &item.file,
                        item.span,
                        ident,
                    ))
                }
                _ => None,
            })
            .collect()
    }

    fn check_methods(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut findings = Vec::new();
        for implementation in index.implementations.iter().filter(|implementation| {
            implementation.syntax.trait_.is_none()
                && Self::is_inner_layer(context, ordinal, implementation.module)
        }) {
            let module = implementation.module;
            for member in &implementation.syntax.items {
                let syn::ImplItem::Fn(method) = member else {
                    continue;
                };
                if Self::returns_foreign_error(context, ordinal, module, &method.sig.output) {
                    let anchor = find_node_span(method.span());
                    let ident = &method.sig.ident;
                    let file = &implementation.file;
                    findings.push(Self::build_finding(index, module, file, anchor, ident));
                }
            }
        }
        findings
    }

    const INNER_KINDS: [&'static str; 3] = ["model", "usecase", "util"];
}

impl Rule for ForeignErrorType {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        (0..context.indexes.len())
            .flat_map(|ordinal| {
                let mut findings = Self::check_functions(context, ordinal);
                findings.extend(Self::check_methods(context, ordinal));
                findings
            })
            .collect()
    }
}
