use crate::config::model::Role;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CrateConfig {
    pub(crate) name: String,
    pub(crate) role: Option<Role>,
    pub(crate) host_items: Vec<String>,
    pub(crate) technology_words: Vec<String>,
    pub(crate) composition: String,
    pub(crate) implementation_access: String,
}
