use crate::check::api::Rule;
use crate::module::rule::ModuleVisibility;
use crate::source::model::{DeclaredVisibility, SourceTree};
use crate::test::check::usecase::read_case_context;

#[test]
fn given_module_trees_when_checked_then_visibility_tracks_reexports_and_test_boundaries() {
    let mut context = read_case_context("ModuleVisibility", "pass-public");
    assert!(ModuleVisibility.check(&context).is_empty());
    for module in &mut context.crates[0].modules {
        module.tree = SourceTree::Test;
        if let Some(declaration) = &mut module.declaration {
            declaration.visibility = DeclaredVisibility::Private;
            declaration.use_visibilities.clear();
        }
    }
    assert!(ModuleVisibility.check(&context).is_empty());
    let declaration = context.crates[0].modules[2]
        .declaration
        .as_mut()
        .expect("model declared");
    declaration.use_visibilities.push(DeclaredVisibility::Crate);
    declaration.visibility = DeclaredVisibility::Restricted;
    assert_eq!(ModuleVisibility.check(&context).len(), 1);
    context.crates[0].modules[2]
        .declaration
        .as_mut()
        .expect("model declared")
        .visibility = DeclaredVisibility::Crate;
    assert!(ModuleVisibility.check(&context).is_empty());
}
