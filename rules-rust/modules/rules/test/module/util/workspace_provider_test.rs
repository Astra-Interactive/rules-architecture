use crate::check::model::RuleContext;
use crate::test::declaration::util::read_code_context;
use crate::workspace::model::Dependency;
use crate::workspace::model::DependencyKind;

pub(crate) fn add_workspace_provider(
    context: &mut RuleContext,
    provider_source: &str,
    code_name: &str,
) {
    let mut provider = read_code_context(provider_source);
    let mut member = provider.workspace.members.remove(0);
    member.name = "case-provider".to_owned();
    member.directory = "modules/provider".to_owned();
    let mut index = provider.indexes.remove(0);
    index.crate_name = member.name.clone();
    for module in &mut index.modules {
        module.module.crate_name = member.name.clone();
    }
    context.workspace.members[0].dependencies.push(Dependency {
        name: member.name.clone(),
        code_name: code_name.to_owned(),
        path: Some(
            context
                .workspace
                .root
                .join(&member.directory)
                .to_string_lossy()
                .into_owned(),
        ),
        kind: DependencyKind::Normal,
        features: Vec::new(),
        is_target_specific: false,
        line: Some(1),
    });
    context.workspace.members.push(member);
    context.indexes.push(index);
}
