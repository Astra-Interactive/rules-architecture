use crate::di::RootModule;
use shop_core::lifecycle::api::Lifecycle;
use std::path::PathBuf;

#[test]
fn given_root_module_when_enabled_and_disabled_then_the_lifecycle_completes() {
    let root_module = RootModule::new(PathBuf::from("config"));

    root_module.on_enable();
    root_module.on_disable();
}
