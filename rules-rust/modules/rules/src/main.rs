use klibs_rules::check::command::run_check_workspace_command;
use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<OsString> = env::args_os().skip(1).collect();
    run_check_workspace_command(&arguments)
}
