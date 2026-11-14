use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::declaration::util::derives;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, find_type_path, is_model_item,
    owns_implementation, resolve_type_uses,
};
use crate::source::model::{CrateIndex, IndexedImpl, IndexedItem, Resolution, SourceTree};
use crate::syntax::util::to_snake_case;
use std::path::Path;

pub(crate) struct ImplPlacement;

impl ImplPlacement {
    fn conversion_source<'syntax>(
        index: &CrateIndex,
        implementation: &'syntax IndexedImpl,
    ) -> Option<&'syntax syn::Type> {
        if !Self::is_standard_conversion(index, implementation) {
            return None;
        }
        let (path, _) = implementation.syntax.trait_.as_ref()?;
        let last = path.segments.last()?;
        let syn::PathArguments::AngleBracketed(arguments) = &last.arguments else {
            return None;
        };
        arguments.args.iter().find_map(|argument| match argument {
            syn::GenericArgument::Type(value) => Some(value),
            _ => None,
        })
    }

    fn is_standard_conversion(index: &CrateIndex, implementation: &IndexedImpl) -> bool {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        match index.resolve_path(implementation.module, path) {
            Resolution::Local => {
                path.segments.len() == 1
                    && path.segments.last().is_some_and(|segment| {
                        ["From", "TryFrom"].iter().any(|name| segment.ident == name)
                    })
            }
            Resolution::External(path) => [
                "std::convert::From",
                "core::convert::From",
                "std::convert::TryFrom",
                "core::convert::TryFrom",
            ]
            .contains(&path.as_str()),
            _ => false,
        }
    }

    fn conversion_allowed(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        implementation: &IndexedImpl,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let Some(source) = Self::conversion_source(index, implementation) else {
            return false;
        };
        if !is_model_item(context, ordinal, item)
            || !Self::is_standard_conversion(index, implementation)
        {
            return false;
        }
        let local = resolve_type_uses(index, implementation.module, source, 0);
        if local.iter().any(|identity| {
            index.items[*identity].file == implementation.file
                && index.items[*identity].file != item.file
        }) {
            return true;
        }
        let Some(path) = Self::external_origin(index, implementation.module, source, 0) else {
            return false;
        };
        Self::technology_scope(context, ordinal, implementation.module, &path)
    }

    fn external_origin(
        index: &CrateIndex,
        module: usize,
        value: &syn::Type,
        depth: usize,
    ) -> Option<String> {
        if depth > 8 {
            return None;
        }
        let path = find_type_path(value)?;
        match index.resolve_path(module, path) {
            Resolution::External(path) => Some(path),
            Resolution::Internal(paths) => paths
                .iter()
                .filter_map(|path| path.item.map(|identity| &index.items[identity]))
                .find_map(|item| match &item.syntax {
                    syn::Item::Type(alias) => {
                        Self::external_origin(index, item.module, &alias.ty, depth + 1)
                    }
                    _ => None,
                }),
            _ => None,
        }
    }

    fn technology_scope(context: &RuleContext, ordinal: usize, module: usize, path: &str) -> bool {
        let index = &context.indexes[ordinal];
        let dependency = path.split("::").next().unwrap_or("");
        let kind = find_kind_name(context, ordinal, module);
        let word = dependency.split('_').next().unwrap_or("");
        let role = context
            .find_role_config(&index.crate_name)
            .and_then(|config| config.role);
        let is_whole_technology_crate = matches!(
            role,
            Some(Role::Technology | Role::Platform | Role::CorePlatform)
        );
        kind == dependency || kind == word || is_whole_technology_crate && kind.is_empty()
    }

    fn stands_alone(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        contract: &syn::Path,
    ) -> bool {
        let index = &context.indexes[ordinal];
        let Some(file) = context
            .crates
            .iter()
            .find(|sources| {
                sources.crate_name == index.crate_name && sources.target == index.target
            })
            .and_then(|sources| {
                sources
                    .files
                    .iter()
                    .find(|file| file.path == implementation.file)
            })
        else {
            return false;
        };
        let contract_name = contract.segments.last().map(|segment| &segment.ident);
        file.syntax.items.iter().all(|item| match item {
            syn::Item::Use(_) => true,
            syn::Item::Impl(other) => other.trait_.as_ref().is_some_and(|(path, _)| {
                path.segments.last().map(|segment| &segment.ident) == contract_name
            }),
            _ => false,
        })
    }

    fn find_pointee(self_type: &syn::Type) -> Option<&syn::Type> {
        match self_type {
            syn::Type::Reference(reference) => Some(&*reference.elem),
            syn::Type::Path(path) => {
                let segment = path.path.segments.last()?;
                if !["Arc", "Box", "Rc"].contains(&segment.ident.to_string().as_str()) {
                    return None;
                }
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return None;
                };
                arguments.args.iter().find_map(|argument| match argument {
                    syn::GenericArgument::Type(inner) => Some(inner),
                    _ => None,
                })
            }
            _ => None,
        }
    }

    fn bounds_name_contract<'bound>(
        mut bounds: impl Iterator<Item = &'bound syn::TypeParamBound>,
        contract: &syn::Ident,
    ) -> bool {
        bounds.any(|bound| {
            matches!(bound, syn::TypeParamBound::Trait(bound) if bound.path.segments.last().is_some_and(|segment| &segment.ident == contract))
        })
    }

    fn is_forwarding_impl(implementation: &IndexedImpl, contract: &syn::Path) -> bool {
        let Some(contract_name) = contract.segments.last().map(|segment| &segment.ident) else {
            return false;
        };
        let Some(pointee) = Self::find_pointee(&implementation.syntax.self_ty) else {
            return false;
        };
        if let syn::Type::TraitObject(object) = pointee {
            return Self::bounds_name_contract(object.bounds.iter(), contract_name);
        }
        let syn::Type::Path(path) = pointee else {
            return false;
        };
        let Some(parameter) = path.path.get_ident() else {
            return false;
        };
        let generics = &implementation.syntax.generics;
        let in_parameters = generics.params.iter().any(|candidate| {
            matches!(candidate, syn::GenericParam::Type(declared) if declared.ident == *parameter
                && Self::bounds_name_contract(declared.bounds.iter(), contract_name))
        });
        let in_where_clause = generics.where_clause.as_ref().is_some_and(|clause| {
            clause.predicates.iter().any(|predicate| {
                matches!(predicate, syn::WherePredicate::Type(bounded)
                    if matches!(&bounded.bounded_ty, syn::Type::Path(path) if path.path.is_ident(parameter))
                        && Self::bounds_name_contract(bounded.bounds.iter(), contract_name))
            })
        });
        in_parameters || in_where_clause
    }

    fn is_prelude_standard_type(self_type: &syn::Type) -> bool {
        let Some(path) = find_type_path(self_type) else {
            return false;
        };
        let [segment] = path.segments.iter().collect::<Vec<_>>()[..] else {
            return false;
        };
        Self::PRELUDE_STANDARD_TYPES.contains(&segment.ident.to_string().as_str())
    }

    fn is_local_contract(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
        contract: &syn::Path,
    ) -> bool {
        let index = &context.indexes[ordinal];
        context
            .resolve_workspace_path(ordinal, implementation.module, contract)
            .iter()
            .any(|target| {
                target.index == ordinal
                    && target.path.item.is_some_and(|identity| {
                        let item = &index.items[identity];
                        matches!(item.syntax, syn::Item::Trait(_))
                            && find_kind_name(context, ordinal, item.module) == "api"
                    })
            })
    }

    fn build_wrapper_finding(implementation: &IndexedImpl, subject: String) -> RuleFinding {
        build_node_finding(
            "ImplPlacement",
            &implementation.file,
            implementation.span,
            subject,
            "A contract is not implemented for a type of the standard library: write a wrapper type (arch-package.rs.md)".to_owned(),
        )
    }

    fn foreign_contract_finding(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        if index.modules[implementation.module].tree != SourceTree::Src {
            return None;
        }
        let (contract, _) = implementation.syntax.trait_.as_ref()?;
        let is_local_contract = Self::is_local_contract(context, ordinal, implementation, contract);
        let is_foreign_type = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .is_empty();
        if !is_local_contract
            || !is_foreign_type
            || Self::is_forwarding_impl(implementation, contract)
        {
            return None;
        }
        if Self::is_prelude_standard_type(&implementation.syntax.self_ty) {
            return Some(Self::build_wrapper_finding(
                implementation,
                "std".to_owned(),
            ));
        }
        let external = Self::external_origin(
            index,
            implementation.module,
            &implementation.syntax.self_ty,
            0,
        )?;
        let first_segment = external.split("::").next().unwrap_or("");
        if ["std", "core", "alloc"].contains(&first_segment) {
            return Some(Self::build_wrapper_finding(implementation, external));
        }
        let in_technology =
            Self::technology_scope(context, ordinal, implementation.module, &external);
        let type_name = external.rsplit("::").next().unwrap_or("");
        let contract_name = contract.segments.last()?.ident.to_string();
        let expected_stem = format!(
            "{}_{}",
            to_snake_case(type_name),
            to_snake_case(&contract_name)
        );
        let stem = Path::new(&implementation.file)
            .file_stem()
            .and_then(|stem| stem.to_str());
        let is_placed = in_technology
            && stem == Some(expected_stem.as_str())
            && Self::stands_alone(context, ordinal, implementation, contract);
        (!is_placed).then(|| build_node_finding("ImplPlacement", &implementation.file, implementation.span, external, "An impl of a contract for a type of another crate stands alone in <type>_<contract>.rs of that crate's technology module (arch-package.rs.md)".to_owned()))
    }

    fn external_conversion_finding(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let source = Self::conversion_source(index, implementation)?;
        let model = resolve_type_uses(index, implementation.module, source, 0)
            .iter()
            .any(|identity| {
                is_model_item(context, ordinal, &index.items[*identity])
                    && index.items[*identity]
                        .cfg
                        .and(&implementation.cfg)
                        .is_satisfiable()
            });
        let external = Self::external_origin(
            index,
            implementation.module,
            &implementation.syntax.self_ty,
            0,
        )?;
        (model && !Self::technology_scope(context, ordinal, implementation.module, &external)).then(|| build_node_finding("ImplPlacement", &implementation.file, implementation.span, external, "A model-to-external conversion lives in the external crate's technology module (arch-domain.rs.md)".to_owned()))
    }

    fn allowed(
        context: &RuleContext,
        ordinal: usize,
        item: &IndexedItem,
        implementation: &IndexedImpl,
    ) -> bool {
        if item.file == implementation.file {
            return true;
        }
        let kind = find_kind_name(context, ordinal, implementation.module);
        let stem = Path::new(&implementation.file)
            .file_stem()
            .and_then(|stem| stem.to_str());
        let extension = implementation.syntax.trait_.is_none()
            && kind == "util"
            && stem == Some(format!("{}_ext", to_snake_case(&item.name)).as_str());
        let error = matches!(&item.syntax, syn::Item::Enum(value) if item.name.ends_with("Error") || derives(&value.attrs, "Error"));
        let entry = context
            .find_role_config(&context.indexes[ordinal].crate_name)
            .and_then(|config| config.role)
            == Some(Role::EntryPoint);
        let framework = external_trait(context, ordinal, implementation).is_some_and(|path| {
            !["std", "core", "alloc"].contains(&path.split("::").next().unwrap_or(""))
        });
        extension
            || Self::conversion_allowed(context, ordinal, item, implementation)
            || (entry && kind == "error" && error && framework)
    }

    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        if context.is_shape_generated_file(&implementation.file) {
            return Vec::new();
        }
        let index = &context.indexes[ordinal];
        let mut findings = Self::external_conversion_finding(context, ordinal, implementation)
            .into_iter()
            .collect::<Vec<_>>();
        findings.extend(Self::foreign_contract_finding(
            context,
            ordinal,
            implementation,
        ));
        findings.extend(index.resolve_type_items(implementation.module, &implementation.syntax.self_ty).into_iter().filter(|identity| owns_implementation(index, *identity, implementation)).filter_map(|identity| {
            let item = &index.items[identity];
            (!Self::allowed(context, ordinal, item, implementation)).then(|| build_node_finding("ImplPlacement", &implementation.file, implementation.span, item.name.clone(), "An impl stays in its type's file, its documented util extension or a conversion/framework boundary (arch-package.rs.md)".to_owned()))
        }));
        findings
    }
}

impl ImplPlacement {
    const PRELUDE_STANDARD_TYPES: [&'static str; 22] = [
        "String", "Vec", "Option", "Result", "Box", "bool", "char", "str", "u8", "u16", "u32",
        "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
    ];
}

impl Rule for ImplPlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(
                index
                    .items
                    .iter()
                    .filter(|item| {
                        matches!(item.syntax, syn::Item::Trait(_))
                            && item.name.ends_with("Ext")
                            && find_kind_name(context, ordinal, item.module) != "util"
                            && !context.is_shape_generated_file(&item.file)
                    })
                    .map(|item| {
                        build_shape_item_finding(
                            index,
                            item,
                            "ImplPlacement",
                            "An extension trait lives in util (arch-package.rs.md)",
                        )
                    }),
            );
            findings.extend(
                index
                    .implementations
                    .iter()
                    .flat_map(|implementation| Self::check_impl(context, ordinal, implementation)),
            );
        }
        findings
    }
}
