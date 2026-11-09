#[test]
fn given_registration_parameter_alias_when_classified_then_broad_shape_keeps_external_paths_and_rejects_primitives()
 {
    let context = crate::test::module::util::read_di_context(
        "mod route { type External = std::path::Path; trait Setup { fn register(&self, context: &External); } trait Primitive { fn register(&self, context: u64); } }",
    );
    let setup = context.indexes[0]
        .items
        .iter()
        .find(|item| item.name == "Setup")
        .expect("trait");
    let primitive = context.indexes[0]
        .items
        .iter()
        .find(|item| item.name == "Primitive")
        .expect("trait");
    assert!(context.has_registration_shape(0, setup));
    assert!(!context.has_registration_shape(0, primitive));
}
