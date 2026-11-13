use crate::check::model::RuleContext;
use crate::source::model::{CfgPredicate, TypeBinding, WorkspaceTypeResult};
use std::collections::BTreeMap;
use syn::{GenericArgument, GenericParam, PathArguments, Type, TypeParamBound};

pub(crate) struct WorkspaceTypeResolver<'a> {
    context: &'a RuleContext,
    nested: bool,
    result: WorkspaceTypeResult,
}

impl<'a> WorkspaceTypeResolver<'a> {
    pub(crate) fn new(context: &'a RuleContext, nested: bool) -> Self {
        Self {
            context,
            nested,
            result: WorkspaceTypeResult {
                items: Vec::new(),
                external: Vec::new(),
            },
        }
    }

    fn collect_bounds(
        &mut self,
        binding: &TypeBinding,
        bounds: &syn::punctuated::Punctuated<TypeParamBound, syn::Token![+]>,
        depth: usize,
        cfg: &CfgPredicate,
    ) {
        for bound in bounds {
            if let TypeParamBound::Trait(bound) = bound {
                self.collect(
                    &TypeBinding {
                        typed: Type::Path(syn::TypePath {
                            attrs: Vec::new(),
                            qself: None,
                            path: bound.path.clone(),
                        }),
                        ..binding.clone()
                    },
                    depth,
                    cfg,
                );
            }
        }
    }

    fn list_type_arguments(alias: &syn::ItemType, path: &syn::Path) -> Vec<Option<Type>> {
        let arguments = path
            .segments
            .last()
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(arguments) => Some(&arguments.args),
                _ => None,
            });
        let mut arguments = arguments.into_iter().flatten().peekable();
        let mut types = Vec::new();
        for parameter in &alias.generics.params {
            match parameter {
                GenericParam::Lifetime(_) => {
                    if matches!(arguments.peek(), Some(GenericArgument::Lifetime(_))) {
                        arguments.next();
                    }
                }
                GenericParam::Const(_) => {
                    arguments.next();
                }
                GenericParam::Type(_) => {
                    types.push(match arguments.next() {
                        Some(GenericArgument::Type(typed)) => Some(typed.clone()),
                        _ => None,
                    });
                }
            }
        }
        types
    }

    fn bind_alias(
        &self,
        binding: &TypeBinding,
        alias: &syn::ItemType,
        path: &syn::Path,
        index: usize,
        module: usize,
    ) -> TypeBinding {
        let arguments = Self::list_type_arguments(alias, path);
        let mut parameters = binding.parameters.clone();
        for (position, parameter) in alias
            .generics
            .params
            .iter()
            .filter_map(|parameter| match parameter {
                GenericParam::Type(parameter) => Some(parameter),
                _ => None,
            })
            .enumerate()
        {
            let value = arguments
                .get(position)
                .and_then(Option::as_ref)
                .map(|typed| TypeBinding {
                    index: binding.index,
                    module: binding.module,
                    typed: typed.clone(),
                    parameters: binding.parameters.clone(),
                })
                .or_else(|| {
                    parameter.default.as_ref().map(|(_, typed)| TypeBinding {
                        index,
                        module,
                        typed: typed.clone(),
                        parameters: parameters.clone(),
                    })
                });
            if let Some(value) = value {
                parameters.insert(parameter.ident.to_string(), value);
            }
        }
        TypeBinding {
            index,
            module,
            typed: (*alias.ty).clone(),
            parameters,
        }
    }

    fn collect_path(
        &mut self,
        binding: &TypeBinding,
        path: &syn::TypePath,
        depth: usize,
        cfg: &CfgPredicate,
    ) {
        if path.qself.is_some() {
            return;
        }
        if path.path.segments.len() == 1
            && let Some(first) = path.path.segments.first()
            && let Some(parameter) = binding.parameters.get(&first.ident.to_string())
        {
            self.collect(parameter, depth, cfg);
            return;
        }
        let targets =
            self.context
                .resolve_canonical_type_path(binding.index, binding.module, &path.path);
        let mut alias_found = false;
        for mut target in targets {
            let Some(item) = target.path.item else {
                continue;
            };
            let declaration = &self.context.indexes[target.index].items[item];
            let predicate = cfg
                .in_scope(binding.index)
                .and(&target.path.cfg.in_scope(target.index));
            if !predicate.is_satisfiable() {
                continue;
            }
            if let syn::Item::Type(alias) = &declaration.syntax {
                alias_found = true;
                let alias =
                    self.bind_alias(binding, alias, &path.path, target.index, declaration.module);
                self.collect(&alias, depth + 1, &predicate);
            } else {
                target.path.cfg = predicate;
                self.result.items.push(target);
            }
        }
        if alias_found {
            return;
        }
        if let Some(external) =
            self.context
                .canonical_external_type_path(binding.index, binding.module, &path.path)
            && external.member.is_none()
        {
            self.result.external.push(external);
        }
        if self.nested {
            self.collect_arguments(binding, &path.path, depth, cfg);
        }
    }

    fn collect_arguments(
        &mut self,
        binding: &TypeBinding,
        path: &syn::Path,
        depth: usize,
        cfg: &CfgPredicate,
    ) {
        for segment in &path.segments {
            let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                continue;
            };
            for argument in &arguments.args {
                let typed = match argument {
                    GenericArgument::Type(typed) => typed,
                    GenericArgument::AssocType(typed) => &typed.ty,
                    _ => continue,
                };
                self.collect(
                    &TypeBinding {
                        typed: typed.clone(),
                        ..binding.clone()
                    },
                    depth,
                    cfg,
                );
            }
        }
    }

    fn collect(&mut self, binding: &TypeBinding, depth: usize, cfg: &CfgPredicate) {
        if depth > 8 {
            return;
        }
        let typed = match &binding.typed {
            Type::Path(path) => {
                self.collect_path(binding, path, depth, cfg);
                return;
            }
            Type::Reference(reference) => &reference.elem,
            Type::Ptr(pointer) => &pointer.elem,
            Type::Paren(paren) => &paren.elem,
            Type::Group(group) => &group.elem,
            Type::TraitObject(object) => {
                self.collect_bounds(binding, &object.bounds, depth, cfg);
                return;
            }
            Type::ImplTrait(object) => {
                self.collect_bounds(binding, &object.bounds, depth, cfg);
                return;
            }
            Type::Array(array) if self.nested => &array.elem,
            Type::Slice(slice) if self.nested => &slice.elem,
            Type::Tuple(tuple) if self.nested => {
                for typed in &tuple.elems {
                    self.collect(
                        &TypeBinding {
                            typed: typed.clone(),
                            ..binding.clone()
                        },
                        depth,
                        cfg,
                    );
                }
                return;
            }
            _ => return,
        };
        self.collect(
            &TypeBinding {
                typed: (**typed).clone(),
                ..binding.clone()
            },
            depth,
            cfg,
        );
    }

    pub(crate) fn resolve(self, index: usize, module: usize, typed: &Type) -> WorkspaceTypeResult {
        self.resolve_under(index, module, typed, &CfgPredicate::All(Vec::new()))
    }

    pub(crate) fn resolve_under(
        mut self,
        index: usize,
        module: usize,
        typed: &Type,
        cfg: &CfgPredicate,
    ) -> WorkspaceTypeResult {
        self.collect(
            &TypeBinding {
                index,
                module,
                typed: typed.clone(),
                parameters: BTreeMap::new(),
            },
            0,
            cfg,
        );
        self.result
            .items
            .sort_by_key(|item| (item.index, item.path.module, item.path.item));
        self.result.items.dedup();
        self.result
            .external
            .sort_by_key(crate::source::model::ExternalPath::format_path);
        self.result.external.dedup();
        self.result
    }
}
