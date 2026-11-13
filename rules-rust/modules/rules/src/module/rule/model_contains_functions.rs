use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::module::rule::InvariantVisitor;
use crate::module::util::{is_model_item, list_try_from_impls, owns_implementation};
use crate::source::model::{CfgPredicate, CrateIndex, IndexedImpl, IndexedItem};
use crate::source::util::find_node_span;
use crate::syntax::util::is_accessor_adapter;
use syn::spanned::Spanned;

pub(crate) struct ModelContainsFunctions;

impl ModelContainsFunctions {
    fn reads_field(expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Field(field) => {
                matches!(field.base.as_ref(), syn::Expr::Path(path) if path.path.is_ident("self"))
            }
            syn::Expr::Reference(reference) => Self::reads_field(&reference.expr),
            syn::Expr::MethodCall(call) if is_accessor_adapter(call) => {
                Self::reads_field(&call.receiver)
            }
            syn::Expr::Return(value) => value
                .expr
                .as_ref()
                .is_some_and(|value| Self::reads_field(value)),
            _ => false,
        }
    }

    fn is_accessor(function: &syn::ImplItemFn) -> bool {
        function.sig.inputs.len() == 1
            && matches!(function.sig.inputs.first(), Some(syn::FnArg::Receiver(_)))
            && function.block.stmts.len() == 1
            && matches!(&function.block.stmts[0], syn::Stmt::Expr(expression, _) if Self::reads_field(expression))
    }

    fn invariant_cfg(
        implementations: &[&IndexedImpl],
        try_from_implementations: &[&IndexedImpl],
    ) -> CfgPredicate {
        let checked_by_try_from = try_from_implementations
            .iter()
            .map(|implementation| implementation.cfg.clone());
        CfgPredicate::Any(
            implementations
                .iter()
                .flat_map(|implementation| {
                    implementation
                        .syntax
                        .items
                        .iter()
                        .filter_map(|item| match item {
                            syn::ImplItem::Fn(function)
                                if function.sig.ident == "new"
                                    && InvariantVisitor::calls_assert(&function.block) =>
                            {
                                Some(
                                    implementation
                                        .cfg
                                        .and(&CfgPredicate::from_attributes(&function.attrs)),
                                )
                            }
                            _ => None,
                        })
                })
                .chain(checked_by_try_from)
                .collect(),
        )
    }

    fn allowed_method(
        function: &syn::ImplItemFn,
        implementation: &IndexedImpl,
        item: &IndexedItem,
        invariant: &CfgPredicate,
    ) -> bool {
        let active = implementation
            .cfg
            .and(&item.cfg)
            .and(&CfgPredicate::from_attributes(&function.attrs));
        if !active.is_satisfiable() {
            return true;
        }
        let without_invariant = active
            .and(&CfgPredicate::Not(Box::new(invariant.clone())))
            .is_satisfiable();
        !without_invariant && (function.sig.ident == "new" || Self::is_accessor(function))
    }

    fn check_type(index: &CrateIndex, identity: usize, item: &IndexedItem) -> Vec<RuleFinding> {
        let implementations = index
            .implementations
            .iter()
            .filter(|implementation| {
                implementation.file == item.file
                    && implementation.syntax.trait_.is_none()
                    && owns_implementation(index, identity, implementation)
            })
            .collect::<Vec<_>>();
        let invariant =
            Self::invariant_cfg(&implementations, &list_try_from_impls(index, identity));
        let mut findings = Vec::new();
        for (number, implementation) in implementations.iter().enumerate() {
            if implementations[..number].iter().any(|previous| {
                previous
                    .cfg
                    .and(&implementation.cfg)
                    .and(&item.cfg)
                    .is_satisfiable()
            }) {
                findings.push(build_node_finding(
                    "ModelContainsFunctions",
                    &item.file,
                    implementation.span,
                    item.name.clone(),
                    "A model has at most one inherent impl in its own file (arch-domain.rs.md)"
                        .to_owned(),
                ));
            }
            for child in &implementation.syntax.items {
                let allowed = match child {
                    syn::ImplItem::Const(_) => true,
                    syn::ImplItem::Fn(function) => {
                        Self::allowed_method(function, implementation, item, &invariant)
                    }
                    _ => false,
                };
                if !allowed {
                    findings.push(build_node_finding("ModelContainsFunctions", &item.file, find_node_span(child.span()), item.name.clone(), "A model's own file holds constants and invariant construction/accessors; rules live in util (arch-domain.rs.md)".to_owned()));
                }
            }
        }
        findings
    }
}

impl Rule for ModelContainsFunctions {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context
            .indexes
            .iter()
            .enumerate()
            .flat_map(|(ordinal, index)| {
                index
                    .items
                    .iter()
                    .enumerate()
                    .filter(move |(_, item)| is_model_item(context, ordinal, item))
                    .flat_map(|(identity, item)| Self::check_type(index, identity, item))
            })
            .collect()
    }
}
