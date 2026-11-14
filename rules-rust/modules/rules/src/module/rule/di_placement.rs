use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::rule::PlacementTypeVisitor;
use crate::module::util::{build_shape_item_finding, build_shape_module_finding, find_kind_name};
use crate::source::model::SourceTree;

pub(crate) struct DiPlacement;

impl DiPlacement {
    fn constructs_dependency(
        context: &RuleContext,
        ordinal: usize,
        usage: &crate::source::model::WorkspaceUse,
    ) -> bool {
        let eligible = usage.source_index == ordinal
            && !usage.is_test
            && !usage.ambiguous
            && crate::module::util::is_construction_use(context, usage)
            && crate::module::util::workspace_implementation_under(
                context,
                usage.target_index,
                usage.target_item,
                &usage.cfg,
            );
        if !eligible {
            return false;
        }
        let index = &context.indexes[ordinal];
        !index.implementations.iter().any(|implementation| {
            implementation.file == usage.file
                && implementation.span.contains(&usage.span)
                && ordinal == usage.target_index
                && crate::module::util::owns_implementation(
                    index,
                    usage.target_item,
                    implementation,
                )
        })
    }

    fn holds_as_field(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        index
            .items
            .iter()
            .filter(|holder| find_kind_name(context, ordinal, holder.module) == "di")
            .any(|holder| {
                let syn::Item::Struct(structure) = &holder.syntax else {
                    return false;
                };
                structure.fields.iter().any(|field| {
                    PlacementTypeVisitor::collect_paths(&field.ty)
                        .into_iter()
                        .any(|path| {
                            let typed: syn::Type = syn::parse_quote!(#path);
                            index
                                .resolve_type_items(holder.module, &typed)
                                .contains(&identity)
                        })
                })
            })
    }

    fn is_owned_by_di(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let constructed_in_di = context.name_index().uses.iter().any(|usage| {
            usage.target_index == ordinal
                && usage.target_item == identity
                && find_kind_name(context, usage.source_index, usage.source_module) == "di"
                && crate::module::util::is_construction_use(context, usage)
        });
        constructed_in_di || Self::holds_as_field(context, ordinal, identity)
    }

    fn has_di_module(context: &RuleContext, crate_name: &str) -> bool {
        context
            .indexes
            .iter()
            .enumerate()
            .filter(|(_, index)| index.crate_name == crate_name)
            .any(|(ordinal, index)| {
                index.items.iter().any(|item| {
                    index.modules[item.module].tree == crate::source::model::SourceTree::Src
                        && crate::module::util::is_di_type(context, ordinal, item)
                })
            })
    }

    fn missing_module(context: &RuleContext, ordinal: usize) -> Vec<RuleFinding> {
        let index = &context.indexes[ordinal];
        let mut findings: Vec<_> = index
            .items
            .iter()
            .enumerate()
            .filter(|(identity, item)| {
                index.modules[item.module].tree == crate::source::model::SourceTree::Src
                    && !crate::module::util::is_di_type(context, ordinal, item)
                    && crate::module::util::lifecycle_implementation(context, ordinal, *identity)
                    && !Self::is_owned_by_di(context, ordinal, *identity)
            })
            .map(|(_, item)| {
                build_shape_item_finding(
                    index,
                    item,
                    "DiPlacement",
                    "a resource with lifecycle has an owning DI module in its crate",
                )
            })
            .collect();
        if Self::has_di_module(context, &index.crate_name) {
            return findings;
        }
        if let Some(usage) = context
            .name_index()
            .uses
            .iter()
            .find(|usage| Self::constructs_dependency(context, ordinal, usage))
        {
            findings.push(crate::check::util::build_node_finding(
                "DiPlacement",
                &usage.file,
                usage.anchor,
                index.crate_name.clone(),
                "a crate that assembles behavioral dependencies has its own DI module".to_owned(),
            ));
        }
        findings
    }

    fn has_module_shape(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        let syn::Item::Struct(structure) = &index.items[identity].syntax else {
            return false;
        };
        let has_constructor = index.implementations.iter().any(|implementation| {
            implementation.syntax.trait_.is_none()
                && crate::module::util::owns_implementation(index, identity, implementation)
                && implementation.syntax.items.iter().any(
                    |member| matches!(member, syn::ImplItem::Fn(function) if function.sig.ident == "new"),
                )
        });
        let holds_module = structure.fields.iter().any(|field| {
            PlacementTypeVisitor::collect_paths(&field.ty)
                .iter()
                .any(|path| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident.to_string().ends_with("Module"))
                })
        });
        has_constructor
            || holds_module
            || crate::module::util::lifecycle_implementation(context, ordinal, identity)
    }

    fn only_contracts(context: &RuleContext, ordinal: usize) -> bool {
        let index = &context.indexes[ordinal];
        let data = index
            .items
            .iter()
            .any(|item| ["api", "model"].contains(&find_kind_name(context, ordinal, item.module)));
        data && index.items.iter().all(|item| {
            ["api", "model", "di"].contains(&find_kind_name(context, ordinal, item.module))
        }) && index
            .modules
            .iter()
            .filter(|module| module.parent == Some(0))
            .all(|module| {
                ["api", "model", "di"]
                    .contains(&module.module.path.last().map(String::as_str).unwrap_or(""))
            })
    }
}

impl Rule for DiPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                if item.name.ends_with("Module")
                    && index.modules[item.module].tree == SourceTree::Src
                    && find_kind_name(context, ordinal, item.module) != "di"
                    && Self::has_module_shape(context, ordinal, identity)
                {
                    findings.push(build_shape_item_finding(
                        index,
                        item,
                        "DiPlacement",
                        "dependency wiring structs live in di",
                    ));
                }
            }
            findings.extend(Self::missing_module(context, ordinal));
            if !Self::only_contracts(context, ordinal) {
                continue;
            }
            for module in &index.modules {
                if !module.is_file_module
                    && module.module.path.last().is_some_and(|name| name == "di")
                {
                    findings.extend(build_shape_module_finding(
                        context,
                        ordinal,
                        module.id,
                        "DiPlacement",
                        "a contracts-and-models-only crate has no dependency wiring module",
                    ));
                }
            }
        }
        findings
    }
}
