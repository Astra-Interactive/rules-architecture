use crate::check::api::Rule;
use crate::config::model::Role;
use crate::module::rule::UnsafePlacement;
use crate::test::declaration::util::{
    assert_code_findings, assert_code_fixture, read_code_context,
};

#[test]
fn given_persistent_placement_cases_when_checked_then_match_exact_node_anchors() {
    assert_code_fixture(&UnsafePlacement, "UnsafePlacement", "bad");
    assert_code_fixture(&UnsafePlacement, "UnsafePlacement", "good");
}

#[test]
fn given_unsafe_extern_block_in_util_kind_when_checked_then_the_block_is_reported() {
    assert_code_findings(
        &UnsafePlacement,
        "UnsafePlacement",
        "mod util {\n    unsafe extern \"C\" {\n        safe fn abs(input: i32) -> i32;\n    }\n    fn load() { unsafe {} }\n}",
        &[2, 5],
    );
}

#[test]
fn given_unsafe_attributes_outside_a_technology_module_when_checked_then_each_attribute_is_reported()
 {
    assert_code_findings(
        &UnsafePlacement,
        "UnsafePlacement",
        "mod util {\n    #[unsafe(no_mangle)]\n    fn exported() {}\n    #[unsafe(export_name = \"shop_run\")]\n    fn run() {}\n    #[inline]\n    fn plain() {}\n}\nmod file { #[unsafe(no_mangle)] fn exported() {} }",
        &[2, 4],
    );
}

fn list_unsafe_lines(source: &str, role: Role) -> Vec<u32> {
    let mut context = read_code_context(source);
    context.project_config.crates[0].role = Some(role);
    let findings = UnsafePlacement.check(&context);
    let mut lines: Vec<u32> = findings
        .iter()
        .map(|finding| {
            assert_eq!("UnsafePlacement", finding.finding.rule_id.0);
            finding.finding.location.line.expect("finding line")
        })
        .collect();
    lines.sort_unstable();
    lines
}

const HOST_FUNCTION: &str = "#[unsafe(no_mangle)]\npub unsafe extern \"C\" fn start_app(token: *mut u8) -> i32 {\n    let value = unsafe { token.as_ref() };\n    i32::from(value.is_some())\n}";
const EXPORTED_HOST_FUNCTION: &str = "#[unsafe(export_name = \"stop_app_symbol\")]\npub unsafe extern \"C\" fn stop_app(token: *mut u8) {\n    let value = unsafe { token.as_ref() };\n    drop(value);\n}";

#[test]
fn given_host_function_in_the_root_of_an_entry_point_when_checked_then_attribute_signature_and_blocks_pass()
 {
    assert_eq!(
        Vec::<u32>::new(),
        list_unsafe_lines(HOST_FUNCTION, Role::EntryPoint)
    );
    assert_eq!(
        Vec::<u32>::new(),
        list_unsafe_lines(EXPORTED_HOST_FUNCTION, Role::EntryPoint)
    );
}

#[test]
fn given_host_function_with_a_bare_symbol_attribute_in_the_root_of_an_entry_point_when_checked_then_it_passes()
 {
    for source in [
        "#[export_name = \"stop_app_symbol\"]\npub unsafe extern \"C\" fn stop_app(token: *mut u8) {\n    let value = unsafe { token.as_ref() };\n    drop(value);\n}",
        "#[no_mangle]\npub unsafe extern \"C\" fn start_app(token: *mut u8) -> i32 {\n    let value = unsafe { token.as_ref() };\n    i32::from(value.is_some())\n}",
    ] {
        assert_eq!(
            Vec::<u32>::new(),
            list_unsafe_lines(source, Role::EntryPoint),
            "{source}"
        );
    }
}

#[test]
fn given_host_function_outside_an_entry_point_root_when_checked_then_each_unsafe_site_reports() {
    assert_eq!(
        vec![1, 2, 3],
        list_unsafe_lines(HOST_FUNCTION, Role::Context)
    );
    assert_eq!(
        vec![1, 2, 3],
        list_unsafe_lines(EXPORTED_HOST_FUNCTION, Role::Context)
    );
    let nested = format!("mod embedding {{\n{HOST_FUNCTION}\n}}");
    assert_eq!(vec![2, 3, 4], list_unsafe_lines(&nested, Role::EntryPoint));
}

#[test]
fn given_unsafe_function_that_is_no_exported_abi_function_when_in_an_entry_root_then_it_reports() {
    assert_eq!(
        vec![1, 2],
        list_unsafe_lines(
            "pub unsafe fn plain(token: *mut u8) {\n    unsafe { token.as_ref(); }\n}",
            Role::EntryPoint,
        )
    );
    assert_eq!(
        vec![1, 2],
        list_unsafe_lines(
            "pub unsafe extern \"C\" fn callback(token: *mut u8) {\n    unsafe { token.as_ref(); }\n}",
            Role::EntryPoint,
        )
    );
    assert_eq!(
        vec![2],
        list_unsafe_lines(
            "fn helper(token: *mut u8) {\n    unsafe { token.as_ref(); }\n}",
            Role::EntryPoint,
        )
    );
}

#[test]
fn given_struct_and_impl_named_in_host_items_when_unsafe_then_the_impl_still_reports() {
    let mut context = read_code_context("pub struct Plugin;\nunsafe impl Send for Plugin {}");
    context.project_config.crates[0].role = Some(Role::EntryPoint);
    context.project_config.crates[0].host_items = vec!["Plugin".to_owned()];
    let lines: Vec<_> = UnsafePlacement
        .check(&context)
        .iter()
        .map(|finding| finding.finding.location.line)
        .collect();
    assert_eq!(vec![Some(2)], lines);
}

fn build_test_tree_source(body: &str) -> String {
    format!(
        "#[unsafe(no_mangle)]\npub unsafe extern \"C\" fn start_app(token: *mut u8) -> i32 {{ 0 }}\nfn helper(token: *mut u8) -> i32 {{ 0 }}\nmod test {{\n    use crate::helper;\n    use crate::start_app;\n    fn calls() -> i32 {{\n        {body}\n    }}\n}}"
    )
}

#[test]
fn given_unsafe_block_whose_body_calls_a_host_function_in_the_test_tree_when_checked_then_it_passes()
 {
    for body in [
        "unsafe { start_app(std::ptr::null_mut()) }",
        "unsafe { crate::start_app(std::ptr::null_mut()) }",
    ] {
        assert_eq!(
            Vec::<u32>::new(),
            list_unsafe_lines(&build_test_tree_source(body), Role::EntryPoint),
            "{body}"
        );
    }
}

#[test]
fn given_unsafe_block_in_the_test_tree_that_is_no_plain_host_call_when_checked_then_it_reports() {
    for body in [
        "unsafe { helper(std::ptr::null_mut()) }",
        "unsafe { let value = start_app(std::ptr::null_mut()); value + 1 }",
    ] {
        assert_eq!(
            vec![8],
            list_unsafe_lines(&build_test_tree_source(body), Role::EntryPoint),
            "{body}"
        );
    }
    assert_eq!(
        vec![1, 2, 8],
        list_unsafe_lines(
            &build_test_tree_source("unsafe { start_app(std::ptr::null_mut()) }"),
            Role::Context
        )
    );
    assert_eq!(
        vec![3],
        list_unsafe_lines(
            "#[unsafe(no_mangle)]\npub unsafe extern \"C\" fn start_app(token: *mut u8) -> i32 { 0 }\nmod embedding { fn calls() -> i32 { unsafe { crate::start_app(std::ptr::null_mut()) } } }",
            Role::EntryPoint
        )
    );
}

#[test]
fn given_host_call_in_the_test_tree_with_a_plain_argument_when_checked_then_it_passes() {
    for body in [
        "unsafe { start_app(std::ptr::null_mut()) }",
        "unsafe { start_app(std::ptr::null()) }",
        "unsafe { start_app(token) }",
        "unsafe { start_app(0) }",
    ] {
        assert_eq!(
            Vec::<u32>::new(),
            list_unsafe_lines(&build_test_tree_source(body), Role::EntryPoint),
            "{body}"
        );
    }
}

#[test]
fn given_host_call_in_the_test_tree_with_a_raw_deref_or_an_unlisted_call_argument_when_checked_then_it_reports()
 {
    for body in [
        "unsafe { start_app(*raw_pair) }",
        "unsafe { start_app(*raw_pair, other()) }",
        "unsafe { start_app(other()) }",
        "unsafe { start_app(std::ptr::null_mut(), other()) }",
        "unsafe { start_app(raw_pair.read()) }",
    ] {
        assert_eq!(
            vec![8],
            list_unsafe_lines(&build_test_tree_source(body), Role::EntryPoint),
            "{body}"
        );
    }
}

#[test]
fn given_unsafe_in_platform_crate_concept_when_checked_then_only_the_util_kind_is_reported() {
    let mut context = read_code_context(
        "mod menu {\n    fn open() { unsafe {} }\n}\nmod util {\n    fn load() { unsafe {} }\n}",
    );
    context.project_config.crates[0].role = Some(crate::config::model::Role::Platform);
    let findings = UnsafePlacement.check(&context);
    let mut lines: Vec<_> = findings
        .iter()
        .map(|finding| {
            assert_eq!("UnsafePlacement", finding.finding.rule_id.0);
            finding.finding.location.line.expect("finding line")
        })
        .collect();
    lines.sort_unstable();
    assert_eq!(vec![5], lines, "{findings:?}");
}
