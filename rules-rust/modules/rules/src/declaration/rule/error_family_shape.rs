use crate::check::api::Rule;
use crate::check::model::{RuleContext, RuleFinding};
use crate::declaration::util::derives;
use crate::module::util::build_shape_item_finding;
use syn::{Item, Type};

pub(crate) struct ErrorFamilyShape;

impl ErrorFamilyShape {
    fn error_derive_path(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        path: &syn::Path,
    ) -> bool {
        let crate::source::model::Resolution::External(path) =
            context.indexes[ordinal].resolve_path(module, path)
        else {
            return false;
        };
        let path = context.resolve_external_identity(ordinal, &path);
        path.package == "thiserror" && path.segments.last().is_some_and(|name| name == "Error")
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
                            .any(|path| Self::error_derive_path(context, ordinal, module, path))
                    })
            })
    }
    fn is_source_box(
        context: &RuleContext,
        ordinal: usize,
        module: usize,
        typed: &Type,
        cfg: &crate::source::model::CfgPredicate,
    ) -> bool {
        context
            .resolve_type_syntax(ordinal, module, typed)
            .iter()
            .filter(|resolved| cfg.and(&resolved.cfg).is_satisfiable())
            .all(|resolved| {
                if resolved.has_unsubstituted_parameters {
                    return true;
                }
                let outer = context.resolve_outer_external_type_paths_under(
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    cfg,
                );
                let boxed = outer.iter().any(|path| {
                    ["std", "alloc"].contains(&path.package.as_str())
                        && path.segments.last().is_some_and(|name| name == "Box")
                });
                let Type::Path(path) = &resolved.typed else {
                    return false;
                };
                let Some(segment) = path.path.segments.last() else {
                    return false;
                };
                let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                    return false;
                };
                let Some(syn::GenericArgument::Type(Type::TraitObject(object))) =
                    arguments.args.first()
                else {
                    return false;
                };
                let paths = context.resolve_external_type_paths_under(
                    resolved.index,
                    resolved.module,
                    &resolved.typed,
                    cfg,
                );
                boxed
                    && object
                        .bounds
                        .iter()
                        .filter(|bound| matches!(bound, syn::TypeParamBound::Trait(_)))
                        .count()
                        == 3
                    && ["Error", "Send", "Sync"].iter().all(|name| {
                        paths.iter().any(|path| {
                            ["std", "core"].contains(&path.package.as_str())
                                && path.segments.last().is_some_and(|segment| segment == name)
                        })
                    })
            })
    }
}

impl Rule for ErrorFamilyShape {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        context.indexes.iter().enumerate().flat_map(|(ordinal, index)| index.items.iter().filter_map(move |item| {
            let Item::Enum(family) = &item.syntax else { return None; };
            if !item.name.ends_with("Error") && !Self::derives_error(context, ordinal, item.module, &family.attrs) { return None; }
            let invalid_variant = family.variants.iter().filter(|variant| item.cfg.and(&crate::source::model::CfgPredicate::from_attributes(&variant.attrs)).is_satisfiable()).any(|variant| !variant.attrs.iter().any(|attribute| attribute.path().is_ident("error")) || variant.fields.iter().any(|field| {
                let cfg = item.cfg.and(&crate::source::model::CfgPredicate::from_attributes(&variant.attrs)).and(&crate::source::model::CfgPredicate::from_attributes(&field.attrs));
                if !cfg.is_satisfiable() { return false; }
                let source = field.attrs.iter().any(|attribute| attribute.path().is_ident("source"));
                let foreign = !source && context.resolve_outer_external_type_paths_under(ordinal, item.module, &field.ty, &cfg).iter().any(|path| path.segments.last().is_some_and(|name| name.ends_with("Error")));
                (source && !Self::is_source_box(context, ordinal, item.module, &field.ty, &cfg)) || foreign
            }));
            let invalid = !derives(&family.attrs, "Debug") || !Self::derives_error(context, ordinal, item.module, &family.attrs) || invalid_variant;
            invalid.then(|| build_shape_item_finding(index, item, "ErrorFamilyShape", "an error enum derives Debug + thiserror::Error, labels each variant, and boxes source errors as dyn Error + Send + Sync"))
        })).collect()
    }
}
