use crate::config::util::parse_project_config;
use crate::test::check::usecase::read_case_context;
use crate::workspace::model::{Dependency, DependencyKind};

#[test]
fn given_dependencies_when_configuration_is_read_then_only_external_non_development_technology_words_are_derived()
 {
    let mut context = read_case_context("RootFileShape", "pass-tree");
    let name = context.workspace.members[0].name.clone();
    let dependencies = &mut context.workspace.members[0].dependencies;
    for dependency in ["aws-sdk-s3", "async-trait", "network", "local", &name] {
        dependencies.push(Dependency {
            name: dependency.to_owned(),
            code_name: dependency.replace('-', "_"),
            path: if dependency == name {
                Some("modules/sample".to_owned())
            } else {
                None
            },
            kind: DependencyKind::Normal,
            features: Vec::new(),
            is_target_specific: false,
            line: None,
        });
    }
    dependencies.push(Dependency {
        name: "test-double".to_owned(),
        code_name: "test_double".to_owned(),
        path: None,
        kind: DependencyKind::Development,
        features: Vec::new(),
        is_target_specific: false,
        line: None,
    });
    let config = parse_project_config(&context.workspace).expect("valid configuration");
    let words = &config.crates[0].technology_words;
    for included in [
        "aws",
        "aws_sdk_s3",
        "async_trait",
        "file",
        "memory",
        "system",
    ] {
        assert!(words.iter().any(|word| word == included));
    }
    for excluded in [
        "async",
        "test",
        "test_double",
        "network",
        "local",
        "case_sample",
    ] {
        assert!(!words.iter().any(|word| word == excluded));
    }
}
