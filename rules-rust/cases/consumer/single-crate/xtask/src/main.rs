use crate::architecture::command::run_architecture_check;
use std::env;
use std::path::Path;
use std::process::ExitCode;

pub(crate) mod architecture {
    pub(crate) mod command {
        pub(crate) use architecture_check_command::run_architecture_check;
        mod architecture_check_command;
    }
}

fn main() -> ExitCode {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("architecture-check") => run_architecture_check(Path::new(env!("CARGO_MANIFEST_DIR"))),
        _ => ExitCode::from(2),
    }
}

#[cfg(test)]
#[path = "../test"]
mod test {
    mod architecture {
        mod command {
            mod architecture_check_command_test;
        }
    }
}
