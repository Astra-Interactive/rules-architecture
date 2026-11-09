use crate::check::api::Rule;
use crate::check::model::RuleContext;
use crate::check::model::RuleFinding;
use crate::check::util::build_node_finding;
use crate::check::util::format_module_path;
use crate::declaration::util::has_kind;
use crate::naming::rule::NameVisitor;
use crate::source::model::CrateIndex;
use crate::source::model::IndexedImpl;
use crate::source::model::IndexedItem;
use crate::source::model::Resolution;

pub(crate) struct ImplementationName;

impl ImplementationName {
    fn check_contract(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        contract: &IndexedItem,
        types: &[usize],
    ) -> Vec<RuleFinding> {
        if !matches!(contract.syntax, syn::Item::Trait(_))
            || !has_kind(index, contract.module, "api")
        {
            return Vec::new();
        }
        let mut findings = Vec::new();
        for target in types {
            let item = &index.items[*target];
            if !contract
                .cfg
                .and(&implementation.cfg)
                .and(&item.cfg)
                .is_satisfiable()
            {
                continue;
            }
            let vague_prefix = {
                ["Database", "Network", "Local", "Remote"]
                    .iter()
                    .any(|word| item.name.starts_with(word))
            };
            if vague_prefix {
                findings.push(build_node_finding("ImplementationName", &item.file, item.span, format!("{}::{}", format_module_path(&index.crate_name, &index.modules[item.module].module.path), item.name), "Database, Network, Local and Remote are not technology prefixes (arch-package.md)".to_owned()));
            }
        }
        findings
    }

    fn check_impl(index: &CrateIndex, implementation: &IndexedImpl) -> Vec<RuleFinding> {
        let Some((trait_path, _)) = &implementation.syntax.trait_ else {
            return Vec::new();
        };
        let Resolution::Internal(traits) = index.resolve_path(implementation.module, trait_path)
        else {
            return Vec::new();
        };
        let types = index.resolve_type_items(implementation.module, &implementation.syntax.self_ty);
        traits
            .iter()
            .filter(|target| target.cfg.and(&implementation.cfg).is_satisfiable())
            .filter_map(|target| target.item.and_then(|identity| index.items.get(identity)))
            .flat_map(|contract| Self::check_contract(index, implementation, contract, &types))
            .collect()
    }
}

impl Rule for ImplementationName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = NameVisitor::check_context(context, "ImplementationName");
        findings.extend(context.indexes.iter().flat_map(|index| {
            index
                .implementations
                .iter()
                .flat_map(|implementation| Self::check_impl(index, implementation))
        }));
        findings.sort_by(|left, right| {
            left.finding
                .location
                .cmp(&right.finding.location)
                .then(left.finding.subject.cmp(&right.finding.subject))
        });
        findings.dedup_by(|left, right| left.finding == right.finding);
        findings
    }
}
