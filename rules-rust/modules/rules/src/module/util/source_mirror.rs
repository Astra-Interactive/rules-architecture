pub(crate) fn find_mirrored_source_path(directory: &str, file: &str) -> Option<String> {
    let prefix = if directory.is_empty() {
        String::new()
    } else {
        format!("{directory}/")
    };
    let relative = file.strip_prefix(&prefix)?;
    let test = relative
        .strip_prefix("tests/it/")
        .or_else(|| relative.strip_prefix("test/"))?;
    Some(format!("{prefix}src/{}.rs", test.strip_suffix("_test.rs")?))
}
