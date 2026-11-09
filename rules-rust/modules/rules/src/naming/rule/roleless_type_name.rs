use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::has_kind;
use crate::source::model::CrateIndex;
use crate::source::model::IndexedImpl;
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;

pub(crate) struct RolelessTypeName;

impl RolelessTypeName {
    fn is_runtime_factory(index: &CrateIndex, module: usize, value: &syn::ItemTrait) -> bool {
        let methods = value
            .items
            .iter()
            .filter_map(|item| match item {
                syn::TraitItem::Fn(function) => Some(function),
                _ => None,
            })
            .collect::<Vec<_>>();
        let markers = value
            .supertraits
            .iter()
            .filter_map(|bound| {
                let syn::TypeParamBound::Trait(bound) = bound else {
                    return None;
                };
                match index.resolve_path(module, &bound.path) {
                    Resolution::External(path)
                        if [
                            "std::marker::Send",
                            "core::marker::Send",
                            "std::marker::Sync",
                            "core::marker::Sync",
                        ]
                        .contains(&path.as_str()) =>
                    {
                        path.rsplit("::").next().map(str::to_owned)
                    }
                    Resolution::Local => bound.path.get_ident().map(ToString::to_string),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        markers.iter().any(|name| name == "Send")
            && markers.iter().any(|name| name == "Sync")
            && markers.len() == value.supertraits.len()
            && markers.iter().all(|name| name == "Send" || name == "Sync")
            && methods.len() == 1
            && methods[0].sig.ident == "invoke"
    }

    fn repeats_trait(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        identity: usize,
        item: &IndexedItem,
    ) -> bool {
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        let same_type = index
            .resolve_type_items(implementation.module, &implementation.syntax.self_ty)
            .contains(&identity);
        let compatible = item.cfg.and(&implementation.cfg).is_satisfiable();
        same_type
            && compatible
            && matches!(index.resolve_path(implementation.module, path), Resolution::Internal(paths)
            if paths.iter().any(|path| path.item.is_some_and(|identity| index.items[identity].name == item.name)))
    }

    fn invalid_factory(index: &CrateIndex, item: &IndexedItem) -> bool {
        let Some(product) = item.name.strip_suffix("Factory") else {
            return false;
        };
        let valid_form = matches!(&item.syntax, syn::Item::Trait(value) if Self::is_runtime_factory(index, item.module, value));
        let has_product = index.items.iter().any(|candidate| {
            candidate.file == item.file
                && candidate.name == product
                && item.cfg.and(&candidate.cfg).is_satisfiable()
                && matches!(
                    candidate.syntax,
                    syn::Item::Struct(_)
                        | syn::Item::Enum(_)
                        | syn::Item::Union(_)
                        | syn::Item::Trait(_)
                )
        });
        !valid_form || !has_product
    }

    fn misplaced_factory_impl(index: &CrateIndex, implementation: &IndexedImpl) -> bool {
        if has_kind(index, implementation.module, "di") {
            return false;
        }
        let Some((path, _)) = &implementation.syntax.trait_ else {
            return false;
        };
        let Resolution::Internal(targets) = index.resolve_path(implementation.module, path) else {
            return false;
        };
        targets.iter().filter_map(|target| target.item.and_then(|identity| index.items.get(identity))).any(|item| {
            item.name.ends_with("Factory") && item.cfg.and(&implementation.cfg).is_satisfiable()
                && matches!(&item.syntax, syn::Item::Trait(value) if Self::is_runtime_factory(index, item.module, value))
        })
    }
}

impl Rule for RolelessTypeName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = Vec::new();
        for index in &context.indexes {
            for (identity, item) in index.items.iter().enumerate() {
                if !matches!(
                    item.syntax,
                    syn::Item::Struct(_)
                        | syn::Item::Enum(_)
                        | syn::Item::Union(_)
                        | syn::Item::Trait(_)
                ) {
                    continue;
                }
                let repeats_trait = index.implementations.iter().any(|implementation| {
                    Self::repeats_trait(index, implementation, identity, item)
                });
                if repeats_trait {
                    continue;
                }
                let misplaced_warmup =
                    item.name.ends_with("WarmUp") && !has_kind(index, item.module, "warmup");
                let misplaced_migration =
                    item.name.ends_with("Migration") && !has_kind(index, item.module, "migration");
                let invalid_factory = Self::invalid_factory(index, item);
                if item.name.ends_with("Validator")
                    || misplaced_warmup
                    || misplaced_migration
                    || invalid_factory
                {
                    findings.push(build_node_finding("RolelessTypeName", &item.file, item.span, format!("{}::{}", format_module_path(&index.crate_name, &index.modules[item.module].module.path), item.name), "Use model rules in util, runtime factories beside their product, and warmup/migration types in their kind (arch-domain.rs.md)".to_owned()));
                }
            }
            for implementation in &index.implementations {
                if Self::misplaced_factory_impl(index, implementation) {
                    findings.push(build_node_finding(
                        "RolelessTypeName",
                        &implementation.file,
                        implementation.span,
                        format_module_path(
                            &index.crate_name,
                            &index.modules[implementation.module].module.path,
                        ),
                        "Implement a runtime factory in a DI module (arch-domain.rs.md)".to_owned(),
                    ));
                }
            }
        }
        findings
    }
}
