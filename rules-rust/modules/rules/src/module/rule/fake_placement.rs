use crate::check::api::Rule;
use crate::check::model::{Anchor, Finding, Location, RuleContext, RuleFinding, RuleId};
use crate::module::util::{
    build_shape_item_finding, build_shape_module_finding, find_kind_name, item_concept,
    owns_implementation,
};
use crate::source::model::{IndexedItem, SourceTree};
use crate::workspace::model::TomlValue;
use syn::Visibility;

pub(crate) struct FakePlacement;

impl FakePlacement {
    fn check_fake(
        context: &RuleContext,
        ordinal: usize,
        identity: usize,
        item: &IndexedItem,
    ) -> Option<RuleFinding> {
        let index = &context.indexes[ordinal];
        let syn::Item::Struct(value) = &item.syntax else {
            return None;
        };
        if !item.name.starts_with("Fake") {
            return None;
        }
        let tree = index.modules[item.module].tree;
        let in_fake = find_kind_name(context, ordinal, item.module) == "fake";
        let correct_visibility = match tree {
            SourceTree::Fixture => matches!(value.vis, Visibility::Public(_)),
            SourceTree::Test => {
                matches!(&value.vis, Visibility::Restricted(restricted) if restricted.in_token.is_none() && restricted.path.is_ident("crate"))
            }
            _ => false,
        };
        let concept = item_concept(context, ordinal, item.module);
        let wrong_concept = index
            .implementations
            .iter()
            .filter(|implementation| owns_implementation(index, identity, implementation))
            .any(|implementation| {
                let Some((path, _)) = &implementation.syntax.trait_ else {
                    return false;
                };
                context
                    .resolve_workspace_path(ordinal, implementation.module, path)
                    .iter()
                    .any(|target| {
                        let Some(contract) = target
                            .path
                            .item
                            .map(|identity| &context.indexes[target.index].items[identity])
                        else {
                            return false;
                        };
                        matches!(contract.syntax, syn::Item::Trait(_))
                            && target.path.cfg.and(&implementation.cfg).is_satisfiable()
                            && (concept != item_concept(context, target.index, contract.module)
                                || target.index != ordinal)
                    })
            });
        let duplicate = index.items.iter().any(|other| {
            let other_tree = index.modules[other.module].tree;
            item.name == other.name
                && concept == item_concept(context, ordinal, other.module)
                && matches!(
                    (tree, other_tree),
                    (SourceTree::Test, SourceTree::Fixture)
                        | (SourceTree::Fixture, SourceTree::Test)
                )
                && item.cfg.and(&other.cfg).is_satisfiable()
        });
        (!in_fake || !correct_visibility || wrong_concept || duplicate).then(|| build_shape_item_finding(index, item, "FakePlacement", "A fake lives once in its trait's concept/fake tree, pub(crate) in test or pub in fixture (arch-package.rs.md)"))
    }

    fn feature_finding(context: &RuleContext, crate_name: &str) -> Option<RuleFinding> {
        let member = context
            .workspace
            .members
            .iter()
            .find(|member| member.name == crate_name)?;
        let feature = member.manifest.root.find(&["features", "fake"]);
        let fixture = context
            .indexes
            .iter()
            .filter(|index| index.crate_name == crate_name)
            .any(|index| {
                index
                    .modules
                    .iter()
                    .any(|module| module.tree == SourceTree::Fixture)
            });
        let valid_feature =
            feature.is_some_and(|value| value.as_array().is_some_and(<[TomlValue]>::is_empty));
        let mismatch = (fixture && !valid_feature) || (!fixture && feature.is_some());
        mismatch.then(|| RuleFinding {
            finding: Finding { rule_id: RuleId("FakePlacement".to_owned()), location: Location { path: member.manifest.path.clone(), line: feature.map(|feature| feature.line) }, subject: format!("{crate_name} features.fake"), message: "The fixture block and an empty fake = [] feature must be declared together (arch-package.rs.md)".to_owned() },
            anchor: Anchor::Crate(member.name.clone()),
        })
    }

    fn build_default_finding(context: &RuleContext, crate_name: &str) -> Option<RuleFinding> {
        let member = context
            .workspace
            .members
            .iter()
            .find(|member| member.name == crate_name)?;
        let default = member.manifest.root.find(&["features", "default"])?;
        let enables_fake = default
            .as_array()?
            .iter()
            .any(|entry| entry.as_str() == Some("fake"));
        enables_fake.then(|| RuleFinding {
            finding: Finding {
                rule_id: RuleId("FakePlacement".to_owned()),
                location: Location {
                    path: member.manifest.path.clone(),
                    line: Some(default.line),
                },
                subject: format!("{crate_name} features.default"),
                message: "A default feature must not enable fake: a release build compiles no fixture tree (arch-package.rs.md)".to_owned(),
            },
            anchor: Anchor::Crate(member.name.clone()),
        })
    }
}

impl Rule for FakePlacement {
    fn check(&self, context: &RuleContext) -> Vec<RuleFinding> {
        let mut findings: Vec<RuleFinding> = context.workspace.members.iter().filter(|member| {
            member.directory.rsplit('/').next() == Some("fake")
                || member.name.rsplit('-').next() == Some("fake")
        }).map(|member| RuleFinding {
            finding: Finding {
                rule_id: RuleId("FakePlacement".to_owned()),
                location: Location { path: member.manifest.path.clone(), line: None },
                subject: member.name.clone(),
                message: "shared fakes belong to the owning crate's fixture tree behind its fake feature, never a separate crate (arch-package.rs.md, Tests)".to_owned(),
            },
            anchor: Anchor::Crate(member.name.clone()),
        }).collect();
        for (ordinal, index) in context.indexes.iter().enumerate() {
            findings.extend(
                index
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(identity, item)| {
                        Self::check_fake(context, ordinal, identity, item)
                    }),
            );
            findings.extend(index.modules.iter().filter(|module| module.tree == SourceTree::Src && !module.is_file_module && module.module.path.last().is_some_and(|name| name == "fake")).filter_map(|module| build_shape_module_finding(context, ordinal, module.id, "FakePlacement", "The fake kind exists only in test or fixture trees (arch-package.rs.md)")));
        }
        findings.extend(
            context
                .workspace
                .members
                .iter()
                .filter_map(|member| Self::feature_finding(context, &member.name)),
        );
        findings.extend(
            context
                .workspace
                .members
                .iter()
                .filter_map(|member| Self::build_default_finding(context, &member.name)),
        );
        findings
    }
}
