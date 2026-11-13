#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TomlValue {
    pub(crate) line: u32,
    pub(crate) kind: TomlKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TomlKind {
    String(String),
    Integer(String),
    Float(String),
    Boolean(bool),
    Datetime(String),
    Array(Vec<TomlValue>),
    Table(Vec<TomlEntry>),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct TomlEntry {
    pub(crate) key: String,
    pub(crate) line: u32,
    pub(crate) value: TomlValue,
}
