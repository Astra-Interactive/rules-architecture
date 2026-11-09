use crate::module::rule::EnumFamilyFile;
use crate::test::module::util::assert_placement_fixture;
use crate::test::module::util::{assert_placement_findings, read_placement_context};

#[test]
fn given_other_indexed_uses_when_checked_then_keeps_a_shared_payload_in_its_file() {
    let root = "mod sample { mod model { pub(crate) use payload::Payload; mod payload; mod outcome; } mod util { mod consume; } }";
    let payload = "pub(crate) struct Payload { value: u64 }";
    let outcome = "use crate::sample::model::Payload as Data; type Alias = Data; pub(crate) enum Outcome { Ready(Alias) }";
    let consumer = "use crate::sample::model::Payload as Data; fn consume(value: Data) {}";
    let context = read_placement_context(&[
        ("src/lib.rs", root),
        ("src/sample/model/payload.rs", payload),
        ("src/sample/model/outcome.rs", outcome),
        ("src/sample/util/consume.rs", consumer),
    ]);
    assert_placement_findings(&EnumFamilyFile, "EnumFamilyFile", &context, &[]);
    let inactive = "#[cfg(all(target_os = \"linux\", target_os = \"windows\"))] fn consume(value: crate::sample::model::Payload) {}";
    let context = read_placement_context(&[
        ("src/lib.rs", root),
        ("src/sample/model/payload.rs", payload),
        ("src/sample/model/outcome.rs", outcome),
        ("src/sample/util/consume.rs", inactive),
    ]);
    assert_placement_findings(
        &EnumFamilyFile,
        "EnumFamilyFile",
        &context,
        &[("src/sample/model/payload.rs", 1, "Node")],
    );
}

#[test]
fn given_two_enum_families_or_payload_behavior_when_checked_then_does_not_merge_the_type() {
    let root = "mod model { pub(crate) use payload::Payload; mod payload; mod outcome; }";
    for payload in [
        "pub(crate) struct Payload; impl Payload { fn read(&self) {} }",
        "pub(crate) struct Payload;",
    ] {
        let outcome = "use crate::model::Payload; enum First { Value(Payload) } enum Second { Value(Payload) }";
        let context = read_placement_context(&[
            ("src/lib.rs", root),
            ("src/model/payload.rs", payload),
            ("src/model/outcome.rs", outcome),
        ]);
        assert_placement_findings(&EnumFamilyFile, "EnumFamilyFile", &context, &[]);
    }
}

#[test]
fn given_generic_payload_shadowing_a_type_name_when_checked_then_does_not_infer_a_family_use() {
    let context = read_placement_context(&[
        (
            "src/lib.rs",
            "mod model { pub(crate) use payload::Payload; mod payload; mod outcome; }",
        ),
        ("src/model/payload.rs", "pub(crate) struct Payload;"),
        (
            "src/model/outcome.rs",
            "use crate::model::Payload; enum Outcome<Payload> { Ready(Payload) }",
        ),
    ]);
    assert_placement_findings(&EnumFamilyFile, "EnumFamilyFile", &context, &[]);
}

#[test]
fn given_exclusive_enum_payload_in_another_file_when_checked_then_reports_its_exact_declaration() {
    assert_placement_fixture(&EnumFamilyFile, "EnumFamilyFile", "bad");
    assert_placement_fixture(&EnumFamilyFile, "EnumFamilyFile", "good");
}
