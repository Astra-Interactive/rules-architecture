use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::source::util::find_node_span;
use crate::syntax::util::find_self_name;
use std::collections::HashSet;
use syn::spanned::Spanned;

pub(crate) struct StructDefaultValue;

impl StructDefaultValue {
    fn derives(attributes: &[syn::Attribute], name: &str) -> bool {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .any(|attribute| {
                attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| {
                        paths.iter().any(|path| {
                            path.segments
                                .last()
                                .is_some_and(|segment| segment.ident == name)
                        })
                    })
            })
    }

    fn is_self_type(value: &syn::Type, name: &str) -> bool {
        matches!(value, syn::Type::Path(path) if path.path.is_ident("Self") || path.path.is_ident(name))
    }

    fn has_empty_new(items: &[syn::Item], name: &str) -> bool {
        items.iter().any(|item| {
            let syn::Item::Impl(value) = item else {
                return false;
            };
            if value.trait_.is_some() {
                return false;
            }
            if find_self_name(&value.self_ty).as_deref() != Some(name) {
                return false;
            }
            value.items.iter().any(|child| {
                let syn::ImplItem::Fn(function) = child else {
                    return false;
                };
                if function.sig.ident != "new" || !function.sig.inputs.is_empty() {
                    return false;
                }
                let syn::ReturnType::Type(_, value) = &function.sig.output else {
                    return false;
                };
                Self::is_self_type(value, name)
            })
        })
    }

    fn delegates_to_new(value: &syn::ItemImpl, name: &str) -> bool {
        value.items.iter().any(|item| {
            let syn::ImplItem::Fn(function) = item else {
                return false;
            };
            if function.sig.ident != "default" || function.block.stmts.len() != 1 {
                return false;
            }
            let syn::Stmt::Expr(syn::Expr::Call(call), None) = &function.block.stmts[0] else {
                return false;
            };
            let syn::Expr::Path(path) = &*call.func else {
                return false;
            };
            let segments = &path.path.segments;
            if !call.args.is_empty() || segments.len() != 2 {
                return false;
            }
            let has_owner = segments[0].ident == "Self" || segments[0].ident == name;
            let has_constructor = segments[1].ident == "new";
            has_owner && has_constructor
        })
    }

    fn requires_explicit_values(
        attributes: &[syn::Attribute],
        items: &[syn::Item],
        name: &str,
    ) -> bool {
        if !Self::derives(attributes, "Default") {
            return false;
        }
        if Self::derives(attributes, "Deserialize") {
            return false;
        }
        !Self::has_empty_new(items, name)
    }

    fn check_items(items: &[syn::Item], own_types: &HashSet<String>) -> Vec<proc_macro2::Span> {
        let mut findings = Vec::new();
        for item in items {
            match item {
                syn::Item::Struct(value) => {
                    if Self::requires_explicit_values(&value.attrs, items, &value.ident.to_string())
                    {
                        findings.push(value.span());
                    }
                }
                syn::Item::Enum(value) => {
                    if Self::requires_explicit_values(&value.attrs, items, &value.ident.to_string())
                    {
                        findings.push(value.span());
                    }
                }
                syn::Item::Impl(value)
                    if value.trait_.as_ref().is_some_and(|(path, _)| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == "Default")
                    }) =>
                {
                    let Some(name) = find_self_name(&value.self_ty) else {
                        continue;
                    };
                    if !own_types.contains(&name) {
                        continue;
                    }
                    let deserialized = items.iter().any(|candidate| match candidate {
                        syn::Item::Struct(value) => {
                            value.ident == name && Self::derives(&value.attrs, "Deserialize")
                        }
                        syn::Item::Enum(value) => {
                            value.ident == name && Self::derives(&value.attrs, "Deserialize")
                        }
                        _ => false,
                    });
                    let constructor_default = if Self::has_empty_new(items, &name) {
                        Self::delegates_to_new(value, &name)
                    } else {
                        false
                    };
                    if !deserialized && !constructor_default {
                        findings.push(value.span());
                    }
                }
                syn::Item::Mod(module) => {
                    if let Some((_, children)) = &module.content {
                        findings.extend(Self::check_items(children, own_types));
                    }
                }
                _ => {}
            }
        }
        findings
    }

    fn collect_types(items: &[syn::Item], names: &mut HashSet<String>) {
        for item in items {
            match item {
                syn::Item::Struct(value) => {
                    names.insert(value.ident.to_string());
                }
                syn::Item::Enum(value) => {
                    names.insert(value.ident.to_string());
                }
                syn::Item::Mod(module) => {
                    if let Some((_, children)) = &module.content {
                        Self::collect_types(children, names);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Rule for StructDefaultValue {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| {
            let mut own_types = HashSet::new();
            for file in &sources.files { Self::collect_types(&file.syntax.items, &mut own_types); }
            sources.files.iter().flat_map(|file| Self::check_items(&file.syntax.items, &own_types).into_iter().map(|span| build_node_finding("StructDefaultValue", &file.path, find_node_span(span), format_module_path(&file.module.crate_name, &file.module.path), "Pass struct and enum values explicitly instead of providing Default (struct-default-values.rs.md)".to_owned())).collect::<Vec<_>>()).collect::<Vec<_>>()
        }).collect()
    }
}
