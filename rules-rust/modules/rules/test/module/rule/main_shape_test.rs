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
fn given_early_exit_between_block_on_and_on_disable_when_checked_then_main_is_rejected() {
    for (early_return, early_try, expected) in [
        (
            "if std::env::var_os(\"SKIP\").is_some() { return Ok(()); }",
            "std::fs::write(\"state\", \"down\")?;",
            vec![Some(2)],
        ),
        (
            "if std::env::var_os(\"SKIP\").is_some() { return Ok(()); }",
            "",
            vec![Some(2)],
        ),
        ("", "std::fs::write(\"state\", \"down\")?;", vec![Some(2)]),
        ("", "", vec![]),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }}\nfn main() -> Result<(), std::io::Error> {{\n    let runtime = tokio::runtime::Builder::new_multi_thread().build()?;\n    let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); tokio::signal::ctrl_c().await; root }});\n    {early_return}\n    {early_try}\n    root.on_disable();\n    std::fs::write(\"state\", \"done\")?;\n    Ok(())\n}}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        let lines: Vec<Option<u32>> =
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context)
                .iter()
                .map(|finding| finding.finding.location.line)
                .collect();
        assert_eq!(lines, expected, "{source}");
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
fn given_shutdown_wait_inside_associated_function_when_checked_then_only_the_signal_wait_passes() {
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
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }} mod util {{ pub struct ShutdownSignal; impl ShutdownSignal {{ pub async fn wait() {{ {helper} }} }} }} fn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); util::ShutdownSignal::wait().await; root }}); root.on_disable(); }}"
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

#[test]
fn given_token_or_workspace_method_wait_when_checked_then_only_a_shutdown_wait_passes() {
    for (helper, wait, expected) in [
        (
            "tokio::signal::ctrl_c().await;",
            "let token = tokio_util::sync::CancellationToken::new(); token.cancelled().await;",
            vec![],
        ),
        (
            "tokio::signal::ctrl_c().await;",
            "let signal = util::ShutdownSignal::new(); signal.wait().await;",
            vec![],
        ),
        (
            "let (sender, mut receiver) = tokio::sync::mpsc::channel::<u8>(1); receiver.recv().await;",
            "let signal = util::ShutdownSignal::new(); signal.wait().await;",
            vec![Some(3)],
        ),
        (
            "tokio::signal::ctrl_c().await;",
            "let token = vendor::CancellationToken::new(); token.cancelled().await;",
            vec![Some(3)],
        ),
    ] {
        let source = format!(
            "mod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }}\nmod util {{ pub struct ShutdownSignal; impl ShutdownSignal {{ pub fn new() -> Self {{ Self }} pub async fn wait(&self) {{ {helper} }} }} }}\nfn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); {wait} root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        let mut dependencies = context.indexes[0].dependencies.clone();
        dependencies.push("tokio_util".to_owned());
        context.indexes = vec![
            crate::source::syn::CrateIndexBuilder::new(&context.crates[0], dependencies).build(),
        ];
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        let lines: Vec<Option<u32>> =
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context)
                .iter()
                .map(|finding| finding.finding.location.line)
                .collect();
        assert_eq!(lines, expected, "{wait} / {helper}");
    }
}

#[test]
fn given_annotated_local_of_workspace_type_when_its_method_waits_then_only_a_shutdown_wait_passes()
{
    for (helper, expected) in [
        ("tokio::signal::ctrl_c().await;", vec![]),
        (
            "let (sender, mut receiver) = tokio::sync::mpsc::channel::<u8>(1); receiver.recv().await;",
            vec![Some(4)],
        ),
    ] {
        let source = format!(
            "use crate::util::ShutdownSignal;\nmod di {{ pub struct RootModule; impl RootModule {{ pub fn new() -> Self {{ Self }} }} }}\nmod util {{ pub struct ShutdownSignal; impl ShutdownSignal {{ pub fn new() -> Self {{ Self }} pub async fn wait(&self) {{ {helper} }} }} }}\nfn main() {{ let runtime = tokio::runtime::Builder::new_multi_thread().build(); let root = runtime.block_on(async {{ let root = di::RootModule::new(); root.on_enable(); let signal: ShutdownSignal = ShutdownSignal::new(); signal.wait().await; root }}); root.on_disable(); }}"
        );
        let mut context = crate::test::declaration::util::read_code_context(&source);
        context.project_config.crates[0].role = Some(crate::config::model::Role::EntryPoint);
        let lines: Vec<Option<u32>> =
            crate::check::api::Rule::check(&crate::module::rule::MainShape, &context)
                .iter()
                .map(|finding| finding.finding.location.line)
                .collect();
        assert_eq!(lines, expected, "{helper}");
    }
}

#[test]
fn given_main_of_a_binary_entry_point_with_a_library_when_checked_then_it_constructs_the_root_module_of_the_library()
 {
    let context =
        crate::test::check::usecase::read_case_context("CrateTargets", "pass-binary-entry-point");
    assert!(crate::check::api::Rule::check(&crate::module::rule::MainShape, &context).is_empty());
}
