use crate::check::model::RuleContext;
use crate::source::model::IndexedItem;

impl RuleContext {
    pub(crate) fn has_external_name_user(&self, ordinal: usize, identity: usize) -> bool {
        let mut pending = vec![(
            ordinal,
            identity,
            self.indexes[ordinal].items[identity].cfg.in_scope(ordinal),
        )];
        let mut visited =
            std::collections::BTreeMap::<(usize, usize), crate::source::model::CfgPredicate>::new();
        while let Some((index, item, cfg)) = pending.pop() {
            if visited
                .get(&(index, item))
                .is_some_and(|covered| cfg.implies(covered))
            {
                continue;
            }
            visited
                .entry((index, item))
                .and_modify(|covered| {
                    *covered =
                        crate::source::model::CfgPredicate::Any(vec![covered.clone(), cfg.clone()])
                })
                .or_insert_with(|| cfg.clone());
            for usage in self
                .name_index()
                .users_of(index, item)
                .filter(|usage| usage.cfg.is_satisfiable())
            {
                let predicate = cfg.and(&usage.cfg);
                if !predicate.is_satisfiable() {
                    continue;
                }
                if usage.source_index != ordinal {
                    return true;
                }
                pending.extend(
                    self.indexes[usage.source_index]
                        .items
                        .iter()
                        .enumerate()
                        .filter(|(_, declaration)| {
                            declaration.file == usage.file && declaration.span.contains(&usage.span)
                        })
                        .map(|(identity, _)| (usage.source_index, identity, predicate.clone())),
                );
            }
        }
        false
    }

    pub(crate) fn can_prove_workspace_only(&self, ordinal: usize) -> bool {
        if self.name_index().has_blind_spots {
            return false;
        }
        let index = &self.indexes[ordinal];
        let Some(member) = self
            .workspace
            .members
            .iter()
            .find(|member| member.name == index.crate_name)
        else {
            return false;
        };
        !member.is_published(&self.workspace)
            && self.indexes.iter().all(|index| {
                index.unanalysable.is_empty()
                    && !index.imports.iter().any(|import| {
                        import.is_glob
                            && matches!(
                                index.resolve_segments(import.module, &import.path, 0),
                                crate::source::model::Resolution::External(_)
                            )
                    })
            })
    }

    pub(crate) fn is_public_item(item: &IndexedItem) -> bool {
        let visibility = match &item.syntax {
            syn::Item::Struct(item) => &item.vis,
            syn::Item::Enum(item) => &item.vis,
            syn::Item::Trait(item) => &item.vis,
            syn::Item::Type(item) => &item.vis,
            syn::Item::Fn(item) => &item.vis,
            _ => return false,
        };
        matches!(visibility, syn::Visibility::Public(_))
    }
}
