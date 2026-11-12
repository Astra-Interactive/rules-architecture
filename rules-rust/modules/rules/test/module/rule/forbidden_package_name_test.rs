use crate::check::api::Rule;
use crate::check::model::Anchor;
use crate::config::model::Role;
use crate::module::rule::ForbiddenPackageName;
use crate::test::module::util::assert_import_pair;
use crate::test::module::util::read_import_context;
use crate::test::module::util::read_shape_context;

#[test]
fn given_file_modules_named_by_a_type_when_checked_then_only_blocks_and_standard_crate_names_report()
 {
    let context = read_shape_context(
        "mod api {\n    mod port;\n    mod std;\n    mod core;\n    pub use port::Port;\n    pub trait Source { fn read(&self) -> Port; }\n}\nmod model {\n    mod port {}\n}\n",
        &[
            ("api/port.rs", "pub struct Port;"),
            ("api/std.rs", "pub struct Handle;"),
            ("api/core.rs", "pub struct Engine;"),
        ],
    );
    let mut actual: Vec<String> = ForbiddenPackageName
        .check(&context)
        .iter()
        .map(|finding| {
            assert_eq!("ForbiddenPackageName", finding.finding.rule_id.0);
            let name = finding.finding.subject.rsplit("::").next().expect("name");
            format!("{} {name}", finding.finding.location.line.expect("line"))
        })
        .collect();
    actual.sort();
    assert_eq!(actual, ["3 std", "4 core", "9 port"]);
}

#[test]
fn given_prelude_file_module_in_a_kind_block_when_checked_then_it_is_reported() {
    let context = read_shape_context(
        "mod api {\n    mod port;\n    mod prelude;\n    pub use port::Port;\n}",
        &[
            ("api/port.rs", "pub struct Port;"),
            ("api/prelude.rs", "pub use super::Port;"),
        ],
    );
    let actual: Vec<String> = ForbiddenPackageName
        .check(&context)
        .iter()
        .map(|finding| {
            let name = finding.finding.subject.rsplit("::").next().expect("name");
            format!("{} {name}", finding.finding.location.line.expect("line"))
        })
        .collect();
    assert_eq!(actual, ["3 prelude"]);
}

#[test]
fn given_persistent_phase3_package_pairs_when_checked_then_only_the_technology_role_allows_storage_kinds()
 {
    assert_import_pair(
        &ForbiddenPackageName,
        "ForbiddenPackageName",
        Role::Technology,
    );
}

#[test]
fn given_technology_only_package_names_when_checked_then_role_controls_all_three_names_and_module_anchors()
 {
    for role in [Role::Technology, Role::Platform, Role::CorePlatform] {
        let context = read_import_context(
            "mod table {} mod entity {} mod database {}",
            role,
            &[],
            &[],
            "app",
        );
        assert!(ForbiddenPackageName.check(&context).is_empty(), "{role:?}");
    }
    for role in [
        Role::Context,
        Role::Api,
        Role::Core,
        Role::CoreApi,
        Role::Contract,
    ] {
        let context = read_import_context(
            "mod table {}\nmod entity {}\nmod database {}",
            role,
            &[],
            &[],
            "app",
        );
        let findings = ForbiddenPackageName.check(&context);
        assert_eq!(findings.len(), 3, "{role:?}");
        assert!(findings.iter().all(|finding| matches!(&finding.anchor, Anchor::Module { file, body_file, .. } if file == "src/lib.rs" && body_file == "src/lib.rs")));
    }
}
