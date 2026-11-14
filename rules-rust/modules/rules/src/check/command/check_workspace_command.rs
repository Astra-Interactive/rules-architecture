use crate::check::di::CheckModule;
use crate::check::util::parse_workspace_directory;
use std::ffi::OsString;
use std::process::ExitCode;

/// The exit code is 0 for a clean workspace, 1 when the report has findings and 2 for an argument,
/// configuration, metadata or write error.
#[expect(
    clippy::print_stdout,
    reason = "the architecture report is this command's product"
)]
pub fn run_check_workspace_command(arguments: &[OsString]) -> ExitCode {
    let workspace_dir = match parse_workspace_directory(arguments) {
        Ok(directory) => directory,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };
    let check_module = CheckModule::new();
    let checked = match check_module
        .check_workspace_use_case
        .invoke_with_ownership(&workspace_dir)
    {
        Ok(report) => report,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };
    let report = &checked.report;
    println!("{}", report.to_violation_lines());
    if let Err(error) = check_module.report_writer.write_with_ownership(&checked) {
        println!("{error}");
        return ExitCode::from(2);
    }
    report.to_exit_code()
}
