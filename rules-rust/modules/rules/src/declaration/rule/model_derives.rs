use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::rule::ErrorFamilyShape;
use crate::declaration::util::derives;
use crate::declaration::util::has_kind;
use crate::declaration::util::is_technology;
use crate::source::util::find_node_span;

pub(crate) struct ModelDerives;

impl ModelDerives {
    fn copy_implementation(
        index: &crate::source::model::CrateIndex,
        identity: usize,
        implementation: &crate::source::model::IndexedImpl,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        let owns_type = crate::module::util::owns_implementation(index, identity, implementation);
        let unconditional =
            implementation.syntax.generics.params.is_empty() && cfg.implies(&implementation.cfg);
        if !owns_type || !unconditional {
            return false;
        }
        match index.resolve_path(implementation.module, path) {
            crate::source::model::Resolution::Local => path.is_ident("Copy"),
            crate::source::model::Resolution::External(path) => {
                ["std::marker::Copy", "core::marker::Copy"].contains(&path.as_str())
            }
            _ => false,
        }
    }

    fn item_copy(
        context: &RuleContext,
        target: &crate::source::model::WorkspaceResolvedPath,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let Some(identity) = target.path.item else {
            return false;
        };
        let index = &context.indexes[target.index];
        let item = &index.items[identity];
        let attributes = match &item.syntax {
            syn::Item::Struct(value) if value.generics.params.is_empty() => &value.attrs,
            syn::Item::Enum(value) if value.generics.params.is_empty() => &value.attrs,
            syn::Item::Union(value) if value.generics.params.is_empty() => &value.attrs,
            _ => return false,
        };
        derives(attributes, "Copy")
            || index.implementations.iter().any(|implementation| {
                Self::copy_implementation(
                    index,
                    identity,
                    implementation,
                    &cfg.and(&target.path.cfg),
                )
            })
    }

    fn is_nonzero_integer(identity: &str) -> bool {
        identity.strip_prefix("num::NonZero").is_some_and(|width| {
            [
                "U8", "U16", "U32", "U64", "U128", "Usize", "I8", "I16", "I32", "I64", "I128",
                "Isize",
            ]
            .contains(&width)
        })
    }

    fn field_holds(item: &crate::source::model::IndexedItem, field: &syn::Field) -> bool {
        item.cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                &field.attrs,
            ))
            .is_satisfiable()
    }

    fn nominal_copy(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        let targets = context.resolve_outer_workspace_type(ordinal, module, typed);
        if !targets.is_empty() {
            return targets
                .iter()
                .filter(|target| cfg.and(&target.path.cfg).is_satisfiable())
                .all(|target| Self::item_copy(context, target, cfg));
        }
        context
            .resolve_outer_external_type_paths(ordinal, module, typed)
            .iter()
            .any(|path| {
                let identity = path
                    .segments
                    .iter()
                    .skip(1)
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .join("::");
                (path.package == "uuid" && identity == "Uuid")
                    || (["std", "core"].contains(&path.package.as_str())
                        && (["time::Duration", "time::Instant", "time::SystemTime"]
                            .contains(&identity.as_str())
                            || Self::is_nonzero_integer(&identity)))
            })
    }

    fn supports_copy(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &syn::Type,
        cfg: &crate::source::model::CfgPredicate,
        generics: &syn::Generics,
        depth: usize,
    ) -> bool {
        if depth > 8 {
            return false;
        }
        let has_copy_bound = crate::declaration::util::has_standard_bound(
            context, ordinal, module, generics, typed, "Copy",
        );
        if has_copy_bound {
            return true;
        }
        let resolved = context.resolve_type_syntax_under(ordinal, module, typed, cfg);
        !resolved.is_empty()
            && resolved.iter().all(|resolved| {
                !resolved.has_unsubstituted_parameters
                    && match &resolved.typed {
                        syn::Type::Reference(reference) => reference.mutability.is_none(),
                        syn::Type::Array(array) => Self::supports_copy(
                            context,
                            resolved.index,
                            resolved.module,
                            &array.elem,
                            &resolved.cfg,
                            generics,
                            depth + 1,
                        ),
                        syn::Type::Path(path) => {
                            let builtin = matches!(
                                context.indexes[resolved.index]
                                    .resolve_path(resolved.module, &path.path),
                                crate::source::model::Resolution::Local
                            );
                            Self::nominal_copy(
                                context,
                                resolved.index,
                                resolved.module,
                                &resolved.typed,
                                &resolved.cfg,
                            ) || builtin
                                && path.path.get_ident().is_some_and(|name| {
                                    [
                                        "bool", "char", "u8", "u16", "u32", "u64", "u128", "usize",
                                        "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
                                    ]
                                    .iter()
                                    .any(|primitive| name == primitive)
                                })
                        }
                        _ => false,
                    }
            })
    }

    fn copy_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        field: &syn::Field,
    ) -> bool {
        let cfg = item
            .cfg
            .and(&crate::source::model::CfgPredicate::from_attributes(
                &field.attrs,
            ));
        let syn::Item::Struct(value) = &item.syntax else {
            return false;
        };
        Self::supports_copy(
            context,
            ordinal,
            item.module,
            &field.ty,
            &cfg,
            &value.generics,
            0,
        )
    }

    fn is_error_derive(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        path: &syn::Path,
    ) -> bool {
        let name = match context.indexes[ordinal].resolve_path(module, path) {
            crate::source::model::Resolution::External(external) => context
                .resolve_external_identity(ordinal, &external)
                .segments
                .last()
                .cloned(),
            _ => path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
        };
        name.is_some_and(|name| name == "Error")
    }

    fn derives_error(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        attributes: &[syn::Attribute],
    ) -> bool {
        attributes
            .iter()
            .filter(|attribute| attribute.path().is_ident("derive"))
            .any(|attribute| {
                attribute
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                    )
                    .is_ok_and(|paths| {
                        paths
                            .iter()
                            .any(|path| Self::is_error_derive(context, ordinal, module, path))
                    })
            })
    }

    fn is_error_struct(context: &RuleContext, ordinal: usize, identity: usize) -> bool {
        let item = &context.indexes[ordinal].items[identity];
        let syn::Item::Struct(value) = &item.syntax else {
            return false;
        };
        Self::derives_error(context, ordinal, item.module, &value.attrs)
            || ErrorFamilyShape::implements_error(context, ordinal, identity)
    }

    fn is_eq_hash_field(
        context: &RuleContext,
        ordinal: usize,
        item: &crate::source::model::IndexedItem,
        field: &syn::Field,
    ) -> bool {
        let syn::Item::Struct(value) = &item.syntax else {
            return false;
        };
        ["Eq", "Hash"].iter().all(|name| {
            crate::declaration::util::has_standard_bound(
                context,
                ordinal,
                item.module,
                &value.generics,
                &field.ty,
                name,
            )
        }) || crate::declaration::util::supports_eq_hash(
            context,
            ordinal,
            item.module,
            &field.ty,
            0,
        )
    }
}

impl Rule for ModelDerives {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            for (identity, item) in index.items.iter().enumerate() {
                let syn::Item::Struct(value) = &item.syntax else {
                    continue;
                };
                if !has_kind(index, item.module, "model")
                    || is_technology(context, index, item.module)
                    || Self::is_error_struct(context, ordinal, identity)
                {
                    continue;
                }
                let mut required = vec!["Debug", "Clone", "PartialEq"];
                if item.name.ends_with("Id")
                    && value.fields.len() == 1
                    && value
                        .fields
                        .iter()
                        .all(|field| Self::copy_field(context, ordinal, item, field))
                {
                    required.push("Copy");
                }
                if item.name.ends_with("Id")
                    || value
                        .fields
                        .iter()
                        .filter(|field| Self::field_holds(item, field))
                        .all(|field| Self::is_eq_hash_field(context, ordinal, item, field))
                {
                    required.extend(["Eq", "Hash"]);
                }
                let missing = required
                    .into_iter()
                    .filter(|name| !derives(&value.attrs, name))
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    findings.push(build_node_finding(
                        "ModelDerives",
                        &item.file,
                        find_node_span(value.ident.span()),
                        format!(
                            "{}::{}",
                            format_module_path(
                                &index.crate_name,
                                &index.modules[item.module].module.path
                            ),
                            item.name
                        ),
                        format!(
                            "Derive {} on this model struct (arch-domain.rs.md)",
                            missing.join(", ")
                        ),
                    ));
                }
            }
        }
        findings
    }
}
