const SKIPPED_NAMES: [&str; 4] = [".git", "build", "node_modules", "resources"];
const SOURCE_DIRECTORIES: [&str; 6] = ["src", "test", "fixture", "tests", "benches", "examples"];

pub(crate) fn is_skipped_directory_name(name: &str) -> bool {
    let is_hidden = name.starts_with('.') && name != ".cargo";
    is_hidden || SKIPPED_NAMES.contains(&name)
}

pub(crate) fn is_member_source_directory(relative: &str, member_directories: &[String]) -> bool {
    member_directories.iter().any(|member| {
        SOURCE_DIRECTORIES.iter().any(|source| {
            let source_root = if member.is_empty() {
                (*source).to_owned()
            } else {
                format!("{member}/{source}")
            };
            relative == source_root || relative.starts_with(&format!("{source_root}/"))
        })
    })
}
