use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::util::is_di_type;
use crate::module::util::{build_shape_item_finding, find_kind_name};

pub(crate) struct DiModuleShape;

impl DiModuleShape {
    fn service_fields(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
    ) -> bool {
        let syn::Item::Struct(structure) = &item.syntax else {
            return false;
        };
        structure.fields.iter().any(|field| {
            let cfg = item
                .cfg
                .and(&crate::source::model::CfgPredicate::from_attributes(
                    &field.attrs,
                ));
            if !cfg.is_satisfiable() {
                return false;
            }
            let deferred = context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .iter()
                .any(|path| {
                    ["std", "core", "tokio", "once_cell"].contains(&path.package.as_str())
                        && path.segments.last().is_some_and(|name| {
                            ["OnceCell", "LazyCell", "LazyLock", "Lazy"].contains(&name.as_str())
                        })
                });
            let concrete = context
                .resolve_workspace_type(ordinal, item.module, &field.ty)
                .iter()
                .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        Self::service_identity(context, target.index, identity)
                    })
                });
            let shared = context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .iter()
                .any(|path| {
                    ["std", "alloc"].contains(&path.package.as_str())
                        && path
                            .segments
                            .last()
                            .is_some_and(|name| ["Arc", "OnceLock"].contains(&name.as_str()))
                });
            deferred
                || (concrete
                    && !shared
                    && !context
                        .resolve_workspace_type(ordinal, item.module, &field.ty)
                        .iter()
                        .any(|target| crate::module::util::resolved_di_type(context, target)))
        })
    }

    fn service_identity(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        item.name != "TaskScope"
            && (crate::module::util::workspace_implementation(context, ordinal, identity)
                || (matches!(item.syntax, syn::Item::Trait(_))
                    && find_kind_name(context, ordinal, item.module) == "api"))
    }

    fn constructs_outside_new(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let index = &context.indexes[ordinal];
        context.name_index().uses.iter().any(|usage| {
            let eligible = usage.source_index == ordinal
                && !usage.is_test
                && !usage.ambiguous
                && usage.function.as_deref().is_some_and(|name| name != "new")
                && crate::module::util::is_construction_use(context, usage)
                && crate::module::util::workspace_implementation(
                    context,
                    usage.target_index,
                    usage.target_item,
                );
            eligible
                && index.implementations.iter().any(|implementation| {
                    implementation.file == usage.file
                        && implementation.span.contains(&usage.span)
                        && crate::module::util::owns_implementation(index, identity, implementation)
                })
        })
    }

    fn item_generics(item: &crate::source::model::IndexedItem) -> Option<&syn::Generics> {
        match &item.syntax {
            syn::Item::Struct(value) => Some(&value.generics),
            syn::Item::Enum(value) => Some(&value.generics),
            _ => None,
        }
    }
    fn generic_contract(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        generics: &syn::Generics,
    ) -> bool {
        let mut bounds: Vec<_> = generics
            .type_params()
            .flat_map(|parameter| parameter.bounds.iter())
            .collect();
        if let Some(clause) = &generics.where_clause {
            for predicate in &clause.predicates {
                if let syn::WherePredicate::Type(predicate) = predicate {
                    bounds.extend(&predicate.bounds);
                }
            }
        }
        bounds.into_iter().any(|bound| {
            let syn::TypeParamBound::Trait(bound) = bound else {
                return false;
            };
            context
                .resolve_workspace_path(ordinal, item.module, &bound.path)
                .iter()
                .any(|target| {
                    target.path.item.is_some_and(|identity| {
                        matches!(
                            context.indexes[target.index].items[identity].syntax,
                            syn::Item::Trait(_)
                        )
                    })
                })
        })
    }
}

impl Rule for DiModuleShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                let di = is_di_type(context, ordinal, item);
                let shape = di && !matches!(item.syntax, syn::Item::Struct(_));
                let kind = find_kind_name(context, ordinal, item.module);
                let relevant =
                    di || kind == "usecase" || context.is_entry_kind(&index.crate_name, kind);
                let bound = relevant
                    && Self::item_generics(item).is_some_and(|generics| {
                        Self::generic_contract(context, ordinal, item, generics)
                    });
                let service = di
                    && (Self::service_fields(context, ordinal, item)
                        || Self::constructs_outside_new(context, ordinal, identity));
                if shape || bound || service {
                    findings.push(build_shape_item_finding(index, item, "DiModuleShape", "a DI module is a struct; DI, use cases and entry-point types receive trait objects instead of workspace-contract generic parameters"));
                }
            }
        }
        findings
    }
}
