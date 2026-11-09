use crate::workspace::model::TomlEntry;
use crate::workspace::model::TomlKind;
use crate::workspace::model::TomlValue;

impl TomlValue {
    pub(crate) fn entries(&self) -> &[TomlEntry] {
        match &self.kind {
            TomlKind::Table(entries) => entries,
            _ => &[],
        }
    }

    pub(crate) fn find_entry(&self, key: &str) -> Option<&TomlEntry> {
        self.entries().iter().find(|entry| entry.key == key)
    }

    pub(crate) fn find(&self, path: &[&str]) -> Option<&TomlValue> {
        let Some((first, rest)) = path.split_first() else {
            return Some(self);
        };
        self.find_entry(first)?.value.find(rest)
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match &self.kind {
            TomlKind::String(string) => Some(string),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self.kind {
            TomlKind::Boolean(boolean) => Some(boolean),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[TomlValue]> {
        match &self.kind {
            TomlKind::Array(elements) => Some(elements),
            _ => None,
        }
    }

    pub(crate) fn is_table(&self) -> bool {
        matches!(self.kind, TomlKind::Table(_))
    }
}
