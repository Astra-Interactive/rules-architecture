#[test]
fn given_entry_main_when_checked_then_runtime_root_enable_and_synchronous_disable_are_required() {
    let source = "mod di { pub struct RootModule; impl RootModule { pub fn new() -> Self { Self } } }\n fn main() { let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async { let root = di::RootModule::new(); root.on_enable(); tokio::signal::ctrl_c().await; root }); root.on_disable(); }";
    let mut context = crate::test::declaration::util::read_code_context(source);
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    assert!(crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).is_empty());
    let mut context = crate::test::declaration::util::read_code_context("fn main() {} ");
    context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
    let findings = crate::check::api::Rule::check(&crate::module::rule::MainShape, &context);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].finding.location.line, Some(1));
}

#[test]
fn given_no_shutdown_wait_or_early_runtime_drop_when_checked_then_main_is_rejected() {
    for (wait, drop_runtime) in [
        ("", ""),
        ("tokio::signal::ctrl_c().await;", "drop(runtime);"),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} fn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); {wait} root }}); {drop_runtime} root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        assert_eq!(
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).len(),
            1
        );
    }
}

#[test]
fn given_persistent_di_policy_pairs_when_checked_then_exact_nodes_and_subjects_match() {
    for half in ["bad", "good"] {
        crate::test::module::util::assert_di_fixture(
            &crate::module::rule::MainShape,
            "MainShape",
            half,
        );
    }
}

#[test]
fn given_on_enable_on_one_branch_when_checked_then_main_is_rejected() {
    for (enable, expected) in [
        ("if flag { root.on_enable(); }", 1),
        (
            "if flag { root.on_enable(); } else { root.on_enable(); }",
            0,
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} fn main() {{ let flag = std::env::args().count() > 1; let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); {enable} tokio::signal::ctrl_c().await; root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        assert_eq!(
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).len(),
            expected,
            "{enable}"
        );
    }
}

#[test]
fn given_on_disable_on_one_branch_after_block_on_when_checked_then_main_is_rejected() {
    for (disable, expected) in [
        ("if flag { root.on_disable(); }", 1),
        (
            "if flag { root.on_disable(); } else { root.on_disable(); }",
            0,
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} fn main() {{ let flag = std::env::args().count() > 1; let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); tokio::signal::ctrl_c().await; root }}); {disable} }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        assert_eq!(
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).len(),
            expected,
            "{disable}"
        );
    }
}

#[test]
fn given_signal_family_wait_when_checked_then_shutdown_is_waited_and_unrelated_receive_is_not() {
    for (wait, expected) in [
        (
            "let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap(); terminate.recv().await;",
            0,
        ),
        (
            "let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap(); tokio::select! { _ = tokio::signal::ctrl_c() => {} _ = terminate.recv() => {} }",
            0,
        ),
        (
            "let (sender, mut receiver) = tokio::sync::mpsc::channel::<u8>(1); receiver.recv().await;",
            1,
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} fn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); {wait} root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        assert_eq!(
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).len(),
            expected,
            "{wait}"
        );
    }
}

#[test]
fn given_shutdown_wait_inside_workspace_helper_when_checked_then_only_the_signal_wait_passes() {
    for (helper, expected) in [
        (
            "if let Err(error) = tokio::signal::ctrl_c().await { drop(error); }",
            0,
        ),
        (
            "let (sender, mut receiver) = tokio::sync::mpsc::channel::<u8>(1); receiver.recv().await;",
            1,
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} mod util {{ pub async fn wait_for_shutdown() {{ {helper} }} }} fn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); util::wait_for_shutdown().await; root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        let findings = crate::check::api::Rule::check(&crate::module::rule::MainShape, &context);
        assert_eq!(findings.len(), expected, "{helper}: {findings:?}");
        for finding in &findings {
            assert_eq!("MainShape", finding.finding.rule_id.0);
            assert_eq!(Some(1), finding.finding.location.line);
        }
    }
}

#[test]
fn given_shutdown_wait_in_one_match_arm_when_checked_then_main_is_rejected() {
    for (wait, expected) in [
        (
            "match mode { 1 => { tokio::signal::ctrl_c().await; } _ => {} }",
            1,
        ),
        (
            "match mode { 1 => { tokio::signal::ctrl_c().await; } _ => { tokio::signal::ctrl_c().await; } }",
            0,
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} fn main() {{ let mode = std::env::args().count(); let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); {wait} root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        assert_eq!(
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).len(),
            expected,
            "{wait}"
        );
    }
}
