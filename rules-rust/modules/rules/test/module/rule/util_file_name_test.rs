use crate::check::api::Rule;
use crate::module::rule::UtilFileName;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_generated_extension_file_when_checked_then_its_free_function_is_exempt() {
    let context = read_shape_context(
        "mod util { mod invoice_ext; }",
        &[(
            "util/invoice_ext.rs",
            "// This file is @generated.\nfn compute_values() {}",
        )],
    );
    assert!(UtilFileName.check(&context).is_empty());
}

#[test]
fn given_type_extensions_when_checked_then_wrong_file_mixed_types_and_local_extension_traits_report()
 {
    let context = read_shape_context(
        "mod model { pub struct Invoice; pub struct Order; } mod util { mod invoice_rules; mod invoice_ext; }",
        &[
            ("util/invoice_rules.rs", "impl crate::model::Invoice {}"),
            (
                "util/invoice_ext.rs",
                "use crate::model::Invoice; trait InvoiceExt {} impl crate::model::Invoice {} impl crate::model::Order {}",
            ),
        ],
    );
    let findings = UtilFileName.check(&context);
    assert_eq!(findings.len(), 3);
    let context = read_shape_context(
        "mod model { pub struct Invoice; } mod util { mod invoice_ext; }",
        &[(
            "util/invoice_ext.rs",
            "use crate::model::Invoice as Alias; #[cfg(unix)] impl Alias {}",
        )],
    );
    assert!(UtilFileName.check(&context).is_empty());
}

#[test]
fn given_declaration_shapes_when_checked_then_exact_wrong_sites_report_and_correct_shapes_pass() {
    let context = read_case_context("UtilFileName", "fail-declaration-shape");
    let mut actual: Vec<String> = UtilFileName
        .check(&context)
        .iter()
        .map(|finding| {
            format!(
                "{}:{} {}",
                finding.finding.location.path,
                finding.finding.location.line.expect("finding line"),
                finding.finding.subject
            )
        })
        .collect();
    actual.sort();
    assert_eq!(
        actual,
        ["modules/sample/src/lib.rs:2 case_sample::util::invoice_ext"]
    );
    let context = read_case_context("UtilFileName", "pass-declaration-shape");
    assert!(UtilFileName.check(&context).is_empty());
}
#[test]
fn given_external_extension_trait_when_checked_then_requires_one_matching_same_file_implementation()
{
    for implementation in [
        "",
        "impl StringExt for u64 {}",
        "impl StringExt for String {} impl StringExt for u64 {}",
        "impl StringExt for String {}",
    ] {
        let context = read_shape_context(
            "mod util { mod string_ext; }",
            &[(
                "util/string_ext.rs",
                &format!("trait StringExt {{}} {implementation}"),
            )],
        );
        assert_eq!(
            UtilFileName.check(&context).is_empty(),
            implementation == "impl StringExt for String {}",
            "{implementation}"
        );
    }
    let context = read_shape_context(
        "mod util { mod string_ext; mod other; }",
        &[
            ("util/string_ext.rs", "pub trait StringExt {}"),
            (
                "util/other.rs",
                "impl super::string_ext::StringExt for String {}",
            ),
        ],
    );
    assert!(!UtilFileName.check(&context).is_empty());
}
#[test]
fn given_cfg_complementary_extension_implementations_when_checked_then_each_configuration_has_one()
{
    let context = read_shape_context(
        "mod util { mod string_ext; }",
        &[(
            "util/string_ext.rs",
            "trait StringExt {} #[cfg(unix)] impl StringExt for String {} #[cfg(not(unix))] impl StringExt for String {}",
        )],
    );
    assert!(UtilFileName.check(&context).is_empty());
}
