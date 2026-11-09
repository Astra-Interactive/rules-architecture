use crate::check::model::CheckError;
use crate::check::model::Report;

pub trait ReportWriter: Send + Sync {
    /// # Errors
    ///
    /// Returns [`CheckError::Io`] when the report directory or one of its two files cannot be written.
    fn write(&self, report: &Report) -> Result<(), CheckError>;
}
