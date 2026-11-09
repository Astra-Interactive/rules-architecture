use crate::workspace::model::TomlValue;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TomlFile {
    pub(crate) path: String,
    pub(crate) root: TomlValue,
}
