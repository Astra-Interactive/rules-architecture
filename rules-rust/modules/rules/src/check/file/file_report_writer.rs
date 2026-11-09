use crate::check::api::ReportWriter;
use crate::check::model::CheckError;
use crate::check::model::Report;
use std::fs;
use std::path::Path;

pub(crate) struct FileReportWriter;

impl FileReportWriter {
    fn write_file(path: &Path, text: &str) -> Result<(), CheckError> {
        fs::write(path, text).map_err(|error| CheckError::Io {
            path: path.to_string_lossy().into_owned(),
            source: error.into(),
        })
    }
}

impl ReportWriter for FileReportWriter {
    fn write_with_ownership(
        &self,
        checked: &crate::check::model::CheckedWorkspace,
    ) -> Result<(), CheckError> {
        self.write(&checked.report)?;
        Self::write_file(
            &checked
                .report
                .target_directory
                .join("klibs-rules/api-ownership.txt"),
            checked.api_ownership.as_text(),
        )
    }

    fn write(&self, report: &Report) -> Result<(), CheckError> {
        let directory = report.target_directory.join("klibs-rules");
        fs::create_dir_all(&directory).map_err(|error| CheckError::Io {
            path: directory.to_string_lossy().into_owned(),
            source: error.into(),
        })?;
        Self::write_file(
            &directory.join("violations.txt"),
            &report.to_violation_lines(),
        )?;
        Self::write_file(&directory.join("policy.txt"), &report.to_policy_lines())
    }
}
