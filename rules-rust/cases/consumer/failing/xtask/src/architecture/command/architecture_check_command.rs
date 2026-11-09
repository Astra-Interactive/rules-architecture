use klibs_rules::check::di::CheckModule;
use std::path::Path;
use std::process::ExitCode;

#[expect(
    clippy::print_stdout,
    reason = "the architecture report is this command's product"
)]
pub(crate) fn run_architecture_check(workspace_dir: &Path) -> ExitCode {
    let check_module = CheckModule::new();
    let report = match check_module.check_workspace_use_case.invoke(workspace_dir) {
        Ok(report) => report,
        Err(error) => {
            println!("{error}");
            return ExitCode::from(2);
        }
    };
    println!("{}", report.to_violation_lines());
    if let Err(error) = check_module.report_writer.write(&report) {
        println!("{error}");
        return ExitCode::from(2);
    }
    report.to_exit_code()
}
