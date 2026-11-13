use crate::check::model::RuleContext;
use crate::module::util::find_kind_name;
use crate::source::model::CfgPredicate;
use std::collections::BTreeMap;
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Generics, Item, Token, Type, TypeParamBound, WherePredicate};

pub(crate) struct UtilContractParameterVisitor<'context, 'ast> {
    context: &'context RuleContext,
    ordinal: usize,
    module: usize,
    cfg: CfgPredicate,
    bounds: BTreeMap<String, Vec<&'ast syn::Path>>,
    found: bool,
}

impl<'context, 'ast> UtilContractParameterVisitor<'context, 'ast> {
    pub(crate) fn new(
        context: &'context RuleContext,
        ordinal: usize,
        module: usize,
        cfg: CfgPredicate,
        generics: &[&'ast Generics],
    ) -> Self {
        let mut bounds: BTreeMap<String, Vec<&'ast syn::Path>> = BTreeMap::new();
        for generic in generics {
            for parameter in generic.type_params() {
                bounds
                    .entry(parameter.ident.to_string())
                    .or_default()
                    .extend(Self::list_trait_paths(&parameter.bounds));
            }
            let predicates = generic
                .where_clause
                .iter()
                .flat_map(|clause| &clause.predicates);
            for predicate in predicates {
                let WherePredicate::Type(predicate) = predicate else {
                    continue;
                };
                let Type::Path(bounded) = &predicate.bounded_ty else {
                    continue;
                };
                let Some(name) = bounded.path.get_ident() else {
                    continue;
                };
                bounds
                    .entry(name.to_string())
                    .or_default()
                    .extend(Self::list_trait_paths(&predicate.bounds));
            }
        }
        Self {
            context,
            ordinal,
            module,
            cfg,
            bounds,
            found: false,
        }
    }

    pub(crate) fn names_api_trait(mut self, typed: &'ast Type) -> bool {
        self.visit_type(typed);
        self.found
    }

    fn list_trait_paths(
        bounds: &'ast Punctuated<TypeParamBound, Token![+]>,
    ) -> Vec<&'ast syn::Path> {
        bounds
            .iter()
            .filter_map(|bound| match bound {
                TypeParamBound::Trait(bound) => Some(&bound.path),
                _ => None,
            })
            .collect()
    }

    fn is_api_trait(&self, path: &syn::Path) -> bool {
        self.context
            .resolve_workspace_path(self.ordinal, self.module, path)
            .iter()
            .any(|target| {
                let Some(identity) = target.path.item else {
                    return false;
                };
                let declaration = &self.context.indexes[target.index].items[identity];
                matches!(declaration.syntax, Item::Trait(_))
                    && find_kind_name(self.context, target.index, declaration.module) == "api"
                    && target
                        .path
                        .cfg
                        .in_scope(target.index)
                        .and(&self.cfg)
                        .is_satisfiable()
            })
    }

    fn inspect_bound_paths(&mut self, paths: &[&'ast syn::Path]) {
        for path in paths {
            self.found |= self.is_api_trait(path);
            self.visit_path(path);
        }
    }

    fn inspect_bounds(&mut self, bounds: &'ast Punctuated<TypeParamBound, Token![+]>) {
        let paths = Self::list_trait_paths(bounds);
        self.inspect_bound_paths(&paths);
    }
}

impl<'ast> Visit<'ast> for UtilContractParameterVisitor<'_, 'ast> {
    fn visit_type_trait_object(&mut self, node: &'ast syn::TypeTraitObject) {
        self.inspect_bounds(&node.bounds);
    }

    fn visit_type_impl_trait(&mut self, node: &'ast syn::TypeImplTrait) {
        self.inspect_bounds(&node.bounds);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        let parameter = node
            .path
            .get_ident()
            .filter(|_| node.qself.is_none())
            .and_then(|name| self.bounds.remove(&name.to_string()));
        if let Some(paths) = parameter {
            self.inspect_bound_paths(&paths);
        }
        syn::visit::visit_type_path(self, node);
    }
}
