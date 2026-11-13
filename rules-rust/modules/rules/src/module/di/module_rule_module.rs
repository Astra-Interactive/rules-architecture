use crate::check::api::Rule;
use crate::module::di::{
    ModuleConstructionRuleModule, ModuleOwnershipRuleModule, ModuleShapeRuleModule,
    ModuleTreeRuleModule,
};
use std::sync::Arc;

pub(crate) struct ModuleRuleModule {
    _module_tree_rule_module: ModuleTreeRuleModule,
    _module_shape_rule_module: ModuleShapeRuleModule,
    _module_construction_rule_module: ModuleConstructionRuleModule,
    _module_ownership_rule_module: ModuleOwnershipRuleModule,
    pub(crate) rules: Arc<[Arc<dyn Rule>]>,
    pub(crate) active_rule_ids: Vec<&'static str>,
}

impl ModuleRuleModule {
    pub(crate) fn new() -> Self {
        let module_tree_rule_module = ModuleTreeRuleModule::new();
        let module_shape_rule_module = ModuleShapeRuleModule::new();
        let module_construction_rule_module = ModuleConstructionRuleModule::new();
        let module_ownership_rule_module = ModuleOwnershipRuleModule::new();
        let groups = [
            &module_tree_rule_module.rules[..],
            &module_shape_rule_module.rules[..],
            &module_construction_rule_module.rules[..],
            &module_ownership_rule_module.rules[..],
        ];
        let rules = groups
            .iter()
            .flat_map(|group| group.iter().cloned())
            .collect::<Vec<_>>()
            .into();
        let groups = [
            &module_tree_rule_module.active_rule_ids[..],
            &module_shape_rule_module.active_rule_ids[..],
            &module_construction_rule_module.active_rule_ids[..],
            &module_ownership_rule_module.active_rule_ids[..],
        ];
        let active_rule_ids = groups
            .iter()
            .flat_map(|group| group.iter().copied())
            .collect();
        Self {
            _module_tree_rule_module: module_tree_rule_module,
            _module_shape_rule_module: module_shape_rule_module,
            _module_construction_rule_module: module_construction_rule_module,
            _module_ownership_rule_module: module_ownership_rule_module,
            rules,
            active_rule_ids,
        }
    }
}
