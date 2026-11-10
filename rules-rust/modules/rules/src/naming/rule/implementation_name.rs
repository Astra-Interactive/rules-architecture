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

pub(crate) struct ImplementationName;

impl ImplementationName {
    fn build_item_finding(index: &CrateIndex, item: &IndexedItem, message: &str) -> RuleFinding {
        build_node_finding(
            "ImplementationName",
            &item.file,
            item.span,
            format!(
                "{}::{}",
                format_module_path(&index.crate_name, &index.modules[item.module].module.path),
                item.name
            ),
            message.to_owned(),
        )
    }

    fn is_api_contract(index: &CrateIndex, contract: &IndexedItem) -> bool {
        matches!(contract.syntax, syn::Item::Trait(_)) && has_kind(index, contract.module, "api")
    }

    fn check_contract(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        contract: &IndexedItem,
        types: &[usize],
    ) -> Vec<RuleFinding> {
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
                    .filter_map(|word| item.name.strip_prefix(word))
                    .any(|rest| rest.chars().next().is_none_or(|next| !next.is_lowercase()))
            };
            if vague_prefix {
                findings.push(Self::build_item_finding(
                    index,
                    item,
                    "Database, Network, Local and Remote are not technology prefixes (arch-package.md)",
                ));
            }
        }
        findings
    }

    fn check_interface_prefix(
        index: &CrateIndex,
        implementation: &IndexedImpl,
        types: &[usize],
    ) -> Vec<RuleFinding> {
        types
            .iter()
            .map(|target| &index.items[*target])
            .filter(|item| {
                matches!(item.syntax, syn::Item::Struct(_))
                    && NameVisitor::has_interface_prefix(&item.name)
                    && implementation.cfg.and(&item.cfg).is_satisfiable()
            })
            .map(|item| {
                Self::build_item_finding(index, item, NameVisitor::IMPLEMENTATION_NAME_MESSAGE)
            })
            .collect()
    }

    fn check_impl(
        context: &RuleContext,
        ordinal: usize,
        implementation: &IndexedImpl,
    ) -> Vec<RuleFinding> {
        let Some((trait_path, _)) = &implementation.syntax.trait_ else {
            return Vec::new();
        };
        let index = &context.indexes[ordinal];
        let types = index.resolve_type_items(implementation.module, &implementation.syntax.self_ty);
        let mut findings = Self::check_interface_prefix(index, implementation, &types);
        let traits = context.resolve_workspace_path(ordinal, implementation.module, trait_path);
        findings.extend(
            traits
                .iter()
                .filter(|target| target.path.cfg.and(&implementation.cfg).is_satisfiable())
                .flat_map(|target| {
                    let contract_index = &context.indexes[target.index];
                    let Some(contract) = target
                        .path
                        .item
                        .and_then(|identity| contract_index.items.get(identity))
                    else {
                        return Vec::new();
                    };
                    if !Self::is_api_contract(contract_index, contract) {
                        return Vec::new();
                    }
                    Self::check_contract(index, implementation, contract, &types)
                }),
        );
        findings
    }
}

impl Rule for ImplementationName {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings = NameVisitor::check_context(context, "ImplementationName");
        findings.extend(
            context
                .indexes
                .iter()
                .enumerate()
                .flat_map(|(ordinal, index)| {
                    index
                        .implementations
                        .iter()
                        .flat_map(move |implementation| {
                            Self::check_impl(context, ordinal, implementation)
                        })
                }),
        );
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
