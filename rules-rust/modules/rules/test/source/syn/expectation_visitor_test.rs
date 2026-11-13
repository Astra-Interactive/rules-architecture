use crate::source::model::{ExpectationPosition, ModuleId};
use crate::source::syn::ExpectationVisitor;
use syn::visit::Visit;

#[test]
fn given_invisible_macro_group_with_an_attribute_when_visited_then_its_node_expectation_is_collected()
 {
    let group = proc_macro2::Group::new(
        proc_macro2::Delimiter::None,
        "value".parse().expect("expression tokens"),
    );
    let tokens =
        quote::quote!(#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason="group"))] #group);
    let expression = syn::parse2::<syn::Expr>(tokens).expect("macro group expression");
    assert!(matches!(expression, syn::Expr::Group(_)));
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: Vec::new(),
    };
    let mut visitor = ExpectationVisitor::new("src/lib.rs".to_owned(), module);
    visitor.visit_expr(&expression);
    let opaque = syn::parse_str::<syn::Expr>("become f()")
        .expect("syn preserves explicit tail calls as opaque expressions");
    assert!(matches!(opaque, syn::Expr::Verbatim(_)));
    visitor.visit_expr(&opaque);
    let expectations = visitor.collect_file(&syn::parse_file("").expect("empty file"), false);
    assert_eq!(expectations.len(), 1);
    assert_eq!(expectations[0].reason, "group");
}

#[test]
fn given_expression_variants_with_attributes_when_visited_then_every_supported_node_retains_its_expectation()
 {
    let expressions = [
        "[1]",
        "x = 1",
        "async {}",
        "x.await",
        "x + 1",
        "{}",
        "break",
        "f()",
        "x as u8",
        "|x| x",
        "const {}",
        "continue",
        "x.field",
        "for x in xs {}",
        "if true {}",
        "x[0]",
        "_",
        "let x = y",
        "1",
        "loop {}",
        "m!()",
        "match x {}",
        "x.f()",
        "(x)",
        "x",
        "0..1",
        "&raw const x",
        "&x",
        "[0; 2]",
        "return",
        "S {}",
        "x?",
        "try {}",
        "(1,2)",
        "-x",
        "unsafe {}",
        "while true {}",
        "yield x",
    ];
    for expression in expressions {
        let source = format!(
            "#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"expression\"))] {expression}"
        );
        let expression = syn::parse_str::<syn::Expr>(&source).expect("expression syntax");
        let module = ModuleId {
            crate_name: "sample".to_owned(),
            target: "sample".to_owned(),
            path: Vec::new(),
        };
        let mut visitor = ExpectationVisitor::new("src/lib.rs".to_owned(), module);
        visitor.visit_expr(&expression);
        let expectations = visitor.collect_file(&syn::parse_file("").expect("empty file"), false);
        assert_eq!(expectations.len(), 1, "{source}");
        assert_eq!(expectations[0].reason, "expression");
    }
}

#[test]
fn given_raw_module_expectations_when_collected_then_anchor_names_are_unraw() {
    let syntax = syn::parse_file("#[cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason=\"outer\"))] mod r#type { #![cfg_attr(klibs_rules, expect(ForbiddenPackageName, reason=\"inner\"))] }").expect("raw module syntax");
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: Vec::new(),
    };
    let expectations =
        ExpectationVisitor::new("src/lib.rs".to_owned(), module).collect_file(&syntax, false);
    assert_eq!(expectations.len(), 2);
    assert!(
        matches!(&expectations[0].position, ExpectationPosition::ModuleDeclaration { module, .. } if module.path == ["type"])
    );
    assert!(
        matches!(&expectations[1].position, ExpectationPosition::InlineModuleInner { module, .. } if module.path == ["type"])
    );
}

#[test]
fn given_configuration_literals_mentioning_the_linter_when_collected_then_they_are_not_architecture_expectations()
 {
    let source = "#[cfg_attr(feature=\"docs\", doc=\"klibs_rules\")] fn sample() {} #[cfg_attr(feature=\"klibs_rules\", inline)] fn other() {} #[cfg_attr = \"klibs_rules\"] fn malformed_rust() {}";
    let syntax = syn::parse_file(source).expect("attribute syntax");
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: Vec::new(),
    };
    assert!(
        ExpectationVisitor::new("src/lib.rs".to_owned(), module)
            .collect_file(&syntax, false)
            .is_empty()
    );
}

#[test]
fn given_expected_expression_inside_macro_when_collected_then_node_scope_is_retained() {
    let syntax = syn::parse_file("fn run() { call!(#[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"macro\"))] { let value: (u8,u8) = (1,2); }, #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason=\"closure\"))] |value: (u8,u8)| value); }").expect("macro syntax");
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: Vec::new(),
    };
    let expectations =
        ExpectationVisitor::new("src/lib.rs".to_owned(), module).collect_file(&syntax, false);
    assert_eq!(expectations.len(), 2);
    assert!(matches!(
        expectations[0].position,
        ExpectationPosition::Node(_)
    ));
}

#[test]
fn given_verbatim_items_and_expression_reasons_when_collected_then_unknown_nodes_are_safe_and_reason_is_rejected()
 {
    let syntax = syn::parse_file("fn missing_body(); impl Thing { type Missing; } trait Contract { pub type Hidden; } extern \"C\" { type Opaque = u8; } #[cfg_attr(klibs_rules, expect(AnonymousTuple, reason = source::TEXT))] struct Value;").expect("syn preserves unsupported items as Verbatim");
    let module = ModuleId {
        crate_name: "sample".to_owned(),
        target: "sample".to_owned(),
        path: Vec::new(),
    };
    let expectations =
        ExpectationVisitor::new("src/lib.rs".to_owned(), module).collect_file(&syntax, false);
    assert_eq!(expectations.len(), 1);
    assert_eq!(
        expectations[0].malformation.as_deref(),
        Some("the reason is not a string literal")
    );
}
