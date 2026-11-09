use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::ordering::model::FunctionBody;
use crate::ordering::rule::CalledFunctionVisitor;
use crate::source::util::find_node_span;
use crate::syntax::util::find_self_name;
use crate::syntax::util::is_constructor;
use std::collections::HashSet;
use syn::spanned::Spanned;

pub(crate) struct CalleeBeforeCaller;

impl CalleeBeforeCaller {
    fn reaches(calls: &[HashSet<String>], names: &[String], start: usize, target: usize) -> bool {
        let mut pending = vec![start];
        let mut visited = HashSet::new();
        while let Some(index) = pending.pop() {
            if index == target {
                return true;
            }
            if !visited.insert(index) {
                continue;
            }
            pending.extend(
                names
                    .iter()
                    .enumerate()
                    .filter(|(_, name)| calls[index].contains(*name))
                    .map(|(position, _)| position),
            );
        }
        false
    }

    fn check_scope(
        functions: &[FunctionBody<'_>],
        own_name: Option<&str>,
    ) -> Vec<proc_macro2::Span> {
        let names: Vec<String> = functions
            .iter()
            .map(|function| function.signature.ident.to_string())
            .collect();
        let calls: Vec<HashSet<String>> = functions
            .iter()
            .map(|function| {
                CalledFunctionVisitor::new(own_name)
                    .collect_calls(function.signature, function.block)
            })
            .collect();
        let mut findings = Vec::new();
        for (caller, function) in functions.iter().enumerate() {
            if own_name.is_some_and(|name| is_constructor(function.signature, name)) {
                continue;
            }
            let has_later_callee =
                names
                    .iter()
                    .enumerate()
                    .skip(caller + 1)
                    .any(|(callee, name)| {
                        calls[caller].contains(name)
                            && !Self::reaches(&calls, &names, callee, caller)
                    });
            if has_later_callee {
                findings.push(function.signature.span());
            }
        }
        findings
    }

    fn check_items(items: &[syn::Item]) -> Vec<proc_macro2::Span> {
        let functions: Vec<FunctionBody<'_>> = items
            .iter()
            .filter_map(|item| match item {
                syn::Item::Fn(function) => Some(FunctionBody {
                    signature: &function.sig,
                    block: &function.block,
                }),
                _ => None,
            })
            .collect();
        let mut findings = Self::check_scope(&functions, None);
        for item in items {
            match item {
                syn::Item::Impl(value) => {
                    let functions: Vec<FunctionBody<'_>> = value
                        .items
                        .iter()
                        .filter_map(|item| match item {
                            syn::ImplItem::Fn(function) => Some(FunctionBody {
                                signature: &function.sig,
                                block: &function.block,
                            }),
                            _ => None,
                        })
                        .collect();
                    let name = find_self_name(&value.self_ty).unwrap_or_default();
                    findings.extend(Self::check_scope(&functions, Some(&name)));
                }
                syn::Item::Mod(module) => {
                    if let Some((_, children)) = &module.content {
                        findings.extend(Self::check_items(children));
                    }
                }
                _ => {}
            }
        }
        findings
    }
}

impl Rule for CalleeBeforeCaller {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.crates.iter().flat_map(|sources| &sources.files).filter(|file| !file.is_generated).flat_map(|file| {
            Self::check_items(&file.syntax.items).into_iter().map(|span| build_node_finding("CalleeBeforeCaller", &file.path, find_node_span(span), format_module_path(&file.module.crate_name, &file.module.path), "Declare a callee above its caller in the same scope (code-ordering.rs.md)".to_owned())).collect::<Vec<_>>()
        }).collect()
    }
}
