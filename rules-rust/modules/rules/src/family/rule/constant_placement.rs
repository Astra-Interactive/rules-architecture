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
use crate::syntax::util::find_type_name;
use crate::syntax::util::to_snake_case;
use syn::spanned::Spanned;
use syn::visit::Visit;

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

    fn mentions(item: &syn::Item, name: &str) -> bool {
        let mut visitor = IdentifierVisitor { name, found: false };
        visitor.visit_item(item);
        visitor.found
    }

    fn has_owner_beside(file: &SourceFile, sources: &CrateSources, holder: &str) -> bool {
        let parent = &file.module.path[..file.module.path.len().saturating_sub(1)];
        sources
            .files
            .iter()
            .filter(|candidate| {
                candidate.path != file.path
                    && candidate.module.path.len() == file.module.path.len()
                    && candidate.module.path.starts_with(parent)
            })
            .flat_map(|candidate| &candidate.syntax.items)
            .any(|item| {
                let prefixed = find_type_name(item).is_some_and(|name| {
                    let name = name.to_string();
                    name != holder && holder.starts_with(&name)
                });
                prefixed || Self::mentions(item, holder)
            })
    }

    fn is_constant_file(file: &SourceFile) -> bool {
        let items = &file.syntax.items;
        !file.module.path.is_empty()
            && items.iter().any(|item| matches!(item, syn::Item::Const(_)))
            && items
                .iter()
                .all(|item| matches!(item, syn::Item::Const(_) | syn::Item::Use(_)))
    }

    fn has_owner_file_beside(file: &SourceFile, sources: &CrateSources) -> bool {
        let Some(module_name) = file.module.path.last() else {
            return false;
        };
        let parent = &file.module.path[..file.module.path.len() - 1];
        sources
            .files
            .iter()
            .filter(|candidate| {
                candidate.path != file.path
                    && candidate.module.path.len() == file.module.path.len()
                    && candidate.module.path.starts_with(parent)
            })
            .filter(|candidate| {
                candidate
                    .syntax
                    .items
                    .iter()
                    .any(|item| find_type_name(item).is_some())
            })
            .any(|candidate| {
                let is_prefixed = candidate.syntax.items.iter().any(|item| {
                    find_type_name(item).is_some_and(|name| {
                        let owner = to_snake_case(&name.to_string());
                        module_name
                            .strip_prefix(owner.as_str())
                            .is_some_and(|rest| rest.starts_with('_'))
                    })
                });
                let is_mentioned = candidate
                    .syntax
                    .items
                    .iter()
                    .any(|item| Self::mentions(item, module_name));
                is_prefixed || is_mentioned
            })
    }

    fn check_file(file: &SourceFile, sources: &CrateSources) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        if Self::is_constant_file(file) && Self::has_owner_file_beside(file, sources) {
            let first = file
                .syntax
                .items
                .iter()
                .find(|item| matches!(item, syn::Item::Const(_)));
            findings.extend(first.map(|item| {
                build_node_finding(
                    "ConstantOutsideImpl",
                    &file.path,
                    find_node_span(item.span()),
                    format_module_path(&file.module.crate_name, &file.module.path),
                    "Keep a type's constants at the end of its inherent impl (code-ordering.rs.md)"
                        .to_owned(),
                )
            }));
            return findings;
        }
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
            let own_file_holder = type_count == 1
                && find_type_name(item)
                    .is_some_and(|name| Self::has_owner_beside(file, sources, &name.to_string()));
            let holder = (beside_type || own_file_holder)
                && Self::is_constant_holder(item, &file.syntax.items);
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

struct IdentifierVisitor<'name> {
    name: &'name str,
    found: bool,
}

impl<'ast> Visit<'ast> for IdentifierVisitor<'_> {
    fn visit_ident(&mut self, ident: &'ast proc_macro2::Ident) {
        self.found |= ident == self.name;
    }
}
