use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::module::model::LifecycleFacts;
use crate::module::rule::LifecycleBodyVisitor;
use crate::module::util::{build_shape_item_finding, owns_implementation};
use crate::module::util::{is_di_type, lifecycle_implementation, resolved_di_type};
use syn::{ImplItem, Item};

pub(crate) struct DiModuleLifecycle;

impl DiModuleLifecycle {
    fn add_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        field: &syn::Field,
        facts: &mut LifecycleFacts,
    ) {
        let cfg = item
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                &field.attrs,
            ));
        if !cfg.is_satisfiable() {
            return;
        }
        let di = is_di_type(context, ordinal, item);
        let targets: Vec<_> = context
            .resolve_workspace_type(ordinal, item.module, &field.ty)
            .into_iter()
            .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
            .collect();
        let child = targets
            .iter()
            .any(|target| resolved_di_type(context, target));
        let scope = targets.iter().any(|target| {
            target.path.item.is_some_and(|identity| {
                context.indexes[target.index].items[identity].name == "TaskScope"
            })
        });
        let by_value = context
            .resolve_type_syntax_under(ordinal, item.module, &field.ty, &cfg)
            .iter()
            .all(|typed| matches!(typed.typed, syn::Type::Path(_)))
            && context
                .resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg)
                .is_empty();
        facts.invalid |= (di && child && !by_value) || (!di && scope && by_value);
        let Some(name) = &field.ident else {
            return;
        };
        if child && by_value {
            facts.owned.push(name.to_string());
        }
        if scope && by_value {
            facts.scopes.push(name.to_string());
        }
        facts.needs_lifecycle |= child
            && by_value
            && targets.iter().any(|target| {
                target.path.item.is_some_and(|identity| {
                    lifecycle_implementation(context, target.index, identity)
                })
            });
    }

    fn fields(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        structure: &syn::ItemStruct,
    ) -> LifecycleFacts {
        let item = &context.indexes[ordinal].items[identity];
        let mut facts = LifecycleFacts {
            owned: Vec::new(),
            scopes: Vec::new(),
            aborted: std::collections::BTreeSet::new(),
            invalid: false,
            needs_lifecycle: false,
        };
        for field in &structure.fields {
            Self::add_field(context, ordinal, item, field, &mut facts);
        }
        facts
    }

    fn parameters(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        method: &syn::ImplItemFn,
    ) -> bool {
        if method.sig.ident != "new" {
            return false;
        }
        method.sig.inputs.iter().any(|parameter| {
            let syn::FnArg::Typed(parameter) = parameter else { return false; };
            let module_type = context.resolve_workspace_type(ordinal, module, &parameter.ty).iter().any(|target| resolved_di_type(context, target));
            let shared = context.resolve_type_syntax(ordinal, module, &parameter.ty).iter().all(|typed| matches!(&typed.typed, syn::Type::Reference(reference) if reference.mutability.is_none()));
            module_type && !shared
        })
    }

    fn methods(context: &RuleContext, ordinal: usize, identity: usize, facts: &mut LifecycleFacts) {
        let index = &context.indexes[ordinal];
        for implementation in index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
        {
            for method in &implementation.syntax.items {
                let ImplItem::Fn(method) = method else {
                    continue;
                };
                if !implementation
                    .cfg
                    .and(&crate::source::model::CfgPredicate::from_attributes(
                        &method.attrs,
                    ))
                    .is_satisfiable()
                {
                    continue;
                }
                facts.invalid |= ["start", "reload", "stop"]
                    .iter()
                    .any(|name| method.sig.ident == *name);
                facts.invalid |= Self::parameters(context, ordinal, implementation.module, method);
                if !["on_enable", "on_disable"]
                    .iter()
                    .any(|name| method.sig.ident == *name)
                {
                    continue;
                }
                let name = method.sig.ident.to_string();
                let body = LifecycleBodyVisitor::inspect(
                    context,
                    ordinal,
                    implementation.module,
                    &method.block,
                    &name,
                    &facts.owned,
                    &facts.scopes,
                );
                facts.invalid |= body.invalid;
                facts.needs_lifecycle |= body.child_lifecycle;
                facts.aborted.extend(body.aborted);
            }
        }
    }

    fn check_item(context: &RuleContext, ordinal: usize, identity: usize) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let item = &index.items[identity];
        let Item::Struct(structure) = &item.syntax else {
            return None;
        };
        let mut facts = Self::fields(context, ordinal, identity, structure);
        if is_di_type(context, ordinal, item) {
            facts.invalid |=
                crate::module::util::has_module_parameter_cycle(context, ordinal, identity);
            Self::methods(context, ordinal, identity, &mut facts);
            facts.invalid |= facts
                .scopes
                .iter()
                .any(|scope| !facts.aborted.contains(scope));
            facts.invalid |= (!facts.scopes.is_empty() || facts.needs_lifecycle)
                && !lifecycle_implementation(context, ordinal, identity);
        }
        facts.invalid.then(|| build_shape_item_finding(index, item, "DiModuleLifecycle", "DI lifecycle is synchronous, manages only owned children, receives modules by shared reference, and aborts owned task scopes on disable"))
    }
}

impl Rule for DiModuleLifecycle {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                (0..index.items.len())
                    .filter_map(move |identity| Self::check_item(context, ordinal, identity))
            })
            .collect()
    }
}
