use crate::check::api::Rule;
use crate::comment::di::CommentRuleModule;
use crate::config::di::ConfigRuleModule;
use crate::declaration::di::DeclarationRuleModule;
use crate::default::di::DefaultRuleModule;
use crate::dependency::di::DependencyRuleModule;
use crate::expression::di::ExpressionRuleModule;
use crate::family::di::FamilyRuleModule;
use crate::import::di::ImportRuleModule;
use crate::layout::di::LayoutRuleModule;
use crate::module::di::ModuleRuleModule;
use crate::naming::di::NamingRuleModule;
use crate::ordering::di::OrderingRuleModule;
use crate::readme::di::ReadmeRuleModule;
use crate::runtime::di::RuntimeRuleModule;
use crate::suppression::di::SuppressionRuleModule;
use crate::tuple::di::TupleRuleModule;
use std::sync::Arc;

pub(crate) struct RuleModule {
    comment_module: CommentRuleModule,
    config_module: ConfigRuleModule,
    declaration_module: DeclarationRuleModule,
    default_module: DefaultRuleModule,
    dependency_module: DependencyRuleModule,
    expression_module: ExpressionRuleModule,
    family_module: FamilyRuleModule,
    import_module: ImportRuleModule,
    layout_module: LayoutRuleModule,
    module_module: ModuleRuleModule,
    naming_module: NamingRuleModule,
    ordering_module: OrderingRuleModule,
    readme_module: ReadmeRuleModule,
    runtime_module: RuntimeRuleModule,
    suppression_module: SuppressionRuleModule,
    tuple_module: TupleRuleModule,
}

impl RuleModule {
    pub(crate) fn new() -> Self {
        Self {
            comment_module: CommentRuleModule::new(),
            config_module: ConfigRuleModule::new(),
            declaration_module: DeclarationRuleModule::new(),
            default_module: DefaultRuleModule::new(),
            dependency_module: DependencyRuleModule::new(),
            expression_module: ExpressionRuleModule::new(),
            family_module: FamilyRuleModule::new(),
            import_module: ImportRuleModule::new(),
            layout_module: LayoutRuleModule::new(),
            module_module: ModuleRuleModule::new(),
            naming_module: NamingRuleModule::new(),
            ordering_module: OrderingRuleModule::new(),
            readme_module: ReadmeRuleModule::new(),
            runtime_module: RuntimeRuleModule::new(),
            suppression_module: SuppressionRuleModule::new(),
            tuple_module: TupleRuleModule::new(),
        }
    }

    pub(crate) fn rules(&self) -> Vec<Arc<dyn Rule>> {
        let groups = [
            &self.comment_module.rules[..],
            &self.config_module.rules[..],
            &self.declaration_module.rules[..],
            &self.default_module.rules[..],
            &self.dependency_module.rules[..],
            &self.expression_module.rules[..],
            &self.family_module.rules[..],
            &self.import_module.rules[..],
            &self.layout_module.rules[..],
            &self.module_module.rules[..],
            &self.naming_module.rules[..],
            &self.ordering_module.rules[..],
            &self.readme_module.rules[..],
            &self.runtime_module.rules[..],
            &self.suppression_module.rules[..],
            &self.tuple_module.rules[..],
        ];
        groups
            .iter()
            .flat_map(|group| group.iter().cloned())
            .collect()
    }

    pub(crate) fn active_rule_ids(&self) -> Vec<&'static str> {
        let groups = [
            &self.comment_module.active_rule_ids[..],
            &self.config_module.active_rule_ids[..],
            &self.declaration_module.active_rule_ids[..],
            &self.default_module.active_rule_ids[..],
            &self.dependency_module.active_rule_ids[..],
            &self.expression_module.active_rule_ids[..],
            &self.family_module.active_rule_ids[..],
            &self.import_module.active_rule_ids[..],
            &self.layout_module.active_rule_ids[..],
            &self.module_module.active_rule_ids[..],
            &self.naming_module.active_rule_ids[..],
            &self.ordering_module.active_rule_ids[..],
            &self.readme_module.active_rule_ids[..],
            &self.runtime_module.active_rule_ids[..],
            &self.suppression_module.active_rule_ids[..],
            &self.tuple_module.active_rule_ids[..],
        ];
        groups
            .iter()
            .flat_map(|group| group.iter().copied())
            .collect()
    }
}
