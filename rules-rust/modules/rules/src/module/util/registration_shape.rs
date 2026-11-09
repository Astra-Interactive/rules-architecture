use crate::check::model::RuleContext;
use crate::config::model::Role;
use crate::source::model::IndexedItem;
use syn::{FnArg, GenericArgument, Item, PathArguments, TraitItem, Type};

impl RuleContext {
    fn is_framework_parameter(&self, ordinal: usize, module: usize, typed: &Type) -> bool {
        let syntax = self.resolve_type_syntax(ordinal, module, typed);
        !syntax.is_empty()
            && syntax.iter().all(|syntax| {
                !syntax.has_unsubstituted_parameters
                    && self.is_framework_parameter_shape(syntax.index, syntax.module, &syntax.typed)
            })
    }

    fn is_framework_parameter_shape(&self, ordinal: usize, module: usize, typed: &Type) -> bool {
        match typed {
            Type::Reference(reference) => {
                self.is_framework_parameter(ordinal, module, &reference.elem)
            }
            Type::Paren(paren) => self.is_framework_parameter(ordinal, module, &paren.elem),
            Type::Path(path) => {
                let identity = self
                    .canonical_external_type_path(ordinal, module, &path.path)
                    .or_else(|| {
                        self.resolve_canonical_type_path(ordinal, module, &path.path)
                            .first()
                            .map(|target| crate::source::model::ExternalPath {
                                package: self.indexes[target.index].crate_name.clone(),
                                segments: Vec::new(),
                                member: self.workspace.members.iter().position(|member| {
                                    member.name == self.indexes[target.index].crate_name
                                }),
                            })
                    });
                let external = identity.as_ref().is_some_and(|identity| {
                    self.is_platform_identity(identity)
                        || self.is_technology_identity(ordinal, identity)
                });
                let arguments: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .flat_map(|segment| match &segment.arguments {
                        PathArguments::AngleBracketed(arguments) => arguments
                            .args
                            .iter()
                            .filter_map(|argument| match argument {
                                GenericArgument::Type(typed) => Some(typed),
                                _ => None,
                            })
                            .collect(),
                        _ => Vec::new(),
                    })
                    .collect();
                let standard = identity.as_ref().is_some_and(|identity| {
                    ["std", "core", "alloc"].contains(&identity.package.as_str())
                });
                let wrapper = standard
                    && path.path.segments.last().is_some_and(|segment| {
                        ["Box", "Arc", "Rc", "Pin", "Option"]
                            .iter()
                            .any(|name| segment.ident == name)
                    });
                (external || (wrapper && !arguments.is_empty()))
                    && arguments
                        .iter()
                        .all(|argument| self.is_framework_parameter(ordinal, module, argument))
            }
            _ => false,
        }
    }

    pub(crate) fn is_platform_registration_trait(
        &self,
        ordinal: usize,
        item: &IndexedItem,
    ) -> bool {
        if !self.has_registration_shape(ordinal, item) {
            return false;
        }
        let index = &self.indexes[ordinal];
        let role = self
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role);
        if matches!(
            role,
            Some(Role::Api | Role::CoreApi | Role::StorageContract | Role::Contract)
        ) {
            return false;
        }
        let kind = crate::module::util::find_kind_name(self, ordinal, item.module);
        if !self.is_entry_kind(&index.crate_name, kind) {
            return false;
        }
        let Item::Trait(contract) = &item.syntax else {
            return false;
        };
        !contract.items.is_empty() && contract.items.iter().all(|member| {
            let TraitItem::Fn(method) = member else { return false; };
            method.sig.inputs.len() > 1 && method.sig.inputs.iter().skip(1).all(|parameter| matches!(parameter, FnArg::Typed(parameter) if self.is_framework_parameter(ordinal, item.module, &parameter.ty)))
        })
    }
}
