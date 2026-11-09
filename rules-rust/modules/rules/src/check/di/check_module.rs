use crate::check::api::ReportWriter;
use crate::check::di::RuleModule;
use crate::check::file::FileReportWriter;
use crate::check::usecase::CheckWorkspaceUseCase;
use crate::source::api::SourceReader;
use crate::source::syn::SynSourceReader;
use crate::workspace::api::RustfmtReader;
use crate::workspace::api::WorkspaceReader;
use crate::workspace::cargo::CargoRustfmtReader;
use crate::workspace::cargo::CargoWorkspaceReader;
use crate::workspace::util::resolve_cargo_path;
use crate::workspace::util::resolve_rustfmt_path;
use std::sync::Arc;

pub struct CheckModule {
    pub check_workspace_use_case: Arc<CheckWorkspaceUseCase>,
    pub report_writer: Arc<dyn ReportWriter>,
}

impl CheckModule {
    pub fn new() -> CheckModule {
        let cargo = resolve_cargo_path();
        let rustfmt = resolve_rustfmt_path(&cargo);
        let workspace_reader: Arc<dyn WorkspaceReader> = Arc::new(CargoWorkspaceReader::new(cargo));
        let source_reader: Arc<dyn SourceReader> = Arc::new(SynSourceReader);
        let rustfmt_reader: Arc<dyn RustfmtReader> = Arc::new(CargoRustfmtReader::new(rustfmt));
        let rule_module = RuleModule::new();
        let check_workspace_use_case = Arc::new(CheckWorkspaceUseCase::new(
            workspace_reader,
            source_reader,
            rustfmt_reader,
            rule_module.rules,
            rule_module.active_rule_ids,
        ));
        CheckModule {
            check_workspace_use_case,
            report_writer: Arc::new(FileReportWriter),
        }
    }
}

impl Default for CheckModule {
    fn default() -> Self {
        Self::new()
    }
}
