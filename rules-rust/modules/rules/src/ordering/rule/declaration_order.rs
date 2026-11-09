use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::ordering::model::FamilyOrder;
use crate::source::util::find_node_span;
use crate::syntax::util::find_self_name;
use crate::syntax::util::find_type_name;
use crate::syntax::util::is_constructor;
use std::collections::HashMap;
use syn::spanned::Spanned;

pub(crate) struct DeclarationOrder;

impl DeclarationOrder {
    fn find_family(item: &syn::Item) -> Option<String> {
        find_type_name(item)
            .map(ToString::to_string)
            .or_else(|| match item {
                syn::Item::Impl(value) => find_self_name(&value.self_ty),
                _ => None,
            })
    }

    fn check_impl(item: &syn::ItemImpl) -> Vec<proc_macro2::Span> {
        let mut findings = Vec::new();
        let mut has_function = false;
        let mut has_constant = false;
        let name = find_self_name(&item.self_ty).unwrap_or_default();
        let is_trait_impl = item.trait_.is_some();
        for child in &item.items {
            match child {
                syn::ImplItem::Fn(function) => {
                    let misplaced_constructor = if !is_trait_impl && has_function {
                        is_constructor(&function.sig, &name)
                    } else {
                        false
                    };
                    let has_inherent_constant = !is_trait_impl && has_constant;
                    if misplaced_constructor || has_inherent_constant {
                        findings.push(function.span());
                    }
                    has_function |= is_trait_impl || !is_constructor(&function.sig, &name);
                }
                syn::ImplItem::Const(constant) => {
                    if is_trait_impl && has_function {
                        findings.push(constant.span());
                    }
                    has_constant = true;
                }
                syn::ImplItem::Type(alias) if is_trait_impl && has_function => {
                    findings.push(alias.span())
                }
                _ => {}
            }
        }
        findings
    }

    fn update_family_order(
        item: &syn::Item,
        state: FamilyOrder,
        has_local_definition: bool,
        findings: &mut Vec<proc_macro2::Span>,
    ) -> FamilyOrder {
        if let syn::Item::Impl(value) = item {
            let precedes_definition = has_local_definition && !state.has_definition;
            let follows_trait_impl = value.trait_.is_none() && state.has_trait_impl;
            if precedes_definition || follows_trait_impl {
                findings.push(item.span());
            }
            findings.extend(Self::check_impl(value));
            return FamilyOrder {
                has_impl: true,
                has_trait_impl: state.has_trait_impl || value.trait_.is_some(),
                ..state
            };
        }
        if state.has_impl {
            findings.push(item.span());
        }
        FamilyOrder {
            has_definition: true,
            ..state
        }
    }

    fn check_items(items: &[syn::Item]) -> Vec<proc_macro2::Span> {
        let mut findings = Vec::new();
        let mut has_type_or_function = false;
        let mut has_function = false;
        let mut definitions = HashMap::new();
        let mut last_family = None;
        let mut closed = Vec::new();
        for item in items {
            if matches!(item, syn::Item::Const(_) | syn::Item::Static(_)) && has_type_or_function {
                findings.push(item.span());
            }
            if matches!(item, syn::Item::Fn(_)) {
                has_function = true;
                has_type_or_function = true;
                continue;
            }
            let Some(name) = Self::find_family(item) else {
                continue;
            };
            if has_function {
                findings.push(item.span());
            }
            let changes_family = last_family
                .as_ref()
                .is_some_and(|previous| previous != &name);
            if changes_family {
                closed.extend(last_family.take());
            }
            if closed.contains(&name) {
                findings.push(item.span());
            }
            last_family = Some(name.clone());
            let has_local_definition = items.iter().any(|candidate| {
                find_type_name(candidate).is_some_and(|ident| ident == name.as_str())
            });
            let state = definitions.remove(&name).unwrap_or(FamilyOrder {
                has_definition: false,
                has_impl: false,
                has_trait_impl: false,
            });
            let next = Self::update_family_order(item, state, has_local_definition, &mut findings);
            definitions.insert(name, next);
            has_type_or_function = true;
        }
        findings.dedup_by(|left, right| left.start() == right.start() && left.end() == right.end());
        findings
    }
}

impl Rule for DeclarationOrder {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| &sources.files).filter(|file| !file.is_generated && !file.module.path.is_empty()).flat_map(|file| {
            Self::check_items(&file.syntax.items).into_iter().map(|span| build_node_finding("DeclarationOrder", &file.path, find_node_span(span), format_module_path(&file.module.crate_name, &file.module.path), "Declare constants, complete type families and functions in dependency order (code-ordering.rs.md)".to_owned())).collect::<Vec<_>>()
        }).collect()
    }
}
