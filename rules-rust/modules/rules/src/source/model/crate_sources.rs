use crate::source::model::Expectation;
use crate::source::model::SourceFile;
use crate::source::model::SourceModule;
use crate::source::model::UnanalysableSource;

#[derive(Clone)]
pub(crate) struct CrateSources {
    pub(crate) crate_name: String,
    pub(crate) target: String,
    pub(crate) modules: Vec<SourceModule>,
    pub(crate) files: Vec<SourceFile>,
    pub(crate) expectations: Vec<Expectation>,
    pub(crate) unanalysable: Vec<UnanalysableSource>,
    pub(crate) undeclared_files: Vec<String>,
}
