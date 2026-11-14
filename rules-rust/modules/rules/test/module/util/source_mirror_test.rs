use crate::module::util::find_mirrored_source_path;

#[test]
fn given_test_files_when_mirrored_then_the_source_file_beside_src_is_named() {
    assert_eq!(
        find_mirrored_source_path(
            "modules/core",
            "modules/core/test/slot/command_slot_test.rs"
        ),
        Some("modules/core/src/slot/command_slot.rs".to_owned())
    );
    assert_eq!(
        find_mirrored_source_path(
            "modules/macros",
            "modules/macros/tests/it/derive/derive_test.rs"
        ),
        Some("modules/macros/src/derive/derive.rs".to_owned())
    );
    assert_eq!(
        find_mirrored_source_path("", "test/slot/command_slot_test.rs"),
        Some("src/slot/command_slot.rs".to_owned())
    );
}

#[test]
fn given_files_that_are_no_tests_of_the_member_when_mirrored_then_there_is_no_source_file() {
    assert_eq!(
        find_mirrored_source_path("modules/core", "modules/core/test/slot/helper.rs"),
        None
    );
    assert_eq!(
        find_mirrored_source_path(
            "modules/core",
            "modules/other/test/slot/command_slot_test.rs"
        ),
        None
    );
    assert_eq!(
        find_mirrored_source_path("modules/core", "modules/core/src/slot/command_slot_test.rs"),
        None
    );
}
