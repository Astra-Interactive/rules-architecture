use crate::workspace::model::TomlValue;
use crate::workspace::util::parse_toml_file;

#[test]
fn given_scalar_kinds_when_boolean_is_requested_then_only_boolean_values_match() {
    let parsed = parse_toml_file("values.toml", "flag=true\nname=\"value\"\n").expect("TOML");
    assert_eq!(
        parsed.root.find(&["flag"]).and_then(TomlValue::as_bool),
        Some(true)
    );
    assert_eq!(
        parsed.root.find(&["name"]).and_then(TomlValue::as_bool),
        None
    );
}
