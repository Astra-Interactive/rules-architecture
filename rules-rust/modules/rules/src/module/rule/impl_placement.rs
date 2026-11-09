use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::check::util::build_node_finding;
use crate::config::model::Role;
use crate::declaration::util::derives;
use crate::module::util::{
    build_shape_item_finding, external_trait, find_kind_name, find_type_path, is_model_item,
    owns_implementation, resolve_type_uses,
};
use crate::source::model::{CrateIndex, IndexedImpl, IndexedItem, Resolution};
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
                path.is_ident("From") || path.is_ident("TryFrom") || path.segments.len() == 1
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
        kind == dependency
            || kind == word
            || context
                .find_role_config(&index.crate_name)
                .and_then(|config| config.role)
                == Some(Role::Technology)
                && kind.is_empty()
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
        findings.extend(index.resolve_type_items(implementation.module, &implementation.syntax.self_ty).into_iter().filter(|identity| owns_implementation(index, *identity, implementation)).filter_map(|identity| {
            let item = &index.items[identity];
            (!Self::allowed(context, ordinal, item, implementation)).then(|| build_node_finding("ImplPlacement", &implementation.file, implementation.span, item.name.clone(), "An impl stays in its type's file, its documented util extension or a conversion/framework boundary (arch-package.rs.md)".to_owned()))
        }));
        findings
    }
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
