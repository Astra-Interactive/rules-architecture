use crate::check::model::CheckError;
use crate::workspace::model::TomlEntry;
use crate::workspace::model::TomlKind;
use crate::workspace::model::TomlValue;
use crate::workspace::util::parse_toml_file;

fn build_entry(key: &str, line: u32, kind: TomlKind) -> TomlEntry {
    TomlEntry {
        key: key.to_owned(),
        line,
        value: TomlValue { line, kind },
    }
}

#[test]
fn given_every_value_kind_when_parsed_then_each_keeps_its_text_and_line() {
    let text = "count = 0x10\nratio = 1.5\nwhen = 1979-05-27\nflags = [true]\n";

    let file = parse_toml_file("values.toml", text).expect("the file parses");

    let flag = TomlValue {
        line: 4,
        kind: TomlKind::Boolean(true),
    };
    assert_eq!(
        vec![
            build_entry("count", 1, TomlKind::Integer("0x10".to_owned())),
            build_entry("flags", 4, TomlKind::Array(vec![flag])),
            build_entry("ratio", 2, TomlKind::Float("1.5".to_owned())),
            build_entry("when", 3, TomlKind::Datetime("1979-05-27".to_owned())),
        ],
        file.root.entries()
    );
}

#[test]
fn given_invalid_toml_when_parsed_then_configuration_error_names_its_line() {
    let result = parse_toml_file("broken.toml", "valid = 1\nbroken = \n");

    assert!(
        matches!(result, Err(CheckError::Configuration { path, line, .. }) if path == "broken.toml" && line == Some(2))
    );
}
