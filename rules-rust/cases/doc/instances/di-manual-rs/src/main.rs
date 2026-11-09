use crate::di::RootModule;
use shop_core::lifecycle::api::Lifecycle;
use std::error::Error;
use std::path::PathBuf;
use std::process::ExitCode;
use tokio::runtime::Builder;

pub(crate) mod di {
    pub(crate) use root_module::RootModule;
    mod root_module;
}

// instances/server: src/main.rs, after the `use` lines and the module tree
fn main() -> ExitCode {
    let runtime = match Builder::new_multi_thread().enable_all().build() {
        Ok(runtime) => runtime,
        Err(error) => {
            tracing::error!(
                error = &error as &dyn Error,
                "Could not start the async runtime"
            );
            return ExitCode::FAILURE;
        }
    };
    let root_module = runtime.block_on(async {
        let root_module = RootModule::new(PathBuf::from("config"));
        root_module.on_enable();
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(
                error = &error as &dyn Error,
                "Could not listen for the shutdown signal"
            );
        }
        root_module
    });
    root_module.on_disable();
    ExitCode::SUCCESS
}
