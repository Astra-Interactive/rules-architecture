use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::family::util::find_primary_type;
use crate::source::model::CrateSources;
use crate::source::model::SourceFile;
use crate::source::util::find_node_span;
use crate::syntax::util::find_self_name;
use syn::spanned::Spanned;

pub(crate) struct ConstantPlacement;

impl ConstantPlacement {
    fn has_only_constants(name: &str, items: &[syn::Item]) -> bool {
        let implementations: Vec<&syn::ItemImpl> = items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Impl(value)
                    if find_self_name(&value.self_ty).as_deref() == Some(name) =>
                {
                    Some(value)
                }
                _ => None,
            })
            .collect();
        !implementations.is_empty()
            && implementations.iter().all(|value| {
                value.trait_.is_none()
                    && !value.items.is_empty()
                    && value
                        .items
                        .iter()
                        .all(|child| matches!(child, syn::ImplItem::Const(_)))
            })
    }

    fn is_constant_holder(item: &syn::Item, items: &[syn::Item]) -> bool {
        match item {
            syn::Item::Mod(module) => module.content.as_ref().is_some_and(|(_, children)| {
                !children.is_empty()
                    && children
                        .iter()
                        .all(|child| matches!(child, syn::Item::Const(_)))
            }),
            syn::Item::Struct(value) if value.fields.is_empty() => {
                Self::has_only_constants(&value.ident.to_string(), items)
            }
            syn::Item::Enum(value) if value.variants.is_empty() => {
                Self::has_only_constants(&value.ident.to_string(), items)
            }
            _ => false,
        }
    }

    fn check_file(file: &SourceFile, sources: &CrateSources) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        let primary = find_primary_type(file, sources);
        let has_primary_data = matches!(primary, Some(syn::Item::Struct(_) | syn::Item::Enum(_)));
        let type_count = file
            .syntax
            .items
            .iter()
            .filter(|item| {
                matches!(
                    item,
                    syn::Item::Struct(_)
                        | syn::Item::Enum(_)
                        | syn::Item::Trait(_)
                        | syn::Item::Union(_)
                )
            })
            .count();
        for item in &file.syntax.items {
            let top_constant = has_primary_data && matches!(item, syn::Item::Const(_));
            let beside_type = type_count > usize::from(!matches!(item, syn::Item::Mod(_)));
            let holder = beside_type && Self::is_constant_holder(item, &file.syntax.items);
            if top_constant || holder {
                findings.push(build_node_finding(
                    "ConstantOutsideImpl",
                    &file.path,
                    find_node_span(item.span()),
                    format_module_path(&file.module.crate_name, &file.module.path),
                    "Keep a type's constants at the end of its inherent impl (code-ordering.rs.md)"
                        .to_owned(),
                ));
            }
        }
        findings
    }
}

impl Rule for ConstantPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .crates
            .iter()
            .flat_map(|sources| {
                sources
                    .files
                    .iter()
                    .filter(|file| !file.is_generated)
                    .flat_map(|file| Self::check_file(file, sources))
            })
            .collect()
    }
}
