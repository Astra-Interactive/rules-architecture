use crate::workspace::model::TomlValue;

pub(crate) struct LockPackage<'lock> {
    pub(crate) name: &'lock str,
    pub(crate) version: &'lock str,
    pub(crate) source: Option<&'lock str>,
    pub(crate) value: &'lock TomlValue,
}
