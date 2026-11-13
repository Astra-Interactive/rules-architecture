use crate::check::api::Rule;
use crate::module::rule::ModelShouldBeImmutable;
use crate::test::declaration::util::read_code_context;

#[test]
fn given_resolved_model_aliases_and_borrowed_fields_when_written_then_assignments_are_reported() {
    for body in [
        "value.total = 2;",
        "let alias = &mut value; alias.total += 2;",
        "let field = &mut value.total; *field = 2;",
    ] {
        let context = read_code_context(&format!(
            "mod model {{ pub struct Invoice {{ pub total: i64 }} }} type Alias = crate::model::Invoice;\nfn change(mut value: Alias) {{ {body} }}"
        ));
        let findings = ModelShouldBeImmutable.check(&context);
        assert_eq!(findings.len(), 1, "{body}: {findings:?}");
        assert_eq!(findings[0].finding.location.line, Some(2));
    }
}

#[test]
fn given_external_values_and_shadowed_models_when_written_then_no_model_assignment_is_inferred() {
    let context = read_code_context(
        "mod model { pub struct Invoice { pub total: i64 } } struct External { total: i64 } fn change(value: crate::model::Invoice) { let mut value = External { total: 1 }; value.total = 2; } fn transition(value: crate::model::Invoice) -> crate::model::Invoice { crate::model::Invoice { total: value.total + 1 } }",
    );
    assert!(ModelShouldBeImmutable.check(&context).is_empty());
}

#[test]
fn given_macro_generated_mutation_when_indexed_then_it_remains_an_explicit_analysis_limit() {
    let context = read_code_context(
        "mod model { pub struct Invoice { pub total: i64 } } macro_rules! change { ($value:expr) => { $value.total = 2; }; } fn apply(mut value: crate::model::Invoice) { change!(value); }",
    );
    assert!(context.name_index().has_blind_spots);
}
#[test]
fn given_scoped_shadowing_when_mutating_an_external_closure_parameter_then_the_outer_model_does_not_leak()
 {
    let context = read_code_context(
        "mod model { pub struct Invoice { pub total: i64 } } struct External { total: i64 } fn change(value: crate::model::Invoice) { let callback = |mut value: External| { value.total = 2; }; } #[cfg(any())] fn disabled(mut value: crate::model::Invoice) { value.total = 2; }",
    );
    assert!(ModelShouldBeImmutable.check(&context).is_empty());
}
#[test]
fn given_for_match_and_if_let_shadowing_when_external_fields_are_written_then_model_identity_does_not_leak()
 {
    for body in [
        "for invoice in &mut [Other { total: 0 }] { invoice.total = 1; }",
        "match (Other { total: 0 }) { mut invoice => { invoice.total = 1; } }",
        "match (Other { total: 0 }) { mut invoice if true => { invoice.total = 1; } _ => {} }",
        "if let Some(mut invoice) = Some(Other { total: 0 }) { invoice.total = 1; }",
    ] {
        let context = read_code_context(&format!(
            "mod model {{ pub struct Invoice {{ pub total: u32 }} }} struct Other {{ total: u32 }} fn run(invoice: &mut model::Invoice) {{ {body} }}"
        ));
        assert!(
            context
                .crates
                .iter()
                .all(|source| source.unanalysable.is_empty()),
            "{body}"
        );
        assert!(ModelShouldBeImmutable.check(&context).is_empty(), "{body}");
    }
}
