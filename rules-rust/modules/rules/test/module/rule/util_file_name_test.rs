use crate::check::api::Rule;
use crate::module::rule::UtilFileName;
use crate::test::check::usecase::read_case_context;
use crate::test::module::util::assert_placement_findings;
use crate::test::module::util::read_placement_context;
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
fn given_extension_trait_of_another_crates_trait_when_its_one_impl_is_the_bounded_blanket_then_passes()
 {
    for source in [
        "use vendor::Buf;\npub trait BufExt: Buf { fn skip_all(&mut self); }\nimpl<B> BufExt for B\nwhere\n    B: Buf + ?Sized,\n{\n    fn skip_all(&mut self) {}\n}",
        "pub trait BufExt: vendor::Buf {}\nimpl<B: vendor::Buf + ?Sized> BufExt for B {}",
        "use vendor::Buf;\npub trait BufExt: Buf {}\nimpl<B: ?Sized + Buf> BufExt for B {}",
    ] {
        let context = read_placement_context(&[
            ("src/lib.rs", "mod util { mod buf_ext; }"),
            ("src/util/buf_ext.rs", source),
        ]);
        assert_placement_findings(&UtilFileName, "UtilFileName", &context, &[]);
    }
}

#[test]
fn given_blanket_impl_with_inline_parameter_and_where_clause_bound_when_checked_then_passes() {
    for source in [
        "use vendor::Buf;\npub trait BufExt: Buf {}\nimpl<B: ?Sized> BufExt for B where B: Buf {}",
        "use vendor::Buf;\npub trait BufExt: Buf {}\nimpl<B> BufExt for B where B: Sized + Buf {}",
    ] {
        let context = read_placement_context(&[
            ("src/lib.rs", "mod util { mod buf_ext; }"),
            ("src/util/buf_ext.rs", source),
        ]);
        assert_placement_findings(&UtilFileName, "UtilFileName", &context, &[]);
    }
}

#[test]
fn given_blanket_impl_of_an_extension_trait_without_the_owner_bound_when_checked_then_reports_the_trait()
 {
    for source in [
        "pub trait BufExt {}\nimpl<T> BufExt for T {}",
        "pub trait BufExt {}\nimpl<B: vendor::Other> BufExt for B {}",
    ] {
        let context = read_placement_context(&[
            ("src/lib.rs", "mod util { mod buf_ext; }"),
            ("src/util/buf_ext.rs", source),
        ]);
        assert_placement_findings(
            &UtilFileName,
            "UtilFileName",
            &context,
            &[("src/util/buf_ext.rs", 1, "Node")],
        );
    }
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod api { pub trait Buf {} } mod util { mod buf_ext; }",
        ),
        (
            "src/util/buf_ext.rs",
            "use crate::api::Buf;\npub trait BufExt {}\nimpl<B: Buf> BufExt for B {}",
        ),
    ]);
    assert_placement_findings(
        &UtilFileName,
        "UtilFileName",
        &context,
        &[("src/util/buf_ext.rs", 2, "Node")],
    );
    let context = read_placement_context(&[
        ("src/lib.rs", "mod util { mod buf_ext; mod other; }"),
        ("src/util/buf_ext.rs", "pub trait BufExt {}"),
        (
            "src/util/other.rs",
            "use vendor::Buf;\nimpl<B: Buf + ?Sized> super::buf_ext::BufExt for B {}",
        ),
    ]);
    assert_placement_findings(
        &UtilFileName,
        "UtilFileName",
        &context,
        &[("src/util/buf_ext.rs", 1, "Node")],
    );
}

fn assert_sized_extension_reports_trait_at(source: &str, line: u32) {
    let context = read_placement_context(&[
        ("src/lib.rs", "mod util { mod sized_ext; }"),
        ("src/util/sized_ext.rs", source),
    ]);
    assert_placement_findings(
        &UtilFileName,
        "UtilFileName",
        &context,
        &[("src/util/sized_ext.rs", line, "Node")],
    );
}

#[test]
fn given_blanket_impl_whose_only_bound_is_a_maybe_bound_when_checked_then_reports_the_trait() {
    assert_sized_extension_reports_trait_at(
        "pub trait SizedExt {}\nimpl<T: ?Sized> SizedExt for T {}",
        1,
    );
    assert_sized_extension_reports_trait_at(
        "pub trait SizedExt {}\nimpl<T> SizedExt for T where T: ?Sized {}",
        1,
    );
    assert_sized_extension_reports_trait_at(
        "use vendor::Sized;\npub trait SizedExt {}\nimpl<T: ?Sized> SizedExt for T {}",
        2,
    );
}

#[test]
fn given_primitive_and_slice_extension_traits_when_checked_then_reports_only_the_missing_impl() {
    let context = read_shape_context(
        "mod util { mod str_ext; mod u64_ext; mod slice_ext; mod string_ext; }",
        &[
            (
                "util/str_ext.rs",
                "trait StrExt { fn is_blank_line(&self) -> bool; } impl StrExt for str { fn is_blank_line(&self) -> bool { self.trim().is_empty() } }",
            ),
            (
                "util/u64_ext.rs",
                "trait U64Ext { fn is_even(&self) -> bool; } impl U64Ext for u64 { fn is_even(&self) -> bool { *self % 2 == 0 } }",
            ),
            (
                "util/slice_ext.rs",
                "trait SliceExt { fn is_ascii_run(&self) -> bool; } impl SliceExt for [u8] { fn is_ascii_run(&self) -> bool { self.is_ascii() } }",
            ),
            ("util/string_ext.rs", "trait StringExt {}"),
        ],
    );
    let findings = UtilFileName.check(&context);
    assert_eq!(1, findings.len(), "{findings:?}");
    assert!(
        findings[0]
            .finding
            .location
            .path
            .ends_with("util/string_ext.rs"),
        "{findings:?}"
    );
    assert_eq!(Some(1), findings[0].finding.location.line);
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
