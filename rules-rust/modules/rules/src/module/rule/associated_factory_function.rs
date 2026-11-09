use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::util::{find_kind_name, is_model_item, owns_implementation};
use crate::source::model::IndexedImpl;
use crate::source::model::{CrateIndex, Resolution};
use crate::source::util::find_node_span;
use syn::spanned::Spanned;

pub(crate) struct AssociatedFactoryFunction;

impl AssociatedFactoryFunction {
    fn returns_model(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        value: &syn::Type,
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        let index = &context.indexes[ordinal];
        let syn::Type::Path(path) = value else {
            return false;
        };
        match index.resolve_path(module, &path.path) {
            Resolution::Internal(targets) => targets.iter().filter_map(|target| target.item.map(|identity| &index.items[identity])).any(|item| {
                is_model_item(context, ordinal, item) || matches!(&item.syntax, syn::Item::Type(alias) if Self::returns_model(context, ordinal, item.module, &alias.ty, depth + 1))
            }),
            resolution => {
                let wrapper = match resolution {
                    Resolution::External(name) => matches!(name.as_str(), "std::result::Result" | "core::result::Result" | "std::option::Option" | "core::option::Option"),
                    Resolution::Local => path.path.segments.len() == 1 && matches!(path.path.segments[0].ident.to_string().as_str(), "Result" | "Option"),
                    _ => false,
                };
                wrapper && path.path.segments.last().is_some_and(|segment| {
                    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else { return false; };
                    arguments.args.iter().find_map(|argument| if let syn::GenericArgument::Type(value) = argument { Some(value) } else { None }).is_some_and(|value| Self::returns_model(context, ordinal, module, value, depth + 1))
                })
            }
        }
    }

    fn returns_result(index: &CrateIndex, module: usize, value: &syn::Type, depth: usize) -> bool {
        if depth > 8 {
            return false;
        }
        let syn::Type::Path(path) = value else {
            return false;
        };
        match index.resolve_path(module, &path.path) {
            Resolution::Local => path.path.segments.len() == 1 && path.path.segments[0].ident == "Result",
            Resolution::External(path) => path == "std::result::Result" || path == "core::result::Result",
            Resolution::Internal(targets) => targets.iter().filter_map(|target| target.item.map(|identity| &index.items[identity])).any(|item| matches!(&item.syntax, syn::Item::Type(alias) if Self::returns_result(index, item.module, &alias.ty, depth + 1))),
            _ => false,
        }
    }

    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let model = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .iter()
            .any(|identity| {
                is_model_item(context, ordinal, &index.items[*identity])
                    && owns_implementation(index, *identity, implementation)
            });
        if !model || implementation.syntax.trait_.is_some() {
            return Vec::new();
        }
        implementation
            .syntax
            .items
            .iter()
            .filter_map(|item| {
                let syn::ImplItem::Fn(function) = item else {
                    return None;
                };
                let receiver = function
                    .sig
                    .inputs
                    .iter()
                    .any(|argument| matches!(argument, syn::FnArg::Receiver(_)));
                (!receiver && function.sig.ident != "new").then(|| {
                    build_node_finding(
                        "AssociatedFactoryFunction",
                        &implementation.file,
                        find_node_span(function.span()),
                        function.sig.ident.to_string(),
                        "A model has no associated factory other than new (arch-domain.rs.md)"
                            .to_owned(),
                    )
                })
            })
            .collect()
    }
}

impl Rule for AssociatedFactoryFunction {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for item in &index.items {
                let syn::Item::Fn(function) = &item.syntax else {
                    continue;
                };
                let invalid = function.sig.ident.to_string().starts_with("create_")
                    && matches!(&function.sig.output, syn::ReturnType::Type(_, value) if Self::returns_model(context, ordinal, item.module, value, 0))
                    && (find_kind_name(context, ordinal, item.module) != "util"
                        || !matches!(&function.sig.output, syn::ReturnType::Type(_, value) if Self::returns_result(index, item.module, value, 0)));
                if invalid {
                    findings.push(build_node_finding("AssociatedFactoryFunction", &item.file, item.span, item.name.clone(), "Create an aggregate with a create_* function returning Result in util (arch-domain.rs.md)".to_owned()));
                }
            }
            findings.extend(
                index
                    .implementations
                    .iter()
                    .flat_map(|implementation| Self::check_impl(context, ordinal, implementation)),
            );
        }
        findings
    }
}
