use crate::check::model::RuleContext;
use std::collections::BTreeMap;
use syn::fold::{self, Fold};

pub(crate) struct AliasSyntaxFolder<'a> {
    pub(crate) context: &'a RuleContext,
    pub(crate) index: usize,
    pub(crate) module: usize,
    pub(crate) types: BTreeMap<String, syn::Type>,
    pub(crate) lifetimes: BTreeMap<String, syn::Lifetime>,
    pub(crate) constants: BTreeMap<String, syn::Expr>,
    pub(crate) incomplete: bool,
    pub(crate) cfg: crate::source::model::CfgPredicate,
}

impl<'a> AliasSyntaxFolder<'a> {
    pub(crate) fn new(context: &'a RuleContext, index: usize, module: usize) -> Self {
        Self {
            context,
            index,
            module,
            types: BTreeMap::new(),
            lifetimes: BTreeMap::new(),
            constants: BTreeMap::new(),
            incomplete: false,
            cfg: crate::source::model::CfgPredicate::All(Vec::new()),
        }
    }

    fn qualify(&mut self, path: syn::Path) -> syn::Path {
        let targets = self
            .context
            .resolve_canonical_type_path(self.index, self.module, &path)
            .into_iter()
            .filter(|target| {
                let target_cfg = target.path.cfg.in_scope(target.index);
                target_cfg
                    .and(&self.cfg.in_scope(self.index))
                    .is_satisfiable()
            })
            .collect::<Vec<_>>();
        let identities = targets
            .iter()
            .filter_map(|target| {
                target.path.item.map(|item| {
                    (
                        &self.context.indexes[target.index].crate_name,
                        &self.context.indexes[target.index].modules[target.path.module]
                            .module
                            .path,
                        &self.context.indexes[target.index].items[item].name,
                    )
                })
            })
            .collect::<std::collections::BTreeSet<_>>();
        if identities.len() > 1 {
            self.incomplete = true;
            return path;
        }
        let canonical = if let Some(target) = targets.first() {
            let index = &self.context.indexes[target.index];
            let Some(item) = target.path.item else {
                return path;
            };
            let declaration = &index.items[item];
            format!(
                "{}::{}",
                crate::check::util::format_module_path(
                    &index.crate_name,
                    &index.modules[declaration.module].module.path
                ),
                declaration.name
            )
        } else if let Some(external) =
            self.context
                .canonical_external_type_path(self.index, self.module, &path)
        {
            external.format_path().replace('-', "_")
        } else {
            return path;
        };
        let Ok(mut qualified) = syn::parse_str::<syn::Path>(&format!("::{canonical}")) else {
            self.incomplete = true;
            return path;
        };
        if let (Some(last), Some(original)) = (qualified.segments.last_mut(), path.segments.last())
        {
            last.arguments = original.arguments.clone();
        }
        qualified
    }

    fn bind_type(
        &mut self,
        parameter: &syn::TypeParam,
        argument: Option<&syn::GenericArgument>,
        caller: &mut Self,
    ) {
        let value = match argument {
            Some(syn::GenericArgument::Type(typed)) => Some(caller.fold_type(typed.clone())),
            _ => parameter
                .default
                .as_ref()
                .map(|(_, typed)| self.fold_type(typed.clone())),
        };
        if let Some(value) = value {
            self.types.insert(parameter.ident.to_string(), value);
        } else {
            self.incomplete = true;
        }
    }

    fn bind_const(
        &mut self,
        parameter: &syn::ConstParam,
        argument: Option<&syn::GenericArgument>,
        caller: &mut Self,
    ) {
        let value = match argument {
            Some(syn::GenericArgument::Const(value)) => Some(caller.fold_expr(value.clone())),
            Some(syn::GenericArgument::Type(value)) => {
                syn::parse2::<syn::Expr>(quote::quote!(#value))
                    .ok()
                    .map(|value| caller.fold_expr(value))
            }
            _ => parameter
                .default
                .as_ref()
                .map(|(_, value)| self.fold_expr(value.clone())),
        };
        if let Some(value) = value {
            self.constants.insert(parameter.ident.to_string(), value);
        } else {
            self.incomplete = true;
        }
    }

    pub(crate) fn bind(
        &mut self,
        alias: &syn::ItemType,
        path: &syn::Path,
        caller_index: usize,
        caller_module: usize,
    ) {
        let arguments = path
            .segments
            .last()
            .and_then(|segment| match &segment.arguments {
                syn::PathArguments::AngleBracketed(arguments) => {
                    Some(arguments.args.iter().collect::<Vec<_>>())
                }
                _ => None,
            })
            .unwrap_or_default();
        let mut position = 0;
        for parameter in &alias.generics.params {
            let mut caller = Self::new(self.context, caller_index, caller_module);
            caller.cfg = self.cfg.clone();
            match parameter {
                syn::GenericParam::Type(parameter) => {
                    self.bind_type(parameter, arguments.get(position).copied(), &mut caller);
                    if arguments.get(position).is_some() {
                        position += 1;
                    }
                }
                syn::GenericParam::Lifetime(parameter) => {
                    let lifetime = if let Some(syn::GenericArgument::Lifetime(value)) =
                        arguments.get(position)
                    {
                        position += 1;
                        (*value).clone()
                    } else {
                        syn::Lifetime::new("'_", proc_macro2::Span::call_site())
                    };
                    self.lifetimes
                        .insert(parameter.lifetime.ident.to_string(), lifetime);
                }
                syn::GenericParam::Const(parameter) => {
                    self.bind_const(parameter, arguments.get(position).copied(), &mut caller);
                    if arguments.get(position).is_some() {
                        position += 1;
                    }
                }
            }
            self.incomplete |= caller.incomplete;
        }
    }
}

impl Fold for AliasSyntaxFolder<'_> {
    fn fold_type(&mut self, typed: syn::Type) -> syn::Type {
        if let syn::Type::Path(path) = &typed
            && path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
            && let Some(value) = self.types.get(&segment.ident.to_string())
        {
            return value.clone();
        }
        fold::fold_type(self, typed)
    }

    fn fold_lifetime(&mut self, lifetime: syn::Lifetime) -> syn::Lifetime {
        self.lifetimes
            .get(&lifetime.ident.to_string())
            .cloned()
            .unwrap_or(lifetime)
    }

    fn fold_expr(&mut self, expression: syn::Expr) -> syn::Expr {
        if let syn::Expr::Path(path) = &expression
            && path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
            && let Some(value) = self.constants.get(&segment.ident.to_string())
        {
            return value.clone();
        }
        fold::fold_expr(self, expression)
    }

    fn fold_path(&mut self, path: syn::Path) -> syn::Path {
        let path = fold::fold_path(self, path);
        self.qualify(path)
    }
}
