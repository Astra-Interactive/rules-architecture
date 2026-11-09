use crate::workspace::model::Dependency;
use crate::workspace::model::Target;
use crate::workspace::model::TomlFile;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Member {
    pub(crate) name: String,
    pub(crate) directory: String,
    pub(crate) manifest: TomlFile,
    pub(crate) targets: Vec<Target>,
    pub(crate) dependencies: Vec<Dependency>,
    pub(crate) readme: Option<String>,
}
